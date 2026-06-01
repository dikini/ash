---
id: ref.runtime.kernel
title: RuntimeKernel
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
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
  code:
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs
    - crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
  explains:
    - ref.getting_started.run_a_program
    - ref.getting_started.run_as_daemon
    - ref.runtime.admission
    - ref.runtime.artifacts
    - ref.runtime.daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-cli/src/commands/run.rs changes
  - crates/ash-cli/src/commands/daemon.rs changes
  - reference/runtime/kernel.md changes
---

# RuntimeKernel

`RuntimeKernel` is Ash's Alpha semantic execution host abstraction. It is the conceptual container for roots, compiler/checker inputs, artifact identity, provider/resource inventory, workflow definitions, workflow instances, process state, admission, reports, traces, and the optional local daemon control endpoint.

The current implementation is intentionally split across existing crates and command paths. The important contract is that `ash run` and `ash daemon ...` are host-lifetime modes for the same language semantics, not separate languages.

## Host Modes

`ash run FILE[:WORKFLOW]` creates a one-shot host process. It reads and checks the selected source, evaluates admission, constructs the current runtime artifact summary for accepted inputs, runs one root workflow instance, reports locally when possible, and exits with an OS status.

`ash daemon ...` creates a long-lived local host mode. It indexes checked workflow definitions under a root, accepts local control requests, starts admitted workflow instance records, reports status, accepts cancellation requests, and reloads roots/config for future starts.

Both host modes share the same SPEC-069 tower semantics. Host lifetime and control-plane shape differ; typed lowering and admitted authority rules do not.

## What the Kernel Owns

The RuntimeKernel boundary includes:

- explicit runtime roots for source, library, config, state, cache, and logs;
- workflow definition identity and workflow instance identity;
- profile/config selection facts;
- source/check-summary based runtime artifact identity;
- provider/resource registry inventory;
- admission decisions and projected grants;
- scheduler/process ownership for the admitted instance;
- report and trace facts that make execution auditable.

In Alpha, these responsibilities are not all implemented by one concrete Rust struct. `crates/ash-core/src/runtime_kernel.rs` provides identity/admission/artifact carriers, while `ash run`, `ash daemon`, `ash-engine`, and `ash-interp` carry current execution seams.

## Integrity Caveats

File presence does not execute code. A source file or daemon-indexed definition becomes executable only after selection and admission.

Verified runtime artifacts are source/check-summary based. They identify accepted source, profile/config facts, runtime-support identity when present, and checker summary facts at the Alpha checked workflow-boundary carrier. They are not a production bytecode package format and do not prove that arbitrary files are executable.

Reload affects future starts. A successful daemon reload swaps the future definition/artifact index; failed reload preserves the previous valid index. Already admitted running instances keep their admitted artifact/source identity.

## Authority Caveats

Provider or resource existence is inventory, not authority. Admission grants authority before user body execution, and capability calls must check the admitted grant state. Fallback host-provider dispatch must fail closed when no admitted grant or binding authorizes the action.

Child `Proc` execution inherits or derives authority only through the runtime's split/join policy. It must not widen authority from ambient provider registry state.

## Non-Goals

The Alpha RuntimeKernel pages do not claim:

- remote or multi-user daemon API;
- distributed scheduling;
- production init-system integration;
- hot-swapping artifacts for already-running instances;
- full semantic selection of arbitrary non-`main` exported workflows through `FILE[:WORKFLOW]`;
- full production artifact packaging or JIT/native execution.

For current evidence and remaining limitations, see [RuntimeKernel status](../status/runtime-kernel.md).
