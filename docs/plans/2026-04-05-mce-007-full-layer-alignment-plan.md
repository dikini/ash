# MCE-007 Full Layer Alignment Plan

> **For Hermes:** Use subagent-driven-development to execute this plan task-by-task. TASK-397 is now reconciled as complete framing work whose intended outputs were materially realized by the published MCE-007 closeout corpus; MCE-006 Phase 63 is closed, and TASK-398 through TASK-400 now stand as the completed ingestion/classification/closeout realization path.

**Goal:** Turn MCE-007 from a broad exploration into an execution-ready five-layer alignment closeout plan that consumes accepted MCE-004 and MCE-005 results, gates on MCE-006 for runtime/interpreter evidence, and ends with a durable drift-prevention checklist.

**Architecture:** Treat MCE-004 and Phase 61 / MCE-005 as fixed upstream inputs. Treat MCE-006 as the owner of runtime/interpreter realization. Scope MCE-007 as a consolidation and verification phase over construct families, not as a new semantics-design or runtime-design effort. Build the matrix/closure contract first, then consume MCE-006 outputs, then classify residual drift and publish future-change safeguards.

**Tech Stack:** Markdown planning/docs only; canonical references are [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-004](../spec/SPEC-004-SEMANTICS.md), [MCE-004](../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md), [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md), [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md), and [MCE-007](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md).

---

## Phase Overview

### Phase 62.1: Matrix Baseline and Closure Contract

Objective:
- Define the canonical five-layer matrix, evidence expectations, and closure criteria without claiming runtime/interpreter alignment is already done.

Primary artifact:
- `docs/plan/tasks/TASK-397-five-layer-alignment-matrix-and-closure-contract.md`

Expected outputs:
- construct-family matrix shape
- per-boundary evidence model
- row status vocabulary
- MCE-006 gating language
- explicit non-goals and exit criteria

### Phase 62.2: Runtime/Interpreter Evidence Ingestion

Objective:
- Consume MCE-006 runtime/interpreter mapping results into the MCE-007 matrix rather than re-deriving them in MCE-007.

Primary artifact:
- `docs/plan/tasks/TASK-398-runtime-interpreter-correspondence-ingestion-for-mce-007.md`

Expected outputs:
- filled Small-step → Interpreter evidence column by construct family
- explicit runtime-preservation notes for blocking, `Par`, spawn/completion, and observables
- clear classification of what is now closed versus what remains a true residual gap

Status after TASK-398:
- complete as an ingestion phase: the current MCE-007 matrix now carries row-level Small-step → Interpreter classifications derived from the frozen Phase 63 packet.
- residual work remains intentionally downstream in TASK-399 and TASK-400.

### Phase 62.3: Residual Drift Classification and Future-Change Safeguards

Objective:
- Resolve or classify remaining cross-layer drift and publish a durable checklist for preventing future layer divergence.

Primary future artifacts:
- `docs/plan/tasks/TASK-399-five-layer-drift-resolution-and-residual-gap-classification.md`
- `docs/plan/tasks/TASK-400-mce-007-closeout-summary-and-drift-prevention-checklist.md`

Expected outputs:
- residual gap table with owners
- explicit alignment signoff conditions
- future-change checklist covering surface lowering, SPEC-001, SPEC-004, MCE-005, MCE-006, and runtime-facing docs

Status after TASK-399:
- complete as a residual-classification phase: MCE-007 now distinguishes packaging-only closeout work and accepted owner-bound partiality from true residual drift.
- the true residual drift set is now frozen to blocked/terminal/invalid runtime classification, authoritative cumulative carrier packaging (`Ω` / `π` / `T` / `ε̂`), retained completion-payload observation, and full helper-backed `Par` aggregation.
- TASK-400 was the planned final closeout/checklist step.

Status after TASK-400:
- complete as a closeout/signoff phase: MCE-007 now publishes the accepted five-layer matrix state, the current residual-gap register, explicit signoff conditions, and a future-change drift-prevention checklist.
- the closeout explicitly preserves mixed rows, especially sequencing / binding / branching as accepted local execution alignment plus unresolved cumulative-carrier drift.
- MCE-007 is now complete as a documentation/planning closeout artifact even though the frozen true residual drift set remains open for later runtime/interpreter follow-on.

---

## Recommended Task Sequence

### Task 1: Create the MCE-007 matrix and closure contract

Status: TASK-397 reconciled complete as earlier scaffold/framing work; its intended outputs were realized incrementally by the final published MCE-007 matrix, residual-gap layer, and closeout/signoff contract.

**Objective:** Freeze the alignment frame so later MCE-007 work has one canonical matrix, one evidence model, and one closure vocabulary.

**Files:**
- Create: `docs/plan/tasks/TASK-397-five-layer-alignment-matrix-and-closure-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`

**Step 1: Record the current planning gap**

Document that MCE-007 currently has:
- a useful problem statement,
- a useful five-layer picture,
- but no execution-ready matrix artifact,
- no status vocabulary for construct-family rows,
- no explicit closure contract for “alignment complete”.

**Step 2: Freeze the alignment unit**

Use construct families, not:
- surface-sugar variants,
- per-rule proof artifacts,
- runtime helper implementation details.

Required baseline construct families:
- sequencing / binding / branching,
- pattern-driven control,
- receive / blocking behavior,
- parallel composition,
- capability / policy / obligation workflows,
- spawn / completion observation contracts.

**Step 3: Freeze the evidence model**

For each row, require explicit evidence or owner references for:
- Surface → IR,
- IR → Big-step,
- Big-step → Small-step,
- Small-step → Interpreter.

**Step 4: Freeze the status vocabulary and exit criteria**

At minimum distinguish:
- closed,
- closed pending packaging,
- pending MCE-006,
- residual drift / follow-up required.

**Step 5: Fence the boundary to MCE-006**

State clearly that MCE-007 consumes runtime/interpreter evidence from MCE-006 and does not redesign:
- blocked-state carriers,
- runtime scheduler/queue structures,
- continuation or residual-workflow representations,
- concrete trace/effect accumulation data structures.

Completed result:
- TASK-397 should now be read as the framing task that defined the intended matrix/closure shape.
- Those outputs were later realized materially by the published MCE-007 corpus, especially through the final matrix, the residual-gap classification layer, and the closeout/signoff contract.
- This reconciliation closes the planning/documentation scaffold without claiming that the remaining runtime-side true residual drift has been resolved.

### Task 2: Runtime/interpreter evidence ingestion

Status: TASK-398 complete.

**Objective:** Import the frozen MCE-006 outputs into the MCE-007 matrix without inventing new runtime theory.

**Files:**
- Create: `docs/plan/tasks/TASK-398-runtime-interpreter-correspondence-ingestion-for-mce-007.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`

**Step 1: Identify the required MCE-006 inputs**

At minimum, MCE-007 must consume MCE-006 evidence for:
- workflow-configuration to runtime-structure mapping,
- blocked/suspended state representation,
- `Par` interleaving realization,
- terminal observable preservation,
- spawn/completion runtime authority handling.

**Step 2: Define the ingestion target**

This task should fill the Small-step → Interpreter column per construct family and classify rows conservatively as:
- closed,
- closed with residual note,
- partial / follow-up required.

**Step 3: Preserve the owner boundary**

State explicitly that TASK-398 consumes evidence from MCE-006 and may summarize it, but may not invent replacement runtime theory.

Completed result:
- the Small-step → Interpreter column is now populated by construct family using the frozen Phase 63 evidence packet;
- all current rows remain conservative `partial / follow-up required` classifications to some degree, with stronger direct evidence for ordinary residual execution and pattern-driven control than for receive/blocking, `Par`, capability/policy/obligation workflows, and spawn/completion retention;
- MCE-007 now consumes, rather than merely references, the frozen Phase 63 evidence packet.
### Task 3: Residual drift classification

Status: TASK-399 complete.

**Objective:** Define how MCE-007 will distinguish true remaining cross-layer drift from merely documented, accepted boundary ownership.

**Files:**
- Create: `docs/plan/tasks/TASK-399-five-layer-drift-resolution-and-residual-gap-classification.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`

**Step 1: Define residual-gap categories**

Use categories such as:
- closed,
- packaging-only,
- accepted partiality with an explicit owner,
- true residual drift requiring follow-on work.

**Step 2: Require explicit owners**

Every non-closed row must name its owner:
- MCE-007 closeout / TASK-400,
- later runtime cleanup,
- future semantics/spec update,
- or explicit follow-on task.

Completed result:
- MCE-007 now includes a dedicated residual-gap classification layer over the ingested matrix.
- Remaining issues are split conservatively into packaging-only work, accepted owner-bound partiality, and true residual drift.
- Direct residual execution and rejected-vs-runtime-failure taxonomy cleanup are no longer treated as generic drift claims.
- The true residual drift set is frozen for downstream closeout work: blocked/terminal/invalid runtime classification, authoritative cumulative carrier packaging, retained completion observation, and helper-backed cumulative-state aggregation for `Par`.

### Task 4: Plan final closeout and drift-prevention checklist

Status: TASK-400 complete.

**Objective:** Define the final artifact that turns MCE-007 into a durable future-change guardrail rather than a one-time review note.

**Files:**
- Create: `docs/plan/tasks/TASK-400-mce-007-closeout-summary-and-drift-prevention-checklist.md`
- Optional Create: `docs/reference/five-layer-alignment-matrix.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

**Step 1: Define the closeout artifact**

It should publish:
- the accepted five-layer matrix,
- the remaining residual issues, if any,
- one future-change checklist for keeping all layers aligned.

**Step 2: Define the future-change checklist scope**

When a canonical construct changes, the checklist should force review of:
- surface lowering docs,
- `SPEC-001`,
- `SPEC-004`,
- MCE-005,
- MCE-006,
- interpreter/runtime-facing docs,
- MCE-007 matrix rows.

Completed result:
- MCE-007 now contains a final closeout/signoff section that freezes the accepted matrix state and residual register in one place.
- The previous packaging-only big-step ↔ small-step item is now consumed by the closeout prose rather than left as open follow-up.
- The signoff conditions distinguish "documentation closeout complete" from the stronger claim that true five-layer runtime drift is fully resolved.
- The drift-prevention checklist now serves as the durable guardrail for future construct, semantics, and runtime changes.

---

## Key Decisions This Plan Must Force

1. **Unit of alignment**
- MCE-007 aligns construct families, not individual helper internals or every surface-sugar variant.

2. **Evidence model**
- Every row must say what counts as evidence for each adjacent layer boundary.

3. **Closed vs pending semantics**
- The plan must distinguish “upstream aligned, runtime pending” from “true drift”.

4. **Owner boundaries**
- Runtime/interpreter realization belongs to MCE-006.
- Full-stack verification and closeout packaging belong to MCE-007.

5. **Drift-prevention output**
- MCE-007 must end with a reusable future-change checklist, not just a one-time summary.

---

## Risks and Over-Scoping Traps

1. Turning MCE-007 into “finish MCE-006 inside MCE-007”.
2. Reopening accepted MCE-004 or Phase 61 / MCE-005 decisions.
3. Going too granular with per-rule or per-helper proof obligations.
4. Mixing runtime cleanup/refactoring with alignment closeout.
5. Expanding beyond canonical minimal-core construct families.
6. Claiming final alignment before MCE-006 provides runtime/interpreter evidence.

---

## Acceptance Criteria

This plan is successful when:
- a dedicated MCE-007 task scaffold exists,
- TASK-397 exists as the original framing scaffold, and the later published MCE-007 corpus now clearly freezes the matrix/closure contract,
- the future tasks cleanly separate matrix setup, runtime evidence ingestion, residual-gap classification, and drift-prevention closeout,
- MCE-006 ownership is preserved,
- later MCE-007 execution can proceed without re-litigating scope.

---

## Suggested Task Titles

- TASK-397: Five-Layer Alignment Matrix and Closure Contract
- TASK-398: Runtime/Interpreter Correspondence Ingestion for MCE-007
- TASK-399: Five-Layer Drift Resolution and Residual Gap Classification
- TASK-400: MCE-007 Closeout Summary and Drift-Prevention Checklist
