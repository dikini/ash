# TASK-833: Add core-owned type-function/equation/pattern/result-expression carriers shared by typeck/engine-facing metadata

## Status: 📋 Planned

## Description

Add core-owned type-function/equation/pattern/result-expression carriers shared by typeck/engine-facing metadata.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-832 parser surface decisions being stable.

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

Add core-owned type-function/equation/pattern/result-expression carriers shared by typeck/engine-facing metadata.

## Requirements

1. Add TypeFunctionDef/Equation/Pattern-style carriers or equivalent.
2. Add a `TypeFunctionResultExpr`-style carrier or explicitly extend `CanonicalTypeExpr` so source equation RHSs can represent sealed-domain marker-constructor applications.
3. Use `TypeComputationHeadId` and sealed-domain constructor IDs, not ordinary nominal constructor IDs.
4. Preserve source anchors, equation order, parameter metadata, result expressions, and decreasing-parameter metadata.
5. Encode pattern and result-expression domain/constraint metadata needed for later result-domain and coverage checks.
6. Add serde/hash/equality tests where carriers cross crate boundaries.

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
  - cargo test -p ash-core --test task_833_type_function_carriers -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Add TypeFunctionDef/Equation/Pattern-style carriers or equivalent.
  - [ ] Add a `TypeFunctionResultExpr`-style carrier or explicitly extend `CanonicalTypeExpr` so source equation RHSs can represent sealed-domain marker-constructor applications.
  - [ ] Use `TypeComputationHeadId` and sealed-domain constructor IDs, not ordinary nominal constructor IDs.
  - [ ] Preserve source anchors, equation order, parameter metadata, result expressions, and decreasing-parameter metadata.
  - [ ] Encode pattern and result-expression domain/constraint metadata needed for later result-domain and coverage checks.
  - [ ] Add serde/hash/equality tests where carriers cross crate boundaries.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Core/Substrate. Estimated effort: 5 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
