---
status: accepted
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-394, TASK-395, TASK-396]
tags: [semantics, small-step, workflows, concurrency, alignment]
---

# MCE-005: Small-Step Semantics

## Status Summary

This exploration is accepted as the Phase 61 planning/design backbone for canonical small-step semantics.

It no longer serves as an open options note. Its job is now narrower and fixed:

- define the canonical workflow-level small-step subject for the `SPEC-001` core workflows;
- refine, rather than replace, the accepted `SPEC-004` big-step worldview;
- fix the configuration contract, observability split, and blocked-vs-stuck distinction;
- record the rule inventory that later formal rule writing must cover;
- hand off runtime/interpreter realization questions to [MCE-006](MCE-006-SMALL-STEP-IR.md).

This is documentation/spec-planning work only. It does not claim a Rust interpreter, abstract machine, or runtime implementation.

## Scope

In scope:

- canonical `SPEC-001` workflow configurations as the semantic subject;
- a single chosen transition backbone for workflow execution;
- stepwise preservation of `SPEC-004` terminal semantic dimensions;
- canonical workflow-form rule inventory, including `Par` interleaving and blocking boundaries;
- a clean semantic handoff to [MCE-006](MCE-006-SMALL-STEP-IR.md) and [MCE-007](MCE-007-FULL-ALIGNMENT.md).

Out of scope:

- Rust/runtime/interpreter implementation changes;
- a concrete abstract-machine layout;
- new user-visible syntax such as `await`;
- reopening the `MCE-004` decisions already fixed by the accepted corpus;
- expression-level micro-stepping in v1.

## Fixed Inputs from the Accepted Corpus

MCE-005 treats the following as fixed inputs rather than open design questions:

1. `Workflow::Seq` remains a primitive canonical workflow form. See [MCE-004](MCE-004-BIG-STEP-ALIGNMENT.md) and [SPEC-001](../../spec/SPEC-001-IR.md).
2. `Expr::Match` remains a primitive canonical expression form.
3. Surface `if let` lowers to canonical `Expr::Match`; MCE-005 does not add a separate core runtime form for it.
4. `Par` big-step aggregation is already fixed in [SPEC-004](../../spec/SPEC-004-SEMANTICS.md): successful branch effects join, while concurrent aggregation details remain helper-backed.
5. Spawned-child completion semantics are already fixed in [SPEC-004](../../spec/SPEC-004-SEMANTICS.md): the runtime seals the child workflow's authoritative terminal state into `CompletionPayload` via `ControlLink` authority.
6. `SPEC-004` helper contracts and determinism boundaries remain normative. MCE-005 may refine execution into steps, but it must not erase helper-owned nondeterminism or invent machine-specific guarantees.

These decisions close the stale drift in the earlier note:

- no generic store `S` is introduced as a new semantic center;
- no user-visible `await` form is introduced;
- the semantics are workflow-first, not expression-centric.

## Chosen Semantic Backbone

### 1. Semantic Subject

The primary subject is the canonical workflow configuration:

```text
A ⊢ κ —μ→ κ'
```

where:

- `A = (C, P)` is the ambient capability/policy context inherited from `SPEC-004`;
- `κ` is a workflow configuration carrying the current workflow plus dynamic execution state;
- `μ` is a step label recording the local observable delta for that transition.

This is a refinement of the `SPEC-004` workflow judgment

```text
Γ, C, P, Ω, π ⊢wf w ⇓ out
```

not a replacement for it. Big-step says what the whole workflow means at termination. Small-step says how that same canonical workflow meaning is reached one configuration transition at a time.

### 2. Configuration Shape

The chosen v1 running configuration is:

```text
κ ::= Running(Γ, Ω, π, T, ε̂, w)
    | Returned(v, Ω, π, T, ε̂)
    | Rejected(err, Ω, π, T, ε̂)
```

with:

- `Γ`: runtime environment, matching the `SPEC-004` workflow/expression/pattern vocabulary;
- `Ω`: current obligation state;
- `π`: current provenance state;
- `T`: cumulative execution trace prefix gathered so far;
- `ε̂`: running effect-summary accumulator whose terminal projection reconstructs the `SPEC-004` effect result and `EffectTrace` view;
- `w`: the residual canonical workflow from `SPEC-001`.

This deliberately reuses the existing semantic carriers instead of introducing a generic mutable store. If a later runtime uses stacks, heaps, queues, or tombstones internally, that belongs to [MCE-006](MCE-006-SMALL-STEP-IR.md), not here.

### 3. Chosen Observability Strategy

MCE-005 adopts a deliberate split across configuration state and transition labels.

The transition judgment is:

```text
A ⊢ κ —μ→ κ'
```

where the label `μ` carries only the local observable delta of the step:

```text
μ ::= silent
    | emit(ΔT, δε)
```

with:

- `ΔT`: the trace fragment emitted by that step, possibly empty;
- `δε`: the effect-layer contribution incurred by that step, if any.

The authoritative cumulative semantic state lives in the configuration, not in the label:

- obligations live in `Ω`;
- provenance lives in `π`;
- cumulative trace lives in `T`;
- cumulative effect summary lives in `ε̂`;
- terminal success/rejection lives in `Returned(...)` / `Rejected(...)`.

Why this split:

- labels make local step observability explicit enough for correspondence and interpreter handoff work;
- state remains the authoritative carrier for the terminal information that `SPEC-004` reports;
- provenance and obligation updates are better read as state transitions than as duplicated label payload.

### 4. Relationship to `SPEC-004` Terminal Outcomes

The terminal projection is direct:

- `Returned(v, Ω', π', T, ε̂')` reconstructs `Return(v, eff', T, Ω', π')`;
- `Rejected(err, Ω', π', T, ε̂')` reconstructs `Reject(err, eff', T, Ω', π')`;
- `eff'` is read from the terminal projection of `ε̂'`, with the same terminal effect/effect-summary intent that `SPEC-004` uses.

So the `SPEC-004` terminal semantic dimensions are preserved as follows:

| `SPEC-004` dimension | Small-step carrier |
|---|---|
| return value / rejection error | terminal configuration form |
| obligation state | configuration state `Ω` |
| provenance | configuration state `π` |
| trace | cumulative configuration state `T`, with per-step `ΔT` labels |
| effect result / effect summary | cumulative configuration state `ε̂`, with per-step `δε` labels |
| helper-owned observable boundaries | preserved at the owning step family; not flattened into machine internals |

This is the core refinement claim of MCE-005: repeated small steps must reconstruct the same terminal meaning already specified big-step.

## Blocked, Suspended, and Stuck

MCE-005 makes the waiting contract explicit.

### Blocked or suspended

A configuration is blocked/suspended when it is well-formed and semantically owned by an external/helper condition, but no progress step is currently available.

Canonical v1 examples:

- `Receive` in blocking mode when no arm is currently selectable;
- helper-owned waiting on a runtime-controlled mailbox/control outcome;
- runtime-owned child-completion observation boundaries relevant to `ControlLink` handoff work, even though no user-visible `await` syntax exists.

Blocked/suspended is not an error. It means the configuration is waiting for a helper/runtime condition that is intentionally outside the pure reduction relation.

### Stuck

A configuration is stuck when:

- it is not terminal;
- it is not classified as blocked/suspended;
- no rule applies.

Stuckness is therefore not a normal execution status. In the accepted corpus it indicates one of two things:

1. an inadmissible state that should already have been owned by a `SPEC-004` failure boundary such as `PatternBindFailure`, `PatternMatchFailure(v)`, or `RuntimeFailure(reason)`; or
2. a defect in the small-step presentation itself.

The small-step corpus must therefore aim for: terminal, progress, or blocked/suspended — but not ordinary user-visible stuckness.

## v1 Atomic Boundaries

MCE-005 chooses conservative granularity in v1.

The following remain atomic subjudgments or helper-owned steps rather than being micro-stepped internally:

1. Pure expression evaluation via `Γ ⊢e expr ⇓ v` from `SPEC-004`.
2. Pure pattern matching and required binding via `Γ ⊢p pat ⇐ v ⇓ ΔΓ` and the existing pattern helpers.
3. Guard evaluation and policy lookup at the currently specified helper/workflow boundaries.
4. Receive-arm selection via `select_receive_outcome(...)`.
5. Parallel outcome aggregation via `combine_parallel_outcomes(...)`.
6. Obligation discharge/join, provenance extension/join, and other existing semantic helpers.
7. Spawn completion sealing and control-authority observation contracts from `SPEC-004`.

This keeps MCE-005 workflow-centric and prevents expression-level or runtime-machine design from swallowing the phase.

## Canonical Workflow Rule Inventory

The v1 rule inventory is defined over the canonical `SPEC-001` workflow forms.

### 1. Terminal / structural families

- `DONE-TERM`: `Done` steps or classifies directly as a terminal no-op boundary, depending on the later formal presentation.
- `RET-EVAL` / `RET-RETURN`: evaluate the return expression atomically, then enter `Returned(...)`.
- `SEQ-STEP`: step the left workflow of `Seq`.
- `SEQ-ADVANCE`: once the left workflow has returned normally, continue with the right workflow.
- `SEQ-REJECT`: propagate rejection from the left side.

### 2. Binding and branching families

- `LET-EVAL`: evaluate the bound expression atomically.
- `LET-BIND`: apply canonical pattern binding and continue under `Γ ⊕ ΔΓ`.
- `LET-REJECT`: map binding failure through the existing `SPEC-004` ownership boundary.
- `IF-COND`: evaluate the condition atomically.
- `IF-TRUE` / `IF-FALSE`: choose the continuation branch.
- `FOREACH-INIT`: evaluate the collection atomically.
- `FOREACH-STEP` / `FOREACH-DONE`: iterate one element at a time while preserving canonical workflow sequencing.

### 3. Capability / policy / obligation families

- `OBSERVE-STEP`: perform capability lookup/observation under the existing helper boundary, bind the result pattern, and continue.
- `ORIENT-STEP`: atomically evaluate the expression and continue.
- `PROPOSE-STEP`: perform proposal formation under the existing helper contract and continue.
- `DECIDE-STEP`: atomically evaluate the decision expression, run the named policy, then continue or reject.
- `CHECK-STEP`: discharge/check the obligation and continue or reject.
- `ACT-STEP`: evaluate the guard/helper-owned action contract and terminate or reject as specified by `SPEC-004`.
- `OBLIG-ENTER` / `OBLIG-EXIT`: enter and leave obligation-scoped execution while preserving obligation/provenance state transitions.
- `WITH-ENTER` / `WITH-EXIT`: enter and leave capability-scoped execution without inventing new runtime structure in the semantic backbone.

### 4. Modal / fallback families

- `MAYBE-PRIMARY`: step the primary branch.
- `MAYBE-FALLBACK`: switch to fallback when the primary rejects in the way allowed by the canonical rule family.
- `MUST-STEP`: step the wrapped workflow.
- `MUST-REJECT`: preserve the strengthened rejection/mandatory-success contract already described at the big-step layer.

### 5. Receive and concurrency families

- `RECEIVE-SELECTED`: selected arm continues with its body.
- `RECEIVE-FALLBACK`: explicit fallback body continues.
- `RECEIVE-FALLTHROUGH`: non-blocking miss proceeds according to the canonical receive contract.
- `RECEIVE-BLOCKED`: blocking receive with no available arm is classified as blocked/suspended, not stuck.
- `PAR-STEP`: one active branch steps at a time.
- `PAR-BRANCH-TERM`: a branch reaches local terminal state.
- `PAR-AGGREGATE`: once all branches are terminal, use helper-backed aggregation to reconstruct the authoritative combined outcome.
- `PAR-REJECT`: preserve helper-owned concurrent rejection/combination behavior rather than imposing fake left-to-right short-circuiting.

### 6. Explicit non-inventory items

The following are intentionally not canonical user-visible workflow rule families in MCE-005:

- `await`;
- surface-syntax concurrency forms not present in `SPEC-001`;
- runtime-only supervisor/control operations that belong to helper or interpreter realization work.

If spawn/completion semantics matter, they matter here only as preserved runtime/helper contracts that later correspondence and interpreter work must honor.

## Concurrency and Determinism Boundary

MCE-005 fixes the semantic stance without over-specifying scheduling internals:

1. `Par` is modeled by interleaving progress of branch-local workflow execution.
2. The semantics must preserve the helper-owned concurrent aggregation contract already accepted in `SPEC-004`.
3. The semantics do not commit to a concrete scheduler, fairness theorem, queue implementation, or runtime representation.
4. Any branch-choice nondeterminism that remains at the semantic level must be carried forward as helper-owned or scheduler-owned latitude for [MCE-006](MCE-006-SMALL-STEP-IR.md), not hidden by accidental left-to-right presentation.

So the determinism boundary remains the same as the big-step corpus: deterministic where the current semantics are deterministic, explicitly runtime/helper-owned where the current corpus already leaves room.

## Correspondence and Handoff to MCE-006

MCE-006 now has a fixed semantic target.

The runtime/interpreter alignment phase must preserve all of the following from MCE-005:

1. Ambient/static context is factored as `A = (C, P)` rather than baked into a generic store.
2. Dynamic execution state is carried in the `SPEC-004` vocabulary: `Γ`, `Ω`, `π`, cumulative trace, and cumulative effect summary.
3. The primary step relation is workflow-first: `A ⊢ κ —μ→ κ'`.
4. Observables are split deliberately:
   - authoritative cumulative state in configurations;
   - local step deltas in labels.
5. Pure expressions and patterns remain atomic in v1.
6. `Receive` blocking is a blocked/suspended state, not semantic stuckness.
7. `Par` uses interleaving plus helper-backed terminal aggregation.
8. Spawn/completion semantics remain runtime/helper contracts, not new surface syntax.

What MCE-006 still owns:

- mapping `κ` onto concrete runtime/interpreter structures;
- deciding how residual workflows, continuations, queues, tombstones, and child handles are represented;
- explaining how helper-owned nondeterminism is realized operationally;
- validating that the executable interpreter preserves the same terminal observables and blocking behavior.

## Resolution / Defer Table

| Question | Status in MCE-005 | Downstream owner |
|---|---|---|
| Is the semantic subject workflow-first or expression-first? | Resolved: workflow-first | Closed here |
| What configuration vocabulary is canonical? | Resolved: `Γ`, `Ω`, `π`, `T`, `ε̂`, residual workflow, with ambient `(C, P)` | Closed here |
| Generic store `S`? | Rejected for the semantic backbone | Closed here |
| User-visible `await`? | Rejected from canonical scope | Closed here |
| Where do observables live? | Resolved: deliberate split across state and labels | Closed here |
| Blocked vs stuck? | Resolved explicitly | Closed here |
| Pure expression micro-steps? | Deferred; v1 stays atomic | Closed for v1, revisit only by later explicit task |
| Exact per-form rule schemata and notation | Partially specified by inventory only | [TASK-395](../../plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md) |
| Formal big-step/small-step correspondence packaging | Deferred | [TASK-396](../../plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md) |
| Runtime machine/interpreter realization | Deferred | [MCE-006](MCE-006-SMALL-STEP-IR.md) |
| Full five-layer verification matrix closeout | Deferred | [MCE-007](MCE-007-FULL-ALIGNMENT.md) |

## Related Documents

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](MCE-007-FULL-ALIGNMENT.md)
- [TASK-394](../../plan/tasks/TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- [TASK-395](../../plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- [TASK-396](../../plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Initial small-step gap capture |
| 2026-04-05 | Accepted as the canonical small-step planning/design backbone | Phase 61 fixed the workflow subject, configuration contract, observability split, blocking distinction, rule inventory, and MCE-006 handoff |
