# TASK-1421: Ash → Rust Symbol Mapping Tools

**Estimate:** 6 hours  
**Status:** 📋 Planned  
**Phase:** 142  

## Description

Implement the `ash_find_rust_implementation` MCP tool that maps Ash symbols to their corresponding Rust implementations. This is the primary navigation tool for finding Rust definitions from Ash code.

## Acceptance Criteria

✅ **All tests pass** - Unit tests, integration tests, property tests  
✅ **Property tests extensive** - Proptest for symbol resolution edge cases  
✅ **Code review** - Simplify logic, check for performance issues  
✅ **Rust tooling** - `cargo fmt`, `cargo clippy`, `cargo doc` clean  
✅ **Documentation** - Tool documentation with examples  

## Specifications

### MCP Tool Interface

```rust
/// Find the Rust implementation corresponding to an Ash symbol
/// 
/// Input: Ash symbol with file location
/// Output: Rust symbol location or error
/// 
/// Example:
///   ash_symbol = "Effect::Epistemic" 
///   → returns Rust location in "crates/ash-core/src/effect.rs:10:9"
pub async fn ash_find_rust_implementation(
    ash_symbol: String,
    file_path: String,
    line: u32,
    column: u32,
) -> Result<Option<RustLocation>> {
    // Implementation
}
```

### Rust Data Structures

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RustLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub symbol: String,
    pub kind: String,
    pub confidence: ConfidenceLevel,
}

#[derive(Debug)]
pub struct AshSymbolRequest {
    pub name: String,
    pub file_path: PathBuf,
    pub position: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
```

### Implementation Algorithm

```rust
impl CrossLanguageMapper {
    pub async fn find_rust_implementation(
        &self,
        request: AshSymbolRequest,
    ) -> Result<Option<RustLocation>> {
        // 1. Resolve Ash symbol from file location
        let ash_symbol = self.resolve_ash_symbol(request).await?;
            
        // 2. Look up in cross-language mappings
        if let Some(mapping) = self.config.mappings.iter()
            .find(|m| m.ash_symbol == ash_symbol.name) {
            
            // 3. Verify Rust symbol exists and get location
            if let Some(rust_loc) = self.find_rust_symbol_location(&mapping.rust_symbol).await? {
                return Ok(Some(RustLocation {
                    file: rust_loc.file,
                    line: rust_loc.line,
                    column: rust_loc.column,
                    symbol: mapping.rust_symbol.clone(),
                    kind: mapping.rust_kind.clone(),
                    confidence: mapping.confidence,
                }));
            }
        }
        
        // 4. Graceful degradation - return None if no mapping found
        Ok(None)
    }
}
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| First lookup latency | <100ms | Benchmark with cold cache |
| Cached lookup latency | <10ms | Benchmark with warm cache |
| Memory usage | <10MB | Memory profiling |
| Success rate | >80% | Test corpus coverage |

## Implementation Steps

1. **Ash symbol resolution** (1h)
   - Parse Ash file at given location
   - Extract symbol name and context
   - Handle qualified names (e.g., `Effect::Epistemic`)

2. **Cross-language lookup** (1.5h)
   - Query configuration mappings
   - Match Ash symbol to Rust symbol
   - Return confidence level

3. **Rust symbol location** (1.5h)
   - Parse Rust files from configured crates
   - Find exact line/column of Rust symbols
   - Cache results for performance

4. **Error handling** (1h)
   - Graceful degradation when mappings missing
   - Helpful error messages for debugging
   - Log failed lookups for monitoring

5. **Testing** (1h)
   - Unit tests for symbol resolution
   - Integration tests with real Ash/Rust files
   - Performance benchmarks

## Testing Strategy

```rust
#[tokio::test]
async fn test_effect_mapping() {
    let mapper = CrossLanguageMapper::from_test_config();
    let request = AshSymbolRequest {
        name: "Effect".to_string(),
        file_path: "std/src/types.ash".into(),
        position: Position { line: 10, column: 1 },
    };
    
    let result = mapper.find_rust_implementation(request).await.unwrap();
    assert!(result.is_some());
    
    let rust_loc = result.unwrap();
    assert_eq!(rust_loc.file, "crates/ash-core/src/effect.rs");
    assert_eq!(rust_loc.symbol, "ash_core::effect::Effect");
}

#[proptest]
async fn test_graceful_degeneration(
    symbol_name: String,
) {
    // Test that unknown symbols return None gracefully
    let mapper = CrossLanguageMapper::from_test_config();
    let request = AshSymbolRequest {
        name: symbol_name,
        file_path: "nonexistent.ash".into(),
        position: Position { line: 1, column: 1 },
    };
    
    let result = mapper.find_rust_implementation(request).await;
    assert!(result.is_ok()); // Should not error, just return None
}
```

## Verification

- All tests pass including edge cases
- Performance benchmarks meet targets
- Error handling provides helpful debugging info
- Memory usage remains within limits

## Dependencies

- TASK-1420: Cross-language configuration schema

## Notes

- This is the primary tool for Ash → Rust navigation
- Focus on accuracy for high-confidence mappings first
- Consider adding fuzzy matching for low-confidence mappings
- Performance is critical - this will be called frequently by agents

---

**Task Authority**: [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)  
**Verification Baseline**: Will be updated with git commit when task starts