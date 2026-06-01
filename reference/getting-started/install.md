---
id: ref.getting_started.install
title: Install Ash
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
    - ref.tools.ashgrove.install
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/install.md changes
---

# Install Ash

Ash Alpha installation is managed by `ashgrove`, the user-local toolchain manager. Installing Ash means installing a coherent toolchain bundle: `ash`, `ashgrove`, the selected standard library, runtime-support metadata, and install metadata.

Use the [Ashgrove install detail page](../tools/ashgrove/install.md) for exact command forms and current limitations.

## Supported Shape

The current Alpha install model supports source and tarball based installs through Ashgrove. Bare hosted-version lookup is not a supported install path until a later authenticated release-index policy exists.

## Not Current Scope

Do not assume hosted registry lookup, OS package-manager integration, global/system install roots, or independent standard-library updates. Those are future scope unless the Ashgrove reference links to implementation evidence.
