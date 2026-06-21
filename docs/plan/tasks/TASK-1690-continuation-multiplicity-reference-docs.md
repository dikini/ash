# TASK-1690: Add Continuation Multiplicity Reference Documentation

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Document the implemented Core/CPS continuation multiplicity behavior and link non-normative design
rationale.

## Specification Reference

- [SPEC-102](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)
- [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)

## Dependencies

- [TASK-1689](TASK-1689-motivational-multishot-fixtures.md)

## Files

- Create or modify reference docs under `reference/` following existing Core/CPS pages.
- Test: `crates/ash-core/tests/task_1690_continuation_multiplicity_docs_consistency.rs`

## Requirements

1. Explain affine versus multi-shot-pure behavior.
2. State that multi-shot-pure requires explicit Core multiplicity and empty row.
3. State that surface syntax is informational and out of scope.
4. Link SPEC-102, PLAN-164, `docs/design/multi-shot-continuations.md`, and NOTE-012.
5. Document current `.core` spelling.
6. Document test fixture names for motivational examples.

## TDD Steps

1. Add a failing docs consistency test for required links and phrases.
2. Write/update reference docs.
3. Run `cargo test -p ash-core --test task_1690_continuation_multiplicity_docs_consistency`.
4. Run `cargo test -p spec_processor spec_links`.

## Completion Checklist

- [ ] Reference docs exist and link the spec/plan.
- [ ] Non-normative commentary is labeled as such.
- [ ] Docs consistency test passes.
- [ ] CHANGELOG has a task entry.
