---
status: drafting
version: v1
created: 2026-04-01
last-revised: 2026-04-01
related-plan-tasks: []
tags: [type-system, effects, effect-typing, semantics, judgments, capabilities]
---

# TYPES-004: Effect Typing Foundations

## Problem Statement

Ash already has a coarse effect lattice and already uses effect information in both typing and
runtime-facing semantics. But the current design space around effect typing is still underspecified.

Several tensions are visible already:

- the language has a workflow effect vocabulary;
- capability uses are one major way effects become observable;
- embedding-side providers expose coarse effect metadata;
- future work may want richer forms of effect classification, effect variables, or effect-aware
  interfaces.

This exploration tries to identify a practical starting point.

It does **not** attempt to solve full effect polymorphism, algebraic effect design, effect rows,
or associated effects. It asks a narrower question:

> What is the smallest useful way to talk about effect typing in Ash, given the current lattice,
> the current workflow forms, and the current split between source-level semantics and embedding
> metadata?

## Scope

- **In scope:**
  - The current coarse effect lattice as Ash's effect grade system
  - Which workflow forms are effect-producing
  - How effect grades compose
  - The relationship between effect typing and capability use
  - Where provider metadata fits, and where it does not
  - A staged path toward richer future effect typing

- **Out of scope:**
  - Replacing the current lattice with rows or algebraic effects
  - Full inference rules for every expression and workflow form
  - Associated effects on interfaces or typeclasses
  - A final surface syntax for effect annotations
  - First-class functions/workflows as a prerequisite

- **Related but separate:**
  - SPEC-001: IR effect lattice and workflow forms
  - SPEC-003: Type system effect vocabulary and effect judgments
  - SPEC-004: Workflow outcome judgment carrying effect results
  - SPEC-010: Provider-level `effect()` metadata
  - SPEC-017: Capability integration
  - TYPES-003: Capability and effect vocabulary

## Anchors in the Current Specs

Ash already has three strong anchors:

1. **A coarse effect vocabulary** in SPEC-003:
   `Epistemic`, `Deliberative`, `Evaluative`, `Operational`

2. **A lattice structure** in SPEC-001:
   effects are ordered and composed by `join`

3. **A workflow evaluation result** in SPEC-004:
   workflow evaluation returns an effect grade as part of the outcome

Taken together, these suggest a practical initial model:

- effect typing should first be understood as computing a **coarse effect grade**;
- that grade should be derived from Ash workflow forms and source-level contracts;
- embedding-side metadata may constrain or validate that result, but should not define it.

One refinement already looks likely:

- Ash probably wants an explicit `Pure` bottom element below `Epistemic`, even if current specs
  mostly speak in terms of the four named operational grades.

## Core Distinction: Grade vs Source

It helps to separate two questions that are easy to blur:

1. **What is the resulting effect grade?**
   This is the lattice answer: pure, epistemic, deliberative, evaluative, operational.

2. **What produced that grade?**
   This is a more detailed explanation layer: observation, proposal, policy evaluation,
   obligation checking, external actuation, message send, and so on.

This note proposes that Ash should start by making the **grade** precise first, while leaving
space for a more detailed **effect source classification** later.

That yields a staged model:

- **Level 1:** coarse effect grade for typing and composition
- **Level 2:** effect source classification for diagnostics and explanation
- **Level 3:** richer future machinery, such as effect variables, associated effects, or rows

## Current Coarse Effect Grade

The current effect lattice already gives Ash a usable foundation, but this note proposes a
normalized bottom element:

| Grade | Current meaning |
|-------|------------------|
| `Pure` | Adds no computational effect of its own; identity element for composition |
| `Epistemic` | Input acquisition and read-only observation |
| `Deliberative` | Analysis, planning, and proposal formation |
| `Evaluative` | Policy and obligation evaluation |
| `Operational` | External side effects and irreversible outputs |

Why `Pure` is useful:

- some forms such as `Let`, `If`, `Seq`, `Par`, `ForEach`, `Ret`, and `Done` look effect-neutral
  in themselves and should not be conflated with `Epistemic`;
- `Pure` gives the lattice a proper identity element for `join`;
- it makes `Epistemic` mean something positive rather than merely "the smallest surfaced grade";
- it improves explanations in typing, traces, logs, provenance, and diagnostics.

`Pure` should probably be surfaced if the other grades are surfaced, even if it is used mainly for
normalization, diagnostics, and judgment-level explanations rather than frequent hand-written
annotations.

This is already enough to support useful judgments such as:

- whether one workflow is strictly more operationally powerful than another;
- whether a composed workflow exceeds some allowed ceiling;
- whether a boundary should reject or flag a workflow as too effectful.

The immediate goal, then, is not to invent a richer theory first. It is to explain how Ash forms
map into this grade system.

## Effect-Producing Workflow Forms

The most practical next step is to enumerate which Ash workflow forms contribute to the effect
grade and how.

The table below is intentionally provisional. It is a starter map, not a frozen spec.

| Workflow form | Default grade pressure | Reason |
|---------------|------------------------|--------|
| `Observe` | `Epistemic` | Read-only acquisition from a capability or input source |
| `Receive` | `Epistemic` | Input acquisition from streams/mailboxes |
| `Orient` | `Deliberative` | Analysis or internal interpretation step |
| `Propose` | `Deliberative` | Proposal formation without direct irreversible output |
| `Decide` | `Evaluative` | Policy evaluation is explicitly evaluative |
| `Check` | `Evaluative` | Obligation checking is explicitly evaluative |
| `Act` | `Operational` | External side effects / irreversible outputs |
| `Set` | likely `Operational` | Mutating external state should count as operational |
| `Send` | likely `Operational` | Emitting externally visible messages should count as operational |
| `Let` / `If` / `Seq` / `Par` / `ForEach` | `Pure` by themselves | Control-flow forms compose subworkflow effects but need not add one directly |
| `Ret` / `Done` | `Pure` by themselves | No additional effect beyond subcomputations |
| `With` | `Pure` by itself | Authority or environment shaping, not an effect in isolation |
| `Maybe` / `Must` | `Pure` by themselves | Modal/control effect on execution, not necessarily a lattice increase |
| `Oblig` | unclear | A governance/deontic form that may not itself be a separate effect producer |

This already suggests a useful working rule:

> Effect typing should begin from the canonical workflow forms, not from provider metadata.

## Composition

Ash already has the right coarse composition operator for the first pass:

- the effect grade of a composed workflow is the `join` of the grades incurred by its parts

That gives a clean starting intuition:

- sequencing joins effects from earlier and later steps
- branching joins effects from both branches conservatively
- parallel composition joins effects from all branches
- helper/control forms do not introduce effect grades unless their premises do

Example:

```text
observe sensor
decide access_policy
act notify
```

would grade to:

```text
join(Epistemic, join(Evaluative, Operational)) = Operational
```

This is useful because it gives Ash a simple, inspectable first model before asking for finer
effect details.

And for a pure control-only fragment:

```text
if cond then ret x else ret y
```

the surrounding control structure should be able to remain:

```text
Pure
```

unless one of its premises already incurs a stronger grade.

## Effect Typing vs Capability Typing

Capabilities matter here, but should not be allowed to swallow the whole discussion.

The cleanest relationship is:

- capability typing answers whether a workflow is authorized to perform some governed operation;
- effect typing answers what grade of computation the workflow performs;
- one capability use may contribute an effect grade, but the capability and the effect are not
  identical.

This matters especially because capability use is not the only source of effect pressure:

- `Decide` and `Check` are effectful in Ash's current sense even though they are not ordinary
  "capability invocation" in the embedding-provider sense;
- control and composition forms affect effect outcomes indirectly through their premises.

So a useful discipline is:

> Start effect typing from workflow forms and source-level semantics. Then ask how capability
> declarations constrain or explain some of those forms.

## What Capability Declarations Might Contribute

Capability declarations may still carry effect-relevant information, but that contribution should
be understood carefully.

At least three levels are possible:

1. **Fixed coarse classification**
   A capability declaration or operation form implies a minimum or exact coarse effect grade.

2. **Contractual consistency**
   A capability declaration says what kinds of operations it permits, and the typing rules ensure
   that usage is compatible with that contract.

3. **Future refinement**
   Capability declarations may later carry richer effect-level contracts or indices.

For now, the safest stance is:

- capability declarations help constrain effect typing;
- they do not replace syntax-directed effect typing over workflow forms.

## The Provider Metadata Tension

The most obvious can of worms is the relationship between effect typing and embedding-side
`provider.effect()`.

The current best framing is:

> `provider.effect()` is not Ash's effect typing. It is embedding metadata that should be
> compatible with Ash's effect typing.

That means:

- provider metadata is useful;
- provider metadata is not sufficient to classify actual source-level effects;
- a single provider-level effect may be too coarse if one provider realizes multiple operations
  with different effect profiles.

So for now:

- effect typing should derive from Ash forms and source-level contracts first;
- provider metadata should be checked for compatibility, not treated as the source of truth.

## Practical Starting Point

A realistic first effect-typing agenda for Ash could be:

1. Freeze the current coarse lattice as the **grade system**.
2. Add `Pure` as the normalized bottom element of that grade system.
3. Enumerate the **effect-producing workflow forms** and their default grades.
4. Define **composition by join** for sequencing, branching, and parallelism.
5. Clarify how **capability declarations** constrain some effect-producing forms.
6. Treat **provider metadata** as a consistency check and diagnostic aid.
7. Add a separate explanatory layer for **effect source classification** later.

This is intentionally modest. It is enough to make progress without solving the full research
problem up front.

## Design Dimensions

| Dimension | Coarse lattice only | Coarse lattice + source classification | Rich effect system now |
|-----------|---------------------|----------------------------------------|------------------------|
| Near-term tractability | High | Medium-high | Low |
| Diagnostic usefulness | Medium | High | Potentially high |
| Spec complexity | Low | Medium | High |
| Implementation burden | Low | Medium | High |
| Fit with current Ash | High | High | Unclear |

## Open Questions

1. How should `Pure` be integrated into the existing effect specs and examples without muddying the
   meaning of the current four named grades?
2. Is `Receive` best understood as epistemic by default, or does some receive/send protocol design
   eventually want a separate distinction?
3. Should `Oblig` itself contribute to the effect grade, or only the operations performed within
   it?
4. Should `With` remain effect-neutral, or can acquiring or narrowing authority ever be effectful
   in Ash's own semantics?
5. What exact relationship should hold between capability declarations and coarse effect grades?
6. When does Ash need effect source classification for diagnostics, rather than only coarse grades
   for typing?
7. At what point would richer designs such as associated effects or effect variables justify their
   complexity?

## Related Explorations

- [TYPES-003](TYPES-003-capabilities-effects-vocabulary.md)
- [TYPES-002 V2](TYPES-002-ad-hoc-polymorphism-v2.md)
- [MCE-003](../minimal-core/MCE-003-FUNCTIONS-VS-CAPS.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-01 | Exploration created | Ash needs a practical starting point for effect typing before richer effect designs are discussed |
| 2026-04-01 | Grade-first framing chosen | The current lattice and workflow forms provide enough structure for a useful first pass |

## Next Steps

- [ ] Cross-check the provisional workflow-form table against SPEC-001, SPEC-003, and SPEC-004
- [ ] Decide how `Pure` should be surfaced in specs, traces, diagnostics, and any future user-facing annotations
- [ ] Clarify how `Set` and `Send` fit the current canonical workflow inventory and effect vocabulary
- [ ] Follow up with a note on declaration-side effect contracts versus provider metadata
