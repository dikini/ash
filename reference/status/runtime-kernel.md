---
id: ref.status.runtime_kernel
title: RuntimeKernel Status
kind: status
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
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-engine/src/runtime_artifact.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
    - crates/ash-interp/src/capability_policy_runtime.rs
  tests:
    - crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs
    - crates/ash-cli/tests/alpha_admission_profile.rs
    - crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs
    - crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs
    - crates/ash-cli/tests/alpha_ashd_child_failure_trace.rs
    - crates/ash-interp/tests/task_736_capability_binding_admission.rs
    - crates/ash-engine/tests/alpha_runtime_kernel_artifact_builder.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.status.index
    - ref.runtime.index
  explains:
    - ref.runtime.kernel
    - ref.runtime.admission
    - ref.runtime.artifacts
    - ref.runtime.daemon
    - ref.runtime.policy_profiles
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - crates/ash-cli/src/commands/** changes
  - crates/ash-interp/src/** changes
  - reference/runtime/** changes
  - reference/status/runtime-kernel.md changes
---

# RuntimeKernel Status

This page records current RuntimeKernel evidence and limitations. Concept explanations live under [Runtime](../runtime/README.md).

Verification baseline: `9fd1b8f` on 2026-06-01.

## Current Claims

| Area | Status | Evidence boundary |
| --- | --- | --- |
| Semantic host model | current Alpha | SPEC-070 defines one `RuntimeKernel` with one-shot `ash run` and local daemon host modes. Current implementation is split across carriers and command paths. |
| One-shot execution | current Alpha | `ash run` reports RuntimeKernel identity/admission/artifact facts on accepted paths and distinguishes admission rejection from body execution. |
| Local daemon | current Alpha | `ash daemon ...` provides same-user local serve/list/start/status/cancel/reload control and indexes definitions without file-presence execution. |
| Artifact identity | current Alpha | Verified artifacts are source/check-summary based at `alpha_checked_workflow_boundary`; run/daemon summaries match for the same accepted source boundary. |
| Admission profiles | current Alpha | `empty` and `allow` admit; `reject` rejects before user body execution. |
| Policy-profile grants | current Alpha | Capability binding/action grants are projected before workflow and spawned-child execution; provider/resource existence is not authority. |
| Reload lifetime | current Alpha | Successful reload affects future starts; failed reload preserves the prior valid index; running instances keep admitted identity. |
| Daemon config | partial Alpha | Default daemon config is supported. Non-default daemon start config IDs reject before instance recording. |
| Resource enforcement | limited Alpha | Resource grant facts may be recorded from current metadata, but full first-class resource-operation enforcement is not broadened here. |

## Verification Anchors

| Evidence | What it covers |
| --- | --- |
| `crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs` | one-shot RuntimeKernel identity/report shape, provider inventory not admission authority, and no verified artifact for parse-invalid source. |
| `crates/ash-cli/tests/alpha_admission_profile.rs` | admission-profile rejection before body output and default empty admission behavior. |
| `crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs` | daemon serve/list/start/status/cancel/reload, start args/config/admission-profile records, rejected admission without instance recording, reload preservation, and local control boundaries. |
| `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs` | run/daemon verifier-normalized artifact equivalence and failed-reload artifact preservation. |
| `crates/ash-interp/tests/task_736_capability_binding_admission.rs` | policy-profile capability binding/action grant projection and alias-scoped authority. |
| `crates/ash-engine/tests/alpha_runtime_kernel_artifact_builder.rs` | shared RuntimeKernel artifact builder behavior. |

## Explicit Non-Goals

RuntimeKernel currently does not provide:

- remote or multi-user daemon API;
- distributed scheduling;
- production init-system integration;
- hot-swapping artifacts for already-running instances;
- full semantic execution selection for arbitrary non-`main` `FILE[:WORKFLOW]` suffixes;
- production artifact packaging, JIT, or native-code deployment;
- authority from provider/resource existence.

## Integrity and Authority Caveats

File presence is not execution. Provider/resource existence is not authority. Admission grants authority before user body execution. Reload affects future starts rather than mutating already admitted running instances.

Verified artifacts are source/check-summary based and scoped to current Alpha evidence. Treat them as reportable identity and equivalence summaries, not as a production deployment artifact format.
