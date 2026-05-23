---
id: ref.language.workflow
title: Workflow Boundaries
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language-runtime
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
  code:
    - std/src/workflow.ash
    - crates/ash-core/src/runtime_kernel.rs
  tests:
    - crates/ash-engine/tests/workflow_contracts_integration.rs
    - crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs
  examples:
    - examples/09-phase108/01-do-workflow-unit.ash
    - examples/09-phase108/04-workflow-explicit-lifts.reference.ash
related:
  depends_on:
    - ref.language.proc
    - ref.status.feature_matrix
  explains:
    - ref.language.generalized_do
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Workflow Boundaries

## Summary

`Workflow<T>` is the top pilot layer in the public tower. It represents workflow-boundary work admitted and executed through the runtime regime. Current alpha documentation must preserve explicit lifts and runtime admission boundaries.

## Concept

A workflow is the unit that the runtime admits, checks, and runs. Workflow status and authority interact with RuntimeKernel evidence from SPEC-070; the reference page does not turn every source file into an executable workflow.

## Status

This page is a Phase 124 pilot page. It is current for the cited alpha evidence and intentionally incomplete outside the pilot slice.

## Syntax / surface

See the cited specs and stdlib files for full syntax. Examples here are small and classified through [example status](../examples/README.md).

## API / stdlib surface

The public stdlib module paths are listed in `verified_against`. Runtime implementation details remain opaque unless a cited spec exposes them.

## Implementation notes

The reference page summarizes current public behavior only. It does not make a new normative contract.

## Authority and traceability

Primary authority is the cited SPEC-069/SPEC-070 text, stdlib files, tests, and example-status entries. Historical specs are linked only where they explain terminology.

## Agent notes

Retrieve this page before using agent cards. Prefer explicit current limitations over older broad design claims.

## Examples

- `examples/09-phase108/01-do-workflow-unit.ash` is normative-pass for the cited pilot shape.
- `examples/09-phase108/04-workflow-explicit-lifts.reference.ash` is reference-only.

## Known limitations

- Local daemon/runtime details are only cited where SPEC-070 and tests support them.
- Full workflow API coverage is outside Phase 124.

## Common confusions

- Workflow is not a general file-presence execution marker. Runtime admission matters.
- Proc/Act operations do not silently lift into Workflow.
