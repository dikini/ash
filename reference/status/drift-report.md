---
id: ref.status.drift_report
title: Reference Drift Report
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 9fd1b8f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-953-reference-corpus-closeout-and-drift-report.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.status.feature_matrix
    - ref.status.known_limitations
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-075 changes
  - reference/runtime/** changes
  - Phase closeout changes reference policy
---

# Reference Drift Report

## SPEC-071 acceptance evidence

| ID | Status | Evidence |
| --- | --- | --- |
| R71-1 | complete | `reference/` skeleton exists with authority, methodology, style, and status indexes. |
| R71-2 | complete | `tools/reference/check_frontmatter.py --pilot` validates required metadata for pilot pages. |
| R71-3 | complete | Pilot pages exist under `reference/language/` and link to specs, code, tests, and examples. |
| R71-4 | complete | Agent cards in `reference/agents/cards/` link back via `canonical_page` and include warnings. |
| R71-5 | complete | Validator checks frontmatter, enum values, repo paths, internal IDs, and Markdown links for the pilot. |
| R71-6 | complete | `reference/examples/README.md` defines normative-pass, illustrative-pass, expected-fail, aspirational, historical, and reference-only. |
| R71-7 | complete | This drift report records caveats and next-slice recommendations without full-corpus migration claims. |

## Drift findings

- DRIFT-124-001: Older Act/Proc/Workflow specs and examples remain useful but can overclaim current alpha behavior if retrieved alone. Mitigation: pilot pages list SPEC-069/SPEC-070 and example labels first.
- DRIFT-124-002: Example executability is not uniformly tested by the new reference validator. Mitigation: examples are classified; broad execution validation remains outside Phase 124.
- DRIFT-124-003: Validator parses a controlled YAML subset instead of full YAML. Mitigation: SPEC-071-compatible pilot frontmatter uses the supported subset; future generated metadata may need a stronger parser or sidecar format.

## Next-slice recommendation

Continue Reference Slice 2 around the remaining stdlib pages, derivative agent cards, and closeout evidence. RuntimeKernel status pages now exist under TASK-996; do not bulk-migrate the full `docs/` tree.
