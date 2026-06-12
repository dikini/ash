# TASK-1421: Ash → Rust Symbol Mapping Tools

**Estimate:** 6 hours  
**Status:** 📋 Planned  
**Phase:** 142  

## Description

Implement the `ash_find_rust_implementation` MCP tool that allows users to find the Rust implementation corresponding to an Ash symbol. The tool should use the cross-language configuration from TASK-1420 to provide fast, accurate symbol mapping from Ash to Rust.

## Acceptance Criteria

✅ **All tests pass** - Unit tests for symbol lookup and error handling  
✅ **Property tests extensive** - Using proptest for various symbol patterns  
✅ **Code review** - Self-review for performance and correctness  
✅ **Rust tooling** - `cargo fmt`, `cargo clippy`, `cargo doc` all pass  
✅ **Documentation** - Tool documentation with examples  

## Specifications

### MCP Tool Definition

```yaml
name: ash_find_rust_implementation
description: Find the Rust implementation corresponding to an Ash symbol
input_schema:
  type: object
  properties:
    ash_symbol:
      type: string
      description: Name of the Ash symbol to find
    file:
      type: string
      description: Path to the Ash file containing the symbol
    line:
      type: integer
      description: 1-indexed line number where symbol appears
    column:
      type: integer
      description: 1-indexed column number where symbol appears
  required: [ash_symbol, file, line, column]
```

### Response Format

```json
{
  "summary": "Found Rust implementation for Effect",
  "result": {
    "found": true,
    "rust_symbol": "ash_core::effect::Effect",
    "rust_kind": "enum",
    "file": "/path/to/ash-core/src/effect.rs",
    "start_line": 25,
    "start_column": 1,
    "end_line": 45,
    "end_column": 2,
    "confidence": "high",
    "source": "manual"
  }
}
```

### Error Response

```json
{
  "summary": "No Rust implementation found for UnknownSymbol",
  "result": {
    "found": false,
    "error": "No mapping found for Ash symbol 'UnknownSymbol'"
  }
}
```

### Rust Implementation

```rust
#[tool]
pub fn ash_find_rust_implementation(
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
            Self::json_success(summary, serde_json::json!(rust_info))
        }
        Ok(None) => {
            let summary = format!(
                "No Rust implementation found for {}",
                params.ash_symbol
            );
            Self::json_success(summary, serde_json::json!({
                "found": false,
                "error": format!("No mapping found for Ash symbol '{}'", params.ash_symbol)
            }))
        }
        Err(e) => Self::json_error(format!("Lookup failed: {}", e)),
    }
}
```

### Symbol Lookup Logic

```rust
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
            
            // Convert crate name to file name
            let crate_file = crate_name.replace('_', "-");
            let rust_file = format!("target/debug/deps/lib{}.rs", crate_file);
            
            if Path::new(&rust_file).exists() {
                // Placeholder - would parse the actual file
                return Ok(Some(SymbolLocation {
                    file: rust_file,
                    start_line: 1,
                    start_column: 1,
                    end_line: 10,
                    end_column: 2,
                }));
            }
        }
        
        Ok(None)
    }
}

#[derive(Debug)]
struct SymbolLocation {
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}
```

## Implementation Steps

1. **Define tool parameters** (45m)
   - Create SymbolLookupParams struct
   - Add MCP tool registration
   - Define response schema

2. **Implement core lookup logic** (2h)
   - Add find_rust_implementation method
   - Integrate with CrossLangConfig from TASK-1420
   - Handle symbol matching

3. **Add Rust file location finding** (1h 30m)
   - Implement find_rust_symbol_location
   - Parse Rust symbol paths
   - Return actual file locations (placeholder for now)

4. **Tests and documentation** (45m)
   - Unit tests for happy path
   - Error handling tests
   - Integration tests with configuration

## Testing Strategy

```rust
#[tokio::test]
async fn test_find_effect_implementation() {
    let server = AshMcpServer::new();
    let params = SymbolLookupParams {
        ash_symbol: "Effect".to_string(),
        file: "std/src/types.ash".to_string(),
        line: 10,
        column: 1,
    };
    
    let result = server.find_rust_implementation(&params).unwrap();
    assert!(result.is_some());
    
    let rust_info = result.unwrap();
    assert_eq!(rust_info.rust_symbol, Some("ash_core::effect::Effect".to_string()));
    assert_eq!(rust_info.found, true);
}

#[test]
fn test_unknown_symbol_returns_none() {
    let server = AshMcpServer::new();
    let params = SymbolLookupParams {
        ash_symbol: "UnknownSymbol".to_string(),
        file: "test.ash".to_string(),
        line: 1,
        column: 1,
    };
    
    let result = server.find_rust_implementation(&params).unwrap();
    assert!(result.is_none());
}

#[proptest]
fn test_symbol_lookup_properties(
    ash_symbol in r"[a-zA-Z_][a-zA-Z0-9_]*",
    line in 1u32..1000u32,
    column in 1u32..100u32,
) {
    let server = AshMcpServer::new();
    let params = SymbolLookupParams {
        ash_symbol,
        file: "test.ash".to_string(),
        line,
        column,
    };
    
    // Should not panic
    let _result = server.find_rust_implementation(&params);
}
```

## Verification

- MCP tool correctly registers and responds to requests
- Known symbols return correct Rust implementation info
- Unknown symbols return graceful error responses
- Configuration loading works with default fallback
- Integration with CrossLangConfig from TASK-1420

## Dependencies

- TASK-1420: Cross-Language Configuration Schema

## Notes

- Rust file location finding is initially a placeholder
- Future phases could add actual Rust source parsing
- Consider performance optimizations for frequent lookups
- Error handling should be robust for missing configurations

---

**Task Authority**: [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)  
**Verification Baseline**: Will be updated with git commit when task starts