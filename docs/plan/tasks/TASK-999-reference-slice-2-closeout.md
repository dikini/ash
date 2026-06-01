# TASK-999: Reference Slice 2 closeout

## Status: 📝 Planned

## Description

Close out Reference Slice 2 by validating every new page, metadata surface, status page, agent card, drift report, feature matrix, and verification evidence surface. This task owns final docs hygiene and independent review.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130

## Dependencies

- 📝 TASK-993: Reference maintenance metadata and staleness substrate completion
- 📝 TASK-994: Reference getting-started journey
- 📝 TASK-995: Reference Ashgrove and CLI procedures
- 📝 TASK-996: Reference RuntimeKernel pages
- 📝 TASK-997: Reference stdlib tower pages
- 📝 TASK-998: Reference agent cards and context pack

## Requirements

1. Verify all SPEC-075 A75-1 through A75-8 acceptance rows.
2. Update `reference/status/drift-report.md`.
3. Update `reference/status/verification-evidence.md`.
4. Update `reference/status/feature-matrix.md`.
5. Update `reference/status/alpha-limitations.md` and reconcile `reference/status/known-limitations.md` if the existing page remains part of the status index.
6. Update `reference/status/reference-maintenance.md`.
7. Update `docs/spec/README.md`, `docs/plan/PLAN-125-...`, `docs/plan/PLAN-INDEX.md`, task files, and `CHANGELOG.md` only after evidence supports completion.
8. Run an independent review focused on overclaiming, metadata sufficiency, stale links, and maintenance-procedure usability.
9. Run the final verification gates.

## Work Steps

1. Build a checklist from SPEC-075 A75-1 through A75-8.
2. Run the reference validator in full and pilot mode.
3. Run or perform the Slice 2 staleness inspection audit over all expanded pages.
4. Reconcile drift/status/feature/verification pages.
5. Update PLAN-125, PLAN-INDEX, task statuses, SPEC-075 status, spec README, and CHANGELOG only after verification.
6. Request independent review; patch findings; rerun checks.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -m py_compile tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - python3 tools/reference/check_staleness.py --slice reference-slice-2
  - cargo fmt --all --check
  - |
    python3 - <<'PY'
    from pathlib import Path
    required_phrases = {
        'reference/status/drift-report.md': ['Reference Slice 2', 'A75'],
        'reference/status/verification-evidence.md': ['Reference Slice 2', 'A75'],
        'reference/status/feature-matrix.md': ['Reference Slice 2'],
    }
    for rel, phrases in required_phrases.items():
        text = Path(rel).read_text()
        for phrase in phrases:
            assert phrase in text, f'{rel}: {phrase}'
    idx = Path('docs/plan/PLAN-INDEX.md').read_text()
    assert 'Phase 130' in idx
    spec = Path('docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md').read_text()
    assert 'Implemented MVP' in spec or 'Accepted' in spec
    PY
checklist:
  - [ ] A75-1 through A75-8 mapped to evidence.
  - [ ] Full reference validator passes.
  - [ ] Pilot reference validator still passes.
  - [ ] Staleness inspection audit completed.
  - [ ] Drift/status/feature/verification pages updated.
  - [ ] Independent review completed and findings addressed.
  - [ ] PLAN-INDEX, PLAN-125, SPEC-075, task files, spec README, and CHANGELOG reconciled.
```

## Notes

If `tools/reference/check_staleness.py` is not implemented by closeout, replace that command with a documented manual staleness-inspection audit over every Slice 2 page and update this task before marking it complete. Do not leave a failing or nonexistent closeout command in a completed task.
