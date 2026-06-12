//! MCP (Model Context Protocol) server exposing Ash language intelligence.
//!
//! Thin wrapper over `ash-lsp-core` that speaks MCP so AI coding agents
//! can query diagnostics, hover, go-to-definition, completion, and symbols
//! with the same precision as a human IDE user.

use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use ash_lint::LintConfig;
use ash_lsp_core::analysis::AnalysisCache;
use ash_lsp_core::completion;
use ash_lsp_core::diagnostics;
use ash_lsp_core::goto;
use ash_lsp_core::hover;
use ash_lsp_core::symbols;
use ash_lsp_core::vfs::Vfs;
use ash_parser::parse_surface_file;

// Cross-language configuration and symbol mapping
pub mod cross_lang;
pub mod rust_parser;

// Daemon mode with persistent state and LRU caching
pub mod daemon;

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

/// Workspace symbol search parameters.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkspaceSymbolParams {
    /// Absolute path to the workspace root to search.
    pub root: String,
    /// Case-insensitive substring to match against symbol names.
    pub query: String,
}

/// File-only parameter.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileParams {
    /// Absolute file path.
    pub file: String,
}

/// Ash symbol lookup parameters for cross-language Rust implementation finding.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SymbolLookupParams {
    /// Name of the Ash symbol to find.
    pub ash_symbol: String,
    /// Path to the Ash file containing the symbol.
    pub file: String,
    /// 1-indexed line number where the symbol appears.
    pub line: u32,
    /// 1-indexed column number where the symbol appears.
    pub column: u32,
}

/// Rust symbol lookup parameters for reverse cross-language usage finding.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RustUsageParams {
    /// Fully-qualified Rust symbol to find in Ash mappings.
    pub rust_symbol: String,
}

/// Rust symbol information returned by cross-language lookup.
#[derive(Debug, Serialize)]
pub struct RustSymbolInfo {
    /// Whether the symbol was found.
    pub found: bool,
    /// Rust symbol name, if mapped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_symbol: Option<String>,
    /// Rust symbol kind, if mapped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_kind: Option<String>,
    /// Rust source file path, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Start line in Rust file, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// Start column in Rust file, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    /// End line in Rust file, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// End column in Rust file, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
    /// Confidence level of the mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    /// Source of the mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Error message, if lookup failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Ash usage of a Rust symbol.
#[derive(Debug, Serialize)]
pub struct AshUsageInfo {
    /// Path to the Ash source file.
    pub file: String,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number.
    pub column: u32,
    /// Matching Ash symbol text.
    pub ash_symbol: String,
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

    /// Create a new MCP server with pre-built VFS and cache.
    ///
    /// Used by the daemon mode to share state across requests.
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

    /// Find the Rust implementation corresponding to an Ash symbol.
    #[tool(description = "Find the Rust implementation corresponding to an Ash symbol")]
    #[allow(clippy::unused_self)]
    fn ash_find_rust_implementation(
        &self,
        Parameters(params): Parameters<SymbolLookupParams>,
    ) -> CallToolResult {
        let workspace_root = Self::workspace_root_for_file(&params.file);
        let config = Self::load_cross_lang_config_for_root(&workspace_root);
        let normalized_ash_symbol = Self::normalize_ash_symbol(&params.ash_symbol);
        let Some(mapping) = Self::find_ash_mapping(&config.mappings, normalized_ash_symbol) else {
            return Self::json_success(
                format!("No Rust implementation found for {}", params.ash_symbol),
                serde_json::json!({
                    "found": false,
                    "error": format!("No mapping found for Ash symbol '{}'", params.ash_symbol),
                }),
            );
        };

        match Self::find_rust_symbol_location(&workspace_root, &mapping.rust_symbol) {
            Ok(Some(location)) => Self::json_success(
                format!("Found Rust implementation for {}", params.ash_symbol),
                serde_json::json!(RustSymbolInfo {
                    found: true,
                    rust_symbol: Some(mapping.rust_symbol.clone()),
                    rust_kind: Some(mapping.rust_kind.clone()),
                    file: Some(location.file.display().to_string()),
                    start_line: Some(location.start_line),
                    start_column: Some(location.start_column),
                    end_line: Some(location.end_line),
                    end_column: Some(location.end_column),
                    confidence: Some(format!("{:?}", mapping.confidence).to_lowercase()),
                    source: Some(format!("{:?}", mapping.source).to_lowercase()),
                    error: None,
                }),
            ),
            Ok(None) => Self::json_success(
                format!(
                    "Rust symbol mapped but source location not found for {}",
                    params.ash_symbol
                ),
                serde_json::json!(RustSymbolInfo {
                    found: false,
                    rust_symbol: Some(mapping.rust_symbol.clone()),
                    rust_kind: Some(mapping.rust_kind.clone()),
                    file: None,
                    start_line: None,
                    start_column: None,
                    end_line: None,
                    end_column: None,
                    confidence: Some(format!("{:?}", mapping.confidence).to_lowercase()),
                    source: Some(format!("{:?}", mapping.source).to_lowercase()),
                    error: Some("Rust symbol location not found".to_string()),
                }),
            ),
            Err(err) => Self::json_error(format!("Lookup failed: {err}")),
        }
    }

    /// Find Ash usages corresponding to a Rust symbol.
    #[tool(description = "Find Ash usages corresponding to a Rust symbol")]
    #[allow(clippy::unused_self)]
    fn ash_find_ash_usage(
        &self,
        Parameters(params): Parameters<RustUsageParams>,
    ) -> CallToolResult {
        let workspace_root = Self::cross_lang_config_root_for_root(&Self::workspace_root());
        let config = Self::load_cross_lang_config_for_root(&workspace_root);
        let mappings: Vec<_> = config
            .mappings
            .iter()
            .filter(|mapping| mapping.rust_symbol == params.rust_symbol)
            .collect();

        if mappings.is_empty() {
            return Self::json_success(
                format!("No Ash usage mapping found for {}", params.rust_symbol),
                serde_json::json!({
                    "rust_symbol": params.rust_symbol,
                    "usages": [],
                    "error": "No mapping found for Rust symbol",
                }),
            );
        }

        let mut usages = Vec::new();
        for mapping in mappings {
            usages.extend(Self::find_ash_usages_for_symbol(
                &workspace_root,
                &mapping.ash_symbol,
                &config.ash_extensions,
            ));
        }

        Self::json_success(
            format!(
                "Found {} Ash usage(s) for {}",
                usages.len(),
                params.rust_symbol
            ),
            serde_json::json!({
                "rust_symbol": params.rust_symbol,
                "usages": usages,
            }),
        )
    }

    /// Get hover/type information at a position, enriched with Rust context when available.
    ///
    /// This tool extends the basic `ash_hover` by attempting to find the corresponding
    /// Rust implementation for the Ash symbol under the cursor. If a cross-language
    /// mapping exists, the response includes both Ash type information and Rust
    /// symbol details.
    #[tool(description = "Get enhanced hover with Rust context at a position in an Ash file")]
    fn ash_hover_with_rust_context(
        &self,
        Parameters(params): Parameters<PositionParams>,
    ) -> CallToolResult {
        // 1. Get basic Ash hover info (existing functionality)
        let entry = match self.ensure_open(&params.file) {
            Ok(e) => e,
            Err(e) => return Self::json_error(e),
        };
        let module = match Self::parse_file(&entry) {
            Ok(m) => m,
            Err(e) => return Self::json_error(e),
        };

        let ash_hover_result =
            hover::hover_at(&module, &entry.content, params.line - 1, params.column - 1);

        let ash_markdown = ash_hover_result.as_ref().map(|h| match &h.contents {
            lsp_types::HoverContents::Markup(mc) => mc.value.clone(),
            lsp_types::HoverContents::Scalar(ms) => marked_string_value(ms),
            lsp_types::HoverContents::Array(arr) => arr
                .iter()
                .map(marked_string_value)
                .collect::<Vec<_>>()
                .join("\n\n"),
        });

        // 2. Try to find corresponding Rust symbol
        let rust_context = Self::find_rust_context_for_hover(
            &module,
            &entry.content,
            params.line - 1,
            params.column - 1,
        );

        let summary = if ash_markdown.is_some() || rust_context.is_some() {
            format!(
                "Hover info at {}:{}:{}",
                params.file, params.line, params.column
            )
        } else {
            format!(
                "No hover info at {}:{}:{}",
                params.file, params.line, params.column
            )
        };

        let payload = serde_json::json!({
            "ash_hover": ash_markdown,
            "rust_context": rust_context,
        });

        Self::json_success(summary, payload)
    }

    /// Find Rust context for a symbol at a hover position.
    ///
    /// Extracts the identifier at the given position and looks up its Rust
    /// implementation via the cross-language configuration.
    fn find_rust_context_for_hover(
        _module: &ash_parser::surface::ModuleFile,
        content: &str,
        line: u32,
        column: u32,
    ) -> Option<serde_json::Value> {
        // Extract the word at the cursor position
        let lines: Vec<&str> = content.lines().collect();
        let line_text = lines.get(line as usize)?;
        let col = column as usize;

        // Find word boundaries
        let start = line_text[..col.min(line_text.len())]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map_or(0, |i| i + 1);
        let end = line_text[col.min(line_text.len())..]
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map_or(line_text.len(), |i| col + i);

        let symbol = line_text[start..end].trim();
        if symbol.is_empty() {
            return None;
        }

        // Clean up qualified names (e.g., "Effect::Epistemic" -> "Effect")
        let base_symbol = symbol.split("::").next()?;

        // Load cross-language config and look up mapping
        let config = Self::load_cross_lang_config();
        let mapping = config
            .mappings
            .iter()
            .find(|m| m.ash_symbol == base_symbol)?;

        Some(serde_json::json!({
            "ash_symbol": mapping.ash_symbol,
            "rust_symbol": mapping.rust_symbol,
            "rust_kind": mapping.rust_kind,
            "confidence": format!("{:?}", mapping.confidence).to_lowercase(),
        }))
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

    /// Find references to the symbol at a position (single-file only).
    #[tool(
        description = "Find references to the symbol at a position in an Ash file (single-file only)"
    )]
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

        let entries: Vec<serde_json::Value> = refs
            .iter()
            .map(|r| {
                serde_json::json!({
                    "file": params.file,
                    "start_line": r.range.start.line + 1,
                    "start_column": r.range.start.character + 1,
                    "end_line": r.range.end.line + 1,
                    "end_column": r.range.end.character + 1,
                })
            })
            .collect();

        let summary = if entries.is_empty() {
            format!(
                "No references found at {}:{}:{}",
                params.file, params.line, params.column
            )
        } else {
            format!(
                "{} reference(s) at {}:{}:{}",
                entries.len(),
                params.file,
                params.line,
                params.column
            )
        };

        Self::json_success(summary, serde_json::Value::Array(entries))
    }

    /// Workspace symbol search across `.ash` files under a directory.
    #[tool(description = "Search workspace symbols by name across .ash files under a directory")]
    #[allow(clippy::unused_self)]
    fn ash_workspace_symbols(
        &self,
        Parameters(params): Parameters<WorkspaceSymbolParams>,
    ) -> CallToolResult {
        let root = std::path::Path::new(&params.root);
        if !root.is_dir() {
            return Self::json_error(format!("not a directory: {}", params.root));
        }

        let matches = symbols::workspace_symbols(root, &params.query);

        let entries: Vec<serde_json::Value> = matches
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "kind": format!("{:?}", s.kind).to_lowercase(),
                    "file": s.file,
                    "line": s.line,
                    "column": s.column,
                })
            })
            .collect();

        let summary = if entries.is_empty() {
            format!(
                "No symbols matching '{}' under {}",
                params.query, params.root
            )
        } else {
            format!(
                "{} symbol(s) matching '{}' under {}",
                entries.len(),
                params.query,
                params.root
            )
        };

        Self::json_success(summary, serde_json::Value::Array(entries))
    }

    /// Code actions (placeholder — deferred).
    #[tool(description = "Get code actions for a range (not yet implemented)")]
    #[allow(clippy::unused_self)]
    fn ash_code_action(&self) -> CallToolResult {
        let _ = self;
        Self::json_success(
            "Code actions not yet implemented (deferred to Phase 5)".into(),
            serde_json::Value::Null,
        )
    }

    /// Health check — report server status, version, and available tools.
    #[tool(description = "Check Ash MCP server health and list available tools")]
    #[allow(clippy::unused_self)]
    fn ash_mcp_health(&self) -> CallToolResult {
        let version = env!("CARGO_PKG_VERSION");
        let tools = [
            "ash_get_diagnostics",
            "ash_hover",
            "ash_hover_with_rust_context",
            "ash_find_rust_implementation",
            "ash_find_ash_usage",
            "ash_goto_definition",
            "ash_complete",
            "ash_document_symbols",
            "ash_find_references",
            "ash_workspace_symbols",
            "ash_code_action",
            "ash_mcp_health",
        ];
        let payload = serde_json::json!({
            "status": "ok",
            "version": version,
            "tools": tools,
        });
        Self::json_success(
            format!("status: ok, version: {version}, tools: {}", tools.len()),
            payload,
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

impl AshMcpServer {
    fn workspace_root() -> std::path::PathBuf {
        std::env::current_dir().unwrap_or_else(|_| Self::manifest_workspace_root())
    }

    fn manifest_workspace_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .map_or_else(
                || std::path::PathBuf::from("."),
                std::path::Path::to_path_buf,
            )
    }

    fn workspace_root_for_file(file: &str) -> std::path::PathBuf {
        let file_path = std::path::Path::new(file);
        let absolute_file = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            Self::workspace_root().join(file_path)
        };
        let start_root = absolute_file
            .parent()
            .map_or_else(Self::workspace_root, std::path::Path::to_path_buf);
        let mut current = start_root.clone();

        let mut cargo_root = None;
        loop {
            if current.join(".ash/cross_lang_config.yaml").exists()
                || current.join("cross_lang_config.yaml").exists()
            {
                return current;
            }
            if cargo_root.is_none() && current.join("Cargo.toml").exists() {
                cargo_root = Some(current.clone());
            }
            if !current.pop() {
                return cargo_root.unwrap_or(start_root);
            }
        }
    }

    const fn normalize_ash_symbol(symbol: &str) -> &str {
        symbol
    }

    fn find_ash_mapping<'a>(
        mappings: &'a [cross_lang::SymbolMapping],
        query: &str,
    ) -> Option<&'a cross_lang::SymbolMapping> {
        let parts: Vec<_> = query.split("::").collect();
        let parent_symbol = (parts.len() >= 2).then(|| parts[..parts.len() - 1].join("::"));

        mappings
            .iter()
            .find(|mapping| mapping.ash_symbol == query)
            .or_else(|| {
                parent_symbol.as_ref().and_then(|parent| {
                    mappings
                        .iter()
                        .find(|mapping| mapping.ash_symbol == parent.as_str())
                })
            })
            .or_else(|| {
                let last = parts.last().copied();
                let penultimate = parts
                    .len()
                    .checked_sub(2)
                    .and_then(|index| parts.get(index).copied());

                mappings.iter().find(|mapping| {
                    last == Some(mapping.ash_symbol.as_str())
                        || penultimate == Some(mapping.ash_symbol.as_str())
                })
            })
    }

    fn find_rust_symbol_location(
        workspace_root: &std::path::Path,
        rust_symbol: &str,
    ) -> Result<Option<rust_parser::RustSymbolLocation>, rust_parser::RustParseError> {
        let parts: Vec<_> = rust_symbol.split("::").collect();
        let Some(file_path) = rust_parser::find_rust_file_for_symbol(workspace_root, rust_symbol)?
        else {
            return Ok(None);
        };

        let symbol_name = if Self::should_search_associated_item(&file_path, &parts) {
            format!("{}::{}", parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            parts.last().copied().unwrap_or(rust_symbol).to_string()
        };

        rust_parser::find_symbol_location(&file_path, &symbol_name)
    }

    fn should_search_associated_item(file_path: &std::path::Path, parts: &[&str]) -> bool {
        if parts.len() < 4 {
            return false;
        }
        let module_path = parts[1..parts.len() - 1].join("/");
        let full_module_file = std::path::PathBuf::from(format!("src/{module_path}.rs"));
        let full_module_mod = std::path::PathBuf::from(format!("src/{module_path}/mod.rs"));
        !file_path.ends_with(&full_module_file) && !file_path.ends_with(&full_module_mod)
    }

    fn find_ash_usages_for_symbol(
        workspace_root: &std::path::Path,
        ash_symbol: &str,
        ash_extensions: &[String],
    ) -> Vec<AshUsageInfo> {
        let mut usages = Vec::new();
        Self::scan_ash_usages_with_extensions(
            workspace_root,
            ash_symbol,
            ash_extensions,
            &mut usages,
        );
        usages
    }

    #[cfg(test)]
    fn scan_ash_usages(path: &std::path::Path, ash_symbol: &str, usages: &mut Vec<AshUsageInfo>) {
        let default_extensions = [String::from(".ash")];
        Self::scan_ash_usages_with_extensions(path, ash_symbol, &default_extensions, usages);
    }

    fn scan_ash_usages_with_extensions(
        path: &std::path::Path,
        ash_symbol: &str,
        ash_extensions: &[String],
        usages: &mut Vec<AshUsageInfo>,
    ) {
        let Ok(metadata) = std::fs::metadata(path) else {
            return;
        };

        if metadata.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return;
            };
            if matches!(name, ".git" | "target" | ".worktrees") {
                return;
            }
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                Self::scan_ash_usages_with_extensions(
                    &entry.path(),
                    ash_symbol,
                    ash_extensions,
                    usages,
                );
            }
            return;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return;
        };
        if !ash_extensions
            .iter()
            .any(|extension| file_name.ends_with(extension))
        {
            return;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };
        let mut in_block_comment = false;
        for (line_idx, line) in content.lines().enumerate() {
            let searchable = Self::mask_ash_non_code(line, &mut in_block_comment);
            for (column_idx, _) in searchable.match_indices(ash_symbol) {
                if !Self::is_identifier_match(&searchable, column_idx, ash_symbol.len()) {
                    continue;
                }
                usages.push(AshUsageInfo {
                    file: path.display().to_string(),
                    line: u32::try_from(line_idx.saturating_add(1)).unwrap_or(u32::MAX),
                    column: u32::try_from(column_idx.saturating_add(1)).unwrap_or(u32::MAX),
                    ash_symbol: ash_symbol.to_string(),
                });
            }
        }
    }

    fn mask_ash_non_code(line: &str, in_block_comment: &mut bool) -> String {
        let mut masked = String::with_capacity(line.len());
        let mut chars = line.chars().peekable();
        let mut in_string = false;
        while let Some(ch) = chars.next() {
            if *in_block_comment {
                masked.push(' ');
                if ch == '*' && chars.peek() == Some(&'/') {
                    masked.push(' ');
                    chars.next();
                    *in_block_comment = false;
                }
                continue;
            }

            if !in_string
                && ((ch == '/' && chars.peek() == Some(&'/'))
                    || (ch == '-' && chars.peek() == Some(&'-')))
            {
                masked.push(' ');
                masked.push(' ');
                chars.next();
                masked.extend(chars.map(|_| ' '));
                break;
            }

            if !in_string && ch == '/' && chars.peek() == Some(&'*') {
                masked.push(' ');
                masked.push(' ');
                chars.next();
                *in_block_comment = true;
                continue;
            }

            if in_string && ch == '\\' {
                masked.push(' ');
                if chars.next().is_some() {
                    masked.push(' ');
                }
            } else if ch == '"' {
                in_string = !in_string;
                masked.push(' ');
            } else if in_string {
                masked.push(' ');
            } else {
                masked.push(ch);
            }
        }
        masked
    }

    fn is_identifier_match(line: &str, start: usize, len: usize) -> bool {
        let before = line[..start].chars().next_back();
        let after = line[start.saturating_add(len)..].chars().next();
        !before.is_some_and(Self::is_identifier_char)
            && !after.is_some_and(Self::is_identifier_char)
    }

    const fn is_identifier_char(ch: char) -> bool {
        ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
    }
}

// ---------------------------------------------------------------------------
// Test-only public wrappers for integration tests
// ---------------------------------------------------------------------------

#[doc(hidden)]
impl AshMcpServer {
    /// Public wrapper for `ash_workspace_symbols` used by integration tests.
    #[must_use]
    pub fn workspace_symbols(&self, root: String, query: String) -> CallToolResult {
        self.ash_workspace_symbols(rmcp::handler::server::wrapper::Parameters(
            WorkspaceSymbolParams { root, query },
        ))
    }

    /// Public wrapper for `ash_find_references` used by integration tests.
    #[must_use]
    pub fn find_references(&self, file: String, line: u32, column: u32) -> CallToolResult {
        self.ash_find_references(rmcp::handler::server::wrapper::Parameters(PositionParams {
            file,
            line,
            column,
        }))
    }

    /// Public wrapper for `ash_goto_definition` used by integration tests.
    #[must_use]
    pub fn goto_definition(&self, file: String, line: u32, column: u32) -> CallToolResult {
        self.ash_goto_definition(rmcp::handler::server::wrapper::Parameters(PositionParams {
            file,
            line,
            column,
        }))
    }

    /// Public wrapper for `ash_find_rust_implementation` used by tests.
    #[must_use]
    pub fn find_rust_implementation_tool(
        &self,
        ash_symbol: String,
        file: String,
        line: u32,
        column: u32,
    ) -> CallToolResult {
        self.ash_find_rust_implementation(rmcp::handler::server::wrapper::Parameters(
            SymbolLookupParams {
                ash_symbol,
                file,
                line,
                column,
            },
        ))
    }

    /// Public wrapper for `ash_find_ash_usage` used by tests.
    #[must_use]
    pub fn find_ash_usage_tool(&self, rust_symbol: String) -> CallToolResult {
        self.ash_find_ash_usage(rmcp::handler::server::wrapper::Parameters(
            RustUsageParams { rust_symbol },
        ))
    }

    /// Load cross-language configuration from common locations.
    ///
    /// Searches for `cross_lang_config.yaml` in the current directory,
    /// `.ash/`, and `~/.ash/`.
    #[must_use]
    pub fn load_cross_lang_config() -> cross_lang::CrossLangConfig {
        Self::load_cross_lang_config_for_root(&Self::workspace_root())
    }

    fn cross_lang_config_root_for_root(root: &std::path::Path) -> std::path::PathBuf {
        let mut current = root.to_path_buf();
        loop {
            if current.join("cross_lang_config.yaml").exists()
                || current.join(".ash/cross_lang_config.yaml").exists()
            {
                return current;
            }
            if !current.pop() {
                break;
            }
        }

        root.to_path_buf()
    }

    fn load_cross_lang_config_for_root(root: &std::path::Path) -> cross_lang::CrossLangConfig {
        let config_root = Self::cross_lang_config_root_for_root(root);
        let config_paths = [
            config_root.join("cross_lang_config.yaml"),
            config_root.join(".ash/cross_lang_config.yaml"),
        ];

        for path in &config_paths {
            if let Ok(config) = cross_lang::CrossLangConfig::from_file(path) {
                return config;
            }
        }
        // Return default empty config if none found
        cross_lang::CrossLangConfig {
            version: 1,
            rust_crates: vec![],
            ash_extensions: vec![".ash".to_string()],
            mappings: vec![],
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
