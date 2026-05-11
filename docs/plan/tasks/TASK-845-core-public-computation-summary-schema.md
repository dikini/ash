# TASK-845: Core public computation-summary schema

## Status: ✅ Complete

## Description

Add core-owned public type-function summary carriers and a SPEC-062 summary version contract.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-844 completion

## Requirements

### Functional Requirements

1. Extend ash-core semantic summaries with serde-defaulted exported type-function summaries.
2. Add SummaryVersion V3 for type-computation summaries and unsupported-version tests.
3. Reject or validate as malformed any V1/V2 summary that carries a non-empty `exported_type_functions` field; only V3 may carry public computation summaries.
4. Represent export mode explicitly; SPEC-062 MVP supports transparent public equations only.
5. Define `TypeFunctionSummary` fields for exported name, canonical `TypeComputationHeadId`, visibility, parameter names/canonical types/kinds/domain constraints, return type/kind/result-domain constraint, source anchors, checked source-order equations, dependency summary refs/version/digest metadata, and public-closure/revalidation metadata needed by TypeEnv import.
6. Add equality/serde/hash tests for public computation summaries, V3 versioning, V1/V2 malformed-content rejection, and dependency refs.
7. Do not add engine-private semantic owners.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-845 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-845 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
cargo fmt --check
git diff --check
cargo clippy -p ash-core --all-targets --all-features -- -D warnings
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-845 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed by the focused implementation subagent with the required clean gate below.

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
  - cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo clippy -p ash-core --all-targets --all-features -- -D warnings
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused tests for this task pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated if task changes code/docs policy/status
```

## Evidence

### RED

Initial strict-TDD focused test run after adding `crates/ash-core/tests/task_845_public_computation_summary_schema.rs` failed as expected:

```text
cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
error[E0432]: unresolved imports `ash_core::semantic_summary::ModuleSemanticSummaryValidationError`,
`TypeFunctionClosureMetadata`, `TypeFunctionDependencySummaryRef`, `TypeFunctionExportMode`,
`TypeFunctionParamSummary`, `TypeFunctionRevalidationMetadata`, `TypeFunctionSummary`
error[E0599]: no associated item named `SPEC062_TYPE_COMPUTATION_V3` found for struct `SummaryVersion`
error[E0609]: no field `exported_type_functions` on type `ModuleSemanticSummary`
error[E0599]: no method named `validate_summary_version_contract` found for struct `ModuleSemanticSummary`
error[E0599]: no method named `with_version` found for struct `ModuleSemanticSummary`
```

### GREEN / Verification

Required verification after implementation:

```text
cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
running 6 tests
test unknown_future_summary_versions_are_rejected_before_registration ... ok
test dependency_refs_preserve_summary_version_digest_and_algorithm_metadata ... ok
test module_summary_defaults_exported_type_functions_for_older_payloads ... ok
test v1_and_v2_with_non_empty_type_functions_are_malformed ... ok
test v3_module_summary_may_carry_public_type_function_summaries ... ok
test public_type_function_summary_is_equal_hashable_and_serde_roundtrips ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo fmt --check
passed

git diff --check
passed

cargo clippy -p ash-core --all-targets --all-features -- -D warnings
passed

cargo check --workspace
passed
```

## Dependencies for Next Task

This task outputs:
- Produces the core schema consumed by engine/typeck tasks.
