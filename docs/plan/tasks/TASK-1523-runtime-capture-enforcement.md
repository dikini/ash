# TASK-1523: Runtime Capture Enforcement

## Status: 📝 Planned

## Description

Update the runtime to remove the blanket `ctx.is_pure()` ban on closures, add fallback enforcement if needed, or trust the typechecker. Update the interpreter to work with refined closures.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)
- [TASK-1522](TASK-1522-typechecker-capture-analysis.md) — Implementation dependency

## Acceptance Criteria

- [ ] Remove blanket `ctx.is_pure()` closure rejection
- [ ] Add `env_frame.is_pure_capture()` check as fallback (optional)
- [ ] Update closure creation to work with refined captures
- [ ] Ensure runtime behavior matches typechecker expectations
- [ ] Add runtime tests for pure closures with pure captures

## Verification

- `cargo test -p ash-engine` passes
- `cargo test -p ash-interp` passes
- Runtime tests for closures pass
- No regressions in existing tests
