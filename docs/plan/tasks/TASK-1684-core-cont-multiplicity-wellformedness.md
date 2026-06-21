# TASK-1684: Core Continuation Multiplicity Well-Formedness

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Make `CoreMultiplicity::MultiShotPure` a legal Core type-checking feature when the continuation row
is normalized closed empty.

## Specification Reference

- [SPEC-102 §4](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#4-core-type-amendment)
- [SPEC-102 §8](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#8-core-type-checking)

## Dependencies

- [TASK-1680](TASK-1680-continuation-multiplicity-spec-plan-packet.md)

## Files

- Modify: `crates/ash-core/src/core_ash_typecheck.rs`
- Modify as needed: `crates/ash-core/src/core_ash_validate.rs`
- Test: `crates/ash-core/tests/task_1684_core_cont_multiplicity_wellformedness.rs`

## Requirements

1. `(cont Int Unit {} multi-shot-pure)` is well formed.
2. Multi-shot-pure with non-empty rows is rejected.
3. Multi-shot-pure with open rows is rejected.
4. Multi-shot-pure with ambiguous row references is rejected.
5. Affine continuation well-formedness remains unchanged.
6. Current text parser spelling remains accepted.

## TDD Steps

1. Add failing Core type well-formedness tests for legal empty-row multi-shot-pure.
2. Add failing rejection tests for non-empty, open, and ambiguous rows.
3. Implement the well-formedness rule using existing row normalization helpers.
4. Run `cargo test -p ash-core --test task_1684_core_cont_multiplicity_wellformedness`.
5. Run `cargo test -p ash-core --test task_1641_core_type_wellformedness`.

## Completion Checklist

- [ ] Empty-row multi-shot-pure continuation types are accepted.
- [ ] Non-empty/open/ambiguous multi-shot-pure rows are rejected.
- [ ] CHANGELOG has a task entry.
