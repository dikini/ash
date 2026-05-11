# TASK-848: Transparent public equation summary lowering

## Status: ✅ Complete

## Description

Lower export-closed public equations into public computation summaries without losing source order or canonical IDs.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-847 completion

## Requirements

### Functional Requirements

1. Emit public type-function summaries for transparent public definitions.
2. Preserve checked source equation order and canonical TypeComputationHeadId.
3. Include public dependency closure metadata for sealed domains, ordinary types, projections, public helper heads, and dependency summary refs/version/digest inputs required by SPEC-062.
4. Reject or diagnose summary emission when closure is incomplete.
5. Preserve helper-head dependency closure as summary metadata for normalizer availability; do not imply source-visible aliases for helpers.
6. Add core/typeck tests for summary payloads and non-public non-emission.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-848 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-848 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-typeck --test task_848_public_equation_summary_lowering -- --nocapture
cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-848 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed.

## Completion Notes

- Added `TypeEnv::export_public_type_function_summaries`, lowering already-validated public local type functions into `ash_core::semantic_summary::TypeFunctionSummary` values with `TransparentEquations` mode.
- Preserves canonical `TypeComputationHeadId`, checked equation vector/ordinals, source anchors, parameter/return metadata, transitive public helper-head closure counts, revalidation metadata, and dependency summary ref placeholders with live module/version metadata.
- Private/non-public local type functions are filtered out; imported summary registration, engine transport, and downstream normalizer lookup remain TASK-849/TASK-851 scope.

## Verification Evidence

```text
cargo test -p ash-typeck --test task_848_public_equation_summary_lowering -- --nocapture
cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

All commands passed locally for this task implementation after the transitive helper-closure remediation (`task_848_public_equation_summary_lowering`: 3 passed; `task_845_public_computation_summary_schema`: 6 passed). Independent remediation review reported no blocking or nonblocking findings.

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
  - cargo test -p ash-typeck --test task_848_public_equation_summary_lowering -- --nocapture
  - cargo test -p ash-core --test task_845_public_computation_summary_schema -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused tests for this task pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- Produces summaries transported by TASK-849.
