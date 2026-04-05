# TASK-403: `Par` Interleaving, Branch-Local State, and Aggregation Correspondence (MCE-006)

## Status: ✅ Complete

## Description

Build on TASK-401 and TASK-402 by explaining how the current interpreter/runtime realizes `Par` execution, branch-local state, and terminal aggregation relative to the accepted MCE-005 concurrency contract. This task is documentation/planning/runtime-correspondence work. It must document the current operational story conservatively, including where the runtime has a usable correspondence story, where it only approximates the accepted semantic contract, and where helper-backed aggregation remains under-specified.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)

## Dependencies

- ✅ [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- ✅ [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)
- ✅ Accepted MCE-005 backbone in `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- 📝 Current interpreter/runtime evidence in `crates/ash-interp/`

## Requirements

### Functional Requirements

1. Explain how the current runtime realizes `Workflow::Par`, including how child workflows are launched and awaited.
2. Document where branch-local state currently lives, covering at minimum:
   - execution context / bindings,
   - mailbox/control/suspension side state,
   - terminal child results.
3. Distinguish clearly between:
   - semantic interleaving promised by MCE-005,
   - current operational concurrency strategy in the interpreter,
   - helper-backed aggregation expectations from SPEC-004,
   - places where the current runtime only approximates the accepted concurrency contract.
4. Document how terminal aggregation currently works for `Par` and whether it is:
   - direct realization,
   - distributed/partial realization,
   - missing / correspondence-risk.
5. Record concrete implementation evidence sources (files, functions, types) for `Par` execution, branch-local state, and aggregation behavior.
6. Produce one explicit `Par` correspondence section in `MCE-006-SMALL-STEP-IR.md` that later tasks and MCE-007 can consume mechanically.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/runtime-correspondence; no runtime redesign.
2. Preserve accepted MCE-005 semantics:
   - `Par` is interleaving-based semantically,
   - helper-backed aggregation remains normative,
   - do not collapse the semantic contract into accidental left-to-right sequential execution.
3. Use repo-relative links throughout.
4. Be conservative: if the current runtime does not yet realize helper-backed branch-local cumulative-state aggregation, say so explicitly.

## TDD Evidence

### Red

Before this task:

- TASK-401 identifies `Par` as a correspondence risk;
- TASK-402 freezes control/blocking/completion behavior but does not yet explain `Par` branch execution and aggregation;
- MCE-006 therefore still lacks one explicit concurrency/aggregation correspondence story.

### Green

This task is complete when:

- MCE-006 contains one explicit section explaining current `Par` execution and aggregation behavior;
- branch-local state and terminal aggregation are described in runtime terms;
- the gap between accepted semantic interleaving/aggregation and current runtime behavior is documented conservatively;
- the result is usable as runtime evidence for later MCE-007 ingestion.

## Files

- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Reference: `docs/plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md`
- Reference: `docs/plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md`
- Evidence source: current interpreter/runtime implementation files

## TDD Steps

### Step 1: Re-read the TASK-401 / TASK-402 baseline (Red)

Confirm the remaining `Par` gaps are still open:

- where branch-local state lives,
- whether the runtime truly realizes interleaving versus bulk concurrency,
- how terminal aggregation relates to the helper-backed semantic contract.

### Step 2: Freeze the current `Par` execution story

Document how `Workflow::Par` is currently executed in runtime terms.

### Step 3: Freeze the branch-local state story

Document what parts of branch execution are isolated per child and what parts are shared or externally coordinated.

### Step 4: Freeze the aggregation story

Document how terminal child results are combined today and how that compares to the accepted helper-backed aggregation contract.

### Step 5: Verify GREEN

Expected pass condition:

- a reader can tell exactly how current `Par` execution works, what it preserves from the accepted semantics, and what remains a correspondence gap.

## Completion Checklist

- [x] TASK-403 task file created
- [x] dedicated `Par` correspondence section added to `MCE-006-SMALL-STEP-IR.md`
- [x] branch-local state story documented
- [x] terminal aggregation story documented
- [x] evidence sources cited
- [x] PLAN-INDEX updated

## Completion Notes

Completed as documentation/planning/runtime-correspondence work only.

What was added:

- one dedicated `Par` correspondence section in [`MCE-006`](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md);
- one concrete description of the current operational `Workflow::Par` path in `crates/ash-interp/src/execute.rs`;
- one explicit branch-local versus shared-state split for cloned `Context` state versus shared mailbox/control/proxy/suspension registries;
- one conservative correspondence classification distinguishing accepted semantic interleaving from the current bulk async `join_all(...)` strategy;
- one explicit aggregation conclusion: current runtime directly aggregates successful terminal child values into `Value::List`, but does not yet evidence full helper-backed cumulative-state aggregation for `Ω`, `π`, `T`, or `ε̂`.

Conservative closeout statement:

- `Par` execution is now documented as a distributed/partial operational realization of the accepted interleaving contract.
- Terminal child-value collation is directly realized for runtime values.
- Full helper-backed semantic aggregation remains partial/missing and stays visible as a correspondence risk for TASK-404 / MCE-007 handoff work.

## Dependencies for Next Task

This task outputs:

- one explicit concurrency/aggregation correspondence story for `Par`,
- one sharper statement of what the current runtime preserves versus approximates,
- one evidence packet for observable-preservation and MCE-007 handoff work.

Required by:

- TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff

## Notes

Important constraints:

- Do not treat `join_all(...)` returning `Value::List(...)` as automatically equivalent to the accepted helper-backed semantic aggregation contract.
- Do not overclaim fairness, scheduler guarantees, or full helper-backed branch-local cumulative-state aggregation if the current runtime does not evidence them.
- Keep MCE-007 as the downstream consumer of this evidence.

Edge cases to keep visible:

- empty `Par`,
- branch failure propagation,
- branch-local mailboxes/control/suspension state,
- ordering assumptions accidentally introduced by list aggregation,
- mismatch between semantic interleaving and current bulk-await operational shape.
