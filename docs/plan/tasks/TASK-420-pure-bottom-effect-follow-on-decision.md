# TASK-420: Pure Bottom-Effect Follow-On

## Status: ✅ Complete

## Description

Evaluate whether `Pure` should become an explicit surfaced bottom element below `Epistemic` in the
coarse effect lattice, and record the concrete repo outcome.

This is intentionally separated from TASK-419 because TASK-414 only promoted `Pure` as an explicit
follow-up question, not as already normative behavior.

## Specification Reference

- [TASK-414: Effect Typing Contract Promotion and Vocabulary Cleanup](TASK-414-effect-typing-contract-promotion.md)
- [TYPES-004: Effect Typing Foundations](../../ideas/type-system/TYPES-004-effect-typing-foundations.md)
- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)

## Dependencies

- ✅ TASK-414 complete
- ✅ TASK-419 landed first, making the current coarse-contract alignment explicit before this decision

## Requirements

### Functional Requirements

1. Decide whether `Pure` should be added as a surfaced lattice element.
2. If yes, update the effect lattice, inference defaults, and affected tests coherently.
3. If no, record the rejection/deferral clearly and keep the current four-grade model.

### Non-Functional Requirements

1. Do not mix this decision into TASK-419 implicitly.
2. If implemented, update all normative/docs/code surfaces coherently.
3. Update `CHANGELOG.md`.

## Decision

Defer surfaced `Pure` for now and keep the current four-grade coarse effect lattice:

`Epistemic < Deliberative < Evaluative < Operational`

## Rationale

1. TASK-419 just aligned code and promoted docs around the current four-grade contract; that
   contract is now explicit across the main normative surfaces.
2. In the current repo, control/modal forms are already modeled honestly as "no extra surfaced
   grade of their own" within that four-grade lattice. There is no landed user-facing contract that
   requires `Pure` to exist today.
3. Promoting `Pure` now would be a broader contract rewrite than this follow-on honestly warrants:
   it would require coordinated changes across `ash-core` effect enums/serialization, workflow and
   runtime contracts, lattice identity/default assumptions, type/runtime docs, and affected tests.
4. The current repo/spec state does not yet show a concrete correctness gap that the existing
   four-grade model fails to express; it mainly shows a possible future normalization.

Given that state, the smallest honest outcome is to close the decision by explicitly deferring
`Pure` rather than silently widening the surfaced contract.

## Completion Checklist

- [x] Decide whether `Pure` should be added as a surfaced lattice element
- [x] Record the deferral clearly and keep the current four-grade model
- [x] Update `CHANGELOG.md`

## Notes

This was a deliberate second-step task, not part of the baseline TASK-419 implementation.

Completion note:
- TASK-420 does not add code changes or tests because the honest contract-first outcome is a
  docs/planning decision to defer surfaced `Pure` for now;
- the current four-grade lattice remains the normative model;
- any future `Pure` implementation should return as a broader contract rewrite with explicit scope
  across core effect carriers, inference defaults, serialization, and normative docs.
