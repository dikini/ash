# AUDIT-194: Contract Evidence Seams

**Status:** Open
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)
**Task:** [TASK-1892](../tasks/TASK-1892-contract-evidence-seam-audit.md)

## Date

2026-07-04

## Purpose

Inventory the live implementation seams before adding target-surface `requires`/`ensures` and evidence rows on top of the firm computation/operation model. This artifact maps existing carriers, identifies exact file ownership, and assigns each gap to the task that will turn it into tests and implementation.

## Background: existing contract carriers from Phase 165

PLAN-165 already landed Core predicate, snapshot, and discharge sidecars in `ash-core`. These are the implementation carriers Phase 194 must extend:

- `crates/ash-core/src/core_ash_contract.rs` — `LoweredPredicate`, `PredicateNode`, `PredicateRef`, `SnapshotRef`, `PredicateEnvironment`, `PredicateClassification`, `RuntimeCheckPlan`, `ProofObligation`, `ContractDischarge`.
- `crates/ash-core/src/core_ash.rs` — `CoreContractDischarge` and `CoreDischargeMode` aliases.
- `crates/ash-core/src/core_ash_typecheck.rs` — contract discharge metadata on typed functions and callable metadata.
- `crates/ash-core/src/core_ash_lower.rs` — lowering of `CoreContractDischarge` to CPS `ContractDischarge`.
- `crates/ash-interp/tests/task_1701_temporal_monitor_runtime_diagnostics.rs` — temporal monitor diagnostic scaffolding.
- `crates/ash-interp/src/runtime/mod.rs` and CPS evaluator — `Trap` and `ContractViolation` payload paths.

## Surface / parser seam

### Current state

- `crates/ash-parser/src/` defines the current surface AST, target surface AST, and target grammar carriers.
- `crates/ash-parser/src/surface/` contains `FnDecl` and related callable declarations.
- `crates/ash-parser/src/surface/to_core.rs` (or equivalent lowering module) bridges parser AST to `ash-core`.
- There is no parser-side `requires`/`ensures` clause on `fn` declarations today. Surface contracts are not parsed.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| `requires`/`ensures` grammar not in target surface | `ash-parser` | TASK-1893 |
| No contract-position predicate AST boundary | `ash-parser` | TASK-1893 |
| `result` not scoped to postconditions | `ash-parser` / `ash-typeck` | TASK-1893 |
| Callable summaries lack contract metadata | `ash-engine` / `ash-core` | TASK-1893 |

## Typecheck / well-formedness seam

### Current state

- `crates/ash-core/src/core_ash_contract.rs` defines predicate classification, but it consumes already-lowered artifacts.
- `crates/ash-typeck` (or equivalent) does not yet have a surface predicate well-formedness pass that rejects operation calls, handler dispatch, unstable observers, etc., before lowering.
- The target type system in `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md` describes the judgment; implementation is pending.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Surface predicate well-formedness pass missing | `ash-typeck` / `ash-core` | TASK-1894 |
| Empty-row + stable-observer classification not enforced | `ash-typeck` / `ash-core` | TASK-1894 |
| `old(...)` root and snapshot validation missing | `ash-typeck` / `ash-core` | TASK-1894 |
| Admitted predicate-function classification missing | `ash-typeck` / `ash-core` | TASK-1894 |

## Lowering seam

### Current state

- `crates/ash-core/src/core_ash_lower.rs` lowers `CoreContractDischarge` to CPS.
- Surface-to-Core contract lowering (NOTE-033) is not yet implemented from the parser/typecheck side.
- The `core_ash_contract.rs` schema is ready to receive `LoweredPredicate` objects, but no surface syntax produces them.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Surface `requires`/`ensures` to `LoweredPredicate` lowering | `ash-parser` / `ash-core` | TASK-1895 |
| Snapshot capture timing before governed body | `ash-core` / `ash-interp` | TASK-1895 |
| Proof obligation vs runtime check plan selection | `ash-core` | TASK-1895 |
| Blame label preservation through lowering | `ash-core` | TASK-1895 |

## Evidence row seam

### Current state

- `crates/ash-core/src/core_ash.rs` defines `CoreRow` and `CoreRowItem` families.
- `crates/ash-engine/src/row_admission.rs` already has a `RowAdmissionRequirement::Evidence` variant, currently mapped to `Unsupported` (fail-closed placeholder).
- Evidence rows are not yet discharged or recorded.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Evidence row item schema and record carriers | `ash-core` | TASK-1896 |
| `test`/`law`/`proof`/`monitor`/`observation` evidence families | `ash-core` / `ash-engine` | TASK-1896 |
| Evidence identity stability across modules | `ash-core` / `ash-engine` | TASK-1896 |
| Evidence discharge fail-closed validation | `ash-engine` | TASK-1896, TASK-1897 |

## Row admission / discharge seam

### Current state

- `crates/ash-engine/src/row_admission.rs` implements `Engine::admit_workflow_with_explicit_rows` for operation, resource, role, policy, process, failure, evidence, and effect-group rows.
- Evidence and contract rows currently fail closed as `Unsupported`.
- Contract discharge metadata from `CoreContractDischarge` is not yet wired into admission.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Contract discharge modes (static/evidence/dynamic) not wired | `ash-engine` / `ash-core` | TASK-1897 |
| Contract rows must not grant authority | `ash-engine` / `ash-interp` | TASK-1897, TASK-1898 |
| Evidence discharge must validate evidence records | `ash-engine` / `ash-core` | TASK-1897 |

## Runtime / dynamic check seam

### Current state

- `crates/ash-interp/src/` CPS interpreter evaluates `Trap`, `Raise`, and `Handle`.
- `ContractViolation` and `ContractPredicateFault` are defined in the CPS/runtime data model but not yet exercised from surface contracts.
- Dynamic predicate evaluator over captured environments is not yet implemented from surface syntax.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Dynamic predicate evaluator over captured env/snapshots | `ash-interp` | TASK-1898 |
| Distinct `ContractViolation` and `ContractPredicateFault` traps | `ash-interp` | TASK-1898 |
| Check insertion at entry/return boundaries | `ash-core` / `ash-interp` | TASK-1898 |
| Predicate evaluator authority neutrality | `ash-interp` | TASK-1898 |

## Diagnostic / blame seam

### Current state

- `crates/ash-core/src/core_ash_contract.rs` has `BlameLabel` and `ContractDiagnostic` shapes.
- `crates/ash-interp` has `Trap` payload metadata but not yet full structured contract diagnostics from surface checks.
- Redaction policy is defined in `core_ash_contract.rs` but not yet wired end-to-end.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Structured blame diagnostics emitted at runtime | `ash-interp` | TASK-1899 |
| Snapshot/evidence refs in diagnostics | `ash-interp` / `ash-core` | TASK-1899 |
| Redaction policy applied to observed values | `ash-interp` | TASK-1899 |
| Blame immutability through handler composition | `ash-interp` | TASK-1899 |

## Runtime monitor / temporal seam

### Current state

- `crates/ash-core/src/core_ash_contract.rs` has trace contract and monitor-plan carriers (from Phase 165).
- `crates/ash-interp/tests/task_1701_temporal_monitor_runtime_diagnostics.rs` has temporal diagnostic tests.
- No surface syntax or row item currently requires a monitor.

### Gaps

| Gap | Owner | First task |
|---|---|---|
| Trace-contract row items not parsed | `ash-parser` | TASK-1900 |
| Monitor-plan evidence records not wired | `ash-core` / `ash-engine` | TASK-1900 |
| Temporal violation / monitor-fault diagnostics end-to-end | `ash-interp` | TASK-1900 |

## Risk summary

1. **Parser/typecheck boundary risk.** The target surface `fn` grammar is still evolving. TASK-1893 must add clause carriers without destabilizing existing function-first syntax.
2. **Authority leakage risk.** The most important invariant is that contract predicates and evidence rows must not grant authority. This needs authority-neutrality tests in every task from TASK-1894 onward.
3. **Core schema compatibility risk.** Phase 165 carriers are already used by tests. TASK-1895 must extend them rather than replace them.
4. **Row admission completeness risk.** `RowAdmissionRequirement::Evidence` currently fails closed. TASK-1896/1897 must turn this into a real evidence family without accidentally opening authority.

## Recommended next action

Start TASK-1893 by adding parser AST nodes and focused parser tests for `requires`/`ensures` on `fn` declarations. Keep the predicate body as an expression-like AST boundary that the well-formedness pass will classify in TASK-1894.
