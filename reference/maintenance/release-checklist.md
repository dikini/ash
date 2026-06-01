---
id: ref.maintenance.release_checklist
title: Reference Release Checklist
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 4fa1eba
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    - tools/reference/check_frontmatter.py
    - tools/reference/check_staleness.py
  tests:
    - check_frontmatter full reference validation
    - check_staleness maintenance path audit
  examples:
    []
related:
  depends_on:
    - ref.maintenance.metadata
    - ref.maintenance.refresh
    - ref.status.index
  explains:
    - ref.status.reference_maintenance
    - ref.status.verification_evidence
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/status/** changes
  - reference/maintenance/** changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
---

# Reference Release Checklist

## Summary

Use this checklist at phase closeout or release-like documentation boundaries. It keeps reference freshness, status pages, and agent derivatives synchronized.

## Checklist

- Every required reference page for the phase exists or has an accepted scope note.
- Every changed reference page has valid SPEC-071 frontmatter.
- Every closeout page has non-`unknown` `verified_against.git_commit`.
- Staleness inspection has been run for the release scope.
- `reference/status/verification-evidence.md` records the relevant verification commands and results.
- `reference/status/drift-report.md` records unresolved mismatches.
- `reference/status/feature-matrix.md` and limitation pages reflect user-visible status changes.
- Agent cards link back to canonical pages and do not fork semantic claims.
- `CHANGELOG.md`, plan rows, and task checklists match the verified state.

## Commands

Minimum reference checks:

```bash
git diff --check
python3 -m py_compile tools/reference/check_frontmatter.py tools/reference/check_staleness.py
python3 tools/reference/check_frontmatter.py
python3 tools/reference/check_staleness.py --path reference
```

Phase tasks may add stricter commands.

## Agent Notes

Do not use this checklist to claim complete runtime or language coverage. It checks the declared reference scope for the phase.
