# MCE-006 Small-Step ↔ IR Execution Plan

> **For Hermes:** Use subagent-driven-development to execute this plan task-by-task. TASK-401 is actionable now. TASK-402 through TASK-404 depend on the carrier mapping baseline from TASK-401 and should preserve MCE-006's runtime/interpreter ownership boundary throughout.
>
> Note: Phase numbers here reflect planning/indexing order rather than dependency order. Phase 62 for MCE-007 was scaffolded first, but Phase 62.2 and Phase 62.3 remain explicitly gated on the runtime evidence produced by this later-indexed Phase 63.
>
> Execution update: Phase 63 is now complete. [TASK-401](../plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md), [TASK-402](../plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md), [TASK-403](../plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md), and [TASK-404](../plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md) together freeze the MCE-006 runtime correspondence corpus and the concise MCE-007 handoff packet.

**Goal:** Turn MCE-006 from a strong exploration note into an execution-ready runtime/interpreter alignment phase that consumes the accepted MCE-005 small-step backbone, documents and verifies how executable IR evaluation realizes or reconstructs that backbone, and produces the runtime-correspondence evidence packet that MCE-007 will later consume.

**Architecture:** Treat MCE-005 as fixed semantic input and MCE-007 as downstream consumer. Scope MCE-006 as a runtime/interpreter correspondence phase over semantic carriers, control/waiting representation, concurrency realization, and observable preservation. Keep it document-first and evidence-driven: inventory the runtime carriers first, then explain control/blocking/concurrency realization, then package observable-preservation and mismatch classification for MCE-007.

**Tech Stack:** Markdown planning/docs only; canonical references are [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-004](../spec/SPEC-004-SEMANTICS.md), [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md), [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md), [MCE-007](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md), and the current interpreter/runtime implementation as evidence source.

---

## Phase Overview

### Phase 63.1: Runtime Carrier Inventory and Mapping Baseline

Objective:
- Freeze where each accepted semantic carrier from MCE-005 lives in the current interpreter/runtime and classify whether the mapping is direct, distributed, implicit, or missing.

Primary artifact:
- `docs/plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md`

Expected outputs:
- semantic-carrier → runtime-structure mapping table
- residual-control representation classification
- first-pass correspondence gap list

### Phase 63.2: Control, Blocking, and Concurrency Realization

Objective:
- Explain how the runtime realizes residual workflows, blocked/suspended states, `Par` interleaving, branch-local state, helper-backed aggregation, and completion/control behavior without redesigning the semantics.

Primary future artifacts:
- `docs/plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md`
- `docs/plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md`

Expected outputs:
- residual workflow/control representation story
- blocked-state carrier story
- `Par` branch stepping and aggregation correspondence
- runtime handling of completion/control authority boundaries

### Phase 63.3: Observable Preservation and MCE-007 Handoff

Objective:
- Verify how runtime execution preserves the accepted terminal observables and classify whether the current interpreter already realizes the accepted backbone, partially realizes it, or needs contract-preserving follow-up.

Primary artifact:
- `docs/plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md`

Expected outputs:
- terminal observable preservation checklist
- divergence classification (`safe indirection`, `documentation gap`, `correspondence risk`, `follow-up required`)
- concise runtime evidence packet for MCE-007 / TASK-398 ingestion

---

## Recommended Task Sequence

### Task 1: Create the runtime carrier inventory and mapping table

Status: Complete.

**Objective:** Freeze one canonical runtime mapping baseline for the accepted MCE-005 semantic carriers. Completed by TASK-401.

**Files:**
- Created: `docs/plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`

**Step 1: Record the current planning gap**

Document that MCE-006 currently has:
- the right runtime-alignment questions,
- but no execution-ready task scaffold,
- no canonical carrier-to-runtime mapping table,
- no explicit direct/distributed/implicit/missing status for each carrier.

**Step 2: Freeze the accepted carrier inventory**

Use the accepted MCE-005 carrier set exactly:
- `A = (C, P)`
- `Γ`
- `Ω`
- `π`
- `T`
- `ε̂`
- residual workflow / terminal class

**Step 3: Freeze the mapping quality vocabulary**

At minimum distinguish:
- direct,
- distributed,
- implicit,
- missing.

**Step 4: Record first-pass gaps**

Separate:
- semantically safe representation indirection,
- documentation-only gaps,
- correspondence-risk gaps.

### Task 2: Plan residual control, blocked-state, and completion realization

**Objective:** Define the task that explains how residual execution, waiting, and completion authority are carried operationally.

**Files:**
- Create: `docs/plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

**Step 1: Freeze the control-representation decision space**

Classify the runtime execution model as:
- residual AST execution,
- frame/continuation execution,
- hybrid current-node + frames.

**Step 2: Freeze the blocked-state question set**

Require explicit runtime representation for:
- blocking `Receive`,
- mailbox/control waits,
- completion observation waits.

**Step 3: Freeze completion/control correspondence**

Require a concrete story for:
- control handles,
- completion payload sealing,
- retained or tombstoned completion records,
- repeated observation boundaries.

### Task 3: Plan `Par` interleaving and aggregation correspondence

**Objective:** Define the task that explains how runtime branch stepping preserves the accepted helper-backed concurrency contract.

**Files:**
- Create: `docs/plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`

**Step 1: Freeze the branch-state questions**

Require explicit documentation of:
- where branch-local state lives,
- where branch-local observables accumulate,
- when aggregation begins.

**Step 2: Preserve the determinism boundary**

State clearly that MCE-006 may explain scheduler latitude, but may not collapse accepted helper-backed concurrency into accidental left-to-right sequential execution.

**Step 3: Freeze the helper-backed aggregation story**

Require a concrete mapping to the runtime/helper path that preserves the accepted `Par` terminal aggregation contract.

### Task 4: Plan observable preservation and MCE-007 handoff

**Objective:** Define the final MCE-006 artifact that packages runtime correspondence evidence for MCE-007.

**Files:**
- Create: `docs/plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md`
- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`

**Step 1: Freeze the observable checklist**

Require explicit preservation or reconstruction stories for:
- return / reject class,
- trace,
- effect summary,
- obligations,
- provenance,
- blocked vs terminal vs invalid status.

**Step 2: Freeze divergence categories**

At minimum distinguish:
- semantically safe indirection,
- documentation gap,
- correspondence risk,
- contract-preserving follow-up required.

**Step 3: Freeze the handoff packet for MCE-007**

Produce a concise evidence packet that TASK-398 can later ingest without reinterpreting MCE-006 from scratch.

---

## Key Decisions This Plan Must Force

1. **Runtime control representation**
- Is residual execution best described as residual AST, continuations/frames, or a hybrid?

2. **Carrier placement**
- Where do `A`, `Γ`, `Ω`, `π`, `T`, `ε̂`, and terminal state live at runtime?

3. **Blocked-state carrier**
- How are blocked/suspended states represented operationally and distinguished from invalid/stuck/failure states?

4. **`Par` execution model**
- How are branch stepping, branch-local state, and helper-backed aggregation realized without violating the accepted semantic latitude?

5. **Observable split preservation**
- How does the runtime preserve or reconstruct the distinction between cumulative state and local step deltas?

6. **Provenance and obligation commit boundaries**
- When and where are `Ω` and `π` updated and committed?

7. **Completion/control realization**
- How do `ControlLink` and `CompletionPayload` semantics map to concrete runtime entities?

8. **Current-interpreter verdict**
- Does the current interpreter already realize the accepted backbone, mostly realize it with documentation gaps, or require contract-preserving follow-up?

---

## Risks and Over-Scoping Traps

1. Letting MCE-006 become a runtime rewrite or cleanup phase.
2. Reopening MCE-005 semantics.
3. Collapsing into MCE-007 full-stack matrix work.
4. Introducing fairness/scheduler-theory obligations.
5. Treating MCE-008 runtime cleanup as part of required MCE-006 scope.
6. Going too low-level too early instead of freezing the carrier mapping baseline first.

---

## Acceptance Criteria

This plan is successful when:
- a dedicated MCE-006 phase scaffold exists,
- TASK-401 exists and clearly freezes the runtime carrier mapping baseline,
- later tasks cleanly separate carrier mapping, control/blocking/completion realization, `Par` correspondence, and observable-preservation/handoff packaging,
- MCE-005 semantic ownership is preserved,
- MCE-007 has a clear runtime-evidence packet to consume later.

---

## Suggested Task Titles

- TASK-401: Runtime Carrier Inventory and Semantic Mapping Table
- TASK-402: Residual Control, Blocked-State, and Completion Realization
- TASK-403: `Par` Interleaving, Branch-Local State, and Aggregation Correspondence
- [TASK-404](../plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md): Observable Preservation, Gap Classification, and MCE-007 Handoff
