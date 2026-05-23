---
id: ref.language.proc
title: Proc Processes
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
    - docs/spec/SPEC-048-PROC-LIBRARY.md
    - docs/spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
  code:
    - std/src/proc.ash
    - std/src/process.ash
  tests:
    - crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs
  examples:
    - examples/07-phase105/03-do-proc-from-act.ash
    - examples/08-phase106/02-proc-comprehension-from-act.ash
related:
  depends_on:
    - ref.language.act
    - ref.status.feature_matrix
  explains:
    - ref.language.workflow
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Proc Processes

## Summary

`Proc<T>` represents process-oriented work above Act and below Workflow in the pilot tower: `Pure < Act < Proc < Workflow`. Current reference claims require explicit crossing from Act to Proc; no implicit tower lift is assumed.

## Concept

Proc is the process layer for concurrent or process-shaped runtime work. It can include Act-derived work through explicit library/runtime bridges, but the bridge is visible.

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

- `examples/07-phase105/03-do-proc-from-act.ash` is illustrative-pass for explicit Act-to-Proc shape.
- `examples/08-phase106/02-proc-comprehension-from-act.ash` is historical unless current tests cite it directly.

## Known limitations

- The pilot does not document the full process runtime API.
- Historical Proc examples may predate SPEC-069 wording.

## Common confusions

- Proc is not just Act with a different name.
- Proc does not silently become Workflow. Use explicit Workflow lifts.
