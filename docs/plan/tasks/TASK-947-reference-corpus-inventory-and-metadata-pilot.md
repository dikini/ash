# TASK-947: Reference corpus inventory and metadata pilot

## Status: 📝 Planned

## Description

Inventory a real pilot slice of the existing Ash corpus and test SPEC-071 metadata against actual artifacts before bulk authoring reference pages.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-946 completion

## Requirements

### Functional Requirements

1. Classify 20-30 artifacts spanning docs/spec, docs/design, docs/plan/tasks, std/src, examples, and code.
2. Record kind, authority, lifecycle status, health, owner subsystem, and source-of-truth links.
3. Identify friction points where SPEC-071 metadata does not fit.
4. Patch SPEC-071 only for resolved-now schema issues; record deferred issues in the inventory.
5. Recommend whether top-level `reference/` remains the name.

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
    candidates = list(Path('docs/plan/audits').glob('TASK-947-*')) + list(Path('reference/status').glob('*inventory*'))
    assert candidates, 'TASK-947 must create an inventory/friction artifact'
    text = '
'.join(p.read_text() for p in candidates if p.is_file())
    assert 'authority' in text and 'lifecycle' in text and 'friction' in text
    PY
checklist:
  - [ ] Documentation impact classified.
  - [ ] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [ ] New/changed links are scoped-checked.
  - [ ] Reference metadata and authority links are honest for this task's maturity.
```
