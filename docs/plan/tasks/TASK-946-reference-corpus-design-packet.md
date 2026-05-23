# TASK-946: Reference corpus design packet

## Status: ✅ Complete

## Description

Create the initial documentation architecture packet: DESIGN-042, SPEC-071, PLAN-120, Phase 124 PLAN-INDEX entry, task files, spec index row, and changelog entry. Use the `ash-documentation-style-guide` skill for documentation tone and style.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- User approval to create the design packet.

## Requirements

### Functional Requirements

1. Create DESIGN-042 with the two-corpus model and tone/methodology principles.
2. Create SPEC-071 with metadata, authority, crosslinking, maintenance, and acceptance rules.
3. Create PLAN-120 with TASK-946 through TASK-953.
4. Create concrete task files for TASK-946 through TASK-953.
5. Update docs/spec/README.md, docs/plan/PLAN-INDEX.md, and CHANGELOG.md.

### Non-goals

- Do not rewrite or move the whole `docs/` corpus.
- Do not create a dynamic wiki/service unless a later phase explicitly owns it.
- Do not duplicate independent semantics for AI-agent material.

## TDD / Work Steps

1. Re-read DESIGN-042, SPEC-071, and PLAN-120 before editing.
2. Make the smallest documentation/tooling change that satisfies this task.
3. Run the focused verification commands listed below.
4. Record any drift or intentionally deferred work instead of overclaiming.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

Codex instructions:

- Work in a dedicated worktree.
- Do not spawn nested agents.
- Keep this task's scope narrow.
- Return exact files changed, commands run, and remaining blockers.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 - <<'PY'
    from pathlib import Path
    required = ['docs/design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md', 'docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md', 'docs/plan/PLAN-120-REFERENCE-CORPUS-ROLLOUT.md'] + [f'docs/plan/tasks/TASK-{n}-' for n in range(946, 954)]
    for rel in required[:3]:
        assert Path(rel).exists(), rel
    for prefix in required[3:]:
        assert list(Path('docs/plan/tasks').glob(Path(prefix).name + '*.md')), prefix
    PY
checklist:
  - [x] Documentation impact classified as docs-policy/reference-governance packet.
  - [x] CHANGELOG.md updated for docs-policy and release-facing planning status.
  - [x] New/changed links are scoped-checked.
  - [x] Reference metadata and authority links are honest for packet maturity.
```

Packet verification performed during creation must include file-existence checks for all created files, new-ID searches, `git diff --check`, and scoped link checks for newly created/changed files.
