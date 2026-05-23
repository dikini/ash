# TASK-949: Pure/Act/Proc/Workflow reference pilot

## Status: 📝 Planned

## Description

Write the first semantic pilot reference pages for the Ash tower: Pure, Act, Proc, Workflow, and generalized do.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-948 completion

## Requirements

### Functional Requirements

1. Create pilot pages under `reference/language/`.
2. Link each page to current specs, code paths, stdlib modules, examples, tasks, and limitations.
3. Distinguish current implemented MVP behavior from historical design notes.
4. Include common confusions such as Act not being Result and no implicit tower lifts.
5. Add provisional example labels only where evidence is already known; otherwise mark examples `classification-pending` and link to TASK-952.

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
    required = ['reference/language/functions.md', 'reference/language/effects-act.md', 'reference/language/processes-proc.md', 'reference/language/workflows.md', 'reference/language/generalized-do.md']
    for rel in required:
        p = Path(rel)
        assert p.exists(), rel
        text = p.read_text()
        for phrase in ['Known limitations', 'Common confusions', 'Authority and traceability', 'Agent notes']:
            assert phrase in text, f'{rel} missing {phrase}'
    PY
checklist:
  - [ ] Documentation impact classified.
  - [ ] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [ ] New/changed links are scoped-checked.
  - [ ] Reference metadata and authority links are honest for this task's maturity.
```
