---
id: ref.maintenance.stale_triage
title: Reference Stale Document Triage
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
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.maintenance.metadata
    - ref.maintenance.staleness
  explains:
    - ref.status.drift_report
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/maintenance/** changes
  - reference/status/drift-report.md changes
  - reference/status/known-limitations.md changes
---

# Reference Stale Document Triage

## Summary

Triage turns a derived `needs-inspection` result or a reported mismatch into an explicit maintenance decision.

## Classification

| Result | Use when | Action |
| --- | --- | --- |
| `no-relevant-changes` | No evidence or trigger path changed. | Leave declared status unchanged. |
| `needs-inspection` | Relevant paths changed, but the page has not been reviewed. | Inspect before changing status. |
| `stale` | The page contradicts current evidence. | Fix immediately or set `status: stale` and record the issue. |
| `partial` | The page is accurate but incomplete or missing important caveats. | Set `status: partial` until expanded. |
| `superseded` | A newer reference page or authority replaces this page. | Set `status: superseded` and `related.superseded_by`. |

## Triage Procedure

1. Identify the exact claim under review.
2. Open the listed evidence and changed trigger paths.
3. Decide whether the claim is current, stale, partial, or superseded.
4. If the claim is current, refresh the page metadata.
5. If the claim is not current, update the page or record a drift finding.

## Drift Recording

Use [Reference Drift Report](../status/drift-report.md) for cross-page or deferred mismatches. For a simple same-task fix, updating the page and changelog entry is enough.

## Agent Notes

Do not mark a page stale just because `HEAD` moved. Mark it stale only after evidence contradicts a page claim.
