# TASK-1689: Add Motivational Multi-Shot Fixtures

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Encode the motivational multi-shot continuation examples as executable Core/CPS tests using current
syntax.

## Specification Reference

- [SPEC-102 §10](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#10-motivational-examples)

## Dependencies

- [TASK-1686](TASK-1686-core-affine-use-discipline-with-multishot.md)
- [TASK-1687](TASK-1687-core-to-cps-multiplicity-lowering.md)

## Files

- Create fixtures under: `crates/ash-core/tests/fixtures/core/`
- Test: `crates/ash-core/tests/task_1689_motivational_multishot_fixtures.rs`
- Add direct CPS tests in `crates/ash-interp/tests/` only if Core text cannot express a scenario
  without unrelated syntax work.

## Required Examples

1. Choice/all-outcomes: a pure handler invokes resume with `true` and `false` and combines results.
2. Backtracking/find-first: a pure handler tries at least two candidates and returns the first
   successful branch.
3. Nested choice: two independent pure choices produce four logical paths or an equivalent
   structured result.
4. Discard resume: a handler ignores a multi-shot-pure resume and returns directly.
5. Affine negative: the choice/all-outcomes shape with affine resume rejects or traps.
6. Effectful negative: a multi-shot-pure resume with non-empty row rejects.

## Requirements

1. Do not copy surface syntax from the design note.
2. Use current `.core` parser syntax and existing Core operations.
3. Prefer small integer/tuple/list-free encodings if list support would distract from the feature.
4. Make the expected outcome obvious in each test name.
5. Keep each fixture focused; do not introduce a general search library.

## TDD Steps

1. Add failing fixture tests for the four positive examples.
2. Add failing tests for the two negative examples.
3. Adjust implementation only if earlier tasks missed behavior needed by these examples.
4. Run `cargo test -p ash-core --test task_1689_motivational_multishot_fixtures`.
5. Run `cargo test -p ash-interp --test task_1682_cps_multishot_runtime`.

## Completion Checklist

- [ ] All four motivational positive examples execute or type-check as intended.
- [ ] Both negative examples reject for the intended reason.
- [ ] Surface syntax remains untouched.
- [ ] CHANGELOG has a task entry.
