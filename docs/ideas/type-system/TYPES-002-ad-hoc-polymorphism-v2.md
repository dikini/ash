---
status: reviewing
version: v2
created: 2026-04-01
last-revised: 2026-04-01
derived-from: TYPES-002-ad-hoc-polymorphism.md
related-plan-tasks: []
tags: [type-system, typeclasses, traits, ad-hoc-polymorphism, effects, capabilities]
---

# TYPES-002 V2: Ad-Hoc Polymorphism

## Purpose

This is a second-pass exploration of ad-hoc polymorphism in Ash.

It does not try to collapse the design space into a premature MVP. Instead, it keeps the
important options open while pruning a few dead ends from the earlier draft and making the
decision pressure more explicit.

The long-term goal remains the same: Ash should support executable, verifiable workflows with
provenance, effect tracking, and governed collaboration between human and AI participants. Any
ad-hoc polymorphism design should be judged by whether it improves that goal without weakening
reasoning, authority boundaries, or diagnostics.

## Relationship to V1

The `v1` document is preserved as a reasoning trace. It captures the meander, the corrections,
and the conceptual false starts.

This `v2` document is a synthesis pass:

- dead ends are reduced or demoted;
- syntax is more schematic and less committal;
- examples are more Ash-native or future-Ash-native;
- evaluation is organized around workloads, not only around familiar language families.

## What Seems Solid

Several points now look more like design constraints than open questions:

1. **Ad-hoc polymorphism is a separate concern from parametric polymorphism.**
   Ash already has generics. The question is how to express interface-constrained generic code.

2. **Capabilities and interfaces are not the same thing.**
   Capabilities represent runtime authority and governed access to external resources. Interface
   constraints describe what operations are available for a type or family of types.

3. **Effects are a distinct typing dimension, not ordinary payloads.**
   Effects should not be modeled as ordinary inhabitants of the same kind as data values. They
   classify computation forms or arrow structure, even if computations themselves may later be
   representable as values in Ash.

   > Effects are not ordinary value-level payloads. They live in a distinct typing dimension and should attach to computation judgments or arrow structure, even if computations themselves may be representable as values.

   > Effects classify computations, not result values alone.

4. **Coherence matters more than surface elegance.**
   Ash is trying to support predictable, inspectable, and eventually proof-friendly workflows.
   Ambiguous instance resolution would cut directly against that.

5. **Ash should prefer design leverage over language cosplay.**
   A familiar Haskell or Rust surface is only good if it fits Ash's deeper semantics.

6. **Authority separation needs an explicit bridge, not only a conceptual split.**
   It is not enough to say that interfaces describe semantic intent while capabilities provide
   runtime access. Ash also needs a way to represent the decision to use higher implementation
   authority when the design-authorized purpose remains lower-grade.

## Big Design Pressure

The central question is not really:

- `Show a =>` or `T: Show`?

The deeper question is:

- what kind of interface abstraction can Ash support while preserving explicit authority,
  understandable effects, good diagnostics, and a path to formal reasoning?

That suggests evaluating the space along these axes:

| Axis | Why it matters in Ash |
|------|------------------------|
| Coherence | Ash needs predictable semantics and debuggable resolution |
| Authority separation | Interface abstraction must not smuggle in capability-style access |
| Effect visibility | Interface methods must compose cleanly with Ash's effect tracking |
| Elaboration model | The implementation strategy affects runtime cost and diagnostics |
| Ergonomics | Boilerplate matters, but not more than clarity |
| Verifiability | The chosen abstraction should not make provenance and proof obligations opaque |

## Candidate Directions

### Direction 1: Closed-World Interfaces

This is the most promising direction at the moment.

The idea is a trait-like or interface-like mechanism with explicit implementation sites and a
coherence story closer to Rust than to Haskell's open global instance space.

Schematic syntax only:

```ash
interface Explain<T> {
  explain : T -> String
}

impl Explain<PolicyDecision> {
  explain(decision) = ...
}

workflow emit_decision<T: Explain>(decision: T) capabilities: [audit_log] {
  let msg = Explain::explain(decision)
  act audit_log.write(msg)
}
```

Why this looks strong:

- it keeps instance lookup relatively disciplined;
- it does not force capabilities and interfaces into one mechanism;
- it fits Ash's preference for explicit, inspectable semantics;
- it leaves room for monomorphization, dictionary passing, or a hybrid elaboration strategy.

Current preference:

- strong coherence;
- explicit impl locality;
- no overlapping impls by default;
- capability invocation remains effect-first and explicit.

What remains open:

- exact surface syntax;
- whether local instances exist at all;
- whether dynamic dispatch or existential packaging is ever needed;
- whether associated items belong in the first serious design pass.

### Direction 2: Haskell-Style Typeclasses

This remains attractive for concision and expressiveness, and it has the best-developed theory.

Schematic syntax:

```ash
class Explain a where
  explain : a -> String

instance Explain PolicyDecision where
  explain(decision) = ...

emit_decision : Explain a => a -> Workflow
```

Why it remains interesting:

- the constraint story is elegant;
- the elaboration story is well-understood;
- typeclass-style reasoning fits a lot of generic programming patterns.

Why I currently prefer Direction 1 over this:

- open-world instance search is a poor default fit for Ash;
- coherence and diagnostics become harder faster;
- it is too easy for the surface to look compact while the semantics become obscure.

I would not rule this out entirely. But if Ash borrows from Haskell, it should likely borrow
more from the elaboration model than from the fully open instance culture.

### Direction 3: Explicit Evidence Passing

This is less attractive as the main user-facing abstraction, but it remains very important as a
semantic model and possibly as an escape hatch.

```ash
type Explainer<T> = {
  explain : T -> String
}

workflow emit_decision<T>(
  decision: T,
  explainer: Explainer<T>
) capabilities: [audit_log] {
  let msg = explainer.explain(decision)
  act audit_log.write(msg)
}
```

Why it matters:

- it makes the semantics obvious;
- it aligns with classic dictionary passing;
- it may be enough for some subsystems even if Ash never adds full implicit interface resolution.

Why I would not make it the main answer by default:

- it pushes a lot of bookkeeping into user code;
- it reads more like a manual encoding than a language feature;
- it can obscure the actual design question by turning everything into records.

Still, it is useful as a reference model: if a more ergonomic design cannot be explained as a
clear elaboration of evidence passing, that is a warning sign.

### Direction 4: Capability Unification

This is the most speculative direction and currently the least convincing as the mainline path.

The motivating idea is understandable: Ash already has a rich capability model, so perhaps it
could also carry interface abstraction.

But the semantic roles are different:

- capabilities are about governed authority and runtime access;
- ad-hoc polymorphism is about type-indexed interface abstraction.

Ash's current capability semantics are explicit that capability values are authorization witnesses,
not ordinary receivers or method dictionaries. That makes full unification feel more like a
semantic rewrite than an incremental extension.

I would keep this direction open only as a research branch, not as the default framing of the
core problem.

## Preferred Big-Picture Leaning

The current leaning is:

- prefer **closed-world interfaces** over open global instance search;
- keep **capabilities separate** from interface abstraction;
- model **authority elevation** as an explicit, scoped source-level decision backed by audit and
  provenance semantics;
- treat **explicit evidence passing** as a semantic model and fallback, not necessarily as the
  primary surface;
- postpone **associated effects** until the core abstraction boundary is clearer.

This is not a conclusion. It is a way to reduce risk while keeping the broader space open.

## Ash-Native and Future-Ash-Native Examples

These examples are intentionally schematic. They are meant to test the shape of the abstraction,
not lock in syntax.

### Example 1: Human-Facing Explanation

Ash workflows will often need to turn structured values into explainable messages for audit logs,
operator consoles, or human review channels.

```ash
interface Explain<T> {
  explain : T -> String
}

impl Explain<PolicyDecision> {
  explain(decision) = ...
}

impl Explain<CapabilityDenied> {
  explain(denial) = ...
}

workflow record_event<T: Explain>(value: T) capabilities: [audit_log] {
  let msg = Explain::explain(value)
  act audit_log.write(msg)
}
```

Why it matters:

- common generic operation over domain types;
- obviously useful in governed workflows;
- no need to conflate explanation with authority.

### Example 2: Redaction Before Human or AI Handoff

Ash will likely need type-directed redaction before passing data into review or model-facing
channels.

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

Why it matters:

- type-driven transformation with governance implications;
- capability use remains explicit at the workflow boundary;
- a good stress test for keeping interfaces and authority separate.

### Example 3: Codec-Like Domain Translation

Generic serialization is not just a textbook example in Ash. It matters for snapshots, audit
records, persisted workflow state, and interop boundaries.

```ash
interface SnapshotCodec<T> {
  encode : T -> Bytes
  decode : Bytes -> Result<T, DecodeError>
}

workflow persist_snapshot<T: SnapshotCodec>(
  snapshot: T
) capabilities: [snapshot_store] {
  let bytes = SnapshotCodec::encode(snapshot)
  act snapshot_store.write(bytes)
}
```

Why it matters:

- likely common across runtime and governance boundaries;
- still mostly about type-indexed behavior, not authority;
- leaves open whether methods must always be pure.

### Example 4: Generic State-Machine or Handler Logic

Future Ash may want reusable state-machine loops, supervision helpers, or provider-neutral
handler abstractions.

```ash
interface Handler<S, Input, Output> {
  step : S -> Input -> Result<(S, Output), HandlerError>
}
```

This is interesting because it pushes toward richer associated items and more ambitious generic
interfaces. It is therefore a good workload, but probably not the first one to optimize for.

### Example 5: Backends With Different Effects

This is where the associated-effects branch becomes attractive.

```ash
interface Store<S, T> {
  put : S -> T -> Result<(), StoreError>
  get : S -> Key -> Result<T, StoreError>
}
```

A memory store, file-backed store, and remote store might all satisfy the same interface while
carrying very different operational consequences.

This is promising, but also where Ash should move carefully. Once interface abstraction starts to
carry effect variables or associated effects, inference and diagnostics become substantially more
complex.

## Authority Elevation at Implementation Boundaries

### Problem Summary

Some Ash operations are authorized by design at one level, but can only be executed by a mechanism
that needs more operational authority.

That gap is not just a provider detail. It is a design decision:

- the workflow has a **design authority** tied to its intended purpose;
- the selected implementation has an **implementation authority** tied to its operational needs;
- when those differ, someone must decide to authorize the difference and Ash should make that
  decision visible.

This matters even when the operation is routine. A hosted LLM call may be used only for
Deliberative drafting, but it still requires outbound network access, credentials, and billable
remote execution. A remote database read may be epistemic in purpose while still operational in
mechanism. More dangerous cases, such as downloading and executing code from the public internet,
raise the same structural issue with a much higher review burden.

### Example

The same deliberative task can require different implementation authority depending on the chosen
backend:

```ash
workflow draft_reply(
  ticket: SupportTicket,
  llm: cap HostedLlm
) -> Result<ReplyDraft, RuntimeError> {
  let prompt = Redact::redact_for_review(ticket)

  with elevation
    purpose "deliberative reply drafting"
    requires [network_outbound, credential_use, billable_api]
    because "the selected model runs remotely"
  {
    act llm.complete(prompt)
  }
}
```

If the same workflow used a local model, the deliberative purpose could remain the same while the
elevation site disappeared because the implementation authority no longer exceeded the design
authority. That is the key point: the semantic task is stable, but the implementation choice can
still force an explicit authority decision.

### Recommended Design

The recommended direction is a single integrated model:

- **Interfaces and workflow contracts** describe the semantic operation and its design authority.
- **Capabilities and providers** describe the implementation mechanism and the operational
  authority it requires.
- **Elevation sites** are explicit, scoped source-level markers used when implementation authority
  exceeds the authority otherwise available to the workflow.
- **Policy** approves or rejects the elevation as a distinct decision, not merely as a side effect
  of using a capability.
- **Audit and provenance** record the elevation event itself, including purpose, justification,
  requested authority, and the operation performed.

This fits the current Direction 1 preference. It keeps interfaces and capabilities separate while
admitting that the crossing between them can itself be semantically important.

The core rule should be:

> No hidden authority escalation. If a design-authorized action requires higher implementation
> authority, Ash should require an explicit elevation site and record it in the runtime semantics.

This gives Ash one design that covers the full range:

- routine remote execution cases such as hosted LLM calls;
- infrastructure-shaped cases such as remote reads and distributed storage;
- high-risk cases such as downloading and executing code from the public internet.

The cases differ in policy burden, not in whether the elevation decision exists at all.

## Decision-Driving Workloads

Rather than choosing based on syntax preference alone, Ash should use a workload table like this
to evaluate candidate designs.

| Workload | Why it matters for Ash | Main pressure on the design |
|----------|-------------------------|-----------------------------|
| Human-readable explanation of typed decisions, denials, and outcomes | Core to auditability and governed workflows | Requires simple, coherent type-indexed behavior |
| Redaction before human/AI handoff | Governance and provenance boundary | Must keep interfaces separate from explicit capability use |
| Snapshot and event encoding/decoding | Persistence, replay, audit, interop | Needs reusable generic behavior without authority confusion |
| Generic equality/order/display for ADTs | Foundational library ergonomics | Favors derivation, coherence, and good diagnostics |
| Reusable handler or state-machine abstractions | Future OTP-like and workflow-runtime design | Presses toward associated items and more expressive interfaces |
| Swappable pure, local, and remote backends | Testing and deployment flexibility | Brings effect-sensitive interfaces into view |
| Proof-carrying or policy-aware abstractions | Long-term verification goals | Requires interfaces to remain inspectable and semantically disciplined |

This table is useful because it prevents the exploration from collapsing into "Haskell vs Rust"
as a matter of aesthetics. Ash should ask which design best serves these workloads with the least
semantic debt.

## Associated Effects

Associated effects remain an important branch, but they should be framed carefully.

The attraction is real:

- generic code could abstract over interface plus operational profile;
- test backends and production backends could share one interface while differing in effect;
- effect inference could, in principle, solve for both type-level and effect-level parameters.

But the risks are also real:

- the interface system becomes entangled with the effect system very early;
- inference becomes harder to explain;
- diagnostics become more important and more difficult;
- the language may be forced into associated-item machinery before the core interface story is
  settled.

Current stance:

- keep associated effects in scope as a second-layer design problem;
- do not let them define the core answer to "should Ash have ad-hoc polymorphism?"

## Questions Worth Carrying Forward

1. What coherence model best fits Ash: strict impl locality, an orphan-style rule, or something
   more novel but still deterministic?
2. Should Ash expose implicit resolution directly, or should it prefer a more explicit interface
   surface and keep elaboration mostly internal?
3. Are effectful interface methods acceptable in the core design, or should early interfaces be
   limited to pure or effect-transparent operations?
4. Is dynamic dispatch or existential packaging a necessary part of the same feature, or should it
   be considered separately?
5. How much deriving or generic programming support is needed before the feature becomes genuinely
   useful?
6. Can Ash gain the benefits of interface abstraction without reopening the function-vs-workflow
   question too aggressively?
7. What tooling and diagnostics would be required before a richer interface system is humane to use?

## Next Steps

- Keep `v1` as the preserved reasoning trace.
- Use this `v2` document as the main discussion surface.
- Expand the workload table with concrete examples from runtime, policy, audit, and review flows.
- Explore at least one closed-world interface sketch and one explicit-evidence sketch in more detail.
- Defer any serious attempt at capability/interface unification until the semantic payoff is clear.
- Revisit associated effects after the core abstraction boundary is better defined.
