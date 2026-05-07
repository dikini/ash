# TASK-843: SPEC-F spec/plan packet

## Status: ✅ Complete

## Description

Promote DESIGN-034 §16.6 into SPEC-062/PLAN-110, create Phase 114 task files, register status surfaces, and update CHANGELOG.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- ✅ TASK-842: Phase 113 review remediation (complete)

## Requirements

### Functional Requirements

1. Create SPEC-062 with normative public computation-summary export/import rules.
2. Create PLAN-110 with ordered task breakdown and verification gates.
3. Create TASK-843 through TASK-856 task files.
4. Register SPEC-062 in docs/spec/README.md and Phase 114 in PLAN-INDEX.
5. Record changelog entry for the planning packet.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-843 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-843 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
  - git diff --check
  - cargo fmt --check
  - scoped markdown link check over new/edited docs
  - cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-843 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is specified and assigned to TASK-847/TASK-852/TASK-854.
- [x] Status docs and CHANGELOG.md are updated for this docs/status packet.
- [x] Independent verification completed.

## Completion Evidence

- Created SPEC-062, PLAN-110, and TASK-843 through TASK-856 in the Phase 114 worktree.
- Registered SPEC-062 in `docs/spec/README.md` as Draft.
- Registered Phase 114 in both PLAN-INDEX progress tables and the detailed Phase 114 section.
- Ran `git diff --check`, `cargo fmt --check`, `cargo check --workspace`, and a scoped markdown-link/task-structure check over Phase 114 docs.
- Independent review completed; review findings around batch summary registration and task verification hygiene were remediated before commit.

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
  - scoped markdown link check over new/edited docs
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused docs/link/task-structure checks pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated for this docs/status packet
```

## Dependencies for Next Task

This task outputs:
- This task outputs the implementation-grade Phase 114 packet. No Rust implementation is owned by TASK-843.
