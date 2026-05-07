# TASK-851: TypeEnv imported head registration and normalizer integration

## Status: 📝 Planned

## Description

Batch-register imported public computation heads/equations and let the normalizer reduce downstream public applications.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-850 completion

## Requirements

### Functional Requirements

1. Replace local-only lookup with a unified computation-head lookup covering local, imported public, and fixture sources.
2. Add or use a batch `TypeEnv::register_module_semantic_summaries(...)`-style API that declares ordinary type identities, sealed-domain identities, interface/member identities, and public computation heads across all imported summaries before validating domains or equations.
3. Register imported transparent public equations only after batch identity declaration and summary closure validation.
4. Preserve neutral/stable results for abstract arguments and unsupported unavailable reductions.
5. Add downstream normalization and definitional equality tests.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-851 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-851 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
  - cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture
  - cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-851 complete until the subagent reports no blocking findings and the commands above pass.

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
  - cargo test -p ash-typeck --test task_851_imported_type_function_normalizer -- --nocapture
  - cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- First task that permits downstream cross-module public reduction.
