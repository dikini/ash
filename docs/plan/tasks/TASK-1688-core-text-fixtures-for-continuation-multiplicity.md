# TASK-1688: Add Core Text Fixtures for Continuation Multiplicity

**Status:** ✅ Complete
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Add `.core` fixtures and golden coverage for continuation multiplicity and answer-binding
continuation invocation.

## Specification Reference

- [SPEC-102 §4](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#4-core-type-amendment)
- [SPEC-102 §9](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#9-core-to-cps-lowering)

## Dependencies

- [TASK-1684](TASK-1684-core-cont-multiplicity-wellformedness.md)
- [TASK-1686](TASK-1686-core-affine-use-discipline-with-multishot.md)
- [TASK-1687](TASK-1687-core-to-cps-multiplicity-lowering.md)

## Files

- Create fixtures under: `crates/ash-core/tests/fixtures/core/`
- Test: `crates/ash-core/tests/task_1688_core_text_continuation_multiplicity.rs`

## Required Fixtures

1. `multishot_resume_text_roundtrip.core`
2. `affine_empty_row_remains_affine.core`
3. `invalid_multishot_nonempty_row.core`
4. `invalid_multishot_open_row.core`
5. `let_cont_call_text_roundtrip.core`

## Requirements

1. Use Core text syntax available after TASK-1686.
2. Round-trip legal fixtures through parser and serializer.
3. Validate/type-check legal fixtures.
4. Assert invalid fixtures fail at validation or type checking with multiplicity-specific errors.
5. Include at least one legal fixture that lowers `let-cont-call` to CPS `LetContCall`.
6. Add `.cps.golden` files only where existing Core fixture tests expect them.

## TDD Steps

1. Add fixture files and a failing round-trip test.
2. Add failing invalid-fixture tests.
3. Adjust parser/serializer only if existing behavior is insufficient.
4. Run `cargo test -p ash-core --test task_1688_core_text_continuation_multiplicity`.
5. Run existing Core text tests.

## Completion Checklist

- [x] Legal fixtures round-trip.
- [x] Invalid fixtures reject for the intended reason.
- [x] CHANGELOG has a task entry.
