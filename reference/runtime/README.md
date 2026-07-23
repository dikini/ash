---
id: ref.runtime.index
title: Runtime Reference Index
kind: index
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
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-engine/src/runtime_artifact.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs
    - crates/ash-cli/tests/alpha_admission_profile.rs
    - crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs
    - crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.runtime.kernel
    - ref.runtime.admission
    - ref.runtime.artifacts
    - ref.runtime.daemon
    - ref.runtime.policy_profiles
    - ref.status.runtime_kernel
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - crates/ash-cli/src/commands/** changes
  - reference/runtime/** changes
---

# Runtime Reference Index

The runtime pages explain the current Alpha `RuntimeKernel`: the semantic execution host abstraction shared by one-shot `ash run` and the local daemon host mode.

RuntimeKernel is not a production deployment framework. It is the boundary that ties roots, checked function artifacts, artifact identity, admission, provider inventory, application instances, reports, and daemon control state to one execution model.

## Pages

- [RuntimeKernel](kernel.md): what the kernel owns and how one-shot and daemon host modes relate.
- [Admission and authority](admission.md): why provider/resource existence is not authority and how admission grants authority before user body execution.
- [Runtime artifacts](artifacts.md): source/check-summary artifact identity, verification caveats, and reload lifetime.
- [Local daemon](daemon.md): local control-plane behavior and daemon non-goals.
- [Policy profiles](policy-profiles.md): Alpha admission-profile labels and policy-profile grant projection boundaries.
- [RuntimeKernel status](../status/runtime-kernel.md): current evidence table, limitations, and verification baseline.

## Boundaries to Keep in Mind

File presence does not execute code. A daemon can index definitions without starting them, and `ash run` still needs a selected checked function artifact plus admission.

Provider and resource inventory is not authority. Admission creates explicit grants before user body execution; unadmitted provider actions must fail closed at the authority boundary.

Verified artifacts are Alpha source/check-summary based. They summarize accepted source, checked-function, and effect-row facts at the `checked_function_artifact` boundary; they are not a claim that arbitrary files execute or that a full production bytecode artifact format is complete.

Reload affects future starts. Already admitted running instances keep their admitted artifact/source identity, and failed reload preserves the previous valid index.

The current Alpha runtime does not claim a remote or multi-user daemon API, distributed scheduling, production init integration, or hot-swapping running instances.
