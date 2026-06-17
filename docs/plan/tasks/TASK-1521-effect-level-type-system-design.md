# TASK-1521: Effect Level Type System Design

## Status: ✅ Complete

## Description

Design `EffectLevel` enum, closure type extension, and capture analysis algorithm.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Design Decisions

### Effect Level Representation

Effect levels are represented as strings for runtime values:
- `"Pure"` = 0
- `"Act"` = 1
- `"Proc"` = 2
- `"Workflow"` = 3

The `effect_level_rank()` function in `ash-core/src/value.rs` converts these to numeric ranks for comparison.

### Value Effect Level Assignment

| Value Type | Effect Level |
|-----------|-------------|
| Int, Float, String, Bool, Null, Time, Ref | Pure |
| Record, Variant, List | Max of children's effects |
| Closure | Pure if all captures are pure, otherwise Act |
| Cap | Act |
| ProcessHandle, Proc* | Proc |
| Instance, InstanceAddr, ControlLink | Workflow |
| Stream, ActEnvToken | Act |

### Capture Analysis Algorithm

```rust
fn check_closure_capture(env: &EnvFrame, context: Context) -> Result<(), EvalError> {
    if context.is_pure() {
        for (name, value) in env.all_bindings() {
            if !value.is_pure() {
                return Err(CaptureEffectViolation { var: name, ... });
            }
        }
    }
    Ok(())
}
```

## Implementation

The design was implemented directly in the runtime:

- `Value::is_pure()` — checks if a value and all its components are pure
- `Value::effect_level()` — returns the effect level as a string
- `EnvFrame::all_bindings()` — iterates over all captured bindings including parent chain
- `effect_level_rank()` — converts effect level string to numeric rank

## Verification

- [x] Design reviewed against SPEC-088
- [x] Algorithm correctly identifies pure vs effectful captures
- [x] No gaps in effect level assignment

## Closeout Checklist

- [x] Design complete
- [x] Implemented in code
- [x] Tests verify correctness
- [x] Committed to branch
