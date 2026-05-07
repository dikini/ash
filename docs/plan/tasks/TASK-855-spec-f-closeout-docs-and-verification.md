# TASK-855: SPEC-F closeout docs and verification

## Status: 📝 Planned

## Description

Reconcile SPEC-062/PLAN-110/Phase 114 status and record broad verification evidence.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-854 completion

## Requirements

### Functional Requirements

1. Promote SPEC-062 status only after implementation and acceptance matrix pass.
2. Update PLAN-110, PLAN-INDEX, task files, docs/spec/README.md, and CHANGELOG.
3. Run scoped markdown link check over Phase 114 docs and CHANGELOG.
4. Run broad workspace fmt/check/clippy/test/doc gates and doc warning grep.
5. Do not mark complete if any background verification remains unresolved.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-855 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-855 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
  - git diff --check
  - cargo fmt --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-855 complete until the subagent reports no blocking findings and the commands above pass.

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
  - git diff --check
  - cargo fmt --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [ ] Implementation matches SPEC-062 and PLAN-110 scope
  - [ ] Focused tests for this task pass
  - [ ] Formatting and diff checks pass
  - [ ] CHANGELOG.md updated if task changes code/docs policy/status
```

## Dependencies for Next Task

This task outputs:
- Closeout is docs/status/evidence only unless verification exposes remediation.
