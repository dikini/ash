# TASK-993: Reference maintenance metadata and staleness substrate

## Status: ✅ Complete

## Description

Create the maintenance metadata and staleness-inspection substrate for Reference Slice 2 before bulk reference pages are written. This task owns `reference/maintenance/` and any small validator/checker updates required to make the metadata model inspectable.

## Specification Reference

- [DESIGN-043](../../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [SPEC-075](../../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- [PLAN-125](../PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
- PLAN-INDEX Phase 130

## Dependencies

- ✅ TASK-992: Reference Slice 2 packet

## Requirements

1. Create `reference/maintenance/README.md`.
2. Create `reference/maintenance/metadata-reference.md` defining the Slice 2 metadata semantics.
3. Create `reference/maintenance/staleness-inspection.md` defining diff-based inspection from `verified_against.git_commit`.
4. Create `reference/maintenance/refresh-procedure.md`.
5. Create `reference/maintenance/stale-doc-triage.md`.
6. Create `reference/maintenance/release-checklist.md`.
7. Create `reference/maintenance/agent-card-procedure.md`.
8. Update `reference/META.md`, `reference/methodology.md`, `reference/INDEX.md`, and status indexes to point to maintenance procedures without adding maintenance playbooks to ordinary pages.
9. Decide whether to extend `tools/reference/check_frontmatter.py` or add `tools/reference/check_staleness.py` in this task or leave automation to TASK-999 with a documented manual audit bridge.
10. Preserve SPEC-071 compatibility unless a validator/schema change is explicitly documented.

## Work Steps

1. Re-read SPEC-071, DESIGN-043, SPEC-075, and existing reference metadata pages.
2. Draft the maintenance pages first.
3. Make `metadata-reference.md` define `git_commit`, optional `release_tag`, optional `ash_version`, evidence paths, refresh triggers, declared status, and derived inspection state.
4. Make `staleness-inspection.md` define the `git diff --name-only <baseline>..HEAD` procedure and derived-state classification.
5. Update indexes and methodology with cross-links only.
6. If tooling changes are made, keep them deterministic and stdlib-only unless a later task explicitly changes tooling dependencies.

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
  - python3 -m py_compile tools/reference/check_staleness.py
  - python3 tools/reference/check_frontmatter.py --pilot
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_staleness.py --path reference/maintenance
  - |
    python3 - <<'PY'
    from pathlib import Path
    required = [
        'reference/maintenance/README.md',
        'reference/maintenance/metadata-reference.md',
        'reference/maintenance/staleness-inspection.md',
        'reference/maintenance/refresh-procedure.md',
        'reference/maintenance/stale-doc-triage.md',
        'reference/maintenance/release-checklist.md',
        'reference/maintenance/agent-card-procedure.md',
    ]
    missing = [p for p in required if not Path(p).exists()]
    assert not missing, missing
    text = Path('reference/maintenance/metadata-reference.md').read_text()
    for term in ['verified_against.git_commit', 'release_tag', 'ash_version', 'needs-inspection']:
        assert term in text, term
    inspect = Path('reference/maintenance/staleness-inspection.md').read_text()
    assert 'git diff --name-only' in inspect
    PY
checklist:
  - [x] Maintenance pages created.
  - [x] Metadata semantics define commit baseline and optional release/version fields.
  - [x] Staleness inspection defines derived `needs-inspection` state.
  - [x] Ordinary pages are not burdened with page-specific maintenance procedures.
  - [x] Validator/checker strategy is implemented or explicitly deferred to TASK-999.
```

## Tooling Strategy

TASK-993 adds `tools/reference/check_staleness.py` as a deterministic, stdlib-only path-diff inspector. It reports derived inspection states from `verified_against.git_commit`, evidence paths, and path-like refresh triggers. Semantic stale/partial/superseded decisions remain human or agent review work and are still reconciled at TASK-999 closeout.

## Dependencies for Next Task

TASK-994 through TASK-998 must follow the metadata/staleness conventions established here.
