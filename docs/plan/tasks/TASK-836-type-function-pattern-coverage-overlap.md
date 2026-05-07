# TASK-836: Implement pattern linearity plus pattern-matrix coverage, overlap, and residual catch-all semantics

## Status: 📋 Planned

## Description

Implement pattern linearity plus pattern-matrix coverage, overlap, and residual catch-all semantics.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-835 signature/source/domain validation completion.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Implement pattern linearity plus pattern-matrix coverage, overlap, and residual catch-all semantics.

## Requirements

1. Reject repeated pattern variables within one equation row while allowing the same variable name in different rows.
2. Build finite symbolic pattern spaces over sealed-domain constructor tuples, splitting nested fields only where explicitly inspected.
3. Define and implement nested residual coverage for explicitly inspected sealed-domain fields; do not unboundedly expand recursive domains.
4. Reject nested constructor patterns inside unconstrained `Type` slots.
5. Reject non-exhaustive partial Head-style definitions.
6. Reject overlapping explicit rows, unreachable rows after defaults, empty residual defaults, and duplicate defaults over empty residual spaces.
7. Verify wildcard/variable catch-all rows cover known residual constructors only and do not reduce abstract scrutinees.
8. Add tests for nested-pattern coverage gaps, nested defaults, accepted positive multiple-default residual rows, later explicit rows after defaults, empty/duplicate defaults, and lowercase constructor/variable disambiguation.

## Files

- Modify/create exact files identified by [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) and the TASK-831 audit gate.
- Update `CHANGELOG.md` for completed implementation/tooling/docs-policy changes.

## TDD Steps

1. Write focused failing tests or docs/audit checks appropriate to task type.
2. Run the focused target and verify the expected failure or missing evidence.
3. Implement the minimal change for this task only.
4. Re-run the focused target and relevant non-regression tests.
5. Update docs/status evidence only after verification.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_836_type_function_patterns -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Reject repeated pattern variables within one equation row while allowing the same variable name in different rows.
  - [ ] Build finite symbolic pattern spaces over sealed-domain constructor tuples, splitting nested fields only where explicitly inspected.
  - [ ] Define and implement nested residual coverage for explicitly inspected sealed-domain fields; do not unboundedly expand recursive domains.
  - [ ] Reject nested constructor patterns inside unconstrained `Type` slots.
  - [ ] Reject non-exhaustive partial Head-style definitions.
  - [ ] Reject overlapping explicit rows, unreachable rows after defaults, empty residual defaults, and duplicate defaults over empty residual spaces.
  - [ ] Verify wildcard/variable catch-all rows cover known residual constructors only and do not reduce abstract scrutinees.
  - [ ] Add tests for nested-pattern coverage gaps, nested defaults, accepted positive multiple-default residual rows, later explicit rows after defaults, empty/duplicate defaults, and lowercase constructor/variable disambiguation.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Type/Semantic. Estimated effort: 7 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
