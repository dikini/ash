# TASK-853: Import-order and re-export determinism

## Status: ✅ Complete

## Description

Prove named/glob/pub-use imports and re-exports preserve canonical identities and deterministic normal forms.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-852 completion

## Requirements

### Functional Requirements

1. Add import-order permutation tests for summaries that reference each other through public sealed-domain fields and public type-function equations; the test must fail under one-at-a-time source-order summary registration.
2. Test pub-use re-export preserves original TypeComputationHeadId and equation order.
3. Test repeated imports are idempotent.
4. Test named import does not expose unrelated sibling heads and does not make dependency-closure helper heads source-visible unless selected; helper heads may remain normalizer-available by canonical ID.
5. Test glob import imports all public heads deterministically.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-853 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-853 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-engine --test task_853_type_computation_import_order -- --nocapture
cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-853 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed.

## Verification Evidence

Completed focused verification for TASK-853:

```bash
cargo test -p ash-engine --test task_853_type_computation_import_order -- --nocapture
cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Initial independent review found engine batch-registration, selected-summary compatibility, and named-import visibility/leakage proof gaps. Remediation batch-registers imported summaries in engine type-check paths, refuses to merge selected summary subsets with conflicting overlapping identity facts, extends named-import leakage coverage, and passed independent re-review with no remaining blockers.

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
  - cargo test -p ash-engine --test task_853_type_computation_import_order -- --nocapture
  - cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture
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
- Feeds final acceptance matrix.
