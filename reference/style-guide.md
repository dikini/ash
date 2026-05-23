---
id: ref.style
title: Reference Style Guide
kind: style-guide
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
    - ref.methodology
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

# Reference Style Guide

Use concise, current, example-led prose. Prefer precise modality: `is`, `requires`, `currently`, `deferred`, `not in the pilot`. Avoid marketing language and broad identity claims.

Rules for examples:

- label executable evidence only when it is backed by tests or status classification;
- mark sketches as illustrative, aspirational, historical, or reference-only;
- do not rewrite old examples to make them look current.

Rules for agent material:

- link back to canonical reference pages;
- repeat only short retrieval cues and warnings;
- do not introduce semantics absent from human pages.
