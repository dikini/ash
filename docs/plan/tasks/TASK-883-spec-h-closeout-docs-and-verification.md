# TASK-883: SPEC-H closeout docs and verification

## Status: ✅ Complete

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

- [x] Docs/status/changelog are coherent.
- [x] Broad gates pass and evidence is recorded.
- [x] TASK-884 remains ready until independent review is complete.

## Completion Evidence

- Promoted `docs/spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md` and `docs/spec/README.md` from Draft to Implemented MVP after TASK-882 acceptance evidence and broad gates were available.
- Reconciled `docs/plan/PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md` and `docs/plan/PLAN-INDEX.md`: TASK-871 through TASK-883 are complete; TASK-884 remains ready for independent review/remediation.
- Fixed the stale TASK-879 engine transport regression fixture exposed by broad `cargo test --workspace`: public exported proposition requirements now use satisfied equality evidence, while unevidenced interface-bound requirements are expected to fail before transport.
- Focused regression after the fixture repair: `cargo test -p ash-engine --test task_879_proposition_summary_transport` passed with 4 tests.
- Broad verification passed after TASK-883 changes:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase116-doc.log`
  - `! grep -i '^warning:' /tmp/ash-phase116-doc.log`
- Scoped docs check passed over SPEC-064, spec index, PLAN-112, PLAN-INDEX, TASK-883, TASK-884, and the TASK-882 acceptance matrix: 7 files checked, 0 trailing-whitespace findings, 0 missing relative links.

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
