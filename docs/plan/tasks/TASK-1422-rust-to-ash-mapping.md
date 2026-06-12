# TASK-1422: Rust → Ash Usage Finder

**Estimate:** 4 hours  
**Status:** 📋 Planned  
**Phase:** 142  

## Description

Implement the `ash_find_ash_usage` MCP tool that finds all Ash code that uses a given Rust symbol. This enables "find all references" functionality from Rust definitions back to Ash usage.

## Acceptance Criteria

✅ **All tests pass** - Unit tests, integration tests  
✅ **Property tests extensive** - Test with various Rust symbol patterns  
✅ **Code review** - Check for performance and clarity  
✅ **Rust tooling** - Clean fmt, clippy, docs  
✅ **Documentation** - Tool documentation with examples  

## Specifications

### MCP Tool Interface

```rust
/// Find all Ash usages of a Rust symbol
///
/// Input: Qualified Rust symbol name
/// Output: List of Ash locations using this symbol
///
/// Example:
///   rust_symbol = "ash_core::effect::Effect"
///   → returns all Ash files that import/use this Effect type
pub async fn ash_find_ash_usage(
    rust_symbol: String,
) -> Result<Vec<AshLocation>> {
    // Implementation
}
```

### Data Structures

```rust
#[derive(Debug, Clone, Serialize)]
pub struct AshLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub usage_kind: UsageKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum UsageKind {
    Import,      // `use Effect;`
    TypeAnnotation, // `x: Effect`
    FunctionCall,   // `Effect::new()`
    Reference,      // `Effect::Epistemic`
}
```

### Implementation Algorithm

```rust
impl CrossLanguageMapper {
    pub async fn find_ash_usage(&self, rust_symbol: String) -> Result<Vec<AshLocation>> {
        let mut usages = Vec::new();
        
        // 1. Find all Ash files that import this Rust symbol
        let ash_files = self.find_ash_files_with_import(&rust_symbol).await?;
        
        // 2. Scan each file for actual usages
        for file in ash_files {
            if let Ok(content) = fs::read_to_string(&file) {
                let file_usages = self.scan_file_for_usages(&content, &rust_symbol, &file);
                usages.extend(file_usages);
            }
        }
        
        Ok(usages)
    }
}
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Scan 100 files latency | <200ms | Benchmark with corpus |
| Memory per file | <100KB | Memory profiling |
| False positive rate | <5% | Manual verification |

## Implementation Steps

1. **Rust symbol parsing** (1h)
   - Parse qualified Rust symbols (e.g., `ash_core::effect::Effect`)
   - Handle different symbol formats
   - Normalize symbol names

2. **File discovery** (1h)
   - Find all Ash files in workspace
   - Filter by files that import target Rust symbols
   - Optimize with file system caching

3. **Usage scanning** (1h)
   - Parse Ash files to find symbol usages
   - Categorize usage types (import, type annotation, etc.)
   - Extract precise locations

4. **Testing** (1h)
   - Unit tests for symbol parsing
   - Integration tests with real Ash files
   - Performance testing with large corpora

## Verification

- Accurately finds all usages in test corpus
- Performance meets latency targets
- Minimal false positives/negatives
- Clean error handling

## Dependencies

- TASK-1420: Cross-language configuration schema
- TASK-1421: Ash → Rust symbol mapping (shared infrastructure)

---

**Task Authority**: [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)  
**Verification Baseline**: Will be updated with git commit when task starts