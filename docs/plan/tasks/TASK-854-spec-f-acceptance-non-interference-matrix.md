# TASK-854: SPEC-F acceptance and non-interference matrix

## Status: 📝 Planned

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

1. Add cross-module acceptance tests for public downstream reduction.
2. Add private-equation opacity and private-helper rejection tests.
3. Add import-order independence tests or cite TASK-853 focused suite.
4. Add stable opaque neutral result tests.
5. Rerun SPEC-057/059/060/061 non-regression suites and record evidence.

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
  - cargo test -p ash-typeck --test task_854_type_computation_summary_acceptance -- --nocapture
  - cargo test -p ash-engine --test task_854_type_computation_summary_acceptance -- --nocapture
  - cargo test -p ash-typeck --test task_840_type_function_acceptance -- --nocapture
  - cargo fmt --check
  - cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-854 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [ ] Requirements above are satisfied.
- [ ] Focused tests exist and pass, or docs-only verification is recorded.
- [ ] Negative leakage/private-opacity behavior is tested where applicable.
- [ ] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [ ] Independent verification completed.

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
  - cargo check --workspace
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- This is the single owner of the final DESIGN-034 §16.6 acceptance matrix.
