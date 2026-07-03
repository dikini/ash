# TASK-1861: Reconcile handler/provider specs

## Description

Update target specs and indexes so handler/provider semantics are stated consistently.

## Requirements

- Specs define handler/provider frames and frame-stack lookup.
- Specs define raise/handle behavior and missing-discharge failure.
- Specs state admission evidence needed to prove operation row requirements.
- Update orientation indexes for handler/provider semantics work.

## Completion criteria

- [x] Specs and indexes route handler/provider semantics through current target docs and PLAN-184.
- [x] Specs do not imply rows install frames or grant authority.
- [x] Docs gate passes.

## Evidence

- Updated SPEC-096b, SPEC-097b, SPEC-098b, SPEC-099b, SPEC-100, SPEC-INDEX, and NOTE-INDEX to route handler/provider operational semantics through PLAN-184 and state frame-stack lookup/missing-discharge behavior.

## Depends on

- TASK-1857; TASK-1858; TASK-1859; TASK-1860.
