# Runtime-Reasoner Separation Rules

## Status

TASK-187 review protocol.

## Purpose

This note freezes the classification rules and review protocol used to separate:

- runtime-only concerns
- runtime-to-reasoner interaction concerns
- mixed concerns that must be split explicitly

It exists to prevent later spec revisions from overloading established runtime features with
reasoner-facing meaning or from accidentally moving runtime authority into advisory interaction
contracts.

This note is authoritative for the design-review phase only. It does not redefine the canonical IR,
runtime semantics, or surface syntax contracts. Instead, it provides the rule set later audits and
spec revisions must use.

## Core Separation Test

The primary test is:

> Does this feature still have complete meaning in a reasoner-free execution model?

Interpretation:

- If yes, it is a runtime-only concern unless there is a separate interaction-layer aspect that
  must be specified independently.
- If no, it is an interaction-layer concern.
- If both a runtime core and an interaction-facing interpretation exist, the feature is a split
  concern and the two meanings must be specified separately.

This test is mandatory for every candidate feature, term, and spec delta in the review phase.

## Classification Outcomes

### Runtime-only

A feature is `runtime-only` when it has full meaning without any external reasoner present.

Runtime-only features may involve:

- authoritative state
- execution
- capability lookup
- mailbox scheduling
- policy evaluation
- obligation checking
- effect execution
- provenance
- trace and observability
- inter-workflow monitoring

Runtime-only features must not be reinterpreted as reasoner-facing projection or advisory
machinery merely because they expose or inspect runtime data.

### Interaction-layer

A feature is `interaction-layer` when it exists only to govern runtime-to-reasoner participation.

Interaction-layer features may involve:

- projection of runtime state into reasoner-visible context
- injected context framing
- advisory outputs
- interaction boundaries for requests returning to the runtime
- acceptance or rejection of reasoner-produced candidates

Interaction-layer features do not own runtime truth, execution, or authoritative state transition.

### Split concern

A feature is `split` when it has:

- a runtime core meaning that remains valid without a reasoner, and
- a separate interaction-facing interpretation relevant only when a reasoner participates

Split concerns must be documented in two parts:

1. the runtime meaning
2. the interaction-layer meaning

One side must not silently replace the other.

## Review Questions

Each audit item in the design-review phase must answer:

1. What is the authoritative runtime meaning of this feature?
2. Does the feature still make sense without a reasoner present?
3. If a reasoner participates, what interaction-layer meaning exists, if any?
4. Is the current document explicit about the separation, or is the distinction only implicit?
5. Does the current wording risk overloading a runtime-only feature with reasoner-facing meaning?

If a feature fails to answer these questions cleanly, it should be recorded as a review finding.

## Evidence Expectations

Audit findings should distinguish three cases:

- `Aligned`: the current document already separates the concern correctly
- `Silent`: the current document does not conflict, but the separation is missing or only implied
- `Tension`: the current wording risks conflation or creates conflicting interpretations

Each finding should cite:

- the file
- the relevant section or lines
- the classification
- the reason the item is aligned, silent, or in tension

## Runtime-only Non-goals

The following are explicitly runtime-only unless a future contract proves otherwise:

- monitor views
- `exposes` clauses
- workflow observability
- monitor authority and `MonitorLink`
- inter-workflow state inspection
- runtime scheduling and mailbox source selection
- capability verification and provider compatibility

These constructs may expose or structure runtime-visible information, but that does not make them
reasoner projection machinery.

In particular:

- monitorability is not projection
- exposed workflow views are not injected reasoner context
- observability is not advisory derivation

## Interaction-layer Constraints

The interaction layer must not:

- redefine runtime truth
- redefine authoritative state transition
- bypass runtime validation or rejection boundaries
- use monitor or exposure features as implicit reasoner-context channels
- assume the runtime has full visibility into reasoner history or internal state

Tool calls are a concrete current interaction boundary, not the sole canonical meaning of advisory
interaction. The interaction contract must therefore stay abstract enough to permit future
reasoners, transports, or execution strategies.

## Runtime Authority Preservation

The following must remain runtime-owned across all later spec work:

- admission of facts into authoritative state
- policy and obligation gates
- capability verification
- effect execution
- provenance capture
- official workflow history

Reasoner participation may guide, propose, or request, but it must not become authoritative merely
by existing.

## Use in Later Review

This note should be cited by:

- runtime and verification audits
- surface and observability audits
- later synthesis of document-by-document spec deltas

If later spec work introduces a feature that seems to mix concerns, the first question should be:

> Does this feature make sense without a reasoner present?

If the answer is yes, keep the runtime meaning primary and add any reasoner-facing interpretation
only as a separate contract.
