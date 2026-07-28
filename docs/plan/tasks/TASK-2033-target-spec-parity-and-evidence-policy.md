# TASK-2033: Target-Spec Parity and Evidence Policy

**Status:** Complete
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** TASK-2030 and the canonical target-spec index

## Description

Make target-Ash implementation status, evidence, and specification parity distinct in the agent instructions, canonical orientation, semantic records, traceability, and validators.

## Requirements

- The target Ash specification defines the full implementation domain for each feature.
- A feature below its target-spec domain is `partial`, not complete.
- A feature beyond its target-spec domain requires a specification update before implementation.
- Every feature report records `implementation`, `evidence`, and `parity` separately.
- Evidence is `proved`, `tested`, or `none`. Tests provide confidence; proofs state their exact theorem and refinement scope. No tests means no test evidence.
- A model proof cannot claim production-runtime proof without a checked refinement bridge.
- Validators reject the prior conflated `bounded` and `general` implementation-status vocabulary.
- Existing live semantic records are migrated without claiming target-spec parity they do not have.

## TDD Steps

1. Add validator tests that require the three-axis status and reject a claim of implementation without evidence, a model proof presented as a runtime proof, and behavior beyond the spec.
2. Run the new tests and confirm that the current schema fails them.
3. Implement the schema and validator changes.
4. Migrate the active records, traceability metadata, and documentation policy.
5. Run semantic-record, traceability, orientation, and documentation gates.

## Completion Checklist

- [x] `AGENTS.md` makes the policy mandatory for every new session.
- [x] Canonical target-Ash documentation defines the three report axes.
- [x] Semantic records and coverage use the new vocabulary.
- [x] Traceability distinguishes implementation from tests and proofs.
- [x] Validator tests demonstrate the policy is enforced.
- [x] Active records are classified against their target-spec domains.
- [x] `CHANGELOG.md` and `PLAN-INDEX.md` are updated.
