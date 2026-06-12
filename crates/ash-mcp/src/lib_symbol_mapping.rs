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

/// Symbol location information for Rust source files.
#[derive(Debug, Clone)]
struct SymbolLocation {
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl AshMcpServer {
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
            
            // Find the Rust file location
            if let Some(location) = self.find_rust_symbol_location(&mapping.rust_symbol)? {
                Ok(Some(RustSymbolInfo {
                    found: true,
                    rust_symbol: Some(mapping.rust_symbol.clone()),
                    rust_kind: Some(mapping.rust_kind.clone()),
                    file: Some(location.file),
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

    /// Find Rust symbol location in source files
    fn find_rust_symbol_location(
        &self,
        rust_symbol: &str,
    ) -> Result<Option<SymbolLocation>, Box<dyn std::error::Error>> {
        // For now, return a placeholder
        // In a real implementation, this would:
        // 1. Parse the Rust symbol into crate::module::Symbol
        // 2. Find the corresponding source file
        // 3. Use syn or similar to find the symbol location
        // 4. Return the exact line and column range
        
        let symbol_parts: Vec<&str> = rust_symbol.split("::").collect();
        if symbol_parts.len() >= 3 {
            // ash_core::effect::Effect -> ash-core/src/effect.rs
            let crate_name = symbol_parts[0];
            let module_name = symbol_parts[1];
            let symbol_name = symbol_parts[2];
            
            // For now, return a placeholder location
            // This would need real Rust source parsing in a complete implementation
            return Ok(Some(SymbolLocation {
                file: format!("target/debug/deps/lib{}.rs", crate_name.replace("_", "-")),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 2,
            }));
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