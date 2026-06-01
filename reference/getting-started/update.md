---
id: ref.getting_started.update
title: Update Ash
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 598a8f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.getting_started.index
    - ref.tools.ashgrove.update
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/update.md changes
---

# Update Ash

Ashgrove updates install a new immutable toolchain and switch selectors only when requested. They do not rewrite installed toolchain contents in place, and they do not update the standard library independently from the selected toolchain.

Use the [Ashgrove update detail page](../tools/ashgrove/update.md) for exact command forms and selector behavior.

## Current Boundaries

The Alpha model is local and evidence-bound. Hosted update channels, unsigned release-index lookup, OS package-manager updates, and arbitrary SemVer solving are not part of the current user journey.
