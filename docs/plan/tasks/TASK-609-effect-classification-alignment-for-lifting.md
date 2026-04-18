# TASK-609: Effect Classification Alignment for Lifting

## Status: ✅ Complete

## Description

Replace parser-local heuristic effect classification in `lift.rs` with a production-quality effect/source-of-truth path aligned with later compiler/runtime stages.

## Specification Reference

- `docs/design/DESIGN-028-STATEMENT-LIFTING.md`
- `docs/spec/SPEC-001-IR.md`

## Dependencies

- 🟡 TASK-608: Statement-Lifting Contract Hardening

## Requirements

1. `lift.rs` must no longer treat `EFFECTFUL_NAMES` as semantic truth.
2. Effectful-vs-pure classification must be derived from an explicit, maintainable source of truth.
3. Classification drift between parser/lifting and runtime/typechecker must be eliminated or made impossible by construction.
4. Tests must cover shadowing and user-defined names so builtin-name collisions do not silently misclassify expressions.

## TDD Steps

1. Add failing tests for shadowed builtin names and misclassification cases.
2. Introduce a shared effect classification path or deferred classification mechanism.
3. Remove or demote `EFFECTFUL_NAMES` from semantic authority.
4. Re-run parser/typechecker/interpreter tests.

## Verification Steps

- [ ] `cargo test -p ash-parser -- --nocapture`
- [ ] `cargo test -p ash-typeck -- --nocapture`
- [ ] `cargo test -p ash-interp -- --nocapture`

## Notes

Production quality here means lifting decisions are based on explicit compiler truth, not fragile parser heuristics.