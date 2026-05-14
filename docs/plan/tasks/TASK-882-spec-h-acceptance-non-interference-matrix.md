# TASK-882: SPEC-H acceptance/non-interference matrix

## Status: ✅ Complete

## Description

Create the Phase 116 acceptance/non-interference matrix and focused aggregator evidence for every SPEC-064 §12 row.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-881 completion

## Files / Ownership

- Create: `docs/plan/audits/TASK-882-proposition-acceptance-matrix.md`
- Modify/add: `crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs`
- Modify/add: `crates/ash-parser/tests/task_882_spec_h_surface_non_interference.rs`
- Modify/add: `crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs`
- Modify/add: `crates/ash-engine/tests/task_882_spec_h_transport_non_interference.rs`
- Modify: task/status docs if evidence changes
- Audit rows: H-FORCE-10, H-RISK-01, H-RISK-02, H-RISK-03, H-RISK-04, H-RISK-05, H-RISK-06, H-AUD-NONINT-01

## TASK-872 Binding Notes

- Acceptance matrix must map SPEC-064 §12 H1-H12 to exact focused test evidence, including deferred/diagnostic rows.
- Non-interference suites to cite or run include existing SPEC-035/SPEC-063 associated-family, SPEC-060 normalizer/equality, SPEC-057/058/059/061/062 type-pipeline/summary, and engine transport regressions named in `docs/plan/audits/TASK-872-proposition-layer-audit.md`.
- No zero-test pass is acceptable; every aggregator must list non-zero `task_882_` tests before running.

## Requirements

### Functional Requirements

1. Map every SPEC-064 §12 row H1 through H12 to exact focused test evidence; H3 and other deferred-behavior rows still require focused tests proving the expected diagnostic/deferred outcome.
2. Include command, test count, expected result, actual result, and owning earlier task for every row.
3. Add focused aggregator tests when a row lacks evidence.
4. Run non-interference suites for SPEC-035 and SPEC-057 through SPEC-063 named by TASK-872.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Create acceptance matrix artifact.

### Step 2

- Run all focused tests with non-zero guards.

### Step 3

- Patch missing evidence with focused tests; a documentation-only deferral is allowed only after SPEC-064 and PLAN-112 are explicitly amended to narrow the acceptance row.

### Step 4

- Record exact commands and outcomes.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Matrix artifact exists and covers H1-H12.
- [x] All required focused evidence passes.
- [x] No zero-test pass is accepted.

## Completion Evidence

- Created `docs/plan/audits/TASK-882-proposition-acceptance-matrix.md` mapping SPEC-064 §12 H1-H12 to focused evidence, expected/actual result, command, test count, and owning earlier task evidence.
- Added four non-zero TASK-882 aggregator suites: `ash-core` summary versioning, `ash-parser` raw-surface/non-interference, `ash-typeck` H1-H8/H11 solver acceptance, and `ash-engine` V5 transport/non-interference.
- Ran TASK-882 non-zero guards: core 2, parser 3, typeck 5, engine 2 `task_882_` tests.
- Verification: `cargo test -p ash-core --test task_882_spec_h_summary_non_interference`; `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference`; `cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix`; `cargo test -p ash-engine --test task_882_spec_h_transport_non_interference`.
- Non-interference verification: TASK-872 typeck SPEC-035/SPEC-057-through-SPEC-063 suites passed with 101 tests; engine SPEC-062/SPEC-063 transport suites passed with 8 tests.
- Clean gates: `cargo fmt --check`; `git diff --check`; `cargo check --workspace`; `cargo clippy -p ash-core -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings`.
- Independent review verdict: PASS.

## Dispatch

```yaml
agent: hermes
reasoning: low
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - test -f docs/plan/audits/TASK-882-proposition-acceptance-matrix.md
  - test -f crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs
  - cargo test -p ash-core --test task_882_spec_h_summary_non_interference -- --list | grep -q task_882_
  - cargo test -p ash-core --test task_882_spec_h_summary_non_interference
  - test -f crates/ash-parser/tests/task_882_spec_h_surface_non_interference.rs
  - cargo test -p ash-parser --test task_882_spec_h_surface_non_interference -- --list | grep -q task_882_
  - cargo test -p ash-parser --test task_882_spec_h_surface_non_interference
  - test -f crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs
  - cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix -- --list | grep -q task_882_
  - cargo test -p ash-typeck --test task_882_spec_h_acceptance_matrix
  - test -f crates/ash-engine/tests/task_882_spec_h_transport_non_interference.rs
  - cargo test -p ash-engine --test task_882_spec_h_transport_non_interference -- --list | grep -q task_882_
  - cargo test -p ash-engine --test task_882_spec_h_transport_non_interference
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-882 for downstream tasks.
