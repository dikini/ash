---
id: plan.ash.contract-system-implementation-handoff
title: Contract System Implementation Handoff
kind: plan
audience: [human, agent]
authority: design
status: planned
stability: alpha
owner: language
last_verified: 2026-06-29
verified_against:
  notes:
    - docs/notes/NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md
    - docs/notes/NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md
    - docs/notes/NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md
    - docs/notes/NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md
    - docs/notes/NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md
    - docs/notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOT-SEMANTICS.md
    - docs/notes/NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md
    - docs/notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md
    - docs/notes/NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md
    - docs/notes/NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md
  specs:
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
---

# Contract System Implementation Handoff

> **For implementers:** This is a planning handoff from the NOTE-014 contract design track into implementation tasks. Do not implement directly from NOTE-014; implement from the task files in this phase, checking the referenced notes and target specs for normative details.

**Goal:** Turn the resolved contract-system design notes into an ordered implementation packet for Core Ash, CPS IR/runtime diagnostics, type checking, capability observation evidence, and trace contracts.

**Architecture:** Build from the smallest runtime-safe substrate outward. First materialize Core predicate artifacts and snapshot metadata. Then add dynamic diagnostics and discharge records. Then implement interface/impl subsumption and capability observation evidence. Trace contracts and temporal monitors come last because they depend on the fact/evidence substrate and on the structured trap boundary.

**Tech Stack:** Rust 2024; `ash-core` Core AST/text/validation/type-checking/lowering modules; `ash-interp` CPS trap/runtime diagnostics; `ash-parser`/`ash-typeck` only for later surface integration once Core artifacts exist; focused tests in `crates/ash-core/tests/task_169x_*.rs` and `crates/ash-interp/tests/task_169x_*.rs`; docs verification through `scripts/check-docs-gate.sh`.

---

## Phase: 165

## Status

Planned: 1/10 tasks complete. `TASK-1693` created this handoff packet and is complete; implementation starts at `TASK-1694`.

## Background

NOTE-014 began as the contract-system gap register. The design gaps are now resolved by NOTE-027 through NOTE-035:

| Gap | Resolution |
|-----|------------|
| GAP 1: Blame/accountability | NOTE-027 |
| GAP 2: Monadic Hoare composition | NOTE-030 |
| GAP 3: Contract subsumption/variance | NOTE-027 |
| GAP 4: Evaluation modes/timing | NOTE-028 |
| GAP 5: Temporal/concurrent contracts | NOTE-035 |
| GAP 6: Failure observability/bottom | NOTE-029 |
| GAP 7: Meta-level soundness | NOTE-032 |
| GAP 8: Contract ↔ capability boundary | NOTE-034 |
| GAP 9: Surface-to-Core lowering | NOTE-033 |

The target specs already contain the reconciled contract vocabulary. This phase turns that vocabulary into implementable tasks without re-opening the design decisions.

## Scope locks

1. Keep `ContractViolation` and `TemporalContractViolation` as structured trap payloads by default. They are not row items and not implicit resumable operations.
2. Recoverable contract behavior must lower through explicit row-accounted `fail` or a declared compensation operation.
3. Contract predicates are authority-free observer code over captured environments. They may inspect operation-produced values but must not call providers or acquire authority.
4. Value-predicate lowering and trace-contract lowering are separate tracks. Do not force temporal formulas into `LoweredPredicate`.
5. `Pure`, `Act`, `Proc`, and `Workflow` are semantic anchors over the ambient computation model, not separate implementation boxes.
6. Preserve existing Core/CPS compatibility unless a task explicitly introduces a migration gate.
7. Prefer sidecar metadata and public summaries over new high-level Core term families unless the referenced spec requires a term.
8. Do not add user-facing surface syntax in this phase unless a task explicitly says so. Most tasks target Core/text fixtures first.

## Implementation dependency graph

```text
TASK-1694 Core predicate/snapshot artifact carriers
  ├─> TASK-1695 Contract predicate validation and lowering
  │     ├─> TASK-1696 Dynamic contract traps and predicate faults
  │     │     ├─> TASK-1697 Contract discharge/evidence metadata
  │     │     │     └─> TASK-1698 Interface/impl subsumption and blame
  │     │     └─> TASK-1699 Capability observation evidence boundary
  │     └─> TASK-1699 Capability observation evidence boundary
  └─> TASK-1700 Trace contract and monitor sidecar carriers
        └─> TASK-1701 Temporal monitor runtime diagnostics
              └─> TASK-1702 Integration fixtures, docs, and closeout
```

## Task overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1693](tasks/TASK-1693-contract-system-implementation-handoff.md) | Close NOTE-014 and create this implementation handoff packet | 2h | NOTE-035 | Done |
| [TASK-1694](tasks/TASK-1694-core-contract-predicate-artifacts.md) | Add Core predicate, snapshot, environment, and runtime-check artifact carriers | 6h | TASK-1693 | Planned |
| [TASK-1695](tasks/TASK-1695-contract-predicate-validation-and-lowering.md) | Validate and lower contract-position predicates into Core artifacts | 8h | TASK-1694 | Planned |
| [TASK-1696](tasks/TASK-1696-dynamic-contract-traps-and-predicate-faults.md) | Implement structured dynamic contract traps and predicate-fault diagnostics | 8h | TASK-1695 | Planned |
| [TASK-1697](tasks/TASK-1697-contract-discharge-and-evidence-metadata.md) | Record static/evidence/dynamic discharge metadata and public summaries | 6h | TASK-1696 | Planned |
| [TASK-1698](tasks/TASK-1698-interface-impl-contract-subsumption-and-blame.md) | Check interface-to-impl contract subsumption and preserve blame labels | 10h | TASK-1697 | Planned |
| [TASK-1699](tasks/TASK-1699-capability-observation-evidence-boundary.md) | Add operation-produced observation evidence without predicate authority leakage | 8h | TASK-1695, TASK-1696 | Planned |
| [TASK-1700](tasks/TASK-1700-trace-contract-monitor-sidecars.md) | Add trace-contract, trace-fact, workflow-ledger, and monitor-plan carriers | 8h | TASK-1694 | Planned |
| [TASK-1701](tasks/TASK-1701-temporal-monitor-runtime-diagnostics.md) | Implement temporal monitor result, violation, and monitor-fault diagnostics | 10h | TASK-1700, TASK-1696 | Planned |
| [TASK-1702](tasks/TASK-1702-contract-system-integration-closeout.md) | Add integration fixtures, docs consistency checks, PLAN-INDEX reconciliation, and closeout | 6h | TASK-1698, TASK-1699, TASK-1701 | Planned |

Estimated implementation effort after the handoff packet: 70 hours.

## Required test families

### Core artifact tests

Add focused tests under `crates/ash-core/tests/` for:

1. stable predicate identity includes binders, snapshots, predicate-function identities, and boundary id;
2. `old(path)` snapshot references are boundary-local and cannot be confused across producer/continuation/outer boundaries;
3. rejected predicate forms do not reach runtime-check artifacts;
4. dynamic runtime checks carry a `PredicateRef` plus captured environment, not source text.

### Runtime diagnostic tests

Add focused tests under `crates/ash-interp/tests/` for:

1. false dynamic predicate traps with `ContractViolation(ContractDiagnostic)`;
2. predicate evaluator fault traps with `ContractPredicateFault(PredicateFaultDiagnostic)`;
3. trap typing remains row `{}` and does not add `fail` or contract row items;
4. explicit recoverable behavior uses a visible `fail` path.

### Subsumption/blame tests

Add tests for:

1. impl precondition weakening accepted;
2. impl precondition strengthening rejected;
3. impl postcondition strengthening accepted;
4. impl postcondition weakening rejected;
5. requires failure blames caller/negative side;
6. ensures failure blames callee/impl/positive side.

### Capability observation tests

Add tests for:

1. predicates may inspect operation-produced boundary values;
2. predicates cannot invoke operation providers;
3. admission failure, operation failure, predicate false, and predicate fault are distinct;
4. observation evidence can be redacted without granting predicate authority.

### Trace contract tests

Add tests for:

1. trace contracts lower to `TraceContract`, not `LoweredPredicate`;
2. operational facts classify as `Proc`-like;
3. workflow ledger facts classify as `Workflow`-like;
4. mixed alphabets are accepted when every fact is in scope;
5. temporal violation and monitor fault produce distinct diagnostics.

## Verification gates

Each implementation task must run its focused task tests, plus:

```bash
cargo fmt --check
cargo test -p ash-core --test task_XXXX_...
cargo test -p ash-interp --test task_XXXX_...   # when runtime touched
cargo clippy -p ash-core --all-targets -- -D warnings
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-claim sweep for closeout

Before closing the phase, search for live normative stale claims:

```text
ContractViolation effects|catchable by handlers|raised ContractViolation|runtime contract handler
predicate = expr|source text.*semantic|re-parse surface predicate
separate `Proc` and `Workflow` contract systems|hard runtime containers|Proc<A>|Workflow<A>
capability calls inside predicates|predicate evaluator receives provider|predicate authority
TemporalContract.*LoweredPredicate|trace contract.*RuntimeCheckPlan
```

Changelog/history and explicit bad-contrast examples may remain. Live normative text must align with NOTE-027 through NOTE-035.

## Non-goals

- No new public surface syntax for temporal formulas in Phase 165.
- No full SMT/proof-assistant integration.
- No distributed runtime implementation beyond trace/monitor metadata needed by tests.
- No workflow UI or human-task execution substrate.
- No optimizer that consumes contract evidence beyond preserving metadata needed for later optimizer work.

## Open decisions for implementers

1. Exact Rust names may differ from note names if the mapping is documented and tests assert the semantics.
2. Whether trace facts are stored directly in `ash-core` or split with runtime-only carriers in `ash-interp` should be decided in `TASK-1700` after reading current trace support from Phase 163.
3. Whether first surface integration uses parser-level contract clauses or `.core` fixtures should be postponed until `TASK-1702`; Core fixtures are sufficient for the first implementation slice.

## Changelog

| Date       | Change |
|------------|--------|
| 2026-06-29 | Initial handoff plan. Closes the NOTE-014 gap-register design track and creates TASK-1693 through TASK-1702 as an ordered implementation packet. |
