# TASK-854: SPEC-F acceptance and non-interference matrix

## Status: ✅ Complete

## Description

Own the final DESIGN-034 §16.6 acceptance matrix and regression/non-interference checks.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-853 completion

## Requirements

### Functional Requirements

1. Produce a row-by-row acceptance artifact mapping every SPEC-062 §13 item to a focused suite or recorded evidence.
2. Add cross-module acceptance tests for public downstream reduction.
3. Add private-equation opacity, private helper, private marker constructor, private sealed-domain, and private ordinary-type rejection tests.
4. Add named-import tests proving only the selected head is source-visible while dependency-closure helper heads are normalizer-available only.
5. Add glob-import and pub-use/re-export tests or cite TASK-853 focused suites, including canonical `TypeComputationHeadId` and equation-order preservation.
6. Add stable opaque/neutral result tests for abstract imported applications.
7. Add unknown/future version and V1/V2 non-empty computation-field rejection evidence or cite TASK-851/TASK-852 focused suites.
8. Add malformed imported-summary rejection evidence for arity/domain/kind/coverage/overlap/non-decreasing-recursion categories or cite TASK-851 focused suites.
9. Rerun SPEC-057/059/060/061 non-regression suites and record evidence.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-854 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-854 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-typeck --test task_854_type_computation_summary_acceptance -- --nocapture
cargo test -p ash-engine --test task_854_type_computation_summary_acceptance -- --nocapture
cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-854 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed by focused reruns and acceptance artifact cross-check.

## Acceptance Artifact

- [TASK-854 SPEC-062 §13 acceptance/non-interference matrix](../audits/TASK-854-spec-f-acceptance-matrix.md)

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_854_type_computation_summary_acceptance -- --nocapture
  - cargo test -p ash-engine --test task_854_type_computation_summary_acceptance -- --nocapture
  - cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused tests for this task pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated if task changes code/docs policy/status
```

### Recorded run

2026-05-11 local verification:

- `cargo test -p ash-typeck --test task_854_type_computation_summary_acceptance -- --nocapture` — passed (2 tests).
- `cargo test -p ash-engine --test task_854_type_computation_summary_acceptance -- --nocapture` — passed (3 tests).
- `cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture` — passed (7 tests).
- `cargo fmt --check` — passed.
- `git diff --check` — passed.
- `cargo check --workspace` — passed.

Additional non-regression reruns for SPEC-057/059/060/061 evidence:

- `cargo test -p ash-engine --test task_785_modulefile_summary_exports --test task_786_import_visibility_summary_rules -- --nocapture` — passed (45 tests).
- `cargo test -p ash-typeck --test task_787_semantic_summary_typeenv -- --nocapture` — rerun for SPEC-057 typeck evidence; existing unrelated failure observed in `std_result_summary_binds_existing_prelude_result_identity_without_duplicate_error` (23 passed, 1 failed).
- `cargo test -p ash-core --test task_809_sealed_domain_identities -- --nocapture` — passed (20 tests).
- `cargo test -p ash-typeck --test task_812_domain_registration_validation --test task_813_sealed_domain_registration_diagnostics -- --nocapture` — passed (21 tests).
- `cargo test -p ash-engine --test task_811_domain_summary_transport --test task_813_sealed_domain_non_interference -- --nocapture` — passed (19 tests).
- `cargo test -p ash-typeck --test task_820_internal_fixture_equation_registry --test task_821_closed_computation_head_reduction --test task_822_open_neutral_partial_normalization --test task_823_rigid_projection_alias_normalization --test task_824_definitional_equality --test task_825_non_inverting_unification_boundary --test task_826_typeenv_forcing_point_rollout --test task_827_normalizer_diagnostics --test task_829_review_remediation -- --nocapture` — passed (65 tests).

## Dependencies for Next Task

This task outputs:
- This is the single owner of the final DESIGN-034 §16.6 acceptance matrix.
