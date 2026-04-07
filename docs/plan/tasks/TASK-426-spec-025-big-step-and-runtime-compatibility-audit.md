# TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit

## Status: 📝 Planned

## Description

Run an explicit audit proving that `SPEC-025` remains compatible with both `SPEC-004` big-step semantics and the current implementation evidence recorded in `MCE-006`. The goal is not to prove full implementation closure, but to ensure `SPEC-025` is faithful, non-contradictory, and honest about where runtime support is partial.

This is docs/spec-audit work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)

## Dependencies

- 📝 [TASK-424: SPEC-025 Faithfulness and Compatibility Contract](TASK-424-spec-025-faithfulness-and-compatibility-contract.md)
- 📝 [TASK-425: SPEC-025 Rule-Schema and Helper-Boundary Consolidation](TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md)
- ✅ [TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)

## Requirements

### Functional Requirements

1. Produce one compatibility matrix from `SPEC-025` sections/claims to the relevant `SPEC-004` contracts.
2. Produce one compatibility matrix from `SPEC-025` runtime-facing claims to the frozen `MCE-006` evidence packet.
3. For each audited row, classify the current state as one of:
   - directly compatible / directly supported,
   - compatible but reconstructed or approximated,
   - compatible but weak/missing implementation support,
   - wording change required to avoid overclaim.
4. Cover at minimum:
   - terminal outcome reconstruction,
   - helper-boundary ownership,
   - `Receive` blocking/fallthrough behavior,
   - `Par` interleaving and terminal aggregation,
   - spawned-child completion/control ownership,
   - cumulative carrier claims for `Ω`, `π`, `T`, and `ε̂`.
5. State the final conservative verdict on whether `SPEC-025` is compatible with both big-step semantics and current implementation evidence.

### Non-Functional Requirements

1. Keep the audit conservative and evidence-based.
2. Do not confuse semantic compatibility with full implementation closure.
3. Use repo-relative links throughout.
4. Call out weak/missing support explicitly rather than smoothing it over.

## TDD Evidence

### Red

Before this task:
- compatibility is plausible, but not frozen in one explicit audit artifact.
- overclaim risk remains unless each runtime-facing statement is checked against MCE-006.

### Green

This task is complete when:
- a reader can trace every major SPEC-025 claim to SPEC-004 and MCE-006,
- any overclaim has been downgraded or corrected,
- the final compatibility verdict is explicit.

## Files

- Create: `docs/plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

## Completion Checklist

- [ ] TASK-426 task file created
- [ ] SPEC-025 → SPEC-004 compatibility matrix created
- [ ] SPEC-025 → MCE-006 compatibility matrix created
- [ ] weak/missing implementation-support rows called out explicitly
- [ ] final conservative verdict recorded

## Notes

Important constraints:
- Do not use this task to redesign runtime behavior.
- Do not imply full correspondence where MCE-006 records only partial realization.
