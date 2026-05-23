---
id: ref.status.verification_evidence
title: Reference Verification Evidence
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-953-reference-corpus-closeout-and-drift-report.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.status.drift_report
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Reference Verification Evidence

## Commands

The Phase 124 closeout used these focused commands:

- `git diff --check`
- `python3 -m py_compile tools/reference/check_frontmatter.py`
- `python3 tools/reference/check_frontmatter.py --pilot`
- inline existence checks from TASK-947 through TASK-953

## Acceptance mapping

R71-1 through R71-7 are mapped in [drift report](drift-report.md). The validator output is intentionally scoped to the pilot. No Rust code or public runtime behavior changed in this phase.
