# Workflow Declaration and Runtime Behavior Design Note

**Date:** 2026-03-29

## Goal

Capture the current exploratory design direction for Ash workflow declarations, their runtime meaning, and their relationship to surface syntax, canonical IR, and `SPEC-004` big-step semantics. This note is intentionally option-preserving, but it records the strongest direction reached so far so that later small-step operational semantics can start from a coherent execution model. It also asks a more precise interpretive question: how much of this workflow-declaration story can already be expressed using the existing `SPEC-004` judgment forms, and where an outer semantic layer is still missing.

## Central Design Conclusion

The strongest current design direction is:

> A workflow declaration defines a callable, workflow-backed capability with boundary contracts. Static checking proves the declared contract is plausible; runtime entry checks requirements; runtime exit checks obligations/postconditions; and `oblige` introduces obligation checkpoints within execution.

This is the center of the note. The remaining options and alternatives should be read as refinements, tensions, or consequences of this core model rather than unrelated proposals.

## Context

The present spec stack does not yet describe workflow declarations in one fully unified way.

- [SPEC-002-SURFACE](../spec/SPEC-002-SURFACE.md) defines top-level `workflow_def` syntax.
- [SPEC-001-IR](../spec/SPEC-001-IR.md) defines canonical `Workflow` terms for executable workflow-body forms, not a first-class top-level declaration carrier.
- [SPEC-004-SEMANTICS](../spec/SPEC-004-SEMANTICS.md) gives big-step semantics for lowered workflow-body/core terms `w`, again not for the full declaration boundary.
- [SPEC-017-CAPABILITY-INTEGRATION](../spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-018-CAPABILITY-MATRIX](../spec/SPEC-018-CAPABILITY-MATRIX.md), [SPEC-019-ROLE-RUNTIME-SEMANTICS](../spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md), [SPEC-020-ADT-TYPES](../spec/SPEC-020-ADT-TYPES.md), [SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), and [SPEC-022-WORKFLOW-TYPING](../spec/SPEC-022-WORKFLOW-TYPING.md) each carry part of the surrounding story: contracts, capabilities, obligations, role context, spawn, instances, and observable runtime state.

The result is a split model:

1. the surface language has workflow declarations;
2. the core IR has executable workflow-body forms;
3. the big-step semantics explains execution of the lowered body;
4. nearby specs partly explain contracts, capability requirements, instance/runtime context, and obligation discipline.

This note systematizes the discussion around that split instead of pretending it has already been collapsed into one canonical declaration model.

The central interpretive pressure comes from the shape of `SPEC-004` itself. The canonical big-step workflow judgment is:

```text
Γ, C, P, Ω, π ⊢wf w ⇓ out
```

where `w` is a lowered core workflow term and

```text
out ::= Return(v, eff, T, Ω', π')
   | Reject(err, eff, T, Ω', π')
```

That judgment already gives a strong semantic account of workflow-body execution. The question is whether the broader workflow-declaration story can be stated directly in those terms, or only by wrapping them in an outer declaration/invocation boundary.

## Problem Statement

The main design question is not merely how to lower a workflow body. It is how to understand a **workflow declaration** as a reusable runtime entity:

- what kind of thing a declared workflow is;
- how it is invoked or instantiated;
- how its declared requirements and obligations behave at runtime;
- how this interacts with capability lookup, authority, policy, supervision, and provenance;
- and what semantic shape would best support a later small-step operational account.

A too-small model such as “a workflow declaration just declares an action” loses important structure. Workflows are richer than actions because they carry contract boundaries, internal execution structure, effectful interaction forms, and obligation evolution.

## Working Definitions

### Workflow declaration

A top-level source declaration that names a callable workflow unit and attaches executable body, capability requirements, and contract-like boundary information. In current spec terms, it is not itself one of the canonical `Workflow` body terms ranged over by `w` in `Γ, C, P, Ω, π ⊢wf w ⇓ out`.

### Workflow body

The executable, lowered core term described by canonical `Workflow` IR and executed by `SPEC-004` big-step semantics. This is the object directly ranged over by `w` in the workflow judgment.

### Workflow-backed capability

A callable capability whose implementation is not an opaque external provider but a workflow body executed under Ash runtime control. This keeps workflows inside the broader capability story without collapsing them into plain external actions.

### Boundary contracts

The entry and exit conditions attached to invoking a workflow-backed capability.

- **Entry side:** requirements, authority assumptions, capability availability assumptions, policy prerequisites, and any required initial obligation context.
- **Exit side:** obligations that must hold, obligations that must be discharged, and postconditions/ensures clauses that characterize a successful completion.

In `SPEC-004` vocabulary, these are not currently part of the syntax of `w` itself. They are better understood as conditions on how initial `(Γ, C, P, Ω, π)` are admitted before `⊢wf`, and on how the resulting `out` is interpreted after `⊢wf` completes.

### Obligation checkpoint

A program point introduced by `oblige` and related checking forms where obligation state is updated, refined, created, discharged, or validated before workflow completion.

### Plausibility checking

Static checking that a declared workflow contract is well-formed, non-contradictory, and compatible with the body and surrounding typing/effect discipline. Plausibility checking does not prove concrete runtime success; it proves that the declaration is semantically admissible.

## Current Cross-Layer Reading

The current best reading of the spec stack is:

- surface `workflow_def` is the programmer-facing declaration boundary;
- canonical IR `Workflow` is the lowered executable body, not the whole declaration;
- big-step `SPEC-004` semantics runs the body in context `Γ, C, P, Ω, π`;
- declaration-level information lowers into split runtime-relevant artifacts such as capability requirements, effect summaries, policy bindings, obligation requirements, exposure metadata, and executable body.

Under that reading, a workflow declaration does not lower to one single existing canonical node. It lowers to a coordinated bundle that the runtime must keep correlated when execution starts and ends.

More tightly aligned to `SPEC-004`, that coordinated bundle can be read as supplying three things around the existing body judgment:

1. **Pre-body admission conditions** deciding whether execution may start with some initial `Γ, C, P, Ω, π`.
2. **The lowered body term** `w` consumed by `Γ, C, P, Ω, π ⊢wf w ⇓ out`.
3. **Post-body interpretation conditions** deciding how to read `out`, especially whether `Return(v, eff, T, Ω', π')` satisfies the declaration's exit contract.

This reading preserves `SPEC-004` as body semantics rather than over-reading it as a complete declaration semantics.

## Why “Workflow-Backed Capability” Is the Strongest Direction

This model best fits the current language shape.

1. **It preserves callability.** A declared workflow can be invoked as a named reusable unit rather than only as the one program root.
2. **It fits the existing capability architecture.** Ash already has capability references, lookup, constraints, and runtime-provided implementations. A workflow-backed capability extends this space rather than inventing a separate, incompatible invocation mechanism.
3. **It preserves workflow richness.** Unlike an ordinary action, a workflow carries structured control flow, internal obligation evolution, policy interactions, and provenance-bearing execution.
4. **It aligns with runtime control and supervision.** A workflow-backed capability can be called synchronously or instantiated under supervision while still sharing one conceptual declaration model.
5. **It gives a clean path toward small-step semantics.** The future small-step account can model declaration entry, execution steps, checkpoint transitions, and completion boundaries as explicit runtime states.

This is stronger than saying a workflow “is an action,” but simpler than inventing a wholly separate class of callable entity unrelated to capabilities.

## Lowered Declaration Shape

A plausible lowered workflow declaration consists of coordinated artifacts rather than a single pre-existing core term:

1. **Callable identity**
   - workflow name
   - parameter interface
   - return/result contract

2. **Executable body**
   - lowered canonical `Workflow` term executed by the core semantics

3. **Capability requirement summary**
   - `observes`
   - `receives`
   - `sets`
   - `sends`
   - any declaration-site capability constraints

4. **Contract summary**
   - `requires`
   - `ensures`
   - obligation declarations and discharge expectations

5. **Runtime control metadata**
   - effect summary / effect ceiling
   - policy bindings
   - monitor exposure metadata
   - provenance attachment rules
   - invocation mode expectations where relevant

The runtime then enters the body with these declaration-derived artifacts projected into the execution context.

Using `SPEC-004` judgment vocabulary, the key projection is:

- declaration-level information determines admissible initial `Γ, C, P, Ω, π`;
- the declaration body lowers to canonical `w`;
- execution then proceeds by deriving `Γ, C, P, Ω, π ⊢wf w ⇓ out`;
- declaration-level completion rules inspect `out`.

This makes the declaration/body split explicit instead of tacit.

## Invocation Model

A workflow declaration should be understood as defining a callable unit with a runtime invocation envelope.

A plausible invocation envelope includes:

- argument values;
- caller provenance / lineage attachment;
- current authority or role context;
- capability environment and policy environment handles;
- supervision placement when the call creates a managed instance;
- initial obligation context;
- declaration identity for monitors, control, and trace.

This envelope may support more than one operational mode:

- **call:** run as a callable workflow-backed capability with boundary checks and return to caller;
- **spawn:** create a new managed workflow instance with its own lifecycle, supervision placement, and control/monitor links.

The declaration can therefore stay constant while the invocation mode changes.

In a `SPEC-004`-aligned reading, the invocation model is best seen as an **outer wrapper relation** around the existing body judgment rather than a reinterpretation of `⊢wf` itself.

A schematic declaration-level relation would have to look something like:

```text
invoke(decl, args, runtime_context) ↝ Γ, C, P, Ω, π, w
Γ, C, P, Ω, π ⊢wf w ⇓ out
validate_exit(decl, out) ↝ out'
```

This note does not propose that as canonical notation yet. It is only a way to state precisely what the current big-step rules can and cannot already express.

## Contract Semantics

### Static phase

Static checking should establish plausibility of the declaration.

At minimum this includes:

- the declaration is well-formed;
- required capabilities and effects are coherent with the body;
- preconditions and postconditions are not trivially contradictory;
- obligation declarations fit the typing/effect model;
- call sites or spawn sites can be checked against declared requirements.

This phase does **not** eliminate the need for runtime checks. It only establishes that execution is meaningful to attempt.

In relation to `SPEC-004`, this phase mostly lives outside the runtime big-step judgment. It is prior to any derivation of `Γ, C, P, Ω, π ⊢wf w ⇓ out`.

### Runtime entry

On entry, the runtime checks the declaration boundary before stepping through the body.

Typical entry checks include:

- argument compatibility;
- required authority/role context;
- capability availability and constraint satisfaction;
- policy-side admissibility;
- declared `requires` predicates;
- any obligation preconditions needed before execution begins.

If entry checks fail, the workflow-backed capability invocation is rejected before body execution meaningfully proceeds.

This is the first important limit of the current big-step account: `SPEC-004` can represent rejection **during** workflow-body execution as `Reject(err, eff, T, Ω', π')`, but declaration-entry failure is not naturally a judgment over body term `w` unless the declaration boundary is encoded separately.

### Internal execution and `oblige`

Within the body, `oblige` should be treated as an explicit checkpoint in obligation state evolution rather than only an annotation.

Plausible checkpoint behavior includes:

- creating a new pending obligation;
- requiring evidence or state needed to discharge an obligation;
- validating that an expected obligation has been met;
- refining or splitting obligation state in compound control flow;
- exposing obligation state to trace, monitor, or provenance mechanisms.

This makes obligations part of runtime state evolution, not just final bookkeeping.

This part fits `SPEC-004` relatively well. The current workflow judgment already carries `Ω` and returns `Ω'`, and the helper contracts already include `check_obligation(obligation, Ω, Γ) ↝ bool` and `discharge(Ω, obligation) ↝ Ω'`. So internal obligation checkpoints are much closer to being directly expressible in the current big-step machinery than declaration entry or declaration exit.

### Runtime exit

On successful completion, the runtime checks exit-side boundary conditions.

Typical exit checks include:

- `ensures` predicates;
- declared postconditions;
- required discharged obligations;
- any residual obligation policy permitted by the chosen semantic model;
- result value admissibility and trace/provenance commitments.

Again, this is only partly present in current `SPEC-004`. The workflow judgment already returns the right shape of observable result:

```text
Return(v, eff, T, Ω', π')
```

So the existing big-step semantics can express the **material to be checked** at exit. But it does not yet directly define declaration-level exit validation as part of the workflow judgment itself.

The best current reading is therefore:

- `⊢wf` computes the body outcome;
- declaration semantics interprets that outcome against `ensures` and exit-obligation requirements.

That is enough to support the conceptual story, but only through an outer interpretive layer.

## What Current `SPEC-004` Can Already Express

The current big-step semantics can already express substantial parts of the story.

### Directly expressible now

1. **Workflow-body execution** through `Γ, C, P, Ω, π ⊢wf w ⇓ out`.
2. **Pure expression evaluation** through `Γ ⊢e expr ⇓ v` for body-local conditions and computations.
3. **Pattern-mediated binding** through `Γ ⊢p pat ⇐ v ⇓ ΔΓ` and `require_pattern(...)`.
4. **Runtime capability interaction** through helper contracts such as `lookup(C, capability_ref) ↝ provider` and `perform_action(action, Γ, C) ↝ v | error reason`.
5. **Policy and obligation checks inside execution** through `policy_decision(...)`, `policy_check(...)`, `check_obligation(...)`, and `discharge(...)`.
6. **Observable outcomes** through `Return(v, eff, T, Ω', π')` and `Reject(err, eff, T, Ω', π')`.

These are enough to model the internal execution of a lowered workflow body with evolving obligation and provenance state.

### Not directly expressible as currently written

1. **Workflow declaration as a judgment subject.** `⊢wf` ranges over lowered body terms `w`, not declarations.
2. **Declaration-entry admission.** There is no current judgment that says whether a declaration invocation is admitted before body execution begins.
3. **Declaration-exit validation.** There is no current rule family that says a body-level `Return(...)` counts as successful declaration completion exactly when postconditions and exit obligations hold.
4. **Invocation mode.** Current `SPEC-004` does not directly distinguish declaration call versus declaration spawn.
5. **Supervisor placement.** Runtime placement into a supervision tree is outside the present core big-step contract.

So the answer is not “no, the story cannot be expressed,” but rather “not by `⊢wf` alone.”

## Best Interpretive Fit for Current `SPEC-004`

The strongest exact reading is a two-layer one:

1. **Outer declaration/invocation layer**
   - admits or rejects invocation,
   - prepares initial `Γ, C, P, Ω, π`,
   - identifies lowered body `w`,
   - later validates `out` against declaration exit conditions.

2. **Inner body-execution layer**
   - uses the existing `SPEC-004` judgment unchanged:

   ```text
   Γ, C, P, Ω, π ⊢wf w ⇓ out
   ```

This lets the note say something precise:

> The current `SPEC-004` big-step semantics is sufficient to express workflow-body execution and the obligation/provenance evolution that occurs during that execution, but not sufficient by itself to express full workflow-declaration invocation semantics. The declaration story is therefore best modeled as an outer boundary relation around the existing body judgment.

That is more faithful than pretending current `SPEC-004` already gives direct declaration semantics, and more constructive than saying the current semantics is unusable for this purpose.

## Core Options for Exit Obligations

The main unresolved design fork concerns what exit obligations mean.

### Option A: Callee-owned guarantees

A workflow’s exit obligations are guarantees that the callee must satisfy before it may successfully return.

#### Reading

- The workflow declaration says what successful completion itself guarantees.
- A successful return means no required exit obligation remains unmet.
- If an obligation cannot be discharged, the workflow does not complete successfully.

#### Strengths

- Clear boundary meaning: success is self-contained.
- Strongest compositional story for callers.
- Simplifies reasoning about call results.
- Fits well with a big-step view where success already summarizes completion obligations.

#### Weaknesses

- May be too rigid for long-lived or delegated responsibility patterns.
- Makes some realistic supervisory or multi-stage workflows awkward.
- Can force artificial splitting into spawned or nested workflows just to defer obligations.

#### Small-step implications

A small-step machine can treat successful terminal states as states in which all exit obligations required by the declaration are satisfied.

In current big-step terms, this corresponds most cleanly to an outer rule that accepts `Return(v, eff, T, Ω', π')` as a successful declaration result only if exit validation over `Ω'`, `v`, and perhaps `Γ` succeeds.

### Option B: Residual obligations may propagate upward

A workflow may discharge some obligations internally and return with residual obligations transferred to the caller or surrounding runtime context.

#### Reading

- The workflow declaration describes both what it attempts locally and what obligations may escape the call boundary.
- Successful return may produce value plus residual obligation state.
- The caller becomes responsible for remaining obligations under explicit transfer rules.

#### Strengths

- Better models delegation and staged accountability.
- Supports workflows that advance obligations without fully closing them.
- Can unify nested workflow calls with broader obligation-carrying execution.

#### Weaknesses

- Makes return values semantically richer and more complex.
- Weakens the simple meaning of success.
- Requires explicit transfer, ownership, and discharge discipline.
- Increases proof and runtime complexity for both big-step and small-step semantics.

#### Small-step implications

The machine state likely needs explicit residual obligation components in continuation/caller frames or runtime instance state.

In current big-step terms, this option would require more than merely reading `Ω'` out of `Return(v, eff, T, Ω', π')`. It would require declaration-level rules stating when a non-empty or partially discharged `Ω'` is an admissible return condition and how responsibility transfers beyond the callee body.

### Option C: Stratified boundary model

Some obligations are callee-owned completion guarantees, while others are explicitly transferable or supervisory.

#### Reading

- The declaration distinguishes obligation classes.
- Local completion obligations must be discharged before success.
- Transferable obligations may be returned upward under declared rules.

#### Strengths

- Preserves strong local guarantees where needed.
- Allows delegation where the language explicitly wants it.
- Likely best reflects the practical variety of workflow obligations.

#### Weaknesses

- Adds classification machinery.
- Requires careful syntax and runtime representation.
- Risks premature complexity if introduced too early.

#### Small-step implications

The future semantics must distinguish local discharge transitions from obligation-transfer transitions.

In current big-step terms, this would likely mean the exit validator classifies components of `Ω'` rather than merely testing emptiness or simple satisfaction.

## Strongest Current Preference

The strongest current direction remains the workflow-backed capability model with **boundary contracts** and explicit `oblige` checkpoints.

Within that direction, the cleanest immediate semantic default is close to **Option A**:

- static checking proves the contract is plausible;
- runtime entry checks requirements;
- body execution evolves obligation state through explicit checkpoints;
- runtime exit checks obligations/postconditions for successful return.

This default is attractive because it gives workflow success a sharp meaning and meshes well with the present big-step style.

It also meshes best with current `SPEC-004` judgment shapes because the body semantics already computes one authoritative `Return(...)` or `Reject(...)`, and Option A gives the declaration boundary a relatively simple way to interpret `Return(...)`.

However, the exploration also exposed a real pressure toward **Option B** or **Option C** for delegation-heavy or supervision-heavy designs. So the note preserves those alternatives rather than ruling them out.

A reasonable current stance is therefore:

- treat **callee-owned guarantees** as the semantic default for successful return;
- keep **residual-obligation propagation** as an open extension point requiring explicit modeling, not an implicit property of all workflow calls.

## Call versus Spawn

A useful separation emerged between two phenomena that should not be collapsed.

### Call

A call uses a workflow declaration as a callable capability and returns through a caller boundary.

Key concerns:

- entry/exit contract checks;
- argument/result behavior;
- obligation ownership at return (callee discharges before return);
- caller-visible success or rejection.

Operationally: `call(w)` blocks until `w` completes and obligations are discharged.

### Spawn

A spawn instantiates a workflow as a managed runtime instance with isolated obligations.

Key concerns:

- instance identity and lineage;
- supervisor placement (control link as supervision contract);
- control and monitor links;
- **no obligation join**—spawned workflow's obligations remain internal.

Operationally: `spawn(w)` is like `par { w | continue }` but **without the final join**. The parent continues immediately; obligation states never combine. The only structural relationship is the supervision contract (control link).

### Operational Semantics

| Form | Obligation Semantics |
|------|---------------------|
| `call(w)` | Callee discharges obligations before return; caller observes result |
| `spawn(w)` | Spawned workflow's obligations are isolated; parent receives control link only |
| `par { w1 \| w2 }` | Both branches complete; obligation states join at synchronization |
| `seq(w1, w2)` | w1 discharges before w2 begins |

**Key insight:** Async workflows (spawn) do not share obligation state with parents. Dependencies must be explicit:
- Message passing (async, decoupled)
- Monitoring (observable exposed state)
- Supervision contract (control link)

These should share the same declaration model but have distinct operational semantics for obligation handling.

## Supervision Placement

One important conclusion from the exploration is that supervision should not be fixed as a compile-time property of the declaration itself.

A better fit is:

- the declaration defines what can be run;
- the invocation envelope determines where and how it is placed into a supervisor tree.

This is especially useful if the same workflow declaration may be:

- called synchronously,
- spawned as a child instance,
- or invoked in a different orchestration context.

That keeps workflow identity stable while leaving runtime placement dynamic.

## Capability Story

The current canonical story for capability invocation is already specialized rather than purely expression-call based. Workflow execution uses forms such as `Observe`, `Receive`, and `Act`, and `SPEC-004` resolves capabilities through runtime capability environment `C` and action/capability helper relations.

The workflow-backed capability direction preserves that architecture:

- some capabilities may remain external-provider-backed;
- some may be library/runtime-backed;
- some may be workflow-backed.

This gives one broader capability taxonomy without requiring the language to pretend that every capability is an opaque external service.

Under the two-layer reading, a workflow-backed capability is therefore not a new core `w` form inside `SPEC-004`. It is a declaration-level capability identity whose implementation path eventually evaluates some lowered body `w` via the existing workflow judgment.

## Plausible Small-Step Consequences

This exploration suggests a future small-step semantics should make declaration boundaries explicit rather than only stepping inside lowered workflow-body terms.

A plausible small-step account would likely need explicit states for:

1. declaration lookup / invocation preparation;
2. entry-check evaluation;
3. body execution steps;
4. obligation checkpoint transitions;
5. completion and exit-check evaluation;
6. rejection/abort propagation;
7. spawned-instance lifecycle transitions.

The key benefit is that workflow declaration semantics can then be stated directly rather than reconstructed indirectly from body execution plus side documents.

Seen from the current big-step side, those future states correspond exactly to what is currently outside direct `⊢wf` coverage: invocation admission, boundary validation, and instance/supervision transitions.

## Open Questions

The following remain genuinely open:

1. **Exit obligation ownership:** must successful return imply full discharge, or may residual obligations transfer upward?
2. **Residual obligation representation:** if transfer is allowed, does the caller receive explicit obligation state, a decorated result, or ambient runtime updates?
3. **Call/spawn correspondence:** should both modes share one invocation judgment with different result forms, or separate judgments?
4. **Supervisor-tree semantics:** how much of supervision belongs in the core semantics versus runtime helper contracts?
5. **Declaration lowering shape:** should the spec eventually introduce a canonical declaration-level IR carrier, or keep declaration metadata split across auxiliary domains?
6. **Root workflow semantics:** how should program entry be defined relative to the same callable workflow model?

## Recommended Next Spec Direction

When moving from the current big-step reading toward small-step semantics, the most productive next move is:

1. preserve the central model that a declaration defines a callable, workflow-backed capability;
2. make entry boundary, internal checkpoint, and exit boundary explicit semantic phases;
3. initially specify successful return using the callee-owned guarantee reading;
4. leave residual-obligation propagation as an explicit open design branch, not an accidental omission.

This recommendation is intentionally conservative with respect to current `SPEC-004`: preserve `Γ, C, P, Ω, π ⊢wf w ⇓ out` as the inner body-execution contract, and add declaration-boundary semantics around it rather than forcing declaration meaning directly into the existing body judgment prematurely.

This gives a crisp default semantics now while protecting room for later obligation-transfer extensions if the language needs them.

## Summary

The exploration points to a coherent reading of workflow in Ash:

- a workflow declaration is more than syntax around a body;
- it defines a callable runtime entity;
- that entity is best understood as a workflow-backed capability;
- the declaration carries boundary contracts;
- the current `SPEC-004` workflow judgment already expresses the inner body execution of that entity but not the full declaration boundary;
- static checking establishes plausibility;
- runtime entry checks requirements;
- runtime execution evolves obligation state, including explicit `oblige` checkpoints;
- runtime exit checks obligations and postconditions;
- and the principal remaining fork is whether exit obligations are always callee-owned or may sometimes propagate upward.

That is the most stable conceptual bridge between the present surface syntax, canonical IR, big-step semantics, and the anticipated future small-step operational model.
