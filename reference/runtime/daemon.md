---
id: ref.runtime.daemon
title: Runtime Daemon Reference Target
kind: reference
audience: [human, agent]
authority: draft
status: draft
stability: alpha
owner: runtime
last_verified: 2026-06-01
verified_against:
  git_commit: 598a8f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    - crates/ash-cli/src/commands/daemon.rs
    - crates/ash-core/src/runtime_kernel.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
    - ref.runtime.kernel
  explains:
    - ref.getting_started.run_as_daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/commands/daemon.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - reference/runtime/daemon.md changes
---

# Runtime Daemon Reference Target

This draft page preserves a valid daemon-detail link for [Run as a local daemon](../getting-started/run-as-daemon.md). TASK-996 owns the complete daemon reference.

## Current Boundary

The Alpha daemon is local and RuntimeKernel-based. It does not provide a remote or multi-user API, distributed scheduling, production init-system integration, or hot-swapping for already-running instances.
