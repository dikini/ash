---
id: ref.tools.cli
title: Ash CLI Reference Target
kind: reference
audience: [human, agent]
authority: draft
status: draft
stability: alpha
owner: cli
last_verified: 2026-06-01
verified_against:
  git_commit: 598a8f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ash-cli/src/main.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.index
  explains:
    - ref.getting_started.run_a_program
    - ref.getting_started.run_as_daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/** changes
  - reference/tools/cli.md changes
---

# Ash CLI Reference Target

This draft page is a link target for the getting-started run and daemon pages. TASK-995 owns the complete command map.

## Current Pointers

- `ash run FILE[:WORKFLOW]` is the one-shot execution path. See [Run a program](../getting-started/run-a-program.md).
- `ash daemon ...` is the local daemon command family. See [Run as a local daemon](../getting-started/run-as-daemon.md) and [Runtime daemon](../runtime/daemon.md).

## Limitation

This page intentionally does not enumerate all CLI flags yet. It does not claim remote daemon support, production service management, or complete workflow-selection behavior.
