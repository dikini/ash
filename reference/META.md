---
id: ref.meta
title: Reference Metadata Contract
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
    - ref.root
  explains:
    - ref.authority
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Reference Metadata Contract

Every Markdown page in `reference/` carries SPEC-071 frontmatter. Required fields identify the page, audience, authority class, lifecycle status, stability, owner, verification sources, related records, and refresh triggers.

Use repo-relative paths inside `verified_against`. Use `ref.*` IDs in `related` when pointing to reference pages. Use `historical_rationale` for old plans or design notes that explain why a feature exists but are not current authority.

The pilot validator is `tools/reference/check_frontmatter.py --pilot`.
