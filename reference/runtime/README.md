---
id: ref.runtime.index
title: Runtime Reference Index
kind: index
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
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.runtime.kernel
    - ref.runtime.artifacts
    - ref.runtime.daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-cli/src/commands/** changes
  - reference/runtime/** changes
---

# Runtime Reference Index

This is a draft detail-target index created so the getting-started journey can link to runtime references. TASK-996 owns the complete RuntimeKernel pages.

## Current Targets

- [RuntimeKernel](kernel.md)
- [Runtime artifacts](artifacts.md)
- [Runtime daemon](daemon.md)

## Limitation

This page is not a complete runtime manual. It does not claim remote daemon support, distributed scheduling, production service management, provider-existence authority, or file-presence execution.
