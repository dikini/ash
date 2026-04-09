# DESIGN-015: Unified Action System

## Status: Draft

## Overview

Eliminate the unnecessary duality between `ash_core::Action` and library provider actions by unifying the representation and provider interface. This design removes the adapter layer and aligns semantic intent with implementation.

## Problem Statement

### Current State (Dual Representation)

```rust
// ash-core/src/ast.rs
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,  // Unevaluated expressions
}

// ash-engine/src/providers/mod.rs (library provider trait)
async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError>;
//                                      ^^^^^    ^^^^^^^^^^^^^^^
//                                      Action name (string) + evaluated values

// ash-interp/src/capability.rs (interpreter provider trait)
async fn execute(&self, action: &Action) -> ExecResult<Value>;
//                                      ^^^^^
//                                      Action struct with Expr[]
```

### Issues

1. **Implementation duality, not semantic distinction**: Both representations serve the same purpose (execute an action with arguments)
2. **Adapter layer required**: `InterpProviderAdapter` bridges the two traits
3. **Scattered evaluation**: Arguments evaluated in adapter, not at execution layer
4. **Error type duplication**: `ExecError` and `ProviderError` serve similar purposes
5. **Confusion for library authors**: Must learn two different provider interfaces

### Root Cause

Historical evolution: `Action` was created for interpreter internal use, while library providers evolved independently with a simpler string-based interface. The adapter was added later to bridge them.

## Design Goals

1. **Single source of truth**: One `Action` type, one `CapabilityProvider` trait
2. **Clear evaluation semantics**: Arguments evaluated at workflow execution layer
3. **No adapter layer**: Direct provider interface
4. **Minimize change scope**: Keep `observe` unchanged (accepts phase misalignment)
5. **Breaking change accepted**: Immediate migration, no compatibility layer

## Design Decisions

### Decision 1: Action Uses `Vec<Value>` (Eager Evaluation)

**Rationale**: `Action` should represent a **ready-to-execute** action with evaluated arguments.

**Before**:
```rust
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,  // Unevaluated
}
```

**After**:
```rust
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Value>,  // Evaluated
}
```

**Implications**:
- Arguments evaluated **once** at ACT execution layer
- Provider receives ready-to-use values
- No expression evaluation in provider code

---

### Decision 2: Unified Provider Trait

**Rationale**: Single trait for all providers eliminates adapter layer.

**Unified trait** (in `ash_core`):
```rust
#[async_trait]
pub trait CapabilityProvider: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn effect(&self) -> Effect;
    
    // Observe: uses unevaluated constraints (delayed evaluation)
    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>;
    
    // Execute: uses Action with evaluated arguments (eager evaluation)
    async fn execute(&self, action: &Action) -> Result<Value, CapabilityError>;
}
```

**Removed**:
- `ash_engine::providers::CapabilityProvider` (old trait)
- `InterpProviderAdapter` (bridge)

---

### Decision 3: Unified Error Type

**Rationale**: Single error type simplifies error handling.

**Before**:
```rust
// ash-interp
pub enum ExecError { ... }

// ash-engine
pub enum ProviderError { ... }
```

**After** (in `ash_core`):
```rust
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("Capability '{0}' not available")]
    NotAvailable(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
```

**Conversion**: `CapabilityError` → `ExecError` at interpreter boundary

---

### Decision 4: Keep `observe` Unchanged (Phase Misalignment Accepted)

**Rationale**: `observe` constraints are more complex and may reference workflow state; keeping them unevaluated minimizes change scope.

**Phase misalignment**:
```rust
// observe: delayed evaluation (provider evaluates constraints)
async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>;

// execute: eager evaluation (workflow evaluates arguments)
async fn execute(&self, action: &Action) -> Result<Value, CapabilityError>;
```

**Trade-off**: Inconsistent evaluation semantics, but:
- `observe` is more complex (constraints vs arguments)
- `execute` is simpler (pure values)
- Clear separation in type signatures
- Minimal change scope

---

### Decision 5: Evaluate Arguments at ACT Layer

**Rationale**: Single evaluation point, clear responsibility.

**Before** (evaluation in adapter):
```rust
Workflow::Act { action, guard, ... } => {
    // Evaluate guard
    // Call provider (adapter evaluates arguments)
    cap_ctx.execute(action, &action.name).await
}
```

**After** (evaluation at ACT):
```rust
Workflow::Act { action, guard, ... } => {
    // 1. Evaluate guard
    let guard_result = eval_guard(guard, &ctx)?;
    
    // 2. Evaluate action arguments (NEW)
    let evaluated_action = Action {
        name: action.name.clone(),
        arguments: action.arguments
            .iter()
            .map(|expr| eval_expr(expr, &ctx)?)
            .collect()?,
    };
    
    // 3. Execute with evaluated arguments
    cap_ctx.execute(&evaluated_action, &evaluated_action.name).await?
}
```

**Benefits**:
- Single evaluation location
- Guard evaluated before arguments (unchanged order)
- Provider receives ready-to-use values
- Clear error boundary

---

## Architecture

### Before (Dual Representation)

```
Surface Syntax (ACT)
    ↓
Action { name, arguments: Expr[] }
    ↓
Adapter (InterpProviderAdapter)
    ↓
(provider.execute(action: &str, args: &[Value]))
    ↓
Library Provider
```

### After (Unified)

```
Surface Syntax (ACT)
    ↓
Action { name, arguments: Expr[] }
    ↓
Evaluate arguments → Action { name, arguments: Value[] }
    ↓
CapabilityContext::execute(&Action)
    ↓
Provider::execute(&Action)
    ↓
Library/Primitive Provider
```

---

## Impact Analysis

### Breaking Changes

1. **`Action.arguments` type change**: `Vec<Expr>` → `Vec<Value>`
   - Affects: Parser, AST, type checker, interpreter
   - Mitigation: Land parser/lowering and ACT execution boundary changes in the same phase as the type change so no crate is left constructing the old representation after `Action` changes

2. **Provider trait change**: Two traits → one unified trait
   - Affects: All provider implementations (FsProvider, StdioProvider, McpProvider)
   - Mitigation: Immediate migration, no compatibility layer

3. **Error type unification**: Two error types → one
   - Affects: Error handling across workspace
   - Mitigation: Conversion at interpreter boundary

### Non-Breaking Changes

1. **`observe` unchanged**: Keeps existing signature
2. **ACT guard evaluation**: Unchanged order
3. **Workflow semantics**: Unchanged, only implementation detail

---

## Migration Strategy

### Immediate Break (No Compatibility Layer)

**Rationale**: Clean break prevents technical debt accumulation.

### Step 1: Core Types (ash-core)
1. Update `Action::arguments` to `Vec<Value>`
2. Add unified `CapabilityProvider` trait
3. Add `CapabilityError` enum

### Step 2: Action Boundary Realignment (ash-parser, ash-interp)
1. Update parser/lowering so unevaluated surface arguments are not stored in the provider-facing `Action`
2. Update ACT execution to evaluate arguments before provider dispatch
3. Preserve existing guard-before-action evaluation order

### Step 3: Execution Layer And Registry Migration (ash-interp)
1. Update `CapabilityContext` to use unified trait
2. Remove wrapper/adapter code that only exists to bridge the old trait split
3. Add error conversion (`CapabilityError` → `ExecError`)

### Step 4: Provider Migrations (ash-engine)
1. Update `FsProvider` to unified trait
2. Update `StdioProvider` to unified trait
3. Update `McpProvider` to unified trait
4. Update `Engine` builder
5. Remove old provider trait
6. Remove `InterpProviderAdapter`

### Step 5: Cleanup
1. Update error handling across workspace
2. Update documentation
3. Update examples
4. Full integration testing

---

## Open Questions

1. **Should constraints also be `Vec<Value>`?**
   - **Decision**: No, keep as `&[Constraint]` (unevaluated)
   - **Rationale**: Minimize change scope, constraints are more complex

2. **Should we add a `Pure` effect to the lattice?**
   - **Decision**: Out of scope for this design
   - **Rationale**: Separate concern, addresses different problem

3. **Should we support action registration/discovery?**
   - **Decision**: Out of scope for this design
   - **Rationale**: Provider registration already handles this

---

## Alternatives Considered

### Alternative 1: Keep `Action` with `Vec<Expr>`, Evaluate in Provider

**Rejected** because:
- Scattered evaluation logic
- Provider becomes responsible for evaluation
- Harder to test providers in isolation

### Alternative 2: Use string-based action dispatch for all

**Rejected** because:
- Loses type safety
- Harder to document and validate
- Prone to typos in action names

### Alternative 3: Support both representations with compatibility layer

**Rejected** because:
- Adds technical debt
- Confusing for library authors
- Maintenance burden

---

## Risks and Mitigations

### Risk 1: Breaking Change Disrupts Downstream Users

**Mitigation**: 
- Comprehensive test suite before change
- Clear migration guide
- Examples updated immediately

### Risk 2: Phase Misalignment Causes Confusion

**Mitigation**:
- Document explicitly in specs
- Clear type signatures show evaluation strategy
- Examples demonstrate correct usage

### Risk 3: Error Handling Edge Cases Missed

**Mitigation**:
- Thorough error handling testing
- Property tests for error propagation
- Review all error paths

---

## Success Criteria

1. **Single trait**: All providers implement `ash_core::CapabilityProvider`
2. **No adapter**: `InterpProviderAdapter` removed
3. **Single error type**: `CapabilityError` unified
4. **Tests pass**: Full test suite green
5. **Docs updated**: All specs and examples updated
6. **Performance**: No regression in action execution

---

## References

- [TASK-001](../plan/tasks/TASK-001-effect-lattice.md) - Effect lattice implementation
- [SPEC-001](../spec/SPEC-001-IR.md) - Core IR definition
- [SPEC-017](../spec/SPEC-017-CAPABILITY-INTEGRATION.md) - Capability integration
- [SPEC-018](../spec/SPEC-018-CAPABILITY-MATRIX.md) - Capability verification

---

*Document Version: 1.0*  
*Status: Draft*  
*Author: hermes*  
*Date: 2026-04-09*
