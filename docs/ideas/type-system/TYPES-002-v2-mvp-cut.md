---
status: candidate
created: 2026-04-06
last-revised: 2026-04-06
derived-from: TYPES-002-ad-hoc-polymorphism-v2.md
related-plan-tasks: [TASK-415]
tags: [type-system, interfaces, ad-hoc-polymorphism, mvp, closed-world]
---

# TYPES-002 V2 MVP Cut: Closed-World Interfaces

## Purpose

This document narrows `TYPES-002 V2` into a realistic MVP cut.

It does not attempt to solve the entire ad-hoc polymorphism design space. It defines the smallest
serious feature worth specifying next if Ash wants interface-constrained generic code without
collapsing capabilities, effects, and authority into one mechanism.

Document roles for `TYPES-002` now look like this:

- `TYPES-002` (`v1`) is the preserved non-normative reasoning trace;
- `TYPES-002 V2` is the broader polished exploration and serious discussion surface;
- this MVP cut is the narrowed follow-on target for planning/spec work.

## MVP Thesis

The MVP should be:

- closed-world;
- coherence-first;
- interface/capability separated;
- syntax-light;
- effect-conservative;
- friendly to explicit elaboration.

In practical terms, the MVP should add:

1. interface declarations;
2. impl declarations for concrete nominal types;
3. constrained type parameters on functions/workflows or equivalent generic declarations;
4. explicit associated methods only;
5. no associated types;
6. no associated effects;
7. no overlapping impls;
8. no capability/interface unification;
9. no dynamic dispatch or existential packaging in the first pass.

## Non-Goals

The MVP should explicitly defer:

- Haskell-style open-world instance search;
- overlapping instances or ad hoc incoherence rules;
- associated types;
- associated effects;
- interface impls for capabilities as such;
- trait objects / dynamic dispatch;
- derivation machinery beyond perhaps future follow-up notes;
- any requirement to settle the entire first-class-functions question first.

## Recommended Surface Shape

Use interface/impl syntax rather than Haskell-style class/instance syntax.

The MVP should also freeze one canonical bound form and one canonical method-call form so later
spec work has one serious source surface to build on.

Canonical first-pass forms:

- bound form: `T: Explain`
- call form: `Explain::explain(value)`

The first-pass spec should not try to support parallel alternatives such as `Explain T =>`,
trailing `where` clauses, receiver-style `value.explain()`, or bare type-directed `explain(value)`.

Schematic MVP surface:

```ash
interface Explain<T> {
  explain : T -> String
}

impl Explain<PolicyDecision> {
  explain(decision) = ...
}

workflow record_event<T: Explain>(value: T) capabilities: [audit_log] {
  let msg = Explain::explain(value)
  act audit_log.write(msg)
}
```

This is only a contract shape, not final syntax. But the MVP should keep these structural
properties:

- declarations are explicit;
- impl sites are explicit;
- constrained generics are explicit;
- the canonical bound syntax family is `T: Interface`;
- the canonical method call syntax is `Interface::method(value)`;
- method lookup is coherence-bounded;
- capability use remains explicit and separate.

## Coherence Rule for MVP

The MVP should adopt a strict coherence stance.

Recommended rule:

- at most one impl per `(Interface, ConcreteNominalType)` pair in the MVP-visible corpus;
- duplicate or conflicting impl declarations for the same pair are rejected rather than resolved by
  priority or shadowing;
- no overlapping impls;
- no ad hoc local shadowing;
- impl locality should be closed-world and repository/module governed rather than globally open.

The exact orphan/locality formulation can still be specified later, but the MVP must preserve
predictable resolution and readable diagnostics.

## Effect Stance for MVP

The MVP should begin with effect-conservative methods.

Recommended first-pass rule:

- interface methods are modeled as pure or effect-transparent typing surfaces;
- interface declarations do not introduce associated effects or per-method capability requirements in
  the first pass;
- capability use remains outside the interface mechanism;
- if an interface-constrained workflow performs operational work, that work is explicit in the
  workflow body via capabilities or ordinary workflow forms.

This avoids prematurely entangling interface resolution with effect inference, authority elevation,
and provider/runtime contracts.

## Relationship to Capabilities

This MVP keeps the separation explicit:

- interfaces describe type-indexed operations;
- capabilities describe governed runtime authority;
- workflows may use both, but neither mechanism should be elaborated as the other.

Example:

```ash
interface Redact<T> {
  redact_for_review : T -> RedactedView
}

workflow request_review<T: Redact>(
  value: T,
  reviewer: cap ReviewChannel
) {
  let payload = Redact::redact_for_review(value)
  send reviewer with payload
}
```

The interface handles type-directed transformation.
The capability witness handles governed message delivery.

## Elaboration Model Guidance

The MVP should be explainable as dictionary passing, even if the implementation later chooses a
more optimized strategy.

That means the feature should be specifiable as if:

- each constrained generic parameter carries evidence for the required interface;
- method calls can be explained through explicit evidence lookup;
- diagnostics can name the missing or conflicting evidence site.

This does not force the final implementation to expose explicit evidence in source syntax. It only
requires that the semantics stay understandable and inspectable.

## MVP Workloads

The MVP should optimize for a small set of clearly Ash-relevant workloads:

1. explanation / display of typed decisions, denials, and outcomes;
2. redaction before human or AI handoff;
3. codec-like translation for snapshots and stored events;
4. basic equality/display/order-like library ergonomics for ADTs.

The MVP should not optimize first for:

- backend-swapping with different effect grades;
- OTP-style behavior frameworks;
- dynamic-dispatch-heavy abstractions;
- associated-effect inference.

Those are future workloads and should not define the first feature boundary.

## Suggested Initial Restrictions

To keep the first spec and implementation tractable, the MVP should likely restrict:

- impl targets to nominal concrete types first;
- interface methods to ordinary type parameters and returns;
- constrained declarations to one simple bound syntax family: `T: Interface`;
- method invocation to one explicit namespace form: `Interface::method(value)`.

The MVP should avoid multiple parallel call styles in its first spec.

## What This MVP Must Specify

Before implementation planning begins, the next normative/spec cut should answer exactly these:

1. What declarations introduce interfaces?
2. What declarations introduce impls?
3. What types may receive impls in MVP?
4. What syntax expresses bounds?
5. What call syntax is canonical?
6. What coherence rule rejects duplicate/conflicting impls?
7. What type-check failure categories are exposed for missing impls and ambiguous/conflicting impls?
8. How does the feature stay separate from capabilities and effect typing?

## What This MVP Deliberately Leaves Open

This MVP does not settle:

- the final long-term elaboration strategy;
- dynamic dispatch;
- associated types/effects;
- derivation;
- generic impls over arbitrary constructor families if that complicates the first pass;
- authority elevation sites at the interface boundary.

Those are second-layer design questions.

## Promotion Recommendation

Promote `TYPES-002 V2` through this MVP cut rather than directly.

TASK-415 now records the documentation/spec narrowing pass that turns this into the repository's
single closed-world-interface follow-on target.

Any later normative interface task should:

- defines the MVP interface surface;
- records the coherence rule;
- fixes the separation from capabilities and effect typing;
- selects one call form and one bound form;
- explicitly defers associated types/effects and dynamic dispatch.

## Planning Anchor

Realized by:

- [TASK-415: Closed-World Interfaces MVP Spec Cut](../../plan/tasks/TASK-415-closed-world-interfaces-mvp-spec-cut.md)
