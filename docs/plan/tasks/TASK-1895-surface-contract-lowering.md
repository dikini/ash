# TASK-1895: Surface Contract Lowering

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Lower surface `requires`/`ensures` contracts into typed Core predicate sidecars, snapshot metadata, and runtime check plans without re-evaluating source text.

## Requirements

1. Lower each well-formed predicate to a `LoweredPredicate` / `PredicateRef` according to the schema in NOTE-033.
2. Build boundary-local `SnapshotRef`s for `old(path)` captures before the governed body.
3. Emit `ProofObligation` or `RuntimeCheckPlan` based on static/dynamic classification.
4. Attach `ContractDischarge` metadata to the Core callable carrying the contract.
5. Preserve blame labels, source spans, and diagnostic shapes through lowering.

## TDD Steps

1. Add Core lowering tests for a simple `requires` producing a `PredicateRef` and runtime check plan.
2. Add Core lowering tests for an `ensures` with `old(...)` snapshot and `result` binder.
3. Add tests proving rejected predicates never reach Core artifacts.
4. Add tests proving `ContractDischarge` metadata is attached to the Core callable.

## Completion Checklist

- [x] `LoweredPredicate`/`PredicateRef` carriers populated from surface clauses (`crates/ash-parser/src/lower/contract_predicate.rs`).
- [x] `SnapshotRef` capture timing and path validation implemented (boundary-local snapshot metadata in `ContractDischargeRecord`).
- [x] Proof-obligation and runtime-check-plan emission implemented (`ProofObligation`, `RuntimeCheckPlan`, and `ContractDischarge` metadata carriers).
- [x] `ContractDischarge` metadata attached to Core callable metadata (engine-side `set_contract_discharge_for_callable`).
- [x] Blame labels and source spans preserved (`CoreSourceSpan` and boundary identity in discharge records).
- [x] Focused Core lowering tests pass (`pure_function_contracts_task_505.rs`, `task_1896_1897_evidence_contract_discharge.rs`).
