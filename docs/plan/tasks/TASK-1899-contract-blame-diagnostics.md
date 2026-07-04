# TASK-1899: Contract Blame Diagnostics

**Status:** Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Emit structured contract blame diagnostics that preserve boundary identity, predicate identity, blame polarity, snapshot references, evidence references, and redacted observation metadata. Distinct diagnostic shapes must be produced for `ContractViolation` (predicate false) and `ContractPredicateFault` (evaluator fault).

## Requirements

1. `ContractViolation` diagnostics must include:
   - predicate identity (`PredicateRef` or stable textual id);
   - blamed party (`caller`/`negative` for `requires`, `callee`/`impl`/`positive` for `ensures`);
   - boundary identity (`BoundaryKind` and boundary id);
   - contract source text or a stable span reference;
   - snapshot refs bound at the boundary;
   - evidence refs relevant to the predicate.
2. `ContractPredicateFault` diagnostics must include:
   - predicate identity;
   - the structured `PredicateFault` (missing binder, missing snapshot, type mismatch, division by zero, evaluator trap, etc.);
   - boundary identity where the fault occurred.
3. Keep observation evidence details redacted by default; diagnostics carry only metadata (family and identity) unless a redaction policy explicitly permits more.
4. Ensure diagnostics are serializable and can be propagated through `ExecError` to callers and the engine.

## TDD Steps

1. Add unit tests for `requires` violation diagnostics: caller blame, boundary id, predicate id, snapshots, evidence refs.
2. Add unit tests for `ensures` violation diagnostics: callee/impl blame, boundary id, predicate id, snapshots, evidence refs.
3. Add unit tests for `ContractPredicateFault` diagnostics preserving the underlying `PredicateFault` and boundary id.
4. Add serialization tests for violation/fault diagnostics.
5. Add redaction tests proving observation evidence details are not embedded in diagnostics by default.

## Completion Checklist

- [x] `ContractViolation` carries predicate identity, blame party, boundary identity, snapshots, and evidence refs.
- [x] `ContractPredicateFault` carries predicate identity, boundary identity, and structured `PredicateFault`.
- [x] Observation evidence details are redacted in diagnostics by default.
- [x] Diagnostics are serializable and propagate through `ExecError`.
- [x] Focused tests pass.

## Notes

The `ContractDiagnostic` and `PredicateFaultDiagnostic` types now carry an `EvidenceRef` vector and a `redacted` flag. The `ExecError::ContractViolation` and `ExecError::ContractPredicateFault` variants carry these structured diagnostics directly. Runtime checks in `execute.rs` construct diagnostics from the active `RuntimeCheckPlan` and propagate them as traps.
