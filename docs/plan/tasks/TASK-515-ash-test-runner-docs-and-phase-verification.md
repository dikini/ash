# TASK-515: Ash Test Runner Docs and Phase Verification

## Status: Planned (Phase 76B)

## Description

Finalize planning/bookkeeping/docs for the Ash test runner phase and run the final verification gate, including targeted `ash test` smoke runs for authored and synthesized test paths.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)
- [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023: Small-World Exploration Substrate](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)
- [TASK-1010: Phase 76B Rescope and Spec-Hardening Packet](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Dependencies

- [TASK-509](TASK-509-ash-test-runner-substrate.md)
- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-512](TASK-512-authored-test-metadata-and-execution-model.md)
- [TASK-1010](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)
- [TASK-513](TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md)
- [TASK-514](TASK-514-property-and-smallworld-execution.md)

## Requirements

1. Update `PLAN-INDEX.md` and related task bookkeeping.
2. Update `CHANGELOG.md`.
3. Run the final verification gate for the affected workspace.
4. Run targeted `ash test` smoke cases for authored tests.
5. Run targeted synthesized-test smoke cases for contracts/policies/obligations.
6. Report residual limitations explicitly if any remain.
7. Verify TASK-513 and TASK-514 implemented the TASK-1010 introspection, generated-input, small-world, and repro artifact contracts before Phase 76B closeout.

## Likely Files

- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Modify: active test-runner design/plan/task docs touched during implementation

## TDD Steps

### Red

- Identify stale docs/bookkeeping and missing verification evidence after the implementation tasks land.

### Green

- Update docs/bookkeeping and run the verification/smoke gate until the recorded closeout state matches the actual repository state.

## Explicit Deferred Follow-Up Items

Deferred until after spec work improvement:
- re-close TASK-513 only when synthesized contract/policy/obligation cases are truly executable end-to-end through TASK-1010 introspection APIs
- re-close TASK-514 only when property/small-world execution moves beyond bounded reruns into true generated/explored cases with TASK-1010 repro artifacts
- update Phase 76 phase-level bookkeeping from in-progress to complete only after those deferred items are either implemented or explicitly re-scoped by plan/spec work

## Baseline Already Satisfied by Phase 76A

- [x] PLAN-INDEX updated for the Phase 76A substrate closeout
- [x] CHANGELOG updated for the Phase 76A substrate closeout
- [x] verification commands run successfully against the bounded v1 implementation
- [x] authored `ash test` smoke cases run successfully against the bounded v1 implementation
- [x] synthesized smoke coverage preserved explicit opt-in planning-level behavior
- [x] residual limitations recorded honestly for the bounded v1 implementation

## Phase 76B Completion Checklist

- [ ] TASK-513 executable synthesized contract/policy/obligation cases complete
- [ ] TASK-514 true generated-input and small-world exploration cases complete
- [ ] final verification commands run successfully against the Phase 76B implementation
- [ ] targeted authored and synthesized `ash test` smoke cases run against the Phase 76B implementation
- [ ] PLAN-INDEX, PLAN-024, task files, and CHANGELOG reflect the final Phase 76B state
- [ ] residual limitations recorded honestly if any remain
