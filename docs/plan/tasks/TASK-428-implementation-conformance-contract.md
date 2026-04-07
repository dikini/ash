# TASK-428: Implementation Conformance Contract

## Status: ✅ Complete

## Description

Freeze the canonical implementation-conformance contract for Ash across the big-step semantic layer, the small-step semantic layer, and the runtime-observable layer. This task should produce the contract that future Rust, Lean, and alternate Ash implementations are judged against, without yet implementing new runtime carriers or proof machinery. The result should make it explicit which semantic surfaces are authoritative, what counts as conformance at each layer, where implementation freedom is permitted, and how allowed nondeterminism is bounded.

This remains contract/spec work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [Formalization Boundary and Proof Targets](../../reference/formalization-boundary.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)

## Dependencies

- ✅ [TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics](TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)
- ✅ [TASK-427: SPEC-025 Faithful Closeout and Corpus Alignment](TASK-427-spec-025-faithful-closeout-and-corpus-alignment.md)
- ✅ [TASK-400: MCE-007 Closeout Summary and Drift-Prevention Checklist](TASK-400-mce-007-closeout-summary-and-drift-prevention-checklist.md)

## Requirements

### Functional Requirements

1. Define the canonical implementation-conformance contract for Ash as an explicit spec/reference artifact rather than leaving conformance distributed across `SPEC-004`, `SPEC-021`, `SPEC-025`, and MCE notes.
2. Define at least three explicit conformance surfaces:
   - big-step / terminal semantic conformance,
   - small-step / state-taxonomy conformance,
   - runtime-observable conformance.
3. For each surface, define:
   - the authoritative source documents,
   - the required preserved semantic dimensions,
   - the permitted implementation freedom,
   - the bounded nondeterminism allowances.
4. State explicitly how implementations are compared when exact step ordering is not required, especially for helper-owned concurrency and receive behavior.
5. Define the relationship between semantic conformance and future differential-testing artifacts so later tasks can build canonical result formats and harnesses against this contract.
6. Keep the contract honest about currently partial runtime evidence from `MCE-006` / `MCE-007`; this task must define the target contract, not falsely claim that the current Rust runtime already satisfies every stronger level.
7. Update planning/reporting surfaces and `CHANGELOG.md` to record that Phase 67 now has an explicit conformance anchor.

### Non-Functional Requirements

1. Keep this task contract-first: no interpreter/runtime code changes.
2. Use repo-relative links throughout.
3. Preserve the existing authority hierarchy: code may implement, but canonical semantic truth still comes from the spec corpus.
4. Distinguish normative conformance requirements from informative examples clearly.
5. Mark complete only if later Phase 67 tasks can cite one explicit conformance artifact instead of re-deriving the contract.

## TDD Evidence

### Red

Before this task:
- conformance expectations are spread across `SPEC-004`, `SPEC-021`, `SPEC-025`, and the MCE closeout artifacts;
- there is no single explicit contract saying what another Ash implementation must preserve to count as semantically conformant;
- later runtime, Lean, and differential-testing work would otherwise need to reconstruct that contract indirectly.

### Green

This task is complete when:
- one explicit conformance artifact exists and is cross-linked from the planning/spec corpus;
- the artifact distinguishes big-step, small-step, and runtime-observable conformance clearly;
- implementation freedom and bounded nondeterminism are explicitly scoped;
- later Phase 67 tasks can treat the contract as fixed input rather than rediscovering it.

## Files

- Create: `docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] conformance artifact created
- [x] big-step conformance defined
- [x] small-step conformance defined
- [x] runtime-observable conformance defined
- [x] bounded nondeterminism rules stated
- [x] current-runtime partiality preserved honestly
- [x] planning/reporting surfaces updated
- [x] `CHANGELOG.md` updated

## Completion Notes

Completed 2026-04-07.

- Added [SPEC-026](../../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) as the explicit
  implementation-conformance contract for Ash, with separate big-step, small-step, and
  runtime-observable conformance surfaces.
- Froze surface-specific authority, preserved semantic dimensions, implementation freedom, bounded
  nondeterminism, and comparison rules for helper-owned concurrency and `receive` behavior.
- Made the differential-testing relationship explicit so later corpus/result-format and harness
  tasks can compare implementations against allowed result sets rather than one accidental step
  order.
- Preserved the honesty boundary from the current runtime-evidence corpus: the Rust runtime is not
  described as already having full small-step carrier closure or full retained-completion / `Par`
  aggregation parity.
- Updated the formalization boundary, spec index, Phase 67 planning surface, ideas/reporting
  surfaces, and `CHANGELOG.md` so the conformance contract is now a named corpus anchor.

## Dependencies for Next Task

This task outputs:
- the canonical conformance contract that Phase 67 runtime, proof, and differential-testing tasks can cite directly.

Required by:
- TASK-429: SPEC-025 Full Rule Definitions
- TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh
- TASK-438: Canonical IR Semantics Corpus and Result Format
- TASK-439: Differential Conformance Harness (Rust First)
- TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Notes

Important constraints:
- Do not silently upgrade current Rust implementation status to full conformance.
- Do not relocate semantic authority away from the canonical specs.
- Prefer one explicit conformance spec over diffuse cross-document prose.
