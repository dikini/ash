---
id: ref.getting_started.index
title: Getting Started with Ash
kind: index
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
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    []
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.getting_started.what_is_ash
    - ref.getting_started.install
    - ref.getting_started.update
    - ref.getting_started.run_a_program
    - ref.getting_started.run_as_daemon
    - ref.getting_started.cleanup
    - ref.getting_started.next_steps
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - docs/plan/tasks/TASK-994-reference-getting-started-journey.md changes
  - reference/getting-started/** changes
---

# Getting Started with Ash

This journey is the thin entry path for the current Alpha reference. It orients readers, then sends them to subsystem pages for exact tool, runtime, and language details.

Ash is framed here as:

- Transform with Pure.
- Effect with Act/Proc.
- Orchestrate with Workflow.

## Path

1. [What is Ash?](what-is-ash.md)
2. [Install Ash](install.md)
3. [Update Ash](update.md)
4. [Run a program](run-a-program.md)
5. [Run as a local daemon](run-as-daemon.md)
6. [Clean up local Ash state](cleanup.md)
7. [Next steps](next-steps.md)

## Detail Pages

Install, update, and cleanup details belong to [Ashgrove](../tools/ashgrove.md). CLI and RuntimeKernel details belong to [CLI tools](../tools/cli.md) and [runtime](../runtime/README.md).

This journey does not define hosted registries, OS package-manager installs, remote daemon deployment, or production service management. Those are future scope unless a later reference page links to implementation evidence.
