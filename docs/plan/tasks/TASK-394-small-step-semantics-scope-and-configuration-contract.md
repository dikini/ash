# TASK-394: Small-Step Semantics Scope and Configuration Contract (MCE-005)

## Status: ✅ Complete

## Description

Close the foundational design decisions for MCE-005 so small-step semantics can be drafted against the accepted canonical IR and big-step semantics without re-opening MCE-004 questions. This is a documentation/planning task only. It must convert MCE-005 from an open-ended exploration into a candidate-quality semantic planning artifact that is precise enough to unblock the actual rule-writing work and the later MCE-006 runtime/interpreter alignment work.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)

## Dependencies

- ✅ [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)
- ✅ [TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics](TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)
- ✅ [TASK-370: IR Core Forms Audit](TASK-370-ir-core-forms-audit.md)

## Requirements

### Functional Requirements

1. Reframe MCE-005 so its primary semantic subject is canonical SPEC-001 workflow configurations rather than surface syntax or an expression-only calculus.
2. Record the fixed inputs inherited from current accepted corpus decisions:
   - `Workflow::Seq` remains primitive.
   - `Expr::Match` remains primitive.
   - surface `if let` lowers to `Expr::Match`.
   - `Par` big-step aggregation is already fixed in `SPEC-004`.
   - spawned-child completion semantics are already fixed in `SPEC-004`.
3. Define one explicit small-step judgment backbone for MCE-005 that is stated as a refinement of the existing workflow-level semantic worldview from `SPEC-004`, rather than as an unrelated calculus. Document:
   - the semantic subject of the transition relation,
   - how workflow configurations relate to the existing `Γ, C, P, Ω, π ⊢wf w ⇓ out` framing,
   - how the small-step presentation preserves or reconstructs the same terminal semantic dimensions already tracked by `SPEC-004` (outcome/rejection, obligations, provenance, effects, and traces, or an explicitly justified stepwise equivalent),
   - terminal configurations,
   - blocked or suspended configurations,
   - rejection/error propagation stance.
4. Choose and document one configuration strategy aligned with the current normative corpus:
   - ambient/static context should be expressed in terms compatible with `C` and `P`,
   - dynamic state should be expressed in terms compatible with the current workflow/effect/obligation/provenance vocabulary,
   - do not introduce a generic store `S` unless justified against the current spec corpus.
5. Decide the v1 treatment of pure expressions and patterns:
   - by default, reuse the explicit `SPEC-004` expression/pattern subjudgments atomically within the small-step presentation,
   - do not assume expression-level micro-stepping unless TASK-394 explicitly chooses and justifies it,
   - or explicitly justify any finer-grained expression stepping if that default is rejected.
6. Decide whether MCE-005 uses labeled transitions or an equivalent explicit observability strategy, and document the choice.
7. Make explicit whether observables such as effects, traces, provenance updates, and other execution-visible facts live in configuration state, transition labels, or a deliberate split across both; this is a central closure item for TASK-394.
8. Distinguish expected blocking/suspension from semantic stuckness so async/control behavior is plan-ready without inventing new user-visible syntax.
9. Add a resolution/defer table that says which design questions are closed by TASK-394 and which are deferred to TASK-395 or MCE-006.

### Non-Functional Requirements

1. Keep scope limited to docs/spec planning; no Rust/runtime/interpreter changes.
2. Preserve canonical terminology from `SPEC-001` and `SPEC-004`.
3. Preserve execution-neutrality: this task must not design an abstract machine or runtime data structure layout.
4. Use repo-relative references throughout.
5. Produce a planning artifact that removes enough ambiguity for follow-on rule-writing tasks to proceed without re-litigating foundations.

## TDD Evidence

### Red

Before this task, `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md` is still an exploration note with unresolved options and stale terminology drift:

- it proposes multiple competing configuration shapes instead of choosing one;
- it uses expression-centric examples rather than canonical workflow-first framing;
- it mentions concepts such as `await` and a generic mutable store that are not part of the current canonical IR/spec backbone;
- it does not clearly separate what belongs to MCE-005 from what should remain deferred to MCE-006.

### Green

This task is complete when:

- MCE-005 reads as a scoped semantic planning artifact rather than a brainstorming note;
- the workflow-level semantic subject, configuration strategy, and observability approach are fixed;
- stale or non-canonical framing is removed or explicitly fenced off;
- MCE-006 has a clear semantic handoff target rather than an open-ended dependency.

## Files

- Modify: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Optional Modify: `docs/ideas/README.md` (if status/notes need planning linkage)
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md` (if MCE-005 status/assessment wording needs to reflect the new planning phase)
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`

## TDD Steps

### Step 1: Identify the current ambiguity (Red)

Confirm that MCE-005 still leaves all of the following open:

- configuration shape,
- workflow-vs-expression semantic subject,
- observability/labeled-step strategy,
- blocking-vs-stuck distinction,
- MCE-005 vs MCE-006 boundary.

### Step 2: Define the planning target

Rewrite the task-facing target state in terms of canonical workflow semantics that refine the accepted `SPEC-004` big-step contract.

### Step 3: Freeze the semantic backbone choices

Document:

- the workflow-first semantic subject,
- the chosen configuration structure,
- the chosen observability strategy,
- the v1 treatment of pure expressions/patterns,
- the blocking/suspension contract.

### Step 4: Fence the downstream boundary

Document what remains for:

- TASK-395 (workflow rule inventory and rule writing),
- MCE-006 (runtime/interpreter correspondence),
- MCE-007 (full-stack alignment).

### Step 5: Verify GREEN

Expected pass condition:

- an implementer can read the revised MCE-005 and know exactly what semantic backbone later tasks must extend, without guessing which open questions are still in scope.

## Completion Notes

- Rewrote `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md` as an accepted planning/design artifact instead of an open exploration note.
- Fixed the canonical subject as workflow configurations over `SPEC-001` forms, refining rather than replacing the accepted `SPEC-004` workflow worldview.
- Chose one configuration contract built from `Γ`, `Ω`, `π`, cumulative trace, cumulative effect summary, and residual workflow state, with ambient `(C, P)` context.
- Chose a deliberate observability split: cumulative semantic state in configurations and local trace/effect deltas in step labels.
- Recorded the explicit blocked/suspended versus stuck distinction and fenced runtime realization work into MCE-006.

## Completion Checklist

- [x] TASK-394 task file created
- [x] `docs/plan/PLAN-INDEX.md` updated with a new MCE-005 phase entry
- [x] MCE-005 reframed as canonical workflow small-step planning artifact
- [x] fixed inputs from MCE-004 recorded explicitly
- [x] configuration strategy chosen and documented
- [x] observability/labeled-step strategy chosen and documented
- [x] blocking vs stuck distinction documented
- [x] downstream boundary to TASK-395 / MCE-006 documented

## Dependencies for Next Task

This task outputs:

- one fixed semantic backbone for small-step planning,
- one fixed vocabulary for follow-on rule-writing,
- one explicit defer list for runtime/interpreter alignment.

Required by:

- TASK-395: Canonical Workflow Small-Step Rule Set
- TASK-396: Big-Step Correspondence Matrix and MCE-006 Handoff

## Notes

Important constraints:

- Do not re-open MCE-004 decisions.
- Do not invent new user-visible syntax such as `await` unless the canonical specs are revised first.
- Do not over-specify scheduler internals or abstract-machine structure at this stage.
- Prefer the current normative vocabulary (`Γ`, `C`, `P`, `Ω`, `π`, workflow outcomes, helper contracts, determinism boundaries) over speculative notation drift.

Edge cases to consider:

- expected waiting states for receive/control interactions,
- parallel interleaving without scheduler over-commitment,
- preserving the already-accepted spawn completion contract,
- ensuring pure-expression treatment does not explode scope.
