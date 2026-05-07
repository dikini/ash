# TASK-840: Add diagnostics and the full SPEC-061 acceptance/non-regression test matrix

## Status: ✅ Complete

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
  - [x] Add named diagnostic families from SPEC-061 §14, including ambiguous marker constructors, unreachable rows, empty defaults, missing/invalid decreases, and forward-reference rejection.
  - [x] Add diagnostics/tests for unknown RHS pattern variables and successful pattern-variable substitution.
  - [x] Prove parser acceptance with accurate spans and parser rejection for malformed case heads, missing semicolons, rejected visibility prefixes, and parser dispatch before ordinary `type` parsing.
  - [x] Prove core/lowering identity, parameter metadata, equation order, result expressions, source-anchor preservation, and marker-constructor RHS carriers.
  - [x] Prove Append known-scrutinee reduction, abstract neutrality, residual catch-all behavior, nested residual coverage/default behavior, accepted positive multiple-default residual rows, partial Head rejection, overlap/unreachable/empty-default rejection, repeated-variable rejection, wrong-domain rejection, lowercase marker/variable disambiguation, marker-constructor ambiguity, result-domain mismatch rejection, no-sealed-scrutinee rejection, ambiguous-head rejection, missing/invalid decreases rejection, recursive negative cases, nested recursive-call detection, mutual recursion / forward-reference rejection, public leakage rejection, and cross-module non-normalization.
  - [x] Collect or cite non-regression evidence for SPEC-057 ordinary summaries, SPEC-059 sealed domains, and SPEC-060 fixture normalizer tests.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Evidence

- Added focused acceptance aggregator: `crates/ash-typeck/tests/task_840_type_function_acceptance.rs`.
- TASK-840 aggregator coverage:
  - Cites/asserts SPEC-061 §14 family names with stable diagnostic substrings for `TypeFunctionNoSealedScrutinee`, `TypePatternUnknownConstructor`, `TypePatternWrongDomain`, `TypePatternRepeatedVariable`, `TypeFunctionNonExhaustive`, `TypeFunctionOverlappingEquation`, `TypeFunctionUnreachableEquation`, `TypeFunctionEmptyDefault`, `TypeFunctionMissingDecreases`, `TypeFunctionInvalidDecreases`, `TypeFunctionNonDecreasingRecursion`, `TypeFunctionResultDomainMismatch`, and `TypeFunctionForwardReferenceUnsupported`.
  - Adds direct acceptance/rejection coverage for ambiguous nominal/type-function heads and ambiguous marker constructors.
  - Adds direct coverage for unknown RHS pattern variables and successful pattern-variable substitution through source-backed `Append` normalization.
  - Adds direct coverage for residual catch-all rows reducing known residual constructors while preserving abstract-scrutinee neutrality.
  - Adds direct acceptance coverage for nested default residual rows, positive multiple-default residual rows, and lowercase marker-constructor disambiguation.
  - Adds direct recursive negative coverage for rebuilt/computed recursive arguments, nested recursive call detection, and mutual-recursion/forward-reference rejection, plus invalid decreases metadata diagnostics.
- Parser matrix cited and verified by `crates/ash-parser/tests/task_832_type_function_parser.rs`: raw `type fn Append` spans/pattern/RHS acceptance, dispatch before ordinary `type`, malformed case heads, missing semicolons, visibility prefixes, zero parameters, and inline-module rejection.
- Core/lowering matrix cited and verified by:
  - `crates/ash-core/tests/task_833_type_function_carriers.rs`: computation-head identity, parameter metadata, equation order, source anchors, pattern/result constraints, all result-expression carrier variants, marker-constructor RHS carrier, serde/equality/hash.
  - `crates/ash-typeck/tests/task_834_type_function_lowering.rs`: source-order registration, provisional self-reference lowering, earlier dependency lowering, invalid non-publication, forward-reference rejection, pattern-variable metadata, marker-constructor RHS carriers.
- Typeck/normalizer/engine matrix cited and verified by:
  - `crates/ash-typeck/tests/task_835_type_function_validation.rs`: signature/result/domain/arity validation, unknown RHS variables, repeated variables, wrong-domain constructors/RHSs, result-domain mismatch, ambiguous type-function/type heads, ambiguous marker constructors in RHS/pattern positions, lowercase pattern-variable precedence, no-sealed-scrutinee rejection, source-order dependency acceptance/rejection, and public `type fn` rejection before SPEC-F.
  - `crates/ash-typeck/tests/task_836_type_function_patterns.rs`: partial `Head` rejection, overlap/unreachable/empty-default rejection, positive multiple defaults, nested residual coverage/defaults, nested constructor rejection in unconstrained `Type` slots, lowercase marker constructors, and same-name variables across different rows.
  - `crates/ash-typeck/tests/task_837_type_function_recursion.rs`: missing/invalid decreases rejection, accepted direct-tail recursion, same/rebuilt/computed argument rejection, nested self-call detection, mutual-recursion source-order rejection, and invalid-head non-publication.
  - `crates/ash-typeck/tests/task_838_type_function_normalizer.rs`: source-backed `Append` known-scrutinee reduction, closed recursive reduction, abstract neutrality, partial-prefix neutrality, bound-variable substitution, and definitional equality.
  - `crates/ash-engine/tests/task_839_type_function_module_boundary.rs`: private same-module aliases, public alias/callable leakage rejection, and non-serialization/non-normalization across imported semantic summaries before SPEC-F.
- SPEC-057/SPEC-059/SPEC-060 non-regression evidence:
  - SPEC-057 ordinary summaries: `cargo test -p ash-engine --test task_785_modulefile_summary_exports -- --nocapture` — 7 passed; `cargo test -p ash-typeck --test task_787_semantic_summary_typeenv -- --nocapture` — 24 passed.
  - SPEC-059 sealed domains: `cargo test -p ash-typeck --test task_812_domain_registration_validation -- --nocapture` — 9 passed; `cargo test -p ash-engine --test task_813_sealed_domain_non_interference -- --nocapture` — 6 passed.
  - SPEC-060 fixture normalizer: `cargo test -p ash-typeck --test task_821_closed_computation_head_reduction -- --nocapture` — 5 passed; `cargo test -p ash-typeck --test task_822_open_neutral_partial_normalization -- --nocapture` — 7 passed; `cargo test -p ash-typeck --test task_824_definitional_equality -- --nocapture` — 7 passed.
- Focused verification commands run clean:
  - `cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture` — 6 passed.
  - `cargo test -p ash-core --test task_833_type_function_carriers -- --nocapture` — 5 passed.
  - `cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture` — 7 passed.
  - `cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture` — 4 passed.
  - `cargo fmt --check` — passed.
  - `git diff --check` — passed.
- Additional cited Phase 113 semantic suites run clean: TASK-835 — 19 passed; TASK-836 — 10 passed; TASK-837 — 11 passed; TASK-838 — 6 passed.
- No SPEC-F/G/H behavior was added; this task only added focused tests and docs/status evidence.


## Notes

Task type: Diagnostics/Tests. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
