---
id: ref.index
title: Reference Index
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
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.root
  explains:
    - ref.language.functions
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
    - ref.language.generalized_do
    - ref.agents.index
    - ref.status.index
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/maintenance/** changes
  - reference/getting-started/** changes
  - reference/tools/** changes
  - reference/runtime/** changes
  - Phase closeout changes reference policy
---

# Reference Index

## Getting started

- [Getting started](getting-started/README.md)
  - [What is Ash?](getting-started/what-is-ash.md)
  - [Install Ash](getting-started/install.md)
  - [Update Ash](getting-started/update.md)
  - [Run a program](getting-started/run-a-program.md)
  - [Run as a local daemon](getting-started/run-as-daemon.md)
  - [Clean up Ash state](getting-started/cleanup.md)
  - [Next steps](getting-started/next-steps.md)

## Language pilot

- [Functions and pure code](language/functions.md)
  - [Function declaration syntax](language/functions/declarations.md)
  - [Function bodies and expressions](language/functions/bodies-and-expressions.md)
  - [Local and anonymous functions](language/functions/local-and-anonymous.md)
  - [Calling functions and function values](language/functions/calls-and-values.md)
  - [Functions with pattern matching](language/functions/patterns.md)
  - [Function boundaries and common mistakes](language/functions/boundaries.md)
  - [Function implementation notes](language/functions/implementation-notes.md)
  - [Function authority and traceability](language/functions/authority-and-traceability.md)
- [Act effects](language/effects-act.md)
- [Proc processes](language/processes-proc.md)
- [Workflow boundaries](language/workflows.md)
- [Generalized do](language/generalized-do.md)

## Tools and runtime

- [Tools index](tools/README.md)
- [CLI tools](tools/cli.md)
- [Ashgrove](tools/ashgrove.md)
  - [Install](tools/ashgrove/install.md)
  - [Update](tools/ashgrove/update.md)
  - [Remove and cleanup](tools/ashgrove/remove-cleanup.md)
- [Runtime index](runtime/README.md)
- [RuntimeKernel](runtime/kernel.md)
- [Runtime artifacts](runtime/artifacts.md)
- [Runtime daemon](runtime/daemon.md)

## Agent derivatives

- [Agent guide](agents/README.md)
- [Context-pack index](agents/context-pack-index.md)
- [Common confusions](agents/common-confusions.md)

## Status

- [Status index](status/README.md)
- [Reference maintenance status](status/reference-maintenance.md)
- [Feature matrix](status/feature-matrix.md)
- [Known limitations](status/known-limitations.md)
- [Drift report](status/drift-report.md)
- [Verification evidence](status/verification-evidence.md)

## Maintenance

- [Maintenance index](maintenance/README.md)
- [Metadata reference](maintenance/metadata-reference.md)
- [Staleness inspection](maintenance/staleness-inspection.md)
- [Refresh procedure](maintenance/refresh-procedure.md)
- [Stale document triage](maintenance/stale-doc-triage.md)
- [Release checklist](maintenance/release-checklist.md)
- [Agent-card procedure](maintenance/agent-card-procedure.md)

## Examples

- [Example classification](examples/README.md)
