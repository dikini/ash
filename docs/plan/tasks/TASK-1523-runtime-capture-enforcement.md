# TASK-1523: Runtime Capture Enforcement

## Status: ✅ Complete

## Description

Update runtime to remove blanket ban, add capture-based enforcement.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Implementation

### Changes Made

**File:** `crates/ash-interp/src/eval.rs`

Replaced the blanket ban in both `eval_expr` (async) and `eval_expr` (sync):

```rust
// Before (blanket ban):
if ctx.is_pure() {
    return Err(EvalError::BoundaryViolation { ... });
}

// After (capture-based rule):
if ctx.is_pure() {
    for (name, value) in env_frame.all_bindings() {
        if !value.is_pure() {
            return Err(EvalError::CaptureEffectViolation {
                var: name,
                var_effect: value.effect_level(),
                context_effect: "Pure".to_string(),
                context: "closure created inside pure-function boundary".into(),
            });
        }
    }
}
```

**File:** `crates/ash-interp/src/error.rs`

Added `CaptureEffectViolation` error variant:
```rust
#[error("capture effect violation: variable '{var}' has effect level {var_effect}, but closure is created in {context_effect} context ({context})")]
CaptureEffectViolation {
    var: String,
    var_effect: String,
    context_effect: String,
    context: String,
}
```

**File:** `crates/ash-core/src/value.rs`

Added:
- `Value::is_pure()` — checks if value and all components are pure
- `Value::effect_level()` — returns effect level string
- `effect_level_rank()` — converts level to numeric rank

**File:** `crates/ash-core/src/env_frame.rs`

Added:
- `EnvFrame::all_bindings()` — iterates over all bindings including parent chain

## Verification

- [x] `cargo test -p ash-interp --lib` — 514 tests pass
- [x] `task559_pure_closure_with_no_captures_allowed` — passes
- [x] `task559_capture_effect_violation_in_pure_context` — passes
- [x] All existing closure tests pass

## Closeout Checklist

- [x] Runtime enforcement implemented
- [x] Error messages clear and actionable
- [x] Tests updated and passing
- [x] Committed to branch
