# TASK-420: Pure Bottom-Effect Follow-On

## Status: 📝 Planned

## Description

Evaluate and, if approved, implement `Pure` as an explicit bottom element below `Epistemic` in the
coarse effect lattice.

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
- 🟡 TASK-419 should land first so current coarse-contract alignment is explicit before any lattice expansion

## Requirements

### Functional Requirements

1. Decide whether `Pure` should be added as a surfaced lattice element.
2. If yes, update the effect lattice, inference defaults, and affected tests coherently.
3. If no, record the rejection/deferral clearly and keep the current four-grade model.

### Non-Functional Requirements

1. Do not mix this decision into TASK-419 implicitly.
2. If implemented, update all normative/docs/code surfaces coherently.
3. Update `CHANGELOG.md`.

## Notes

This is a deliberate second-step task, not part of the baseline 414 follow-on implementation.
