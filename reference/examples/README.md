---
id: ref.examples.index
title: Reference Example Classification
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-07-07
verified_against:
  git_commit: phase-201-worktree
  specs: []
  tasks:
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code: []
  tests:
    - crates/ash-cli/tests/example_corpus_check.rs
  examples:
    - examples/10-testing-helpers/testing_helpers.ash
    - examples/11-process-channel-helpers/process_channel_helpers.ash
related:
  depends_on:
    - ref.status.feature_matrix
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale: []
refresh_trigger:
  - examples/README.md changes
  - crates/ash-cli/tests/example_corpus_check.rs changes
---

# Reference Example Classification

Phase 201 removed older workflow-era example paths from productive repository code. The current
checked examples are:

| Example | Label | Note |
| --- | --- | --- |
| `examples/10-testing-helpers/testing_helpers.ash` | current-pass | Productive testing-helper example. |
| `examples/11-process-channel-helpers/process_channel_helpers.ash` | current-pass | Productive process/channel helper example. |

Historical example names may appear in old plans or design documents as prose only. They are not
current examples and must not be copied into new Ash code.
