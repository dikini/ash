# TASK-801: Transparent Alias Canonicalization Helper

## Status: ✅ Complete

## Description

Add focused alias-canonicalization helpers and diagnostic rendering rules without rolling them out everywhere yet.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-798](TASK-798-canonical-type-ir-lowering-from-surface-and-core.md)

## Objective

Prepare the helper layer that later boundary-adoption tasks will use.

## Requirements

1. Add helper APIs that canonicalize transparent aliases to canonical heads/forms.
2. Preserve readable user-facing alias spellings for diagnostics where practical.
3. Do not implement a full normalizer or broad equality rollout in this task.

## Files

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/types.rs`
- Create or modify alias helper module if needed
- Add focused `ash-typeck` alias helper tests

## TDD Steps

1. Write failing tests for transparent alias canonical-head behavior and readable diagnostic rendering.
2. Implement the minimal helper layer.
3. Re-run focused alias-helper tests.
4. Stop before changing current equality or pattern boundaries.

## Verification Steps

- [x] `cargo test -p ash-typeck` for alias helper tests
- [x] `cargo fmt --check`
- [x] `git diff --check`

## Notes

Single-crate task. Junior implementers should not widen this into generalized equality work.
