# TASK-1668: Preserve mode facts in public summaries

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Extend public Core summaries to preserve mode types and thunk latent-row metadata.

## Specification Reference

- [SPEC-101 §7](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#7-row-accounting)
- [SPEC-101 §10](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#10-core-type-checking)

## Dependencies

- [TASK-1667](TASK-1667-core-letmode-force-typechecking.md)
- [TASK-1649](TASK-1649-core-public-summary-scaffold.md)

## Requirements

1. Public function parameter and return summaries preserve `Strict`, `Lazy`, and `Memo`.
2. Type constructors inside mode-wrapped inner types are collected.
3. Latent thunk row facts stored in `CoreType::Mode.latent_row` are not erased from public
   metadata.
4. Private row alias/group leakage diagnostics still apply inside mode metadata.
5. Public summaries mirror mode-type latent rows; they do not introduce a second independent
   source of truth for thunk body rows.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1668_core_mode_public_summary.rs`.
2. Run the focused test and confirm summary omissions.
3. Extend summary walkers and diagnostics.
4. Re-run `task_1668` and `task_1649_core_public_summary`.

## Completion Checklist

- [ ] Mode wrappers appear in summaries.
- [ ] Latent rows are exported where required.
- [ ] Existing public summary behavior is preserved.
- [ ] Type constructors inside latent row payload types are included.
