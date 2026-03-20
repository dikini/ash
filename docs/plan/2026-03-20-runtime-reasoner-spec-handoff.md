# Runtime-Reasoner Spec Handoff

Date: 2026-03-20
Status: TASK-195 handoff

## Purpose

This handoff closes the docs-only runtime-reasoner follow-up phase.

It states:

- which docs are now authoritative for runtime-to-reasoner interaction boundaries
- which runtime-only areas remain protected
- what remains explicitly out of scope
- and what later implementation-planning work may use as its starting surface

It does not create implementation tasks.
It closes the documentation phase cleanly so implementation planning can happen later against a
stable reference set.

## Authoritative Output of the Follow-Up Phase

The follow-up phase produced the following authoritative docs and references:

- [Runtime-to-Reasoner Interaction Contract](../reference/runtime-to-reasoner-interaction-contract.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
  with its minimal runtime-authority and advisory-boundary framing
- [Ash Language Terminology Guide](../design/LANGUAGE-TERMINOLOGY.md)
- [Runtime-Reasoner Interaction Model](../design/RUNTIME_REASONER_INTERACTION_MODEL.md)
- [Surface Guidance Boundary](../reference/surface-guidance-boundary.md)
- [Runtime-Reasoner Separation Rules](../reference/runtime-reasoner-separation-rules.md)

These documents now form the stable reference set for:

- projection versus runtime visibility
- advisory outputs versus authoritative commitment
- runtime authority boundaries
- terminology around monitorability and exposed workflow views
- human-facing stage guidance ownership

## Protected Runtime-Only Areas

The following remain explicitly runtime-only and must not be reused as reasoner-projection
machinery:

- monitor views
- `exposes` clauses
- workflow observability
- `MonitorLink`
- capability verification
- approval routing
- mailbox scheduling
- effect execution
- provenance capture
- official workflow history

These retain full meaning in a reasoner-free execution model and therefore remain outside the
interaction-layer contract except where they are mentioned as non-overlap constraints.

## What the Follow-Up Phase Resolved

This phase resolved five concrete gaps:

1. The runtime-to-reasoner interaction contract now exists as a standalone reference.
2. `SPEC-004` now acknowledges runtime authority and advisory interaction boundaries without
   changing its canonical operational rules.
3. Terminology now distinguishes:
   - projection
   - monitorability
   - exposed workflow view
   - workflow `observe`
4. Human-facing surface guidance is now explicitly explanatory-first rather than syntax-first.
5. The repository now has a stable docs-only corpus that later implementation planning can target.

## What Remains Out of Scope

This handoff does not authorize:

- parser changes
- lowering changes
- type-system changes
- interpreter changes
- capability-runtime implementation changes
- CLI or REPL implementation changes
- new surface syntax for stages
- explicit runtime/reasoner placement markers in the language

Those belong to later planning work, not to this handoff.

## Later Implementation-Planning Surface

When implementation planning begins later, it should treat the following questions as the initial
planning surface:

1. Where does the current implementation need to surface or preserve the interaction contract?
2. Which runtime entry points need explicit projection or advisory-boundary handling?
3. Which observable behaviors need to remain runtime-only even if interaction-aware tooling is
   added?
4. Which parser or surface-language areas, if any, need explanatory documentation updates only,
   rather than syntax changes?

Implementation planning should start from the contracts above and should not reopen the docs-only
phase unless a real contradiction is found.

## Recommended Next Planning Boundary

The next planning phase should be:

- implementation-planning only

It should consume this handoff and decide:

- whether current Rust code needs convergence tasks for interaction-aware runtime boundaries
- whether documentation-only `SPEC-002` explanatory edits should be scheduled before code work
- whether any new syntax design is genuinely needed or still avoidable

That later phase should not re-open the separation rules unless evidence requires it.

## Closure Statement

The runtime-reasoner follow-up phase is complete as docs-only work.

The project now has:

- a stable interaction contract
- stable terminology
- minimal runtime-semantics framing
- a stable boundary for human-facing surface guidance

This is sufficient to begin later implementation planning without overloading runtime-only
constructs or silently extending the language surface.
