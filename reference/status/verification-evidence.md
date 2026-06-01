---
id: ref.status.verification_evidence
title: Reference Verification Evidence
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 01bafb4
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
    - tools/reference/check_frontmatter.py
    - tools/reference/check_staleness.py
  tests:
    - py_compile reference checkers
    - frontmatter full reference validation
    - frontmatter pilot validation
    - staleness reference-slice-2 audit
  examples:
    []
related:
  depends_on:
    - ref.status.drift_report
    - ref.status.feature_matrix
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-075 changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
  - reference/status/** changes
  - Phase closeout changes reference policy
---

# Reference Verification Evidence

## Commands

The Phase 124 closeout used these focused commands:

- `git diff --check`
- `python3 -m py_compile tools/reference/check_frontmatter.py`
- `python3 tools/reference/check_frontmatter.py --pilot`
- inline existence checks from TASK-947 through TASK-953

The Reference Slice 2 TASK-999 closeout evidence adds these commands:

- `git diff --check`
- `python3 -m py_compile tools/reference/check_frontmatter.py tools/reference/check_staleness.py`
- `python3 tools/reference/check_frontmatter.py`
- `python3 tools/reference/check_frontmatter.py --pilot`
- `python3 tools/reference/check_staleness.py --slice reference-slice-2`
- `cargo fmt --all --check`
- TASK-999 inline assertion for drift-report, verification-evidence, feature-matrix, PLAN-INDEX Phase 130, and SPEC-075 status wording.

Closeout staleness audit result recorded during TASK-999: `python3 tools/reference/check_staleness.py --slice reference-slice-2` completed successfully. The `needs-inspection` rows are expected for evidence-bound Slice 2 pages whose verification baselines predate their task-owned page groups; TASK-999 maps those rows to A75 evidence in the drift report instead of treating the path-based checker as semantic freshness proof.

## Acceptance mapping

R71-1 through R71-7 and SPEC-075 A75-1 through A75-8 are mapped in [drift report](drift-report.md).

The Reference Slice 2 validator evidence is intentionally docs/reference scoped. The full frontmatter validator checks every Markdown page under `reference/`; pilot mode remains available for the Phase 124 required page set. The staleness inspector is stdlib-only and path-based; `--slice reference-slice-2` audits the SPEC-075 Slice 2 page set plus touched Slice 2 index, language cross-link, compatibility limitation, and agent surfaces. It does not prove semantic freshness by itself.

No Ash runtime, parser, typechecker, or stdlib semantics changed for TASK-999.
