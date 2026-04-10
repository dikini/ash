# TASK-487: Extend Surface AST and Lowering for Act Continuation

## Status: Done

## Description

Extend the surface `Workflow::Act` variant in `ash-parser/src/surface.rs` with `result_name` and
`continuation` fields. Update the lowering pass in `ash-parser/src/lower.rs` to propagate these
fields to the core `Workflow::Act`.

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-001](../../spec/SPEC-001-IR.md)

## Dependencies

- [TASK-486](TASK-486-core-act-continuation-shape.md) — core Act must have the new fields first

## Requirements

1. Add `result_name: Option<Name>` to `SurfaceWorkflow::Act` in `surface.rs`.
2. Add `continuation: Option<Box<Workflow>>` to `SurfaceWorkflow::Act` (None = terminal).
3. Update `Spanned` impl for `Workflow` if needed.
4. Update lowering in `lower.rs`:
   - `result_name` maps directly to core `result_name`.
   - `continuation` maps: `Some(w)` → lower recursively, `None` → `CoreWorkflow::Done`.
5. All existing surface Act nodes (no result_name, no continuation) lower identically to before.
6. `cargo test -p ash-parser` passes.

## TDD Steps

### Red

- Write tests in `lower.rs` test module that construct `SurfaceWorkflow::Act` with `result_name`
  and `continuation`, assert the lowered core Act has the correct fields.

### Green

- Implement the surface AST changes and lowering updates.
- Verify new tests pass and existing tests still green.

### Refactor

- Ensure lowering helper for Act is clean and doesn't duplicate the continuation-defaulting logic.

## Completion Checklist

- [ ] `SurfaceWorkflow::Act` has `result_name` and `continuation` fields
- [ ] Lowering maps new fields correctly
- [ ] Existing tests pass unchanged
- [ ] New lowering tests added for `result_name` and `continuation`
- [ ] `cargo test -p ash-parser` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md entry added
