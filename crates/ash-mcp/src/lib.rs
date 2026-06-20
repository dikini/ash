//! MCP (Model Context Protocol) server exposing Ash language intelligence.
//!
//! Thin wrapper over `ash-lsp-core` that speaks MCP so AI coding agents
//! can query diagnostics, hover, go-to-definition, completion, and symbols
//! with the same precision as a human IDE user.

pub mod daemon;

use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;

use ash_lint::LintConfig;
use ash_lsp_core::analysis::AnalysisCache;
use ash_lsp_core::completion;
use ash_lsp_core::diagnostics;
use ash_lsp_core::goto;
use ash_lsp_core::hover;
use ash_lsp_core::symbols;
use ash_lsp_core::vfs::Vfs;
use ash_parser::parse_surface_file;

// ---------------------------------------------------------------------------
// Shared parameter types
// ---------------------------------------------------------------------------

/// Position within a file (1-indexed line and column).
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PositionParams {
    /// Absolute file path.
    pub file: String,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column (character offset within the line).
    pub column: u32,
}

/// File-only parameter.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileParams {
    /// Absolute file path.
    pub file: String,
}

/// Workspace symbol search parameters.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkspaceSymbolParams {
    /// Absolute workspace root path.
    pub root: String,
    /// Case-insensitive substring to match against symbol names.
    pub query: String,
}

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

/// The Ash MCP server state.
///
/// Holds a VFS and analysis cache so files stay in memory across tool calls
/// within a session, as required by SPEC-038 §8.5.
pub struct AshMcpServer {
    vfs: Arc<Vfs>,
    cache: Arc<AnalysisCache>,
    config: LintConfig,
    #[allow(dead_code)] // Used by #[tool_handler] generated code
    tool_router: ToolRouter<Self>,
}

impl std::fmt::Debug for AshMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AshMcpServer").finish()
    }
}

impl AshMcpServer {
    /// Create a new MCP server with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vfs: Arc::new(Vfs::new()),
            cache: Arc::new(AnalysisCache::new()),
            config: LintConfig::default(),
            tool_router: Self::tool_router(),
        }
    }

    /// Create a new MCP server with a custom lint configuration.
    #[must_use]
    pub fn with_config(config: LintConfig) -> Self {
        Self {
            vfs: Arc::new(Vfs::new()),
            cache: Arc::new(AnalysisCache::new()),
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Create a new MCP server sharing an existing VFS and analysis cache.
    #[must_use]
    pub fn with_vfs_and_cache(
        vfs: Arc<Vfs>,
        cache: Arc<AnalysisCache>,
        config: LintConfig,
    ) -> Self {
        Self {
            vfs,
            cache,
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Build a `file://` URI from an OS path using `FromStr`.
    fn file_uri(file_path: &str) -> Result<lsp_types::Uri, String> {
        let uri_str = if file_path.starts_with('/') {
            format!("file://{file_path}")
        } else {
            format!("file:///{file_path}")
        };
        uri_str
            .parse()
            .map_err(|e| format!("invalid URI '{uri_str}': {e}"))
    }

    /// Ensure a file is open in the VFS, reading from disk if needed.
    fn ensure_open(&self, file_path: &str) -> Result<Arc<ash_lsp_core::vfs::VfsEntry>, String> {
        let uri = Self::file_uri(file_path)?;

        if let Some(entry) = self.vfs.get(&uri) {
            return Ok(entry);
        }

        let content =
            std::fs::read_to_string(file_path).map_err(|e| format!("read {file_path}: {e}"))?;

        self.vfs.open(uri.clone(), 0, content);
        self.cache.invalidate(&uri);

        self.vfs
            .get(&uri)
            .ok_or_else(|| "VFS entry missing after open".into())
    }

    /// Parse a file from the VFS entry content.
    fn parse_file(
        entry: &ash_lsp_core::vfs::VfsEntry,
    ) -> Result<ash_parser::surface::ModuleFile, String> {
        parse_surface_file(&entry.content).map_err(|e| format!("parse error: {e:?}"))
    }

    /// Build a simple JSON success response with a summary.
    fn json_success(summary: String, payload: serde_json::Value) -> CallToolResult {
        let mut contents = vec![Content::text(summary)];
        if !payload.is_null() {
            contents.push(
                Content::json(payload)
                    .unwrap_or_else(|_| Content::text(String::from("[serialization error]"))),
            );
        }
        CallToolResult::success(contents)
    }

    /// Build a JSON error response.
    fn json_error(msg: String) -> CallToolResult {
        CallToolResult::error(vec![Content::text(msg)])
    }

    /// Public convenience wrapper for workspace symbol tests and direct callers.
    #[must_use]
    pub fn workspace_symbols(&self, root: String, query: String) -> CallToolResult {
        self.ash_workspace_symbols(Parameters(WorkspaceSymbolParams { root, query }))
    }

    /// Public convenience wrapper for same-file find-references tests and direct callers.
    #[must_use]
    pub fn find_references(&self, file: String, line: u32, column: u32) -> CallToolResult {
        self.ash_find_references(Parameters(PositionParams { file, line, column }))
    }

    /// Public convenience wrapper for go-to-definition tests and direct callers.
    #[must_use]
    pub fn goto_definition(&self, file: String, line: u32, column: u32) -> CallToolResult {
        self.ash_goto_definition(Parameters(PositionParams { file, line, column }))
    }
}

impl Default for AshMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

#[tool_router]
impl AshMcpServer {
    /// Get lint and parse diagnostics for an Ash source file.
    #[tool(description = "Get lint and parse diagnostics for an Ash source file")]
    fn ash_get_diagnostics(&self, Parameters(params): Parameters<FileParams>) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };

        let diags = diagnostics::compute_diagnostics(&entry.content, &self.config);

        let entries: Vec<serde_json::Value> = diags
            .iter()
            .map(|d| {
                serde_json::json!({
                    "severity": format!("{:?}", d.severity).to_lowercase(),
                    "message": d.message,
                    "line": d.range.start.line + 1,
                    "column": d.range.start.character + 1,
                })
            })
            .collect();

        let summary = if entries.is_empty() {
            format!("No issues in {}", params.file)
        } else {
            let errors = entries
                .iter()
                .filter(|e| e["severity"].as_str() == Some("some(error)"))
                .count();
            let warnings = entries.len() - errors;
            format!(
                "{} error(s), {} warning(s) in {}",
                errors, warnings, params.file
            )
        };

        Self::json_success(summary, serde_json::Value::Array(entries))
    }

    /// Get hover/type information at a position.
    #[tool(description = "Get hover/type information at a position in an Ash file")]
    fn ash_hover(&self, Parameters(params): Parameters<PositionParams>) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };

        // MCP uses 1-indexed; lsp-core uses 0-indexed.
        let result = hover::hover_at(&module, &entry.content, params.line - 1, params.column - 1);

        if let Some(h) = result {
            let markdown = match &h.contents {
                lsp_types::HoverContents::Markup(mc) => mc.value.clone(),
                lsp_types::HoverContents::Scalar(ms) => marked_string_value(ms),
                lsp_types::HoverContents::Array(arr) => arr
                    .iter()
                    .map(marked_string_value)
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            };
            let summary = format!(
                "Hover info at {}:{}:{}",
                params.file, params.line, params.column
            );
            Self::json_success(summary, serde_json::json!({ "markdown": markdown }))
        } else {
            let summary = format!(
                "No hover info at {}:{}:{}",
                params.file, params.line, params.column
            );
            Self::json_success(summary, serde_json::Value::Null)
        }
    }

    /// Find the definition of the symbol at a position.
    #[tool(description = "Find the definition of the symbol at a position in an Ash file")]
    fn ash_goto_definition(
        &self,
        Parameters(params): Parameters<PositionParams>,
    ) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };

        let uri = match Self::file_uri(&params.file) {
            Ok(u) => u,
            Err(e) => return Self::json_error(e),
        };

        let result = goto::goto_definition(
            &module,
            &entry.content,
            &uri,
            params.line - 1,
            params.column - 1,
        );

        match result {
            Some(lsp_types::GotoDefinitionResponse::Scalar(loc)) => {
                let summary = format!(
                    "Definition at {}:{}:{}",
                    params.file,
                    loc.range.start.line + 1,
                    loc.range.start.character + 1
                );
                let payload = serde_json::json!({
                    "file": params.file,
                    "start_line": loc.range.start.line + 1,
                    "start_column": loc.range.start.character + 1,
                    "end_line": loc.range.end.line + 1,
                    "end_column": loc.range.end.character + 1,
                });
                Self::json_success(summary, payload)
            }
            Some(lsp_types::GotoDefinitionResponse::Array(locs)) => {
                let summary = format!("{} definition(s) found", locs.len());
                let payload = serde_json::json!(
                    locs.iter()
                        .map(|l| {
                            serde_json::json!({
                                "start_line": l.range.start.line + 1,
                                "start_column": l.range.start.character + 1,
                                "end_line": l.range.end.line + 1,
                                "end_column": l.range.end.character + 1,
                            })
                        })
                        .collect::<Vec<_>>()
                );
                Self::json_success(summary, payload)
            }
            _ => {
                let summary = format!(
                    "No definition found at {}:{}:{}",
                    params.file, params.line, params.column
                );
                Self::json_success(summary, serde_json::Value::Null)
            }
        }
    }

    /// Get completion suggestions at a position.
    #[tool(description = "Get completion suggestions at a position in an Ash file")]
    fn ash_complete(&self, Parameters(params): Parameters<PositionParams>) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };

        let result =
            completion::completions(&module, &entry.content, params.line - 1, params.column - 1);

        let items = match result {
            lsp_types::CompletionResponse::Array(items) => items,
            lsp_types::CompletionResponse::List(list) => list.items,
        };

        let entries: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "label": item.label,
                    "kind": item.kind
                        .map_or_else(|| "unknown".into(), |k| format!("{k:?}").to_lowercase()),
                    "insert_text": item.insert_text,
                })
            })
            .collect();

        let summary = format!(
            "{} completion(s) at {}:{}:{}",
            entries.len(),
            params.file,
            params.line,
            params.column
        );

        Self::json_success(summary, serde_json::Value::Array(entries))
    }

    /// Get document symbol outline for a file.
    #[tool(description = "Get document symbol outline for an Ash file")]
    fn ash_document_symbols(&self, Parameters(params): Parameters<FileParams>) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };

        let syms = symbols::document_symbols(&module);

        let mut entries = Vec::new();
        flatten_symbols(&syms, &mut entries);

        let summary = format!("{} symbol(s) in {}", entries.len(), params.file);
        Self::json_success(summary, serde_json::Value::Array(entries))
    }

    /// Find same-file references to the symbol at a position.
    #[tool(description = "Find references to the symbol at a position (single-file)")]
    fn ash_find_references(
        &self,
        Parameters(params): Parameters<PositionParams>,
    ) -> CallToolResult {
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };
        let uri = match Self::file_uri(&params.file) {
            Ok(u) => u,
            Err(e) => return Self::json_error(e),
        };

        let refs = goto::find_references(
            &module,
            &entry.content,
            &uri,
            params.line - 1,
            params.column - 1,
        );
        let entries = refs
            .iter()
            .map(|loc| {
                serde_json::json!({
                    "file": loc.uri.to_string(),
                    "start_line": loc.range.start.line + 1,
                    "start_column": loc.range.start.character + 1,
                    "end_line": loc.range.end.line + 1,
                    "end_column": loc.range.end.character + 1,
                })
            })
            .collect::<Vec<_>>();

        Self::json_success(
            format!("{} reference(s) found", entries.len()),
            serde_json::Value::Array(entries),
        )
    }

    /// Workspace symbol search.
    #[allow(clippy::unused_self)]
    #[tool(description = "Search workspace symbols by name")]
    fn ash_workspace_symbols(
        &self,
        Parameters(params): Parameters<WorkspaceSymbolParams>,
    ) -> CallToolResult {
        let symbols = symbols::workspace_symbols(std::path::Path::new(&params.root), &params.query);
        let entries = symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "name": symbol.name,
                    "kind": format!("{:?}", symbol.kind).to_lowercase(),
                    "file": symbol.file,
                    "line": symbol.line,
                    "column": symbol.column,
                })
            })
            .collect::<Vec<_>>();

        Self::json_success(
            format!("{} symbol(s) found", entries.len()),
            serde_json::Value::Array(entries),
        )
    }

    /// Code actions (placeholder — deferred).
    #[tool(description = "Get code actions for a range (not yet implemented)")]
    fn ash_code_action(&self) -> CallToolResult {
        let _ = self;
        Self::json_success(
            "Code actions not yet implemented (deferred to Phase 5)".into(),
            serde_json::Value::Null,
        )
    }
}

fn marked_string_value(ms: &lsp_types::MarkedString) -> String {
    match ms {
        lsp_types::MarkedString::String(s) => s.clone(),
        lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
    }
}

fn flatten_symbols(syms: &[lsp_types::DocumentSymbol], out: &mut Vec<serde_json::Value>) {
    for s in syms {
        out.push(serde_json::json!({
            "name": s.name,
            "kind": format!("{:?}", s.kind).to_lowercase(),
            "line": s.range.start.line + 1,
            "column": s.range.start.character + 1,
        }));
        if let Some(children) = &s.children {
            flatten_symbols(children, out);
        }
    }
}

// ---------------------------------------------------------------------------
// ServerHandler — generated by #[tool_handler]
// ---------------------------------------------------------------------------

#[tool_handler]
impl ServerHandler for AshMcpServer {}

// ---------------------------------------------------------------------------
// Binary entry point helper
// ---------------------------------------------------------------------------

/// Run the MCP server over stdio.
///
/// # Errors
///
/// Returns an error if the MCP handshake fails or the transport closes
/// unexpectedly.
pub async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let server = AshMcpServer::new();
    let transport = rmcp::transport::stdio();
    let service = server
        .serve(transport)
        .await
        .inspect_err(|e| tracing::error!("MCP serve error: {e}"))?;
    service.waiting().await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
