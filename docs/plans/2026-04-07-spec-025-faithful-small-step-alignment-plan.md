# SPEC-025 Faithful Small-Step Alignment Plan

> **For Hermes:** Use subagent-driven-development to execute this phase task-by-task. The phase is docs/spec-planning only. Do not redesign runtime behavior or introduce new user-visible workflow forms. The target is one faithful [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) that preserves the accepted [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md) semantic backbone, remains compatible with [SPEC-004](../spec/SPEC-004-SEMANTICS.md), and is honest about the current runtime/interpreter evidence recorded in [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).

**Goal:** Add a new planning phase that turns SPEC-025 from an initial distilled small-step spec into a faithful long-lived normative document grounded jointly in accepted MCE-005 semantics and MCE-006 runtime correspondence, without drifting away from SPEC-004 big-step semantics or overclaiming current implementation support.

**Architecture:** Treat MCE-005 as the accepted semantic owner for workflow-first small-step structure and MCE-006 as the accepted runtime-correspondence evidence packet. Scope the work as a contract-first docs/spec alignment phase: first freeze the faithfulness/compatibility contract, then tighten rule/helper wording in SPEC-025, then run an explicit compatibility audit against SPEC-004 and current interpreter evidence, then close out the surrounding corpus so SPEC-025 becomes the stable spec home for this material.

**Tech Stack:** Markdown planning/spec documents only; canonical references are [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-004](../spec/SPEC-004-SEMANTICS.md), [SPEC-021](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md), and [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).

---

## Phase Overview

### Phase 66.1: Faithfulness and Compatibility Contract

Objective:
- Freeze what “faithful SPEC-025” means relative to MCE-005, MCE-006, SPEC-004, and the current implementation.

Primary artifact:
- `docs/plan/tasks/TASK-424-spec-025-faithfulness-and-compatibility-contract.md`

Expected outputs:
- one explicit faithfulness contract for SPEC-025
- one list of required preserved semantic decisions from MCE-005
- one list of required runtime-correspondence constraints from MCE-006
- one list of non-goals preventing runtime redesign or speculative formal overreach

### Phase 66.2: Rule and Helper-Boundary Consolidation

Objective:
- Tighten SPEC-025 so its rule inventory, helper boundaries, blocked-state story, and observability split read as a faithful spec rather than only as an initial distillation.

Primary artifact:
- `docs/plan/tasks/TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md`

Expected outputs:
- clearer rule-family presentation in SPEC-025
- explicit preservation of helper-owned semantic boundaries from SPEC-004/MCE-005
- clearer distinction between normative semantics and informative runtime notes

### Phase 66.3: Big-Step and Runtime Compatibility Audit

Objective:
- Run a conservative compatibility pass proving that SPEC-025 does not contradict SPEC-004 and does not overclaim beyond current runtime evidence.

Primary artifact:
- `docs/plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md`

Expected outputs:
- one compatibility matrix from SPEC-025 sections to SPEC-004 terminal/boundary contracts
- one compatibility matrix from SPEC-025 runtime-alignment statements to MCE-006 evidence
- explicit weak/missing labels where implementation support is partial

### Phase 66.4: SPEC-025 Closeout and Corpus Alignment

Objective:
- Apply the faithful rewrite to SPEC-025 and update surrounding planning/reporting surfaces so SPEC-025 becomes the canonical docs/spec home for the accepted small-step story.

Primary artifact:
- `docs/plan/tasks/TASK-427-spec-025-faithful-closeout-and-corpus-alignment.md`

Expected outputs:
- revised SPEC-025
- updated MCE-005 / MCE-006 cross-references where needed
- updated planning/reporting/index surfaces
- concise changelog entry for the faithful-spec promotion pass

---

## Recommended Task Sequence

### Task 1: Freeze the faithfulness contract for SPEC-025

**Objective:** Define the exact contract that a faithful SPEC-025 must satisfy.

**Files:**
- Create: `docs/plan/tasks/TASK-424-spec-025-faithfulness-and-compatibility-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

**Step 1:** Record the current gap: SPEC-025 exists, but it is still only an initial distillation and lacks one explicit phase-owned faithfulness contract.

**Step 2:** Freeze the required preserved inputs from MCE-005:
- workflow-first semantic subject
- canonical configuration vocabulary
- state/label observability split
- blocked vs stuck distinction
- atomic expression/pattern/helper boundaries
- interleaving plus helper-backed `Par` aggregation

**Step 3:** Freeze the required preserved runtime-evidence constraints from MCE-006:
- do not claim authoritative cumulative carriers where the runtime only partially realizes them
- keep runtime notes explicitly evidence-based
- separate semantic contract from current implementation approximation

**Step 4:** Freeze non-goals:
- no runtime redesign
- no new workflow syntax
- no fairness theorem
- no overclaim of `π`, `T`, `ε̂`, or completion-payload closure

### Task 2: Plan the faithful rule/helper consolidation for SPEC-025

**Objective:** Define how SPEC-025 should be rewritten so its semantics remain faithful and legible.

**Files:**
- Create: `docs/plan/tasks/TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md`
- Reference: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`

**Step 1:** Freeze the rule-presentation goals:
- preserve rule inventory coverage
- make helper-owned boundaries explicit
- distinguish schematic rule families from fully formal inference schemata

**Step 2:** Freeze the normative/informative split:
- normative: judgments, configurations, labels, blocking contract, correspondence contract, rule families
- informative: current runtime evidence, implementation examples, operational notes

**Step 3:** Require explicit wording that helper names in SPEC-025 are ownership markers, not required Rust API names.

### Task 3: Plan the SPEC-025 compatibility audit

**Objective:** Define the evidence-driven audit that checks SPEC-025 against both big-step semantics and current implementation.

**Files:**
- Create: `docs/plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

**Step 1:** Require a SPEC-025 → SPEC-004 compatibility table covering:
- terminal outcome reconstruction
- helper boundaries
- receive blocking semantics
- `Par` aggregation/determinism boundaries
- spawned-child completion/control ownership

**Step 2:** Require a SPEC-025 → MCE-006 evidence table covering:
- blocked/suspended runtime classification
- residual control representation
- `Par` realization notes
- retained completion / control evidence
- partial/missing cumulative carriers

**Step 3:** Require explicit classification for each row:
- faithful and directly supported
- faithful but reconstructed/approximated
- faithful but weak/missing implementation support
- wording change required to avoid overclaim

### Task 4: Plan the final closeout and corpus alignment

**Objective:** Define the final docs/spec alignment pass that lands the faithful SPEC-025.

**Files:**
- Create: `docs/plan/tasks/TASK-427-spec-025-faithful-closeout-and-corpus-alignment.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Optional Modify: `CHANGELOG.md`

**Step 1:** Apply the approved faithful rewrite to SPEC-025.

**Step 2:** Align nearby planning/reporting surfaces so they reference SPEC-025 as the docs/spec home for the accepted small-step contract.

**Step 3:** Verify the final wording remains:
- faithful to MCE-005
- compatible with SPEC-004
- honest about MCE-006 runtime evidence
- explicit about what is still only partial in the implementation

---

## Key Decisions This Phase Must Force

1. What exact claims SPEC-025 may make normatively versus only informatively.
2. How much of MCE-005’s rule inventory should be restated directly in SPEC-025 versus referenced.
3. How runtime evidence from MCE-006 should constrain wording in SPEC-025.
4. Which implementation notes belong in SPEC-025 at all, versus remaining in MCE-006.
5. What exact compatibility checklist should gate calling SPEC-025 “faithful.”

---

## Risks and Over-Scoping Traps

1. Reopening MCE-005 semantic decisions rather than promoting them faithfully.
2. Letting SPEC-025 drift into abstract-machine or runtime redesign territory.
3. Overstating current runtime support for cumulative carriers or completion payloads.
4. Writing formal inference schemata that exceed what the current accepted corpus actually fixes.
5. Duplicating MCE-006 evidence inside SPEC-025 instead of citing it conservatively.

---

## Acceptance Criteria

This plan is successful when:
- Phase 66 exists in `PLAN-INDEX.md`.
- TASK-424 through TASK-427 exist and are concrete enough to execute without guesswork.
- The phase clearly targets a faithful SPEC-025 rather than new runtime implementation work.
- The planned deliverable explicitly requires compatibility with both SPEC-004 and current implementation evidence.

---

## Planned Task Titles

- TASK-424: SPEC-025 Faithfulness and Compatibility Contract
- TASK-425: SPEC-025 Rule-Schema and Helper-Boundary Consolidation
- TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit
- TASK-427: SPEC-025 Faithful Closeout and Corpus Alignment
