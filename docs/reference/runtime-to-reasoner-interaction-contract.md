# Runtime-to-Reasoner Interaction Contract

## Status

TASK-191 interaction handoff.

## Purpose

This note defines the contract boundary between:

- the authoritative Ash runtime, and
- an external reasoner such as an LLM, agent, or future thinker

It exists to make projection, advisory output, and runtime acceptance boundaries explicit without
overloading runtime-only contracts such as core operational semantics, capability verification,
monitoring, or workflow observability.

This note is authoritative for runtime-to-reasoner interaction boundaries. It does not redefine the
canonical IR or the canonical runtime semantics. Instead, it explains how reasoner participation is
governed around those canonical contracts.

## Relationship to Canonical Contracts

This interaction contract depends on, but does not replace:

- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-017: Capability Integration](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-018: Capability Runtime Verification Matrix](../spec/SPEC-018-CAPABILITY-MATRIX.md)
- [Runtime-Reasoner Separation Rules](runtime-reasoner-separation-rules.md)
- [Interaction Obligations Extension Path](../design/INTERACTION_OBLIGATIONS_EXTENSION_PATH.md)

The canonical runtime contracts still own:

- authoritative state
- evaluation and execution
- policy and obligation gates
- capability verification
- effect execution
- trace and provenance
- official workflow history

This note defines only the interaction-facing layer around those runtime-owned responsibilities.

## Core Interaction Model

The interaction model distinguishes four domains:

- `R`: runtime state
- `I`: injected context provided by the runtime to the reasoner
- `A`: advisory output returned by the reasoner
- `R'`: next runtime state after runtime acceptance or rejection

These domains are related by three abstract maps:

- `P: R -> I` projection or injection
- `D: I + H -> A` reasoner turn over injected context plus hidden reasoner history `H`
- `K: R × A -> R'` runtime acceptance, rejection, or commitment

This yields the interaction shape:

```text
R --P--> I
I + H --D--> A
R × A --K--> R'
```

The runtime owns `P` and `K`.
The reasoner owns the internal turn `D`.

## Injected Context

Injected context is the governed runtime output that the reasoner is allowed to see and use.

Injected context may include:

- selected workflow values
- selected observations
- selected obligations
- selected policy summaries or policy-relevant conditions
- selected capability information
- selected task framing and instructions

Injected context is not assumed to be a complete copy of runtime state.
It may:

- select
- omit
- redact
- summarize
- reorder
- frame

runtime information for reasoning.

The runtime is therefore responsible both for access control and for context shaping quality.

## Hidden Reasoner History

The reasoner may operate over historical context `H` that is not fully visible to the runtime.

`H` may include:

- prior turns
- prior outputs
- hidden internal state
- latent model state or other future reasoner memory structures

The runtime does not need to observe or model all of `H` in order to govern interaction safely.
The runtime governs interaction by:

- controlling injected context,
- constraining accepted interaction boundaries,
- and retaining sole authority over acceptance and commitment.

## Advisory Outputs

Advisory outputs are non-authoritative artifacts produced by the reasoner.

Examples include:

- interpretations
- summaries
- candidate plans
- candidate tests
- candidate edits
- candidate commands
- candidate tool requests
- candidate next-step selections

Advisory outputs do not become authoritative merely by being produced.
They remain outside authoritative workflow state until accepted by the runtime.

This is the central protection against treating reasoner output as if it were already execution.

## Acceptance Boundaries

Acceptance boundaries are the places where advisory outputs return to runtime authority.

At an acceptance boundary, the runtime may:

- accept the advisory output,
- reject it,
- require more information,
- require further checks,
- transform it into a constrained executable form,
- or stop progression entirely

Acceptance boundaries may apply to:

- candidate observations
- candidate writes or updates
- candidate sends or external requests
- candidate verification operations
- candidate state transitions

The runtime remains the only component that can turn advisory outputs into authoritative state
transition.

## Commitment and Rejection

If an advisory artifact is accepted, the runtime may commit it into authoritative behavior through:

- policy evaluation
- obligation or invariant checking
- capability verification
- guard checking
- execution of an effectful operation
- provenance capture

If an advisory artifact is rejected, the rejection belongs to runtime authority, not to reasoner
self-assessment.

This means the interaction layer never owns the final decision about authoritative progression.

## Current Concrete Boundary: Tool Calls

At present, tool calls are the concrete interaction boundary most often used by external reasoners.

They are one current realization of the acceptance boundary, not the only abstract model.

Tool-call boundaries are useful because they let the runtime:

- validate the requested operation
- constrain arguments and capabilities
- execute or reject the request
- surface results back into injected context
- record provenance around accepted effects

Future reasoners or transports may use different concrete interfaces, but they must preserve the
same contract:

- injected context is runtime-governed
- advisory outputs are non-authoritative
- acceptance and commitment are runtime-owned

## Explicit Non-Overlap with Runtime-Only Visibility

The following runtime-visible features are not part of projection machinery:

- monitor views
- `exposes` clauses
- workflow observability
- `MonitorLink`
- runtime tracing
- user/tool-visible observable outcomes

These are runtime-only visibility constructs.
They may reveal runtime state to users, tooling, or other workflows, but they are not the same as
runtime-to-reasoner injected context.

In particular:

- monitorability is not projection
- an exposed workflow view is not injected reasoner context
- runtime observability is not advisory derivation

If a later design needs both runtime visibility and reasoner projection over the same data, the two
contracts must be specified separately.

## Interaction Constraints

The interaction layer must not:

- redefine runtime truth
- redefine runtime semantics
- bypass policy or obligation gates
- bypass capability verification
- bypass effect ceilings or guard checks
- treat monitor authority as implicit reasoning authority
- require the runtime to know the full contents of reasoner history

These constraints preserve the runtime-only contracts already frozen elsewhere in the spec set.

## Authoritative Runtime Responsibilities Preserved

Across all interaction scenarios, the runtime remains authoritative for:

- admission of facts into authoritative state
- workflow progression
- policy and obligation decisions
- capability/provider compatibility decisions
- effect execution
- provenance capture
- official workflow history

The reasoner may guide, interpret, propose, and request.
The runtime alone may accept and commit.

## Handoff Use

This note should be cited by later work that:

- adds minimal advisory-boundary framing to `SPEC-004`
- tightens projection versus monitorability terminology
- defines human-facing surface guidance for advisory, gated, and committed stages
- plans later implementation convergence against the interaction contract
- extends runtime/type verification with future interaction-obligation checks

If a later change appears to move a runtime-only feature into the interaction layer, the first
question remains:

> Does this still have complete meaning without a reasoner present?

If yes, the runtime meaning stays primary and the interaction-facing meaning must be added as a
separate layer rather than as a silent reinterpretation.
