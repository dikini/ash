# TASK-234: Implement Capability Definition Parser

## Status: Blocked on TASK-233

## Description

Implement parser support for capability definitions in `.ash` source files, replacing the current stub in `parse_module.rs`.

## Background

After TASK-233 completes, SPEC-017 will contain the full capability definition syntax. This task implements the parser to recognize and parse that syntax into the Surface AST.

## Requirements

### 1. Surface AST Extension

Extend `ash-parser/src/surface.rs`:

```rust
pub struct CapabilityDef {
    pub name: Name,
    pub effect: EffectType,
    pub params: Vec<Param>,
    pub returns: Option<TypeExpr>,
    pub constraints: Vec<Expr>,
    pub span: Span,
}

pub enum EffectType {
    Observe,
    Read,
    Analyze,
    Decide,
    Act,
    Write,
    External,
    // Direct Effect lattice variants
    Epistemic,
    Deliberative,
    Evaluative,
    Operational,
}
```

### 2. Parser Implementation

Implement in `ash-parser/src/parse_module.rs`:

```rust
pub fn capability_def(input: &mut Input) -> PResult<CapabilityDef> {
    // Parse: capability name : effect (params) returns type where constraints
}
```

### 3. Lowering to Core AST

Extend `ash-parser/src/lower.rs`:

```rust
// Surface CapabilityDef -> Core Capability
fn lower_capability_def(def: &CapabilityDef) -> ash_core::Capability {
    // Map effect types
    // Lower parameters
    // Lower constraints
}
```

### 4. Integration with Module System

- Add `CapabilityDef` to `Module` struct
- Populate capability registry from parsed definitions
- Support `pub capability` for visibility

## Test Requirements

- [ ] Parse minimal capability: `capability read_temp : observe ()`
- [ ] Parse capability with params: `capability read_file : read (path: String)`
- [ ] Parse capability with returns: `capability get_temp : observe () returns Int`
- [ ] Parse capability with constraints: `capability transfer : act (amount: Int) where amount > 0`
- [ ] Parse all effect types (observe, read, analyze, decide, act, write, external)
- [ ] Parse effect lattice variants (epistemic, deliberative, evaluative, operational)
- [ ] Error on invalid effect type
- [ ] Error on duplicate capability name
- [ ] Property tests for roundtrip parsing

## Acceptance Criteria

- [ ] Capability definitions parse from `.ash` files
- [ ] Surface AST correctly represents all syntax elements
- [ ] Lowering to Core AST works correctly
- [ ] All test cases pass
- [ ] Pre-registered capabilities still work (backward compatibility)

## Dependencies

- TASK-233: SPEC-017 Revision (must be complete)

## Estimated Effort

2-3 weeks (1 week parser, 1 week lowering/integration, 3-5 days tests)

## Related Documents

- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md` (revised)
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lower.rs`
