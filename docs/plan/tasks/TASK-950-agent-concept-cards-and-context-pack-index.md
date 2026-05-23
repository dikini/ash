# TASK-950: Agent concept cards and context-pack index

## Status: 📝 Planned

## Description

Add AI-agent-facing derivatives for the pilot slice without forking semantic content from the reference pages.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-949 completion

## Requirements

### Functional Requirements

1. Create `reference/agents/README.md`, `reference/agents/context-pack-index.md`, `reference/agents/common-confusions.md`, and pilot cards under `reference/agents/cards/`.
2. Each card links to its canonical reference page.
3. Include retrieval tags, dependency order, common-confusion warnings, and must-check-before-editing links.
4. Add forbidden stale-claim entries for known traps.
5. Verify cards do not introduce semantics absent from linked pages.

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
    required = ['reference/agents/README.md', 'reference/agents/context-pack-index.md', 'reference/agents/common-confusions.md']
    for rel in required:
        assert Path(rel).exists(), rel
    cards = list(Path('reference/agents/cards').glob('*.yaml')) + list(Path('reference/agents/cards').glob('*.md'))
    assert cards, 'agent cards required'
    text = '
'.join(p.read_text() for p in cards)
    assert 'canonical_page' in text or 'canonical_pages' in text
    PY
checklist:
  - [ ] Documentation impact classified.
  - [ ] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [ ] New/changed links are scoped-checked.
  - [ ] Reference metadata and authority links are honest for this task's maturity.
```
