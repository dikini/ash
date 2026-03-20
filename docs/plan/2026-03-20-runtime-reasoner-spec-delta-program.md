# Runtime-Reasoner Spec Delta Program

Date: 2026-03-20
Status: TASK-190 synthesis note

## Purpose

This document synthesizes:

- [TASK-187](tasks/TASK-187-freeze-runtime-reasoner-separation-rules.md)
- [TASK-188](tasks/TASK-188-audit-runtime-and-verification-specs-for-reasoner-boundaries.md)
- [TASK-189](tasks/TASK-189-audit-surface-and-observability-docs-for-reasoner-boundaries.md)

into one ordered program of follow-up spec work.

Its purpose is not to revise the specs directly. Its purpose is to state:

- what should change
- what should remain unchanged
- which changes are framing-only versus normative
- and how the design-review outcome affects later convergence work

## Inputs

Primary references:

- [Runtime-Reasoner Interaction Model](../design/RUNTIME_REASONER_INTERACTION_MODEL.md)
- [Runtime-Reasoner Separation Rules](../reference/runtime-reasoner-separation-rules.md)
- [Audit: Runtime and Verification Reasoner Boundaries](../audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md)
- [Audit: Surface and Observability Reasoner Boundaries](../audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md)

Reviewed canonical contracts:

- [SPEC-001: IR](../spec/SPEC-001-IR.md)
- [SPEC-002: Surface Language](../spec/SPEC-002-SURFACE.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-017: Capability Integration](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-018: Capability Runtime Verification Matrix](../spec/SPEC-018-CAPABILITY-MATRIX.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

## Synthesis Summary

The design-review outcome is stable:

1. The runtime-facing contracts are already correctly runtime-only.
2. Monitor views, `exposes`, and workflow observability are correctly runtime-only.
3. The largest remaining gap is not semantic contradiction. It is missing framing and terminology
   around runtime-to-reasoner interaction.
4. The next spec work should add an explicit interaction contract family without overloading
   runtime-only constructs.

So the follow-up program should be conservative:

- preserve the runtime-only meaning of existing runtime and monitoring constructs
- add explicit interaction framing where it is currently silent
- tighten terminology where visibility-oriented words risk drift

## Protected Runtime-Only Areas

The following areas should not be repurposed as runtime-to-reasoner machinery:

- `exposes` clauses
- monitor views
- `MonitorLink`
- runtime observability
- capability verification
- approval routing
- mailbox scheduling
- effect execution
- provenance capture

These all retain full meaning in a reasoner-free execution model and therefore stay runtime-only.

## Delta Classes

This synthesis uses three delta classes:

- `Framing-only`
  Adds or clarifies explanatory boundary text without changing canonical feature meaning.
- `Normative`
  Changes or extends the canonical contract.
- `New document`
  Introduces a new contract or boundary note rather than stretching an existing spec beyond its
  intended ownership.

## Prioritized Delta Program

### Priority 1: Add a dedicated runtime-to-reasoner interaction contract

Class: `New document`

Target:

- new interaction-facing contract document, most likely under `docs/reference/` first and later
  promotable to a spec if needed

Why:

- the runtime audit found the core runtime and verification specs correctly runtime-only
- the missing projection and advisory-boundary story should not be injected ad hoc into
  `SPEC-001`, `SPEC-017`, or `SPEC-018`

Required content:

- injected or projected context as a governed runtime output
- advisory outputs as non-authoritative artifacts
- acceptance boundaries for outputs returning to the runtime
- current tool-call boundaries as a concrete transport, not the sole abstract model
- explicit statement that monitor views and `exposes` are not projection machinery

Rationale:

This is the cleanest way to add the missing interaction story without contaminating runtime-only
specs.

### Priority 2: Add minimal runtime-facing framing to SPEC-004

Class: `Framing-only`

Target:

- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)

Why:

- `SPEC-004` is the natural place to state runtime authority, validation ownership, and the fact
  that any external reasoner remains outside authoritative state transition unless accepted
- the runtime audit classified `SPEC-004` as correct but silent

Recommended delta:

- add a short subsection near the front describing:
  - authoritative runtime state ownership
  - advisory interaction as external to the semantic judgment until accepted
  - preservation of execution neutrality

Must not do:

- introduce concrete LLM mechanics into the operational rules
- redefine `exposes`, monitoring, or capability verification as reasoner-facing mechanisms

### Priority 3: Tighten terminology around projection, monitorability, and observation

Class: `Framing-only`

Targets:

- [LANGUAGE-TERMINOLOGY](../design/LANGUAGE-TERMINOLOGY.md)
- possibly a short non-overlap note in
  [Runtime-Reasoner Interaction Model](../design/RUNTIME_REASONER_INTERACTION_MODEL.md)

Why:

- the surface/observability audit found no semantic contradiction, but terminology is still loose
- `projection`, `monitorability`, and `exposed workflow view` are not yet frozen as distinct terms
- `observe` is overloaded between workflow input acquisition and generic monitor-view access wording

Recommended delta:

- reserve terminology for:
  - `projection`
  - `monitorability`
  - `exposed workflow view`
- add an explicit note that:
  - monitorability is runtime visibility
  - projection is runtime-to-reasoner context transfer
  - the two must not be conflated

### Priority 4: Add human-facing stage guidance to the surface contract only after the interaction contract exists

Class: `Normative` or `Framing-only`, depending on final design

Target:

- [SPEC-002: Surface Language](../spec/SPEC-002-SURFACE.md)

Why:

- humans need help understanding where stages are advisory, gated, or committed
- but surface guidance should not be added before the interaction contract and terminology are
  frozen, otherwise the language risks encoding an unstable story

Recommended delta:

- defer actual surface-language guidance until after the interaction contract exists
- when added, keep it explanatory unless there is a deliberate decision to introduce syntax or
  annotations

Must not do:

- overload existing runtime-only constructs such as `exposes`
- imply that monitor visibility is the same thing as reasoner projection

### Priority 5: Leave SPEC-001, SPEC-017, SPEC-018, and SPEC-021 semantically unchanged unless a later concrete contradiction appears

Class: `No immediate delta`

Targets:

- [SPEC-001: IR](../spec/SPEC-001-IR.md)
- [SPEC-017: Capability Integration](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-018: Capability Runtime Verification Matrix](../spec/SPEC-018-CAPABILITY-MATRIX.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

Why:

- the audits found these documents runtime-only and aligned
- current issues in these documents are mostly phrasing or terminology adjacency, not contract
  failure

Allowed future work:

- small wording or terminology clarifications

Disallowed immediate move:

- adding reasoner projection semantics directly into these documents without first proving that the
  contract is truly split rather than purely runtime-owned

## Document-by-Document Outcome

### SPEC-001

Current classification: `runtime-only`, `Aligned`

Immediate action: none

Follow-up note:
- keep execution-neutral IR meaning separate from reasoner transport or context models

### SPEC-004

Current classification: `runtime-only`, `Silent`

Immediate action:
- add a small framing section in a later follow-up task

Follow-up note:
- use this file to clarify runtime authority, not to encode detailed reasoner machinery

### SPEC-002

Current classification: runtime-facing surface contract with terminology drift risk

Immediate action: none

Follow-up note:
- revisit only after the interaction contract and terminology pass are complete

### SPEC-017

Current classification: `runtime-only`, `Aligned`

Immediate action: none

Follow-up note:
- monitor views and capability integration remain runtime-owned

### SPEC-018

Current classification: `runtime-only`, `Aligned`

Immediate action: none

Follow-up note:
- approval routing and verification remain runtime verification concerns

### SPEC-021

Current classification: runtime observability contract, aligned but terminology-adjacent

Immediate action: none

Follow-up note:
- preserve monitor visibility as observable runtime behavior, not as projection

### LANGUAGE-TERMINOLOGY

Current classification: design aid with terminology gaps

Immediate action:
- plan a small terminology reservation pass

### RUNTIME_REASONER_INTERACTION_MODEL

Current classification: design note with correct separation but one implicit non-overlap

Immediate action:
- plan a small clarification that monitor views and `exposes` are not projection machinery

## Impact on Existing Convergence Phases

### Unchanged

The following existing convergence areas remain unchanged by this review outcome:

- parser and lowering convergence
- type and verification convergence
- runtime convergence for `receive` and policy outcomes
- REPL and CLI convergence
- ADT convergence

Reason:

These phases remain about existing runtime and language contracts, not about adding reasoner-facing
interaction semantics.

### Newly constrained

Future work that touches:

- runtime projection
- advisory outputs
- runtime-to-reasoner boundaries
- human-facing stage guidance for advisory versus authoritative work

must now respect the frozen separation rules and this delta ordering.

### Practical effect

Do not modify runtime-only features in service of reasoner interaction until the interaction
contract and terminology pass are completed.

## Recommended Follow-up Task Shapes

The next tasks should be introduced as docs-only tasks, likely in this order:

1. define the runtime-to-reasoner interaction contract
2. add the minimal runtime-authority framing to `SPEC-004`
3. reserve terminology for projection, monitorability, and exposed workflow views
4. only then revisit surface-language guidance for humans

These should be separate tasks so runtime-only and interaction-layer edits do not get mixed in one
change.

## Conclusion

The design-review phase found no need to restructure the existing runtime-only contracts.

The correct next move is to:

- preserve runtime-only constructs as they are
- add a separate interaction contract
- tighten terminology
- delay human-facing surface guidance until the interaction story is explicit

That ordering preserves the clean boundary established by the separation rules and keeps monitor and
observability features out of reasoner-projection semantics.
