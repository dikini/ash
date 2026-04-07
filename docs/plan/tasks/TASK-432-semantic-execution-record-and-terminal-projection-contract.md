# TASK-432: Semantic Execution Record and Terminal Projection Contract

## Status: 📝 Planned

## Description

Freeze the runtime-facing semantic execution-record contract that later `ash-interp` work must realize for cumulative semantic carrier alignment. This task should define the first authoritative contract for how runtime execution state and terminal observation package the semantic dimensions that the accepted big-step and small-step corpus care about: obligations `Ω`, provenance `π`, cumulative trace `T`, cumulative effect summary `ε̂`, and terminal outcome classification/projection. The output should be a contract that implementation work can target directly, without guessing how partial current runtime carriers should evolve.

This is contract/spec/reference work only. It must not implement the runtime substrate yet.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-405: Authoritative Runtime Outcome/State Classification](TASK-405-authoritative-runtime-outcome-state-classification.md)
- [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)

## Dependencies

- ✅ [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- 📝 [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- 📝 [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)

## Requirements

### Functional Requirements

1. Define the canonical runtime-facing semantic execution record that later runtime work must target for cumulative semantic-carrier alignment.
2. The contract must account for at minimum:
   - current/terminal obligation state `Ω`,
   - provenance state `π`,
   - cumulative trace `T`,
   - cumulative effect summary `ε̂`,
   - terminal outcome class and payload projection.
3. Distinguish explicitly between:
   - what must be exact for semantic conformance,
   - what may be conservative or partial during staged runtime adoption,
   - what remains intentionally out of scope for the first runtime slice.
4. Define how terminal projection from the execution record reconstructs or exposes the `SPEC-004` workflow outcome dimensions.
5. Keep the contract compatible with the accepted small-step state taxonomy and helper boundaries, especially for blocked/suspended and completion-observation boundaries.
6. State how the execution-record contract relates to the already-started runtime surfaces from TASK-405 through TASK-412 without pretending those tasks already provide full carrier closure.
7. Update planning/reporting/reference surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not implement new runtime structs or APIs here.
2. Do not falsely require exact runtime parity where the corpus has explicitly accepted staged/conservative slices.
3. Keep the contract implementation-neutral enough that Rust, Lean, and future runtimes can target it.
4. Use repo-relative links throughout.
5. Mark complete only if the subsequent runtime task can implement against this contract without rediscovering the semantic packaging story.

## TDD Evidence

### Red

Before this task:
- the runtime correspondence corpus identifies authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging as true residual drift;
- current runtime surfaces expose useful slices, but there is no one explicit contract for the execution record that should eventually carry those semantic dimensions;
- runtime follow-on work would otherwise risk growing ad hoc carriers that only partially align with the semantics corpus.

### Green

This task is complete when:
- one explicit semantic execution-record contract exists;
- the contract states exact vs conservative/staged obligations clearly;
- terminal projection to `SPEC-004` outcomes is explicit;
- the next runtime implementation task can target the contract directly.

## Files

- Create: `docs/reference/semantic-execution-record-contract.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] semantic execution-record contract created
- [ ] cumulative carrier expectations defined for `Ω` / `π` / `T` / `ε̂`
- [ ] exact vs conservative/staged requirements stated
- [ ] terminal projection contract stated explicitly
- [ ] compatibility with TASK-405 through TASK-412 preserved honestly
- [ ] planning/reference surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- the runtime-facing semantic execution-record contract for later `ash-interp` implementation.

Required by:
- TASK-433: `ash-interp` Execution-Record Substrate
- TASK-434: `Par` Branch-State and Aggregation Contract
- TASK-436: Completion-Payload Parity Contract
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Do not confuse retained-completion payload work with the full execution-record contract.
- Keep the contract semantically authoritative, not Rust-API accidental.
- Prefer explicit exact-vs-conservative labeling to aspirational ambiguity.
