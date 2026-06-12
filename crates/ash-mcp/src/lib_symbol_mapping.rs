/// Ash symbol lookup parameters for cross-language Rust implementation finding.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SymbolLookupParams {
    /// Name of the Ash symbol to find
    pub ash_symbol: String,
    /// Path to the Ash file containing the symbol
    pub file: String,
    /// 1-indexed line number where symbol appears
    pub line: u32,
    /// 1-indexed column number where symbol appears
    pub column: u32,
}

/// Rust symbol information returned by cross-language lookup.
#[derive(Debug, Serialize)]
pub struct RustSymbolInfo {
    /// Whether the symbol was found
    pub found: bool,
    /// Rust symbol name (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_symbol: Option<String>,
    /// Rust symbol kind (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_kind: Option<String>,
    /// Rust file path (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Start line in Rust file (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// Start column in Rust file (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    /// End line in Rust file (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// End column in Rust file (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
    /// Confidence level of mapping (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    /// Source of mapping (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Error message (if not found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

use crate::cross_lang::{CrossLangConfig, SymbolMapping};
use crate::rust_parser::{find_rust_file_for_symbol, find_symbol_location};

impl crate::AshMcpServer {
    /// Find Rust implementation for an Ash symbol
    pub fn find_rust_implementation(
        &self,
        params: &SymbolLookupParams,
    ) -> Result<Option<RustSymbolInfo>, Box<dyn std::error::Error>> {
        // Load cross-language configuration
        let config = self.load_cross_lang_config()?;

        // Look up symbol in mappings
        if let Some(mapping) = config.mappings.iter()
            .find(|m| m.ash_symbol == params.ash_symbol) {

            // Find the Rust file location using real parsing
            if let Some(location) = self.find_rust_symbol_location_real(&mapping.rust_symbol)? {
                Ok(Some(RustSymbolInfo {
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
                }))
            } else {
                Ok(Some(RustSymbolInfo {
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
                }))
            }
        } else {
            Ok(None)
        }
    }

    /// Load cross-language configuration
    fn load_cross_lang_config(&self) -> Result<CrossLangConfig, Box<dyn std::error::Error>> {
        use std::path::Path;

        // Try to load from common locations
        let config_paths = vec![
            "cross_lang_config.yaml",
            ".ash/cross_lang_config.yaml",
            "~/.ash/cross_lang_config.yaml",
        ];

        for path in config_paths {
            if let Ok(config) = CrossLangConfig::from_file(Path::new(path)) {
                return Ok(config);
            }
        }

        // Return default config if none found
        Ok(CrossLangConfig {
            version: 1,
            rust_crates: vec![],
            ash_extensions: vec![".ash".to_string()],
            mappings: vec![],
        })
    }

    /// Find Rust symbol location in source files (real implementation with syn parsing)
    fn find_rust_symbol_location_real(
        &self,
        rust_symbol: &str,
    ) -> Result<Option<crate::rust_parser::RustSymbolLocation>, Box<dyn std::error::Error>> {
        use std::path::PathBuf;

        // Get workspace root from current file
        let workspace_root = std::env::current_dir()
            .or_else(|_| PathBuf::from(".").canonicalize())?;

        // Find the Rust source file
        let base_name = rust_symbol.split("::").last().unwrap_or(rust_symbol);

        // Try to find the file first
        let rust_file = find_rust_file_for_symbol(&workspace_root, rust_symbol);

        if let Some(file_path) = rust_file? {
            // Parse the file and find the symbol location
            if let Some(mut location) = find_symbol_location(&file_path, base_name)? {
                location.file = file_path;
                return Ok(Some(location));
            }
        }

        // Fallback: try to parse directly from the qualified symbol path
        let parts: Vec<&str> = rust_symbol.split("::").collect();
        if parts.len() >= 3 {
            let crate_name = parts[0].replace('_', "-");
            let module_path = parts[1..parts.len() - 1].join("/");
            let symbol_name = parts[parts.len() - 1];

            // Try to find and parse the file
            let possible_files = vec![
                format!("crates/{}/src/{}.rs", crate_name, module_path),
                format!("crates/{}/src/{}/mod.rs", crate_name, module_path),
                format!("crates/{}/src/lib.rs", crate_name),
            ];

            for possible_file in possible_files {
                let full_path = workspace_root.join(&possible_file);
                if full_path.exists() {
                    if let Some(mut location) = find_symbol_location(&full_path, symbol_name)? {
                        location.file = full_path;
                        return Ok(Some(location));
                    }
                }
            }
        }

        Ok(None)
    }

    /// MCP tool for finding Rust implementation of Ash symbols
    #[tool(description = "Find the Rust implementation corresponding to an Ash symbol")]
    fn ash_find_rust_implementation(
        &self,
        Parameters(params): Parameters<SymbolLookupParams>,
    ) -> CallToolResult {
        let result = self.find_rust_implementation(&params);

        match result {
            Ok(Some(rust_info)) => {
                let summary = format!(
                    "Found Rust implementation for {}",
                    params.ash_symbol
                );
                let payload = serde_json::json!(rust_info);
                Self::json_success(summary, payload)
            }
            Ok(None) => {
                let summary = format!(
                    "No Rust implementation found for {}",
                    params.ash_symbol
                );
                let payload = serde_json::json!({
                    "found": false,
                    "error": format!("No mapping found for Ash symbol '{}'", params.ash_symbol)
                });
                Self::json_success(summary, payload)
            }
            Err(e) => {
                Self::json_error(format!("Lookup failed: {}", e))
            }
        }
    }
}