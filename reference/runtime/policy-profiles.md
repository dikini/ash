---
id: ref.runtime.policy_profiles
title: Runtime Policy Profiles
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
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
    - crates/ash-interp/src/capability_policy_runtime.rs
  tests:
    - crates/ash-cli/tests/alpha_admission_profile.rs
    - crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs
    - crates/ash-interp/tests/task_736_capability_binding_admission.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
    - ref.runtime.admission
  explains:
    - ref.runtime.kernel
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-cli/src/commands/** changes
  - crates/ash-interp/src/capability_policy_runtime.rs changes
  - reference/runtime/policy-profiles.md changes
---

# Runtime Policy Profiles

Runtime policy profiles are the Alpha names and grant-projection facts used during RuntimeKernel admission. They are not a production policy-management system.

## Admission Profile Labels

The current CLI-facing labels are:

- `empty`: preserve the default Alpha path with no requested grants and no explicit rejection;
- `allow`: explicitly admit the instance;
- `reject`: reject before user body execution.

`ash run` accepts `--admission-profile`. Daemon start requests also carry an `admission_profile` field.

## Config Interaction

Daemon start records carry a `config_id`, but the current Alpha daemon only supports the daemon's default config. Non-default daemon start config IDs are rejected before instance recording until config-specific daemon artifacts exist.

Profile and config names are identity facts. They do not grant authority by themselves.

## Grant Projection

The current policy-profile enforcement projects admitted capability binding IDs into provider/action grants before workflow and spawned-child execution. Execution records carry admission facts for projected bindings, action grants, and resource IDs reachable through implementation binding dependencies.

Provider/resource inventory remains separate from authority. A registered provider without an admitted grant is still unavailable to user body execution.

## Non-Goals

These pages do not claim:

- a remote policy service;
- multi-user policy isolation;
- distributed policy propagation;
- arbitrary policy language authoring;
- automatic authority from provider/resource existence;
- broader first-class resource-operation enforcement than current Alpha evidence.

For the authority boundary itself, see [Runtime admission and authority](admission.md).
