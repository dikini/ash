# TASK-840: Add diagnostics and the full SPEC-061 acceptance/non-regression test matrix

## Status: 📋 Planned

## Description

Add diagnostics and the full SPEC-061 acceptance/non-regression test matrix.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-839 engine/module boundary completion.

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

Add diagnostics and the full SPEC-061 acceptance/non-regression test matrix.

## Requirements

1. Add named diagnostic families from SPEC-061 §14, including ambiguous marker constructors, unreachable rows, empty defaults, missing/invalid decreases, and forward-reference rejection.
2. Add diagnostics/tests for unknown RHS pattern variables and successful pattern-variable substitution.
3. Prove parser acceptance with accurate spans and parser rejection for malformed case heads, missing semicolons, rejected visibility prefixes, and parser dispatch before ordinary `type` parsing.
4. Prove core/lowering identity, parameter metadata, equation order, result expressions, source-anchor preservation, and marker-constructor RHS carriers.
5. Prove Append known-scrutinee reduction, abstract neutrality, residual catch-all behavior, nested residual coverage/default behavior, accepted positive multiple-default residual rows, partial Head rejection, overlap/unreachable/empty-default rejection, repeated-variable rejection, wrong-domain rejection, lowercase marker/variable disambiguation, marker-constructor ambiguity, result-domain mismatch rejection, no-sealed-scrutinee rejection, ambiguous-head rejection, missing/invalid decreases rejection, recursive negative cases, nested recursive-call detection, mutual recursion / forward-reference rejection, public leakage rejection, and cross-module non-normalization.
6. Collect or cite non-regression evidence for SPEC-057 ordinary summaries, SPEC-059 sealed domains, and SPEC-060 fixture normalizer tests.

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
  - cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture
  - cargo test -p ash-core --test task_833_type_function_carriers -- --nocapture
  - cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Add named diagnostic families from SPEC-061 §14, including ambiguous marker constructors, unreachable rows, empty defaults, missing/invalid decreases, and forward-reference rejection.
  - [ ] Add diagnostics/tests for unknown RHS pattern variables and successful pattern-variable substitution.
  - [ ] Prove parser acceptance with accurate spans and parser rejection for malformed case heads, missing semicolons, rejected visibility prefixes, and parser dispatch before ordinary `type` parsing.
  - [ ] Prove core/lowering identity, parameter metadata, equation order, result expressions, source-anchor preservation, and marker-constructor RHS carriers.
  - [ ] Prove Append known-scrutinee reduction, abstract neutrality, residual catch-all behavior, nested residual coverage/default behavior, accepted positive multiple-default residual rows, partial Head rejection, overlap/unreachable/empty-default rejection, repeated-variable rejection, wrong-domain rejection, lowercase marker/variable disambiguation, marker-constructor ambiguity, result-domain mismatch rejection, no-sealed-scrutinee rejection, ambiguous-head rejection, missing/invalid decreases rejection, recursive negative cases, nested recursive-call detection, mutual recursion / forward-reference rejection, public leakage rejection, and cross-module non-normalization.
  - [ ] Collect or cite non-regression evidence for SPEC-057 ordinary summaries, SPEC-059 sealed domains, and SPEC-060 fixture normalizer tests.
  - [ ] focused tests/evidence recorded in this task file
  - [ ] no SPEC-F/G/H scope creep
```


## Notes

Task type: Diagnostics/Tests. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
