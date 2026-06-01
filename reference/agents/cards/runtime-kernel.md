---
id: ref.agents.card.runtime_kernel
title: RuntimeKernel Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 7fc92f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
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
  examples:
    []
related:
  depends_on:
    - ref.runtime.kernel
    - ref.status.runtime_kernel
  explains:
    - ref.runtime.admission
    - ref.runtime.artifacts
    - ref.runtime.daemon
    - ref.runtime.policy_profiles
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - reference/runtime/** changes
  - reference/status/runtime-kernel.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - crates/ash-cli/src/commands/** changes
  - crates/ash-interp/src/** changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
---

# RuntimeKernel Card

canonical_page: ref.runtime.kernel
canonical_page_path: ../../runtime/kernel.md
dependency_order: tools-runtime-3
warning: Read the canonical page, runtime index, and status page first. This card is derivative and must not broaden RuntimeKernel claims.

## Use

Retrieve the canonical page first, then read the relevant runtime subpage before editing admission, artifact, daemon, reload, or policy-profile claims.

## Retrieval tags

- ash
- reference-slice-2
- RuntimeKernel
- runtime-admission
- runtime-artifacts
- local-daemon
- policy-profiles
- verified-artifact
- reload-lifetime
- fail-closed-authority

## Must check before editing

- ../../runtime/README.md
- ../../runtime/kernel.md
- ../../runtime/admission.md
- ../../runtime/artifacts.md
- ../../runtime/daemon.md
- ../../runtime/policy-profiles.md
- ../../status/runtime-kernel.md
- ../../../crates/ash-core/src/runtime_kernel.rs
- ../../../crates/ash-engine/src/runtime_artifact.rs
- ../../../crates/ash-cli/src/commands/run.rs
- ../../../crates/ash-cli/src/commands/daemon.rs
- ../../../docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md

## Forbidden stale claims

- RuntimeKernel provides a remote or multi-user daemon API.
- RuntimeKernel provides distributed scheduling or production init-system integration.
- Daemon reload hot-swaps already-running instances.
- File presence or daemon indexing executes code.
- Provider or resource inventory grants authority without admission.
- Alpha verified artifacts are production bytecode packages, JIT artifacts, native-code deployments, or distributed cache protocols.
- Agent cards are normative specs.
