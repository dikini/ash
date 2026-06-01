---
id: ref.getting_started.run_as_daemon
title: Run as a Local Daemon
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
    - crates/ash-cli/src/commands/daemon.rs
    - crates/ash-core/src/runtime_kernel.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.getting_started.index
    - ref.tools.cli
    - ref.runtime.daemon
    - ref.runtime.kernel
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/commands/daemon.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - reference/runtime/daemon.md changes
  - reference/tools/cli.md changes
---

# Run as a Local Daemon

Use `ash daemon ...` for the local daemon surface. The daemon is a long-lived local host mode for the same RuntimeKernel semantics used by one-shot `ash run`.

CLI command detail belongs in [CLI tools](../tools/cli.md). Daemon behavior, reload scope, and local control-plane limits belong in [Runtime daemon](../runtime/daemon.md).

## Current Boundaries

The Alpha daemon is local-first. It is not a remote or multi-user API, not distributed scheduling, not production init-system integration, and not a hot-swap mechanism for already-running instances.
