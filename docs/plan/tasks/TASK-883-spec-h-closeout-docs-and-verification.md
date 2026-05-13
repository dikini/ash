# TASK-883: SPEC-H closeout docs and verification

## Status: 🟡 Ready

## Description

Reconcile SPEC-064, PLAN-112, PLAN-INDEX, task statuses, spec index, CHANGELOG, and broad verification evidence after implementation.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-882 completion

## Files / Ownership

- Modify: `docs/spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md`
- Modify: `docs/spec/README.md`
- Modify: `docs/plan/PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-871-*.md` through `TASK-883-*.md`
- Modify: `CHANGELOG.md`

## Requirements

### Functional Requirements

1. Promote SPEC-064 status only if implementation and acceptance matrix are complete.
2. Update PLAN-112 and PLAN-INDEX task/status counts honestly.
3. Record focused evidence from TASK-882 and broad workspace gates.
4. Run scoped Markdown link/trailing-whitespace checks over Phase 116 docs.
5. Do not mark TASK-884 complete; reserve it for independent review remediation.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Reconcile statuses and changelog.

### Step 2

- Run broad gates: fmt, diff, check, clippy, test, doc warning grep.

### Step 3

- Run scoped docs checks.

### Step 4

- Record evidence in this task file.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Docs/status/changelog are coherent.
- [ ] Broad gates pass and evidence is recorded.
- [ ] TASK-884 remains ready until independent review is complete.

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
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase116-doc.log
  - ! grep -i '^warning:' /tmp/ash-phase116-doc.log
checklist:
  - "[ ] Task requirements are satisfied"
  - "[ ] Focused verification is recorded"
  - "[ ] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-883 for downstream tasks.
