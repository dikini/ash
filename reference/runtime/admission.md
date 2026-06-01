---
id: ref.runtime.admission
title: Runtime Admission and Authority
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
    - crates/ash-interp/src/runtime_state.rs
    - crates/ash-interp/src/capability_policy_runtime.rs
    - crates/ash-engine/src/lib.rs
  tests:
    - crates/ash-cli/tests/alpha_admission_profile.rs
    - crates/ash-interp/tests/task_736_capability_binding_admission.rs
    - crates/ash-interp/tests/invoke_runtime_dispatch.rs
    - crates/ash-engine/tests/task_715_workflow_admission_red.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
    - ref.runtime.kernel
  explains:
    - ref.runtime.policy_profiles
    - ref.runtime.daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-interp/src/** changes
  - crates/ash-engine/src/** changes
  - reference/runtime/admission.md changes
---

# Runtime Admission and Authority

Admission is the RuntimeKernel step that turns a selected workflow definition into an authorized workflow instance. It happens before user body execution.

Provider and resource existence is not authority. A host may have providers registered and resources known, but user code may use them only when admission projects the required grant into the workflow instance.

## Current Admission Shape

The Alpha runtime has a minimal admission-profile surface:

- `empty`: default Alpha behavior; no requested grants and no explicit rejection;
- `allow`: explicitly admits the instance;
- `reject`: rejects before user body execution.

For accepted execution paths, RuntimeKernel reports record admission facts such as profile, status, capability grant count, resource grant count, and action grant count where the current host can report them.

## Authority Rules

Admission creates explicit capability/resource/action authority. Capability invocation checks admitted grant state. Fallback host-provider dispatch must fail closed if no admitted grant or binding authorizes the provider action.

Action grants are scoped to admitted binding identity and name. Aliases that share a backing provider must not merge into broader authority.

Child process execution must inherit or derive authority through runtime policy. It must not widen authority from ambient provider registry state.

## Rejection Behavior

Rejected admission is not a workflow body failure. It reports an admission failure before user output, body execution, or verified artifact reporting for the rejected path.

Parse/check failure is also distinct from admission failure. Parse/check-invalid source remains a source/check diagnostic and does not receive a verified runtime artifact summary.

## Current Caveats

Resource grant facts are recorded from current metadata where available, but full first-class resource-operation enforcement is not broadened by these pages.

The RuntimeKernel admission model is Alpha and evidence-bound. These pages do not claim a production policy language, remote policy service, or multi-tenant authorization boundary.

For profile labels and grant projection details, see [Policy profiles](policy-profiles.md).
