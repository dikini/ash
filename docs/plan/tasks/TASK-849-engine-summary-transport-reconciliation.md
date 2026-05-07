# TASK-849: Engine summary transport reconciliation

## Status: 📝 Planned

## Description

Transport public computation summaries through engine module loading without making engine-private carriers semantic owners.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-848 completion

## Requirements

### Functional Requirements

1. Extend ModuleExports transport to carry core summaries with public type-function dimensions.
2. Update named/glob import selection to include public computation dependency closure.
3. Update summary merge/dedup keys to include sealed domains and type functions.
4. Keep parser capability metadata and engine-private ModuleExports non-normative.
5. Add engine tests for named import, glob import, and no sibling leakage.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-849 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-849 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
  - cargo test -p ash-engine --test task_849_type_computation_summary_transport -- --nocapture
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-849 complete until the subagent reports no blocking findings and the commands above pass.

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
  - cargo test -p ash-engine --test task_849_type_computation_summary_transport -- --nocapture
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- Provides imported public computation summaries for TypeEnv registration.
