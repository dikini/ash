---
id: ref.maintenance.refresh
title: Reference Refresh Procedure
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
    - ref.maintenance.staleness
  explains:
    - ref.maintenance.stale_triage
    - ref.maintenance.release_checklist
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
  - reference/maintenance/** changes
---

# Reference Refresh Procedure

## Summary

Refreshing a reference page means rechecking its evidence, updating content when needed, and advancing verification metadata only after the check is complete.

## Procedure

1. Run staleness inspection for the page or page group.
2. Re-read every changed evidence source and every applicable refresh trigger.
3. Compare the page's user-facing claims, examples, limitations, and agent notes against current evidence.
4. Fix the page, or classify it as `stale`, `partial`, or `superseded`.
5. Update `verified_against` lists if evidence ownership changed.
6. Set `last_verified` to the inspection date and `verified_against.git_commit` to the checked commit.
7. Run the reference validators.

## Metadata Rules

Do not advance `last_verified` or `verified_against.git_commit` for prose-only edits unless the page evidence was rechecked. Do not remove evidence paths merely to avoid a `needs-inspection` result.

Keep `release_tag` and `ash_version` advisory. Before Alpha tags exist, `verified_against.git_commit` remains the strongest freshness anchor.

## Validation

Use:

```bash
python3 tools/reference/check_frontmatter.py
python3 tools/reference/check_staleness.py --path reference/maintenance
```

Run narrower or broader paths as appropriate for the refresh scope.

## Agent Notes

When a page is refreshed because a semantic claim changed, check any agent card that derives from it. Agent cards must not keep older wording after the canonical page changes.
