# TASK-401: Runtime Carrier Inventory and Semantic Mapping Table (MCE-006)

## Status: ✅ Complete

## Description

Create the execution-ready planning backbone for MCE-006 by inventorying the current interpreter/runtime structures that realize the accepted MCE-005 workflow configuration and producing one canonical mapping table from small-step semantic carriers to runtime carriers. This task is documentation/planning/correspondence work first. It may inspect implementation structure as evidence, but it must not reopen semantic decisions already fixed by MCE-005.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

## Dependencies

- ✅ [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)
- ✅ [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- ✅ [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- ✅ Accepted MCE-005 backbone in `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`

## Requirements

### Functional Requirements

1. Define one canonical mapping table covering at minimum:
   - ambient context `A = (C, P)`;
   - environment `Γ`;
   - obligation state `Ω`;
   - provenance state `π`;
   - cumulative trace `T`;
   - cumulative effect summary `ε̂`;
   - residual workflow / control state;
   - terminal result classes (`Returned`, `Rejected`, blocked/suspended class).
2. For each carrier, record:
   - the current runtime/interpreter holder(s),
   - whether the mapping is direct, distributed, implicit, or currently missing,
   - the mutation/update boundary,
   - the observability role (cumulative state, local delta source, or both by reconstruction).
3. Classify the runtime's residual-execution model as one of:
   - direct residual workflow/AST execution,
   - continuation/frame-based execution,
   - hybrid control representation.
4. Identify first-pass gaps where runtime structure does not yet cleanly explain the accepted semantic carrier.
5. Distinguish representational indirection that is semantically safe from genuine observable-risk mismatches.
6. Produce one explicit output artifact that later MCE-006 tasks can consume mechanically.
7. For each mapping row, cite concrete runtime/interpreter evidence sources (files, types, functions, or equivalent implementation anchors) wherever available.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/runtime-correspondence; no runtime redesign or semantic redesign.
2. Preserve accepted MCE-005 terminology exactly.
3. Use repo-relative links throughout.
4. Keep the mapping at carrier/structure granularity rather than per-rule proof granularity.
5. Be precise enough that later tasks can build on this table without redefining the carrier set.

## TDD Evidence

### Red

Before this task:

- MCE-006 names the accepted semantic carriers but does not map them concretely onto runtime/interpreter structures;
- there is no canonical table saying where each carrier lives, how it is updated, or whether its mapping is direct versus distributed;
- later runtime-alignment questions therefore lack one frozen baseline artifact.

### Green

This task is complete when:

- every accepted carrier from MCE-005 has a documented runtime home or an explicitly labeled gap;
- the residual-control representation is classified;
- direct versus distributed carriers are identified;
- later MCE-006 tasks can reuse the mapping without redefining the carrier set.

## Files

- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md` (add a dedicated carrier-mapping table section as the canonical artifact location)
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Evidence source: current interpreter/runtime implementation files

## TDD Steps

### Step 1: Identify the missing baseline (Red)

Confirm that the current corpus still lacks:

- one carrier-to-runtime mapping table,
- one residual-control representation classification,
- one direct/distributed/missing status for each semantic carrier.

### Step 2: Freeze the carrier inventory

Use the accepted MCE-005 carriers exactly and do not add a new generic semantic center such as a store `S`.

### Step 3: Freeze the runtime mapping table

For each carrier, document:

- runtime holder,
- update point,
- observability role,
- mapping quality (`direct`, `distributed`, `implicit`, `missing`),
- concrete evidence source(s) where available.

### Step 4: Record first-pass gaps

Separate:

- semantically safe indirection,
- documentation-only gaps,
- real correspondence risks.

### Step 5: Verify GREEN

Expected pass condition:

- an implementer can read the table and know where every accepted semantic carrier lives at runtime or why it is currently unresolved.

## Completion Checklist

- [x] canonical carrier-to-runtime mapping table defined
- [x] dedicated carrier-mapping table section added to `MCE-006-SMALL-STEP-IR.md`
- [x] residual-control representation classified
- [x] direct / distributed / implicit / missing statuses recorded
- [x] first-pass correspondence gaps identified
- [x] PLAN-INDEX updated with dedicated MCE-006 phase

## Completion Notes

Completed 2026-04-05.

- Added the canonical semantic-carrier → runtime mapping table to [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).
- Classified the current interpreter as a hybrid control representation: primarily direct residual AST execution, with receive/yield/control state partly externalized into runtime-owned registries and stored continuations.
- Recorded concrete runtime evidence for `A = (C, P)`, `Γ`, `Ω`, `π`, `T`, `ε̂`, residual workflow/control state, and terminal result classes.
- Separated safe indirection, documentation gaps, and correspondence risks so TASK-402 through TASK-404 can build on one frozen baseline.

## Dependencies for Next Task

This task outputs:

- one frozen semantic-carrier inventory for MCE-006,
- one runtime mapping baseline,
- one first-pass gap list.

Required by:

- TASK-402: Residual Control, Blocked-State, and Completion Realization
- TASK-403: Par Interleaving, Branch-Local State, and Aggregation Correspondence
- TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff

## Notes

Important constraints:

- Do not reopen MCE-005 semantics.
- Do not let runtime cleanup sprawl from MCE-008 absorb this phase.
- Do not claim full interpreter correspondence yet.
- Prefer a thin, evidence-driven mapping story over speculative runtime redesign.

Edge cases to keep visible:

- blocked receive and control-owned waits,
- distributed trace/effect accumulation,
- branch-local `Par` state and terminal aggregation,
- retained completion payloads and repeated observation boundaries.
