---
id: ref.status.reference_maintenance
title: Reference Maintenance Status
kind: status
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
    - check_staleness reference-slice-2 audit
  examples:
    []
related:
  depends_on:
    - ref.status.index
    - ref.maintenance.index
  explains:
    - ref.maintenance.metadata
    - ref.maintenance.staleness
    - ref.maintenance.refresh
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/maintenance/** changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
---

# Reference Maintenance Status

TASK-993 establishes the Phase 130 maintenance substrate:

- [Maintenance index](../maintenance/README.md)
- [Metadata reference](../maintenance/metadata-reference.md)
- [Staleness inspection](../maintenance/staleness-inspection.md)
- [Refresh procedure](../maintenance/refresh-procedure.md)
- [Stale document triage](../maintenance/stale-doc-triage.md)
- [Release checklist](../maintenance/release-checklist.md)
- [Agent-card procedure](../maintenance/agent-card-procedure.md)

TASK-999 closeout adds the named `reference-slice-2` staleness scope to the checker. The named scope covers the SPEC-075 minimum Slice 2 pages plus touched root/status/agent indexes, language cross-link pages, compatibility limitation surfaces, and TASK-998 context-pack/common-confusion surfaces, while preserving the existing `--path` behavior for ad hoc audits.

Automation is intentionally path-based. `needs-inspection` is derived from changed evidence or refresh-trigger paths and remains separate from declared page status. Semantic stale, partial, and superseded decisions still require human or agent review against the canonical specs and implementation evidence.
