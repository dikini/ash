# TASK-430: Small-Step Helper Contracts and State Taxonomy

## Status: 📝 Planned

## Description

Make the helper-owned boundaries and runtime-facing state taxonomy of the small-step corpus fully explicit and proof-usable. This task should define the exact small-step helper contracts that remain atomic in v1 and sharpen the accepted state classification vocabulary around progress, blocked/suspended waiting, terminal success/rejection, and invalid/runtime-failure boundaries. The goal is to stop later proof, runtime, and alternate-implementation work from inferring these boundaries indirectly from mixed prose across `SPEC-004`, `SPEC-025`, and MCE closeout notes.

This remains docs/spec work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)
- [TASK-405: Authoritative Runtime Outcome/State Classification](TASK-405-authoritative-runtime-outcome-state-classification.md)
- [TASK-429: SPEC-025 Full Rule Definitions](TASK-429-spec-025-full-rule-definitions.md)

## Dependencies

- ✅ [TASK-405: Authoritative Runtime Outcome/State Classification](TASK-405-authoritative-runtime-outcome-state-classification.md)
- ✅ [TASK-429: SPEC-025 Full Rule Definitions](TASK-429-spec-025-full-rule-definitions.md)

## Requirements

### Functional Requirements

1. Revise the spec/reference corpus so the helper-owned boundaries used by small-step semantics are each stated as explicit contracts rather than only named informally.
2. Cover at minimum the v1 helper-owned boundaries for:
   - receive-arm selection,
   - parallel terminal aggregation,
   - policy decision / rejection ownership,
   - obligation transition/discharge ownership,
   - spawned-child completion observation / sealing ownership,
   - any other helper boundary already frozen by `SPEC-004` and `SPEC-025` as atomic in v1.
3. For each helper boundary, define:
   - input domain,
   - output domain,
   - ownership of failure / blocking / terminality,
   - determinism vs bounded nondeterminism status,
   - preserved semantic dimensions.
4. Freeze the state taxonomy used across the small-step/runtime correspondence story so the corpus distinguishes explicitly between:
   - progress transitions,
   - blocked/suspended waiting,
   - terminal success,
   - terminal rejection/failure,
   - invalid/runtime-failure or inadmissible states.
5. Keep the taxonomy compatible with existing `SPEC-004` failure ownership and the runtime-side follow-on work started by TASK-405.
6. Update planning/reporting/reference surfaces so Phase 67 runtime tasks can cite one explicit helper/state-taxonomy contract.
7. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not redesign the runtime or choose a concrete machine representation.
2. Do not flatten helper contracts into Rust-specific API requirements.
3. Keep the distinction between normative semantic taxonomy and informative current-runtime evidence explicit.
4. Use repo-relative links throughout.
5. Mark complete only if the corpus no longer relies on ambiguous mixed usage of blocked/suspended/invalid terminology.

## TDD Evidence

### Red

Before this task:
- helper-owned boundaries are accepted, but several remain distributed across `SPEC-004`, `SPEC-025`, and MCE discussion rather than packaged as one precise small-step-facing contract;
- blocked/suspended/invalid vocabulary is materially present but still spread across spec and runtime-alignment artifacts;
- later runtime and proof work would still need to reconstruct exact ownership of failure/blocking/taxonomy boundaries.

### Green

This task is complete when:
- helper-owned boundaries are each documented as explicit contracts;
- the state taxonomy is explicit enough for proof work and runtime follow-on tasks to cite directly;
- the accepted distinction between blocked/suspended and stuck/invalid is preserved without ambiguity;
- the corpus remains compatible with `SPEC-004`, `SPEC-025`, and the current conservative runtime evidence.

## Files

- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md` (if helper-contract wording needs alignment)
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] helper-owned boundaries packaged explicitly
- [ ] state taxonomy made explicit and coherent
- [ ] blocked/suspended vs invalid/stuck boundary clarified
- [ ] determinism / bounded nondeterminism per helper stated
- [ ] compatibility with TASK-405 runtime classification preserved
- [ ] planning/reference surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- one explicit helper/state-taxonomy contract for later proof and runtime alignment work.

Required by:
- TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh
- TASK-432: Semantic Execution Record and Terminal Projection Contract
- TASK-434: `Par` Branch-State and Aggregation Contract
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Do not let helper naming become accidental Rust API standardization.
- Keep current-runtime evidence informative, not normative.
- Treat taxonomy clarity as a semantic contract improvement, not as license to broaden runtime claims.
