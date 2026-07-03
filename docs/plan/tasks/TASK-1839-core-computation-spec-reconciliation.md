# TASK-1839: Reconcile target Core computation specs

## Description

Update target specs and indexes so the Core computation model is stated consistently: Core Ash is the checked direct-style layer, computation rows are requirements, and target `do { ... }` is sequencing sugar.

## Requirements

- Update `SPEC-095b`, `SPEC-098c`, `SPEC-099`, and/or `SPEC-100` where needed.
- Update orientation indexes when routing changes.
- Avoid legacy target vocabulary.

## Completion criteria

- [x] Specs state the target Core computation model without implying a separate tower semantic path.
- [x] Indexes route target Core computation work through the reconciled docs.
- [x] Docs gate passes.

## Evidence

- Updated `docs/spec/SPEC-095b-TARGET-GRAMMAR.md` to describe target `do { ... }` as direct-style sequencing sugar and profiled `do:K` as compatibility.
- Updated `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md` to specify target `do { ... }` lowering to ordinary Core sequencing and rows as callable metadata.
- Updated `docs/notes/NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md` to use one checked direct-style Core computation model wording.
- Updated `docs/spec/SPEC-INDEX.md` and `docs/notes/NOTE-INDEX.md`.
- Verification: `python3 tools/docs/validate_orientation_indexes.py --self-test` passed.

## Depends on

- TASK-1838.
