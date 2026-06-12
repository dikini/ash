# TASK-1424: Enhanced Hover with Rust Context

**Estimate:** 3 hours  
**Status:** 📋 Planned  
**Phase:** 142  

## Description

Extend the existing `ash_hover` MCP tool to include Rust context when available. Shows both Ash type information and the corresponding Rust implementation details in a unified tooltip.

## Acceptance Criteria

✅ **All tests pass** - Hover tests with Rust context  
✅ **Property tests extensive** - Various symbol types and locations  
✅ **Code review** - Clean hover composition logic  
✅ **Rust tooling** - Clean fmt, clippy, docs  
✅ **Documentation** - Enhanced hover examples  

## Specifications

### Enhanced Hover Response

```rust
#[derive(Debug, Serialize)]
pub struct HoverInfo {
    pub ash_info: Option<AshTypeInfo>,
    pub rust_info: Option<RustTypeInfo>,
    pub relationship: SymbolRelationship,
}

#[derive(Debug, Serialize)]
pub struct AshTypeInfo {
    pub name: String,
    pub kind: String,
    pub type_signature: String,
    pub documentation: String,
}

#[derive(Debug, Serialize)]
pub struct RustTypeInfo {
    pub name: String,
    pub kind: String,
    pub module_path: Vec<String>,
    pub definition: String,
    pub documentation: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum SymbolRelationship {
    Implements,   // Ash symbol implements Rust symbol
    Uses,         // Ash symbol uses Rust symbol
    Equivalent,   // Symbols are equivalent across languages
    None,         // No clear relationship
}
```

### MCP Tool Enhancement

```rust
/// Enhanced hover that shows both Ash and Rust context when available
pub async fn ash_hover_with_rust_context(
    file: String,
    line: u32,
    column: u32,
) -> Result<HoverInfo> {
    // 1. Get Ash hover info (existing functionality)
    let ash_info = ash_hover(&file, line, column).await?;
    
    // 2. Try to find corresponding Rust symbol
    let rust_info = if let Some(ash_symbol) = ash_info.as_ref() {
        find_rust_context(&ash_symbol.name, &file).await?
    } else {
        None
    };
    
    // 3. Determine relationship
    let relationship = determine_relationship(&ash_info, &rust_info);
    
    Ok(HoverInfo {
        ash_info,
        rust_info,
        relationship,
    })
}
```

### Example Output

```json
{
  "ash_info": {
    "name": "Effect",
    "kind": "type",
    "type_signature": "Effect: Type",
    "documentation": "The Effect lattice defines epistemic, deliberative, evaluative, and operational effect types."
  },
  "rust_info": {
    "name": "ash_core::effect::Effect",
    "kind": "enum",
    "module_path": ["ash_core", "effect"],
    "definition": "pub enum Effect { Epistemic, Deliberative, Evaluative, Operational }",
    "documentation": "Core effect types for Ash runtime system."
  },
  "relationship": "Equivalent"
}
```

## Implementation Steps

1. **Enhanced hover structure** (45m)
   - Define HoverInfo and related types
   - Add JSON serialization
   - Design for future extensibility

2. **Rust context lookup** (1h)
   - Reuse cross-language mapping from TASK-1421
   - Get Rust symbol details from source
   - Handle missing Rust info gracefully

3. **Relationship detection** (45m)
   - Implement heuristics for symbol relationships
   - Use module paths and naming conventions
   - Provide clear relationship explanations

4. **Integration and testing** (30m)
   - Integrate with existing ash_hover implementation
   - Add unit tests for various symbol combinations
   - Test with real Ash/Rust files

## Testing Strategy

```rust
#[tokio::test]
async fn test_effect_hover_with_rust() {
    let hover_info = ash_hover_with_rust_context(
        "std/src/types.ash".to_string(),
        10, 1,  // Effect definition
    ).await.unwrap();
    
    assert!(hover_info.ash_info.is_some());
    assert!(hover_info.rust_info.is_some());
    assert_eq!(hover_info.relationship, SymbolRelationship::Equivalent);
    
    let rust_info = hover_info.rust_info.unwrap();
    assert_eq!(rust_info.name, "ash_core::effect::Effect");
}

#[tokio::test]
async fn test_graceful_degradation() {
    // Test with symbol that has no Rust mapping
    let hover_info = ash_hover_with_rust_context(
        "unknown.ash".to_string(),
        1, 1,
    ).await.unwrap();
    
    assert!(hover_info.rust_info.is_none());
    assert_eq!(hover_info.relationship, SymbolRelationship::None);
}
```

## Verification

- Enhanced hover works for all existing Ash symbols
- Rust context appears when mappings are available
- Graceful degradation when Rust info missing
- Performance impact is minimal (<5ms additional)

## Dependencies

- TASK-1421: Ash → Rust symbol mapping
- TASK-1423: Latency optimization (for cached Rust info)

---

**Task Authority:** [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)  
**Verification Baseline**: Will be updated with git commit when task starts