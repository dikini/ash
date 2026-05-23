# TASK-952: Reference examples and status classification

## Status: ✅ Complete

## Description

Classify cited pilot examples, known limitations, and feature status entries so reference pages do not overclaim executable coverage. Use the `ash-documentation-style-guide` skill for documentation tone and style.

## Specification Reference

- [DESIGN-042](../../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [PLAN-120](../PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
- PLAN-INDEX Phase 124

## Dependencies

- TASK-949 completion
- TASK-951 completion

## Requirements

### Functional Requirements

1. Create or update `reference/status/feature-matrix.md`, `reference/status/known-limitations.md`, and `reference/examples/README.md`.
2. Classify cited examples as normative-pass, illustrative-pass, expected-fail, aspirational, historical, or reference-only.
3. Link each pilot reference page to relevant status entries.
4. Record any mismatches between example status and current docs/spec claims as drift findings.
5. Do not silently fix historical examples by rewriting their intent.

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
    required = ['reference/status/feature-matrix.md', 'reference/status/known-limitations.md', 'reference/examples/README.md']
    for rel in required:
        p = Path(rel)
        assert p.exists(), rel
    text = Path('reference/examples/README.md').read_text()
    for label in ['normative-pass', 'illustrative-pass', 'expected-fail', 'aspirational', 'historical', 'reference-only']:
        assert label in text, label
    PY
checklist:
  - [x] Documentation impact classified.
  - [x] CHANGELOG.md updated if docs policy or release-facing status changed.
  - [x] New/changed links are scoped-checked.
  - [x] Reference metadata and authority links are honest for this task's maturity.
```


## Completion Evidence

Completed in Phase 124. Focused evidence is recorded in `reference/status/verification-evidence.md`; drift and remaining limits are recorded in `reference/status/drift-report.md`. The pilot validator passed with `python3 tools/reference/check_frontmatter.py --pilot`.
