---
id: ref.runtime.kernel
title: RuntimeKernel Reference Target
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
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-engine/src/runtime_artifact.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
  explains:
    - ref.getting_started.run_a_program
    - ref.getting_started.run_as_daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - reference/runtime/kernel.md changes
---

# RuntimeKernel Reference Target

This draft page preserves a valid RuntimeKernel link for the getting-started journey. TASK-996 owns the complete RuntimeKernel concept and status reference.

## Current Boundary

RuntimeKernel is the Alpha execution host abstraction used by one-shot `ash run` and local daemon host modes. Authority comes from admission, not from provider existence or file presence.
