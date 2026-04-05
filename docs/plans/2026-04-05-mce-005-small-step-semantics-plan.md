# MCE-005 Small-Step Semantics Plan

**Goal:** Turn MCE-005 from an open-ended exploration into a scoped planning artifact aligned with the accepted canonical IR and big-step semantics, with backbone choices fixed for later rule-writing and correspondence work.

**Status:** Complete as documentation/planning work. Phase 61 landed TASK-394, TASK-395, and TASK-396; MCE-005 is now an accepted small-step planning/design backbone, and MCE-006 now consumes that fixed target.

**Architecture:** Treat [MCE-004](../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md) and the current [SPEC-001](../spec/SPEC-001-IR.md) / [SPEC-004](../spec/SPEC-004-SEMANTICS.md) corpus as fixed inputs. Split MCE-005 into a narrow semantic-backbone task, a rule-completion task, and a big-step correspondence / MCE-006 handoff task. Keep MCE-005 workflow-centric and execution-neutral; defer runtime-machine realization to [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).

**Tech Stack:** Markdown planning/docs only; canonical references are [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-004](../spec/SPEC-004-SEMANTICS.md), and the MCE exploration notes.

---

## Phase Overview

Historical note:
- This document now serves as a completed historical plan for Phase 61.
- The task sequence below records what Phase 61 set out to do and what was subsequently completed in the corpus.

### Phase 61.1: Semantic Backbone Closure

Objective:
- Freeze the semantic subject, configuration shape, observability strategy, and scope boundary for MCE-005.

Primary artifact:
- `docs/plan/tasks/TASK-394-small-step-semantics-scope-and-configuration-contract.md`

Expected outputs:
- workflow-first framing for MCE-005
- fixed inputs inherited from MCE-004
- chosen configuration strategy
- chosen observability/labeled-step strategy
- explicit decision about whether observables live in configuration state, transition labels, or a deliberate split across both
- explicit blocked vs stuck distinction
- explicit defer list to TASK-395 / MCE-006

### Phase 61.2: Workflow Rule Completion

Objective:
- Define the per-form small-step rule inventory and fill in the canonical workflow rule set.

Primary artifact:
- `docs/plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md`

Expected outputs:
- rule families over canonical workflow forms
- concurrency/interleaving contract
- preserve `SPEC-004` helper contracts, helper-owned nondeterminism, and determinism boundaries
- explicit statement of what remains atomic (e.g. pure expressions, if retained as atomic in v1)

### Phase 61.3: Correspondence and Handoff

Objective:
- Make MCE-005 usable as a prerequisite for MCE-006 and MCE-007.

Primary artifact:
- `docs/plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md`

Expected outputs:
- big-step ↔ small-step correspondence matrix
- checklist of semantic observables/step contracts MCE-006 must preserve
- deferred runtime/interpreter questions fenced into MCE-006

---

## Executed Work

### Task 1: Create and land TASK-394

Status: completed in Phase 61.

**Objective:** Establish the foundational planning task for MCE-005.

**Files:**
- Create: `docs/plan/tasks/TASK-394-small-step-semantics-scope-and-configuration-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`

**Step 1: Record the problem precisely**

Document that MCE-005 is currently too exploratory because it still leaves open:
- configuration shape,
- workflow-vs-expression semantic subject,
- observability/labeled-step strategy,
- blocked-vs-stuck distinction,
- MCE-005 vs MCE-006 boundary.

**Step 2: Define the target state**

Record that the target is a canonical-IR small-step planning/design artifact aligned with:
- `SPEC-001` workflow forms,
- `SPEC-004` semantic domains and helper contracts,
- accepted MCE-004 decisions.

**Step 3: Add the planning phase to PLAN-INDEX**

Add Phase 61 with TASK-394, TASK-395, and TASK-396 planning entries. This was completed in the final Phase 61 closeout with real task files and complete status tracking.

**Step 4: Verify**

Check:
- `TASK-394` exists,
- `PLAN-INDEX` includes a new MCE-005 phase,
- the phase deliverable clearly states what MCE-005 must produce.

### Task 2: Rewrite MCE-005 around canonical workflow semantics

Status: completed in Phase 61.

**Objective:** Replace open-ended exploratory framing in `MCE-005-SMALL-STEP.md` with a planning backbone for later rule-writing and correspondence work.

**Files:**
- Modify: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md`
- Reference: `docs/spec/SPEC-001-IR.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`

**Step 1: Remove stale/non-canonical framing**

Specifically fence off or replace:
- expression-only emphasis,
- speculative `await` user-syntax framing,
- generic mutable-store assumptions not grounded in current specs.

**Step 2: Add fixed-input section**

Explicitly carry forward from accepted corpus:
- `Workflow::Seq` primitive,
- `Expr::Match` primitive,
- `if let` lowers to `Expr::Match`,
- `Par` big-step aggregation fixed,
- spawn completion payload semantics fixed.

**Step 3: Add chosen semantic-backbone section**

Include:
- workflow-level configuration subject,
- ambient vs dynamic state split,
- observability strategy,
- blocked vs stuck semantics,
- v1 treatment of pure expressions/patterns.

**Step 4: Add defer/boundary section**

State clearly:
- TASK-395 owns rule completion,
- TASK-396 owns correspondence/handoff packaging,
- MCE-006 owns runtime/interpreter realization.

### Task 3: Surrounding planning/reporting consistency updates

Status: completed in Phase 61.

**Objective:** After TASK-394 materially reshapes MCE-005, update nearby planning/reporting docs that would otherwise lag behind the new planning state.

**Files:**
- Optional Modify: `docs/ideas/README.md` (if notes/status text should mention planning linkage)
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`

**Step 1: Update implementability framing**

Change MCE-005 from an open theory note to an accepted planning/design backbone. This was completed during the Phase 61 closeout.

**Step 2: Tighten MCE-006 dependency wording**

Make explicit that MCE-006 consumes the frozen semantic backbone from MCE-005, especially:
- configuration model,
- step granularity,
- helper-owned nondeterminism,
- terminal/observable contracts.

**Step 3: Optional MCE-007 consistency sweep**

If needed, update MCE-007 to reference the new Phase 61 / TASK-394 planning surface.

---

## Key Backbone Decisions This Plan Must Fix for Follow-On Work

1. **Semantic subject**
- MCE-005 must be workflow-first, not surface-first and not expression-only.

2. **Configuration shape**
- Choose a representation compatible with the current canonical semantic vocabulary instead of leaving multiple options open.

3. **Observability strategy**
- Decide whether transitions are labeled or otherwise explicitly observable.

4. **Pure expression treatment**
- Decide whether pure expressions remain atomic in v1 by reusing `SPEC-004` subjudgments.

5. **Blocked vs stuck**
- Separate expected waiting/suspension from malformed/no-rule states.

6. **MCE-005 vs MCE-006 boundary**
- Prevent runtime-machine/interpreter structure design from leaking into MCE-005.

---

## Risks and Over-Scoping Traps

1. Re-opening MCE-004 decisions through MCE-005.
2. Inventing user-visible `await` or other non-canonical forms.
3. Micro-stepping pure expressions before the workflow-level semantics are stabilized.
4. Over-specifying scheduler/fairness internals too early.
5. Turning MCE-005 into an abstract machine design instead of a semantics plan.
6. Forgetting to preserve `SPEC-004` helper contracts and determinism boundaries.

---

## Acceptance Criteria

This plan is successful when:
- Phase 61 is marked complete in `PLAN-INDEX`.
- TASK-394, TASK-395, and TASK-396 exist and are concrete enough to execute without guesswork.
- MCE-005 is revised into an accepted planning/design artifact without re-opening MCE-004.
- MCE-006 has a clean, explicit upstream dependency contract.

---

## Phase 61 Delivered Task Titles

- TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics
- TASK-396: Small-Step / Big-Step Correspondence Matrix and MCE-006 Handoff
