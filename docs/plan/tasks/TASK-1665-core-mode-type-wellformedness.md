# TASK-1665: Type-check mode type well-formedness

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Extend Core type well-formedness and equivalence for `Strict`, `Lazy`, and `Memo` mode types.

## Specification Reference

- [SPEC-101 §4](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#4-core-type-shape)
- [SPEC-101 §10](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#10-core-type-checking)

## Dependencies

- [TASK-1662](TASK-1662-core-mode-validation.md)
- [TASK-1641](TASK-1641-core-type-wellformedness.md)

## Requirements

1. Mode types are well formed only when their inner type is well formed.
2. Mode equality is invariant: `Strict A`, `Lazy A`, and `Memo A` are distinct.
3. Diagnostics distinguish mode mismatch from ordinary type mismatch.
4. Existing refinement, record, function, continuation, and row checks still work inside mode wrappers.
5. `CoreType::Mode { mode: Strict, latent_row: Some(_) }` is ill formed.
6. `CoreType::Mode { mode: Lazy|Memo, latent_row: None }` is ill formed.
7. Lazy/memo latent rows are recursively well formed: validate row tails, effect-group references,
   and typed row-item payload types using the same rules as function and continuation rows.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1665_core_mode_type_wellformedness.rs`.
2. Run the focused test and confirm missing mode checks.
3. Extend `core_ash_typecheck.rs` type well-formedness/equivalence.
4. Re-run `task_1665` and `task_1641_core_type_wellformedness`.

## Completion Checklist

- [x] Inner type validation is recursive.
- [x] Strict mode rejects `Some(row)`.
- [x] Lazy and memo modes reject missing latent rows.
- [x] Lazy and memo latent rows are recursively well formed.
- [x] Mode invariance is enforced.
- [x] Mode mismatch has a structured diagnostic.
