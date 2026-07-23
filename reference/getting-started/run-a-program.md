---
id: ref.getting_started.run_a_program
title: Run a Program
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
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-engine/src/runtime_artifact.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.getting_started.index
    - ref.tools.cli
    - ref.runtime.kernel
    - ref.runtime.artifacts
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/commands/run.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - reference/runtime/** changes
  - reference/tools/cli.md changes
---

# Run a Program

Use `ash run FILE` for one-shot execution. This creates a RuntimeKernel host for the run, checks the selected `main` function artifact, admits one root application instance, runs it to a terminal outcome, and exits with an OS status.

CLI command detail belongs in [CLI tools](../tools/cli.md). Runtime identity, admission, and artifact behavior belong in [RuntimeKernel](../runtime/kernel.md) and [runtime artifacts](../runtime/artifacts.md).

## Current Boundaries

`ash run` does not require the daemon. File presence is not execution; the run path still needs a checked `main` function artifact and admission. Arbitrary non-`main` entry selection remains an Alpha caveat until the full selection path is wired through the implementation.
