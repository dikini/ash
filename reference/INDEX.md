---
id: ref.index
title: Reference Index
kind: index
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
slice: reference-slice-3
owner: reference-corpus
last_verified: 2026-06-11
verified_against:
  git_commit: fb685740
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
    - docs/plan/tasks/TASK-1019-reference-ash-test-daily-use.md
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
  - reference/stdlib/** changes
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
- [The Ash Tower](language/tower.md)
- [CPS IR](language/cps-ir.md)
- [CPS Operational Semantics](language/cps-operational-semantics.md)
- [Record types](language/types/records.md)
- [Tuple types](language/types/tuples.md)

## Standard library

- [Stdlib tower index](stdlib/README.md)
- [Act stdlib](stdlib/act.md)
- [Proc stdlib](stdlib/proc.md)
- [Workflow stdlib](stdlib/workflow.md)
- [Result stdlib](stdlib/result.md)
- [Standard algebra](stdlib/algebra.md)

## Tools and runtime

- [Tools index](tools/README.md)
- [CLI tools](tools/cli.md)
- [Ash test](tools/test.md)
- [Ashgrove](tools/ashgrove.md)
  - [Install](tools/ashgrove/install.md)
  - [Update](tools/ashgrove/update.md)
  - [List, current, and default](tools/ashgrove/list-current-default.md)
  - [Remove and cleanup](tools/ashgrove/remove-cleanup.md)
  - [Project dependencies](tools/ashgrove/project-dependencies.md)
  - [Vendor and deploy](tools/ashgrove/vendor-deploy.md)
  - [Trust and signing](tools/ashgrove/trust-and-signing.md)
  - [Source payload and local state](tools/ashgrove/source-payload.md)
- [Runtime index](runtime/README.md)
- [RuntimeKernel](runtime/kernel.md)
- [Runtime admission and authority](runtime/admission.md)
- [Runtime artifacts](runtime/artifacts.md)
- [Runtime daemon](runtime/daemon.md)
- [Runtime policy profiles](runtime/policy-profiles.md)
- [CPS Interpreter](runtime/cps-interpreter.md)

## Agent derivatives

- [Agent guide](agents/README.md)
- [Context-pack index](agents/context-pack-index.md)
- [Common confusions](agents/common-confusions.md)
  - [Stdlib Act card](agents/cards/stdlib-act.md)
  - [Stdlib Proc card](agents/cards/stdlib-proc.md)
  - [Stdlib Workflow card](agents/cards/stdlib-workflow.md)
  - [Stdlib Result card](agents/cards/stdlib-result.md)
  - [Stdlib Algebra card](agents/cards/stdlib-algebra.md)
  - [CPS IR card](agents/cards/cps-ir.md)
  - [CPS Interpreter card](agents/cards/cps-interpreter.md)
  - [CPS Operational Semantics card](agents/cards/cps-operational-semantics.md)
  - [Ash CLI card](agents/cards/ash-cli.md)
  - [Ashgrove card](agents/cards/ashgrove.md)
  - [RuntimeKernel card](agents/cards/runtime-kernel.md)

## Status

- [Status index](status/README.md)
- [Ashgrove status](status/ashgrove.md)
- [RuntimeKernel status](status/runtime-kernel.md)
- [Reference maintenance status](status/reference-maintenance.md)
- [Feature matrix](status/feature-matrix.md)
- [Alpha limitations](status/alpha-limitations.md)
- [Known limitations](status/known-limitations.md) (retained Phase 124 alias)
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
