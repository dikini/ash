---
id: ref.root
title: Ash Reference Corpus
kind: index
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
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
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
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/getting-started/** changes
  - reference/stdlib/** changes
  - reference/status/** changes
  - Phase closeout changes reference policy
---

# Ash Reference Corpus

The reference corpus is the curated reading surface for current Ash behavior. It does not replace `docs/`; specs, plans, tasks, audits, and design notes remain in `docs/` as the working and historical corpus.

Start with [Getting started](getting-started/README.md) for the Alpha reader journey or [INDEX](INDEX.md) for the full reference map. Use [authority](authority.md), [methodology](methodology.md), and [style guide](style-guide.md) to judge scope and freshness.

Reference coverage remains intentionally scoped, but Reference Slice 2 is now closed for its planned Alpha manual slice. Current reader paths cover what Ash is, install, update, one-shot run, local daemon mode, cleanup, and next steps while linking to subsystem pages for exact behavior. The [standard library tower index](stdlib/README.md) covers the current `Act`, `Proc`, `Workflow`, and `Result` public library surfaces separately from the language concept pages; [Alpha limitations](status/alpha-limitations.md) records the no-overclaim boundaries.
