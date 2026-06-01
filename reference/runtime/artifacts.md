---
id: ref.runtime.artifacts
title: Runtime Artifacts Reference Target
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
    - crates/ash-engine/src/runtime_artifact.rs
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
    - ref.getting_started.run_a_program
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - reference/runtime/artifacts.md changes
---

# Runtime Artifacts Reference Target

This draft page preserves a valid artifact-detail link for [Run a program](../getting-started/run-a-program.md). TASK-996 owns the complete runtime artifact reference.

## Current Boundary

Artifact identity and verification matter for run and daemon behavior. This placeholder does not define the full artifact lifecycle or cache policy.
