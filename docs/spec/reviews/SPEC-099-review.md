# Review: SPEC-099 — Core Ash Language

**Date:** 2026-06-20
**Reviewer:** hermes
**Status:** PASS

## Summary

SPEC-099 now reconciles the Core Ash draft with SPEC-096b and SPEC-098b. The second review's blockers have been addressed:

1. `Handle.row` is now the **local residual body row only**. The total row is explicitly `Handle.row ∪ row(Handle.cont)`, matching SPEC-098b §5.4.
2. `ContractViolation` is no longer modeled as a row item or raised operation. Dynamic contract checks now lower to `RecordDischarge` plus `Trap { reason: ContractViolation(...) }`, or to an explicit `fail` effect if recoverable behavior is chosen.
3. Handler resume continuations now have a real continuation type shape: `Cont<OpResult, Ans, ρ_resume, Affine>`, with affine use rules matching SPEC-098b.
4. Arbitrary user-defined resumable effects are marked out of scope. Upper-layer declarations may only lower to known SPEC-096b/SPEC-098b operation kinds unless future specs extend the taxonomy.
5. Evidence is no longer an ordinary user-visible value. Proven/refuted/unknown/statistical evidence lowers to refinement metadata, `ContractDischarge`, `DischargeMarker`, `RecordDischarge`, or sidecar records.
6. The stale open question about `ContractViolation` being an effect item was removed.
7. The dictionary-lowering row now states that it is an upper-layer-to-Core note, not a Core type-system commitment.

## Coherence check

### Alignment with SPEC-098b row accounting

The Core-to-CPS lowering section now preserves local-vs-total row accounting:

- `Call.row` = callee body row ∪ current continuation row.
- `Raise.row` = operation local row only.
- `Handle.row` = local residual body row only.
- total `Handle` term row = `Handle.row ∪ row(Handle.cont)`.
- `If.row` = local branch-row union.
- `Jump.row` = target continuation row.

This avoids continuation-effect double counting and keeps cached row fields consistent with SPEC-098b.

### Alignment with SPEC-096b effect taxonomy

The spec no longer introduces a new Core effect namespace for arbitrary user-defined resumable effects. `EffectOp` is restricted to representable operation kinds:

- capability operations;
- channel operations;
- process operations;
- failure operations.

Surface effect declarations may still be used as upper-layer aliases or adapters that lower to those known operation kinds.

### Continuation discipline

Handler resumes are no longer plain untyped parameters. They are typed as affine continuations and subject to one-shot use rules. This is sufficient for Core Ash to expose Frank-style command-pattern mechanics without contradicting SPEC-098b's affine resume semantics.

### Evidence and diagnostics

The evidence model is now scoped correctly:

- proven evidence can harden refinements;
- disproved evidence emits errors;
- unknown evidence may demote to dynamic checks;
- statistical evidence remains advisory;
- evidence that affects auditability lowers to discharge metadata or sidecars, not ordinary values.

Diagnostics are still included as a cross-phase shape. That is appropriate for this spec because Core refinements, dynamic contract discharge, law evidence, and lowering errors all need source-preserving explanations.

## Remaining non-blocking notes

1. `RefinementEvidence` and diagnostics should probably split into dedicated specs once the language design stabilizes.
2. If arbitrary user-defined resumable effects become a target feature, they must be specified across SPEC-096b, SPEC-097b, SPEC-098b, and SPEC-099 together.
3. A future Core Ash revision may admit `Match` directly for diagnostics and optimization, but the current decision-tree lowering path is coherent.
4. `MultiShotPure` is present only as a hook; it should remain clearly optional until multi-shot semantics are specified normatively.

## Verdict

**PASS.** SPEC-099 is now structurally coherent as a target Core Ash draft and is lowerable to the existing CPS IR without contradicting SPEC-096b/SPEC-098b.
