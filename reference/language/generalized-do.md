---
id: ref.language.generalized_do
title: Generalized Do
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language-typechecker
last_verified: 2026-06-01
verified_against:
  git_commit: 710340f
  specs:
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md
  tasks:
    - docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/act.ash
    - std/src/proc.ash
    - std/src/workflow.ash
    - std/src/result.ash
  tests:
    - crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs
  examples:
    - examples/08-phase106/03-deferred-pure-targets.ash
    - examples/09-phase108/02-do-workflow-contract-statements.ash
related:
  depends_on:
    - ref.language.functions
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
  explains:
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - std/src/act.ash changes
  - std/src/proc.ash changes
  - std/src/workflow.ash changes
  - std/src/result.ash changes
  - reference/stdlib/** changes
  - Phase closeout changes reference policy
---

# Generalized Do

## Summary

Generalized `do:K` lowering uses selected public `Monad<K>` evidence where the current alpha implementation supports it. The canonical public Monad unit method is `unit`; older `return`-named fixture evidence is superseded. Built-in Act, Proc, and Workflow paths remain explicit tower surfaces. There are no implicit tower lifts, and a final expression in a do block is not an implicit return for arbitrary targets.

## Concept

A `do` block sequences computations for an explicit target. Current behavior is conservative: wrong target shapes, missing evidence, and unsupported implicit returns fail rather than guessing.

## Status

This page is a Phase 124 pilot page. It is current for the cited alpha evidence and intentionally incomplete outside the pilot slice.

## Syntax / surface

See the cited specs and stdlib files for full syntax. Examples here are small and classified through [example status](../examples/README.md).

## API / stdlib surface

The public stdlib module paths are listed in `verified_against`. Runtime implementation details remain opaque unless a cited spec exposes them. For current public operation lists, use the [stdlib tower index](../stdlib/README.md); this page remains the generalized-do concept page.

## Implementation notes

The reference page summarizes current public behavior only. It does not make a new normative contract.

## Authority and traceability

Primary authority is the cited SPEC-069/SPEC-070 text, stdlib files, tests, and example-status entries. Historical specs are linked only where they explain terminology.

## Agent notes

Retrieve this page before using agent cards. Prefer explicit current limitations over older broad design claims.

## Examples

- `examples/09-phase108/02-do-workflow-contract-statements.ash` is illustrative-pass for workflow do shape.
- `examples/08-phase106/03-deferred-pure-targets.ash` is expected-fail/historical for deferred pure targets.

## Known limitations

- Full free inference and arbitrary user computation targets remain limited by the current alpha evidence model.
- Do not claim automatic lifts or final-expression return unless a current test/spec row says so.

## Common confusions

- `do` is not syntax for hidden coercions across Pure/Act/Proc/Workflow.
- Result-domain failure and operational bottom are distinct in current alpha references.
