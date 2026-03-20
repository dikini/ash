# Runtime-Reasoner Interaction Model

Date: 2026-03-20
Status: Design note

## Purpose

This note captures the intended design story behind Ash as a workflow language for governed
cognition. Its purpose is to preserve the core conceptual model precisely enough that the runtime,
IR, semantics, and later interaction-facing specifications can be derived or revised against it.

This document is intentionally pre-spec. It does not define full normative syntax or operational
rules. Instead, it defines the system intent and the two focused views that Ash must support:

- the runtime view, which is authoritative and execution-centered
- the reasoner view, which is advisory and interaction-centered

These are not competing models. They are differently focused descriptions of the same larger
system.

## System Intent

Ash is a workflow language for governed cognition.

It is meant to structure the interaction between:

- an authoritative deterministic runtime
- a non-authoritative but useful reasoner such as an LLM, agent, or future thinker

The runtime owns execution, state, validation, commitment, and official workflow history.
The reasoner contributes interpretation, synthesis, proposal, and local judgment over the context
it is given.

The central design goal is to preserve a strict distinction between:

- what is true
- what is visible
- what is inferred
- what is accepted

That distinction is what makes Ash readable by humans, enforceable by the runtime, and usable for
LLM-guided workflows.

## View 1: Runtime

The runtime view is the minimal semantic backbone. It should remain stable and simple even when the
full reasoner model becomes more detailed.

### Core domains

The runtime view distinguishes four domains:

- `R`: runtime state
- `C`: projected context
- `A`: advisory result
- `R'`: next runtime state

These are related by three abstract maps:

- `P: R -> C` projection
- `D: C -> A` derivation
- `K: R × A -> R'` validation and commitment

This yields the minimal interaction pattern:

```text
R --P--> C --D--> A
 \--------------K--------------> R'
```

This model is intentionally trimmed. It is not meant to capture how a concrete LLM internally
represents memory or context. It is meant to capture the authority structure that the runtime must
preserve.

### Runtime responsibility

The runtime is authoritative. It owns:

- workflow state
- values and bindings
- obligations
- policies
- capability availability
- execution of effectful operations
- trace and provenance
- validation and rejection boundaries

The runtime may expose parts of its state for reasoning, but exposed material remains runtime-owned.

The runtime is the only component that can:

- admit facts into authoritative state
- authorize or reject progression
- commit world-affecting effects
- define the official workflow history

### Projection

Projection is a governed map from runtime state into reasoning context.

Projection is not assumed to be lossless. It may:

- select relevant data
- omit irrelevant data
- redact protected data
- summarize large state
- frame exposed material for reasoning

Projection exists to control both safety and reasoning quality. The runtime does not hand over raw
state unchanged. It prepares a view suitable for reasoning while preserving governance constraints.

### Derivation

Derivation produces advisory results from projected context.

Advisory results are not authoritative state transitions. They may be workflow-relevant and useful,
but they do not become part of machine truth merely by being produced.

This is the key boundary that prevents model output from collapsing into system action.

### Commitment

Commitment is the authoritative acceptance boundary.

The runtime consumes the current state together with an advisory result and determines whether that
result:

- is accepted
- is rejected
- requires further checks
- can be committed as part of the next state transition

Commitment may include:

- policy evaluation
- invariant checking
- obligation checking
- guard checking
- capability validation
- effect execution
- provenance capture

The essential principle is that all workflow progress that matters must pass back through runtime
authority.

## Runtime Classification of Primitives

Within the runtime view, the primitive story is:

- `OBSERVE` is authoritative acquisition
- `ORIENT` is advisory interpretation
- `PROPOSE` is advisory candidate generation
- `DECIDE` is authoritative authorization
- `CHECK` is authoritative verification
- `ACT` is authoritative commitment

These primitives should be understood by their role in the `R -> C -> A -> R'` model.

### `OBSERVE`

`OBSERVE` acquires admissible facts and places them into runtime state.

It is not the reasoner noticing something. It is the runtime updating what is authoritatively
known. Its result may later be projected, but the authoritative step is prior to reasoning.

### `ORIENT`

`ORIENT` consumes projected context and yields advisory understanding.

It covers interpretation, decomposition, explanation, diagnosis, restatement, and planning-oriented
internal work. Its outputs may influence later workflow progression, but they do not directly
mutate authoritative state.

### `PROPOSE`

`PROPOSE` consumes projected context and yields candidate actions or artifacts.

It is action-shaped advisory output: drafts, candidate edits, candidate tests, candidate plans,
candidate messages, or candidate next steps.

`PROPOSE` forms intent without commitment.

### `DECIDE`

`DECIDE` is an authorization gate.

It determines whether a candidate progression is admissible under explicit policy or governance
rules. `DECIDE` is runtime-owned even if the candidate it judges arose from reasoner output.

### `CHECK`

`CHECK` is a verification gate.

It determines whether factual, normative, or readiness conditions hold. It is narrower than
authorization: it verifies obligations, invariants, or outcome predicates.

`CHECK` and `DECIDE` are both gating primitives, but with different emphasis:

- `DECIDE` asks "may we proceed?"
- `CHECK` asks "does this condition hold?"

### `ACT`

`ACT` is commitment into authoritative transition and possibly external effect.

It is the point at which accepted intent becomes executed behavior. This is where provenance matters
most strongly, because this is where advisory cognition crosses into authoritative action.

## View 2: Reasoner

The runtime view describes the system from the side of authority and execution. The reasoner view
describes the same system from the side of cognition and interaction.

The runtime sees a clean `C -> A` derivation. A concrete reasoner does not actually operate over an
isolated `C`. It operates over its own evolving historical context, only part of which is supplied
by the runtime.

The runtime-facing derivation map remains valid as an abstraction. The reasoner view explains what
that abstraction hides.

### Reasoner responsibility

The reasoner is responsible for producing advisory cognition.

It may:

- interpret projected context
- synthesize descriptions
- generate candidates
- choose among alternatives heuristically
- formulate tool requests
- sequence its own intermediate thought

The reasoner is not authoritative with respect to:

- runtime truth
- policy satisfaction
- obligation discharge
- effect execution
- provenance
- accepted state transition

Its value is cognitive, not sovereign.

### Hidden historical context

A reasoner is best modeled as operating over a historical context `H`.

`H` includes prior context, prior turns, prior outputs, and internal state not fully visible to the
runtime. The runtime neither controls nor fully observes this whole context.

The runtime can, however, inject governed material into it.

This means that the runtime-facing derivation map:

- `D: C -> A`

is a simplification.

From the reasoner side, the system looks more like:

- the runtime projects and injects material
- the reasoner combines that material with its own history
- the reasoner performs a turn
- the reasoner emits outward requests or outputs

### Injected context

Let `I` denote the runtime-injected portion of context.

Then the reasoner view is approximately:

```text
R --project--> I
I + H --turn--> outputs
```

Where:

- `I` is governed and runtime-supplied
- `H` is reasoner-held historical context
- `outputs` are advisory products or external requests

The exact internal representation of `I` inside `H` does not matter at this level. What matters is
that the runtime shapes only part of the reasoner's effective context.

### Turn execution

The reasoner executes in turns.

A turn may produce:

- an advisory result
- one or more candidate tool calls
- requests for further observation
- or no acceptable result

This means derivation is not best understood as one pure mathematical step in implementation terms.
It is better understood as a bounded reasoning turn over injected plus pre-existing context.

The simple runtime model remains valid as an abstraction: the runtime only needs to treat the turn
as producing advisory outputs.

### Tool calls as interaction boundary

At present, tool calls are the concrete interaction boundary between reasoner and runtime.

They are how the reasoner:

- asks for more information
- requests effectful operations
- attempts verification
- or proposes external interaction

From the reasoner side, tool calls are outbound requests.
From the runtime side, tool calls are candidate commitments or candidate observations subject to
validation.

So the practical system picture is:

```text
Runtime state -> projected injection
Projected injection + reasoner history -> reasoner turn
Reasoner turn -> tool calls / outputs
Tool calls / outputs -> runtime validation
Runtime validation -> accepted state change or rejection
```

This is where the runtime reasserts authority.

## Relationship Between the Views

The runtime view and reasoner view must remain distinct.

The runtime view is normative for:

- semantics
- execution
- acceptance
- observability

The reasoner view is explanatory for:

- cognition
- context shaping
- interaction design
- boundary discipline

The runtime view answers:

- What is authoritative?
- What can change workflow state?
- Where do validation and effects occur?

The reasoner view answers:

- What does the thinker actually operate over?
- How does runtime context reach it?
- How does it communicate back?

Neither view should absorb the other.

In particular:

- the runtime model should not depend on the runtime knowing the whole internal reasoner state
- the reasoner model should not imply that advisory output is authoritative by default

## Design Invariants

The combined story implies the following invariants:

- runtime state is authoritative
- projection is selective and governed
- reasoner outputs are advisory until accepted
- tool-call or action boundaries are validation boundaries
- effectful commitment is runtime-owned
- provenance belongs to accepted and executed workflow history, not merely to raw reasoner thought
- the simple runtime model remains the canonical abstraction even if real reasoners have opaque
  historical context

These invariants should survive later elaboration.

## Consequences for Later Specs

This design story is precise enough to drive later specification work.

### IR and semantics

The IR should remain aligned with the runtime view. It should classify workflow meaning in terms of:

- acquisition
- advisory derivation
- gating
- commitment

It should not bake in assumptions about the full internal structure of reasoner context.

### Runtime specification

The runtime specification should define:

- authoritative state contents
- projection as a governed operation
- validation and rejection boundaries
- commitment behavior
- provenance and trace semantics
- the role of tool-call boundaries

### Reasoner interaction specification

A separate reasoner-facing specification should define:

- injected context
- advisory turn behavior
- outbound requests
- the relationship between reasoner outputs and runtime acceptance

This should explain how the runtime guides the reasoner without assuming full observability or full
control of the reasoner's internal history.

### Gap analysis targets

The first gaps to identify against this design story are:

- where projection is implicit rather than explicit
- where advisory outputs are not clearly separated from accepted state
- where tool-call boundaries are underspecified as validation points
- where `DECIDE`, `CHECK`, and `ACT` responsibilities blur
- where provenance is attached too early or too late
- where current specifications assume a reasoner-free runtime model without saying how reasoner
  interaction fits

## Short form

Ash is a language for governed cognition with two coordinated views.

From the runtime side, Ash is an authoritative deterministic system that projects governed context,
receives advisory outputs, and alone validates and commits workflow progression.

From the reasoner side, Ash is a structured interaction protocol in which a thinker operates over
injected runtime context plus its own historical context, then communicates back through bounded
requests whose acceptance is runtime-governed.
