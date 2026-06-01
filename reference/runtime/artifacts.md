---
id: ref.runtime.artifacts
title: Runtime Artifacts
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
    - crates/ash-engine/src/runtime_artifact.rs
    - crates/ash-core/src/runtime_kernel.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - crates/ash-engine/tests/alpha_runtime_kernel_artifact_builder.rs
    - crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs
    - crates/ash-cli/tests/alpha_ash_run_runtime_kernel_mode.rs
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.runtime.index
    - ref.runtime.kernel
  explains:
    - ref.getting_started.run_a_program
    - ref.runtime.daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
refresh_trigger:
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-engine/src/runtime_artifact.rs changes
  - crates/ash-core/src/runtime_kernel.rs changes
  - crates/ash-cli/src/commands/** changes
  - reference/runtime/artifacts.md changes
---

# Runtime Artifacts

Runtime artifacts are the current Alpha identity and verification summaries that tie an admitted workflow definition to source, check results, profile/config facts, and runtime-support identity. They are used by both one-shot `ash run` and the local daemon host mode.

## What Is Verified

The current artifact builder consumes:

- root identity;
- relative module path;
- workflow name;
- runtime profile and config identity;
- source text;
- check-summary text;
- selected runtime-support identity when present.

It produces deterministic source and check-summary hashes plus a verifier-normalized Alpha language summary. The checked carrier is explicitly scoped to `alpha_checked_workflow_boundary`.

## What Is Not Verified

File presence is not execution. A file can exist under a daemon root without starting any workflow. A runtime artifact summary exists only after source selection and parse/check success.

The Alpha artifact is source/check-summary based. It is not a claim that a complete production bytecode package exists, that every imported-module dependency is captured by a final digest closure, or that bytecode verification reparses source.

Parse/check-invalid source does not receive a verified artifact summary. The failure remains a source/check diagnostic, not a verified runtime artifact.

## One-Shot and Daemon Equivalence

For the same accepted workflow source and profile/config facts, `ash run` and `ash daemon` report matching verifier-normalized artifact summaries at the Alpha checked workflow boundary. The host mode is still recorded separately because one-shot and daemon lifetimes differ.

This equivalence does not erase host-mode differences. One-shot execution exits after the root instance. The daemon keeps an index and control surface for future starts.

## Reload Lifetime

Successful daemon reload stages and publishes new checked definitions and artifact summaries for future starts. Failed reload preserves the previous valid index. Existing running instances keep the artifact/source identity they were admitted with.

Reload is not hot-swapping. It does not mutate already-running instances.

## Current Caveats

The current daemon start path executes from the source bytes already read and hash-checked for the admitted definition. Imported-module drift is still future artifact-dependency work.

The artifact summary is suitable for Alpha status, report, and cross-host equivalence checks. It should not be treated as a production deployment manifest or distributed cache protocol.
