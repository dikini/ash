---
id: ref.stdlib.index
title: Historical Standard Library Tower Index
kind: index
audience: [human, agent]
authority: historical-summary
status: superseded
stability: alpha
slice: reference-slice-3
owner: stdlib
last_verified: 2026-07-07
verified_against:
  git_commit: phase-201-worktree
  release_tag: null
  ash_version: unreleased-alpha
  specs: []
  tasks:
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
  examples:
    - examples/10-testing-helpers/testing_helpers.ash
    - examples/11-process-channel-helpers/process_channel_helpers.ash
related:
  depends_on:
    - ref.index
  explains:
    - ref.stdlib.result
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/plan/PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md
refresh_trigger:
  - std/src/lib.ash changes
  - crates/ash-cli/tests/stdlib_corpus_check.rs changes
---

# Historical Standard Library Tower Index

This page is retained as historical orientation after Phase 201. The former public tower pages for
Act, Proc, and Workflow described stdlib source files and examples that are no longer productive
repository artifacts.

Current productive standard-library orientation is through checked target Ash files under `std/src`
and the current examples:

- `examples/10-testing-helpers/testing_helpers.ash`
- `examples/11-process-channel-helpers/process_channel_helpers.ash`

The former tower-carrier pages remain as prose-only historical records. They must not be treated as
current stdlib APIs or copied into new Ash source.
