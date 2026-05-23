---
id: ref.authority
title: Reference Authority Model
kind: guide
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
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.meta
  explains:
    - ref.status.drift_report
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Reference Authority Model

Authority is typed. A reference page may summarize a spec, code path, example, test, limitation, or historical note, but it must not merge those roles.

Default precedence for current behavior:

1. live code and passing tests;
2. current implemented-MVP specs;
3. reference pages as curated explanation;
4. plans, tasks, and audits as implementation history;
5. design notes as rationale;
6. changelog entries as release-facing history.

If sources disagree, record drift in [drift-report](status/drift-report.md). Do not silently choose the more convenient claim.
