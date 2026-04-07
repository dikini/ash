# TASK-438: Canonical IR Semantics Corpus and Result Format

## Status: 📝 Planned

## Description

Define the canonical IR semantics corpus and machine-readable expected-result format for cross-implementation verification. This task should create the shared input/output substrate that Rust, Lean, and future Ash implementations can all consume when being checked for semantic conformance. The corpus should focus on canonical `SPEC-001` workflows and related semantic boundaries rather than surface syntax convenience.

This remains contract/test-infrastructure planning work only.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh](TASK-431-big-step-small-step-meta-properties-and-formalization-boundary-refresh.md)
- [TASK-434: `Par` Branch-State and Aggregation Contract](TASK-434-par-branch-state-and-aggregation-contract.md)

## Dependencies

- 📝 [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- 📝 [TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh](TASK-431-big-step-small-step-meta-properties-and-formalization-boundary-refresh.md)
- 📝 [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)
- 📝 [TASK-434: `Par` Branch-State and Aggregation Contract](TASK-434-par-branch-state-and-aggregation-contract.md)
- 📝 [TASK-436: Completion-Payload Parity Contract](TASK-436-completion-payload-parity-contract.md)

## Requirements

### Functional Requirements

1. Define the canonical IR-level semantics corpus used for cross-implementation conformance testing.
2. The corpus must cover at minimum:
   - sequencing / binding / branching,
   - pattern-driven control,
   - capability / policy / obligation workflows,
   - receive / blocking / fallback behavior,
   - `Par`,
   - spawn / control / completion observation,
   - representative failure paths.
3. Define one machine-readable expected-result format that can represent, at minimum as applicable:
   - terminal outcome class,
   - return/error payload,
   - cumulative effect summary,
   - obligations summary,
   - provenance summary,
   - trace summary or trace payload policy,
   - blocked/suspended classification where relevant,
   - retained completion observations where relevant.
4. Define how the result format handles bounded nondeterminism, especially for `Par` and receive-related behaviors.
5. Keep the corpus and result format aligned with the implementation-conformance contract from TASK-428.
6. Update planning/reporting/reference surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not implement the harness here.
2. Keep the corpus canonical-IR-first, not surface-syntax-first.
3. Keep the result format implementation-neutral so Rust and Lean can both emit/consume it.
4. Use repo-relative links throughout.
5. Mark complete only if TASK-439 can build directly on this corpus and format.

## TDD Evidence

### Red

Before this task:
- there is no single canonical IR semantics corpus for cross-implementation verification;
- there is no machine-readable result format aligned with the new Phase 67 conformance contract;
- differential testing would otherwise rely on ad hoc case selection and result comparison.

### Green

This task is complete when:
- one canonical IR semantics corpus definition exists;
- one machine-readable expected-result format exists;
- bounded nondeterminism handling is defined explicitly;
- TASK-439 can implement a harness directly against this task’s outputs.

## Files

- Create: `docs/reference/canonical-ir-semantics-corpus.md`
- Create: `docs/reference/canonical-semantics-result-format.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] canonical IR semantics corpus defined
- [ ] machine-readable result format defined
- [ ] bounded nondeterminism representation defined
- [ ] conformance-contract alignment preserved
- [ ] planning/reference surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- the canonical semantics corpus and result format for differential conformance work.

Required by:
- TASK-439: Differential Conformance Harness (Rust First)
- TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Notes

Important constraints:
- Keep corpus cases canonical and semantic, not UI-level.
- Do not hide nondeterminism behind single-outcome golden files where the semantics allows variation.
- Prefer one explicit format over multiple ad hoc fixture shapes.
