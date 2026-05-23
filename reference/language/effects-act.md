---
id: ref.language.act
title: Act Effects
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
    - docs/spec/SPEC-047-ACT-MONAD.md
    - docs/spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md
  tasks:
    - docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
  code:
    - std/src/act.ash
    - std/src/result.ash
  tests:
    - crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs
  examples:
    - examples/07-phase105/01-do-act.ash
related:
  depends_on:
    - ref.language.functions
    - ref.status.feature_matrix
  explains:
    - ref.language.proc
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Act Effects

## Summary

`Act<T>` represents the first public effectful layer above pure code in the alpha tower. Act is an opaque, runtime-managed state-threading effect; it is not `Result<T, E>`. Effectful operations go through runtime capability/provider machinery rather than arbitrary user-visible state mutation.

## Concept

Act describes effectful work that the runtime can sequence. Ash exposes public Act algebra, while the runtime state and provider dispatch remain implementation-managed.

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

- `examples/07-phase105/01-do-act.ash` is classified as illustrative-pass for the pilot when used as a small Act/do example.

## Known limitations

- Arbitrary algebraic effect handlers are not part of the alpha pilot claim.
- Act failure must not be documented as direct `Result` construction unless a cited API says so.

## Common confusions

- Act is not Result. Use Result pages/specs for domain success/failure values.
- Act does not implicitly become Proc or Workflow. Use explicit lifts when crossing the tower.
