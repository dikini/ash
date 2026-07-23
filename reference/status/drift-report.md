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
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-953-reference-corpus-closeout-and-drift-report.md
    - docs/plan/tasks/TASK-992-reference-slice-2-packet.md
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.status.feature_matrix
    - reference/status/alpha-limitations.md
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-075 changes
  - reference/getting-started/** changes
  - reference/tools/** changes
  - reference/runtime/** changes
  - reference/stdlib/** changes
  - reference/agents/** changes
  - reference/maintenance/** changes
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

## SPEC-075 Reference Slice 2 acceptance evidence

| ID | Status | Evidence |
| --- | --- | --- |
| A75-1 | complete | TASK-992 created DESIGN-043, SPEC-075, PLAN-125, TASK-992 through TASK-999, spec index, PLAN-INDEX Phase 130, and CHANGELOG links. |
| A75-2 | complete | TASK-993 created `reference/maintenance/` metadata, staleness-inspection, refresh, stale-doc triage, release, and agent-card procedures plus the path-based staleness checker. |
| A75-3 | complete | TASK-994 created `reference/getting-started/` pages that link into toolchain/runtime/stdlib details without duplicating subsystem authority. |
| A75-4 | complete | TASK-995 created Ash CLI and Ashgrove procedure pages for install, update, selectors, remove/cleanup, project dependencies, vendor/deploy, trust/signing, and source-payload policy with explicit non-goals. |
| A75-5 | complete | TASK-996 created RuntimeKernel concept/status pages for kernel, admission, artifacts, daemon, and policy profiles with integrity and authority caveats. |
| A75-6 | historical | TASK-997 created the older stdlib tower pages. Phase 201 now keeps those pages only as historical routing records while current guidance points to target functions, runtime admission, checked examples, and `Result`. |
| A75-7 | complete | TASK-998 created derivative agent cards and context-pack/common-confusion updates for stdlib, CLI/Ashgrove, and RuntimeKernel pages without forking canonical claims. |
| A75-8 | complete | TASK-999 closeout updates this drift report, verification evidence, feature matrix, limitations, maintenance status, plan/spec/task surfaces, CHANGELOG, and the `--slice reference-slice-2` staleness audit command. |

## Drift findings

- DRIFT-124-001: Older Act/Proc/Workflow specs and examples remain useful but can overclaim current alpha behavior if retrieved alone. Mitigation: current pages point to target authority, removed-form status, and checked examples first.
- DRIFT-124-002: Example executability is not uniformly tested by the new reference validator. Mitigation: examples are classified; broad execution validation remains outside Phase 124.
- DRIFT-124-003: Validator parses a controlled YAML subset instead of full YAML. Mitigation: SPEC-071-compatible pilot frontmatter uses the supported subset; future generated metadata may need a stronger parser or sidecar format.
- DRIFT-130-001: Reference Slice 2 remains docs/reference scope only. It does not change runtime/parser/typechecker/stdlib semantics.
- DRIFT-130-002: The staleness checker is path-based. It can identify pages whose declared evidence changed, but semantic freshness still requires human or agent review.
- DRIFT-130-003: Alpha limitations are centralized at [Alpha limitations](alpha-limitations.md). [Known limitations](known-limitations.md) remains as a retained Phase 124 alias for older links.

## Next-slice recommendation

Future reference slices should either automate deeper semantic checks or keep the current path-based inspection model explicit. Do not bulk-migrate the full `docs/` tree, do not over-promote reference pages above canonical-adjacent authority, and do not claim hosted registry, global install, production daemon, distributed scheduling, implicit tower lifts, or `Result`/operational-bottom behavior beyond the linked implementation evidence.
