---
id: ref.runtime.daemon
title: Local Runtime Daemon
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: runtime
last_verified: 2026-06-01
verified_against:
  git_commit: 9fd1b8f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    - crates/ash-cli/src/commands/daemon.rs
    - crates/ash-core/src/runtime_kernel.rs
  tests:
    - crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs
    - crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
    - ref.runtime.kernel
  explains:
    - ref.getting_started.run_as_daemon
    - ref.runtime.artifacts
    - ref.runtime.admission
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/commands/daemon.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - reference/runtime/daemon.md changes
---

# Local Runtime Daemon

The Alpha daemon is the long-lived local host mode for the same `RuntimeKernel` semantics used by one-shot `ash run`. It is controlled through `ash daemon ...`.

## Current Role

The daemon can:

- serve a same-user local control endpoint;
- index checked function artifacts under a root without executing them;
- list definitions and instance records;
- start admitted application instance records pinned to the active artifact/source identity;
- report instance status;
- request cancellation;
- reload roots/config for future starts.

File presence does not execute code. The daemon may know about a checked function artifact while no application instance is running.

## Local Control Boundary

The current control surface is local-first. The implementation validates root/socket-parent/state/cache/log directory ownership and rejects unsafe group/world-writable or non-socket control paths before binding or stale-socket cleanup.

This is an Alpha same-user local-control design. It is not a remote daemon protocol and not a multi-user service boundary.

## Start, Admission, and Config

Daemon start requests record args, `config_id`, and `admission_profile` fields. The default `config_id` is supported. Non-default daemon start config IDs are rejected before instance recording until config-specific daemon artifacts exist.

Admission happens before user body execution. A rejecting admission profile fails without recording a running instance and without using provider/resource inventory as authority.

## Reload

Reload stages new definitions and artifact summaries. Successful reload affects future starts. Failed reload preserves the previous valid index.

Reload is not hot reload. Already admitted running instances keep their admitted artifact/source identity and are not hot-swapped to newly indexed artifacts.

## Non-Goals

The current daemon does not provide:

- remote or multi-user daemon API;
- distributed scheduling;
- production init-system integration;
- cluster service discovery;
- hot-swapping for already-running instances;
- production report/log-path projection beyond current Alpha evidence.
