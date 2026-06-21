# TASK-1680: Continuation Multiplicity Spec and Plan Packet

**Status:** Done
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Freeze the SPEC-102 and Phase 164 planning packet before implementation begins.

## Specification Reference

- [SPEC-102](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)
- [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)

## Dependencies

- Phase 163 complete.

## Requirements

1. Keep SPEC-102 scoped to Core Ash, CPS IR, semantics, type checking, and Core-to-CPS lowering.
2. Mark surface syntax and motivational surface examples as informational only.
3. Link `docs/design/multi-shot-continuations.md` and NOTE-012 as rationale, not normative syntax.
4. Keep explicit multi-shot opt-in; do not specify empty-row inference.
5. Register PLAN-164 in `docs/plan/PLAN-INDEX.md`.
6. Add task files TASK-1681 through TASK-1691.
7. Update `CHANGELOG.md`.

## TDD Steps

1. Add a docs consistency test only if the implementation branch already has a suitable docs-link harness.
2. Run `cargo test -p spec_processor spec_links` after writing docs.
3. Fix broken links or anchors.

## Completion Checklist

- [x] SPEC-102 exists and states the normative boundaries.
- [x] PLAN-164 exists and lists TASK-1680 through TASK-1691.
- [x] PLAN-INDEX has a Phase 164 summary row and detail section.
- [x] CHANGELOG has an Unreleased entry for the planning packet.

## Closeout Evidence

- `cargo test -p spec_processor spec_links`
