---
id: ref.language.functions
title: Functions and Pure Code
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
  tasks:
    - docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
  code:
    - std/src/prelude.ash
  tests:
    - crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs
  examples:
    - examples/01-basics/03-expressions.ash
related:
  depends_on:
    - ref.status.feature_matrix
  explains:
    - ref.language.act
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Functions and Pure Code

## Summary

Pure Ash code computes values without entering the runtime-managed effect tower. In the pilot tower model, Pure is below Act, Proc, and Workflow: `Pure < Act < Proc < Workflow`.

## Concept

A function can prepare values, select branches, and call other pure functions. It does not perform runtime-managed capability operations by itself. Effectful operations must enter the appropriate tower surface, normally through Act/Proc/Workflow APIs and provider-backed runtime execution.

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

- `examples/01-basics/03-expressions.ash` is classified as historical in [example status](../examples/README.md).

## Known limitations

- The pilot does not attempt a full pure-language manual.
- Do not infer that every old pure example is current normative evidence.

## Common confusions

- Pure code is not automatically lifted into Act, Proc, or Workflow.
- A final expression in a `do` block is not an implicit return for arbitrary computation targets.
