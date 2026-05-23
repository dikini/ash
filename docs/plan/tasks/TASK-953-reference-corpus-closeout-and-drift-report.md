# TASK-953: Reference corpus closeout and drift report

## Status: 📝 Planned

## Description

Close out the PLAN-120 pilot by reconciling SPEC-071 acceptance criteria, validator evidence, drift findings, and next-slice recommendations.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-947 completion
- TASK-948 completion
- TASK-949 completion
- TASK-950 completion
- TASK-951 completion
- TASK-952 completion

## Requirements

### Functional Requirements

1. Map SPEC-071 R71-1 through R71-7 to concrete evidence.
2. Create or update `reference/status/drift-report.md` and `reference/status/verification-evidence.md`.
3. Update PLAN-120, PLAN-INDEX, task statuses, spec index, and CHANGELOG only where evidence supports promotion.
4. Run broad docs/tooling verification and focused validator checks.
5. Obtain independent review focused on overclaiming, stale authority links, and agent/human semantic divergence.

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
  - python3 tools/reference/check_frontmatter.py --pilot
  - python3 - <<'PY'
    from pathlib import Path
    required = ['reference/status/drift-report.md', 'reference/status/verification-evidence.md']
    for rel in required:
        p = Path(rel)
        assert p.exists(), rel
    text = Path('reference/status/drift-report.md').read_text()
    assert 'R71-1' in text and 'R71-7' in text
    PY
checklist:
  - [ ] Documentation impact classified.
  - [ ] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [ ] New/changed links are scoped-checked.
  - [ ] Reference metadata and authority links are honest for this task's maturity.
```

Closeout verification must also run any final reference validator command, scoped markdown-link checks over `reference/`, and broad workspace gates only if code/Rust/public tooling changed.
