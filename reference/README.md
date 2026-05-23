---
id: ref.root
title: Ash Reference Corpus
kind: index
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
    []
  explains:
    - ref.index
    - ref.meta
    - ref.authority
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Ash Reference Corpus

The reference corpus is the curated reading surface for current Ash behavior. It does not replace `docs/`; specs, plans, tasks, audits, and design notes remain in `docs/` as the working and historical corpus.

Start with [INDEX](INDEX.md), then use [authority](authority.md), [methodology](methodology.md), and [style guide](style-guide.md) to judge scope and freshness.

Pilot coverage is intentionally narrow: functions, Act, Proc, Workflow, generalized do, matching agent cards, status pages, and example labels.
