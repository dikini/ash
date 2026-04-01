---
status: drafting
version: v1
created: 2026-04-01
last-revised: 2026-04-01
related-plan-tasks: []
tags: [type-system, capabilities, effects, semantics, vocabulary, judgments]
---

# TYPES-003: Capability and Effect Vocabulary

## Problem Statement

Ash discussions currently use the words `capability`, `provider`, `effect`, `authority`,
`constraint`, `obligation`, and `provenance` in ways that are often adjacent but not identical.
That makes exploration difficult: one note may talk about capabilities as external interfaces,
another as runtime authority, another as provider contracts, and another as effectful operations.

This exploration is a bridge note. Its purpose is not to lock in final syntax or a complete
formalism. Its purpose is to disambiguate the different concerns, relate them back to Ash's
judgment structure, and improve the language we use when reasoning about future features.

## Scope

- **In scope:**
  - Distinguishing the semantic roles currently bundled under "capability"
  - Clarifying the relationship between capabilities and effects
  - Using the existing workflow judgment as an anchor for vocabulary
  - Identifying which concerns are surface-language concerns and which are judgment-level concerns
  - Producing terminology that future explorations can reuse

- **Out of scope:**
  - Final surface syntax changes
  - Full formalization of all judgment families
  - Replacing the current capability or effect specs
  - Deciding first-class workflow/function semantics

- **Related but separate:**
  - SPEC-003: Type system boundaries and effect vocabulary
  - SPEC-004: Big-step workflow judgment backbone
  - SPEC-010: Embedding-side provider model
  - SPEC-017: Capability integration and usage-site `cap`
  - TYPES-002: Ad-hoc polymorphism and interface abstraction
  - MCE-003: Functions vs capabilities

## Anchor: The Workflow Judgment

The best current anchor is the workflow big-step judgment in SPEC-004:

```text
Γ, C, P, Ω, π ⊢wf w ⇓ out
```

At minimum, this reminds us that Ash is already tracking multiple distinct semantic dimensions:

- `Γ` for ordinary typing and binding context
- `C` for capability or authority context
- `P` for policy context
- `Ω` for obligations
- `π` for provenance
- `out` for workflow outcome, including effect and trace

This does not force Ash to expose all of these dimensions directly in the surface language.
But it does suggest a useful discipline:

- not every concern belongs in one undifferentiated notion of "type";
- not every concern should be collapsed into one surface construct;
- the same surface word may currently be covering several different semantic roles.

## Current Understanding

### What we know

- Ash already uses an explicit effect vocabulary: `Epistemic`, `Deliberative`, `Evaluative`,
  `Operational`.
- Usage-site capability types are already specified separately from capability declarations:
  `cap C` is an authorization witness at a workflow boundary, not a method receiver.
- Capability invocation is currently explicit and effect-first.
- Embedding-side providers are a separate concept from source-level capability declarations.
- Provenance and obligations are tracked as distinct semantic concerns already.

### What we're uncertain about

- Which of these distinctions should remain purely semantic, and which should have clearer surface
  names?
- Whether the word `capability` should continue to cover multiple related facets, or whether Ash
  should adopt a more split vocabulary.
- How tightly effect classification should be tied to capability operations in future designs.
- Whether future features such as ad-hoc polymorphism, interfaces, or first-class workflows should
  reuse any of this vocabulary.

## Working Hypothesis

Ash should stop treating "capability" as one thing.

Instead, Ash should talk about several related but distinct facets:

1. a **capability declaration** in source
2. a **capability identity** used at invocation sites
3. a **capability value** or witness, where the boundary admits one
4. a **provider implementation** on the embedding/runtime side
5. an **effect classification** of the computation performed
6. separate **policy**, **obligation**, and **provenance** contexts that constrain or record use

This separation does not mean every facet needs its own syntax. It means the prose should stop
pretending they are interchangeable.

## Disambiguation Table

| Facet | Semantic home | What it is | What it is not | Notes |
|------|---------------|------------|----------------|-------|
| Capability declaration | Source declaration layer | Named contract for an external or governed operation family | Not itself a runtime provider instance | Example: `capability Args : observe (index : Int) returns Option<String>` |
| Capability identity | Name-resolution and invocation layer | The resolved identity named by `observe C ...` or `act C ...` | Not a general object receiver | Important for explicit effect-first invocation |
| Capability value / witness | Workflow boundary / authority context `C` | Authorization witness that a workflow may use capability `C` at a boundary | Not a method dictionary or trait object by default | Current `cap C` wording in SPEC-017 points here |
| Provider | Embedding/runtime layer | Concrete host-side implementation of capability behavior | Not the same as source capability declaration | Lives in SPEC-010 style embedding discussions |
| Effect | Effect judgment / workflow outcome | Classification of the computation performed | Not the capability itself | A capability use may incur an effect; the capability is not identical to the effect |
| Policy context | Judgment context `P` | Governing rules for allowed or rejected actions | Not the capability declaration itself | Policy may constrain capability use |
| Obligation context | Judgment context `Ω` | Duties introduced, discharged, or left outstanding | Not the same as policy or authority | Local logical burden, not external authority |
| Provenance context | Judgment context `π` | Origin and audit lineage of values and actions | Not the effect lattice itself | Tracks where things came from, not only what was done |
| Trace / outcome | Evaluation result `out` | What happened during workflow execution | Not the same as typing context | Contains effects and emitted events |

## A More Precise Relationship Between Capabilities and Effects

The current prose around capabilities and effects is easy to blur because capability operations are
often the places where effects become observable.

But the cleanest framing is:

- a capability is a governed interface or authority-bearing operation family;
- an effect is a classification of the computation carried out;
- capability use may incur an effect, require policy evaluation, change obligations, and extend
  provenance, but those are not all the same thing.

That implies a useful distinction:

> A capability is not an effect. A capability use is a computation whose evaluation may incur one
> or more effects and may be subject to authority, policy, obligation, and provenance judgments.

And a second one:

> Capability vocabulary should answer "what operation family or authority is being exercised?",
> while effect vocabulary should answer "what kind of computation has occurred?"

This is not merely stylistic. It affects how we reason about typing, diagnostics, future surface
syntax, and possible features like interfaces or effect-indexed abstractions.

## Cross-Section Through One Example

Consider a simplified read-like capability:

```ash
pub capability Args : observe (index : Int) returns Option<String>

workflow main(args: cap Args) -> Result<(), RuntimeError> {
  let first = observe Args 0
  ...
}
```

Several distinct things are happening here:

1. `Args` as a **declaration** names a capability contract.
2. `args: cap Args` introduces a **boundary witness** that this workflow is authorized to use it.
3. `observe Args 0` uses the **capability identity** explicitly in source.
4. The host runtime must have some **provider implementation** that can serve that operation.
5. The call incurs an **epistemic effect** because the computation is read-like.
6. The step may be subject to **policy checks**, may interact with **obligations**, and may record
   **provenance** in the enclosing judgmental machinery.

Using one word, "capability", for all six points is possible. It is also a reliable way to create
confusion.

## Common Confusions to Avoid

| Confusion | Better phrasing |
|-----------|-----------------|
| "The capability is epistemic" | "Using this capability in this form incurs an epistemic effect" |
| "A provider is the capability" | "A provider implements a capability declaration at the embedding boundary" |
| "`cap C` is a receiver or object" | "`cap C` is an authorization witness at a workflow boundary" |
| "Policies are capability constraints" | "Policies govern capability use, but are tracked in a separate semantic dimension" |
| "Obligations are permissions" | "Obligations are duties; capabilities are authority" |
| "Provenance is just trace" | "Trace records execution events; provenance tracks origin and lineage" |

## Design Dimensions

| Dimension | Keep one overloaded term | Split vocabulary explicitly | Hybrid |
|-----------|--------------------------|-----------------------------|--------|
| Short-term familiarity | High | Medium | Medium |
| Precision in explorations | Low | High | High |
| Surface churn | Low | Medium | Low-to-medium |
| Semantic clarity | Low | High | High |
| Risk of future confusion | High | Low | Medium |

## Possible Vocabulary Directions

### Direction 1: Minimal Clarification, Same Surface Terms

Keep current surface forms but tighten prose consistently across specs and explorations.

**Pros:**
- Low disruption
- Preserves existing syntax
- Good first step before larger design moves

**Cons:**
- The same word still does too much work
- Readers must infer distinctions from context

### Direction 2: Split Prose Vocabulary, Preserve Surface Syntax

Keep the syntax mostly as-is, but adopt a stricter writing discipline:

- `capability declaration`
- `capability identity`
- `capability witness`
- `provider`
- `effect classification`
- `policy context`
- `obligation context`
- `provenance context`

**Pros:**
- Best tradeoff for near-term clarity
- Helps future explorations immediately
- Does not require syntax commitment

**Cons:**
- Adds terminology overhead
- May reveal deeper inconsistencies that later need real design work

### Direction 3: Future Surface Differentiation

Eventually expose some distinctions more directly in source language or docs structure.

Possible examples:

- a more explicit term than `cap` for boundary witnesses
- clearer syntax/documentation separation between declaration and use
- separate vocabulary for provider registration versus source-level capability contracts

**Pros:**
- Maximum long-term precision
- Could improve onboarding and diagnostics

**Cons:**
- Too early to commit
- Risks syntax churn before semantics are settled

## Current Leaning

The current best move looks like Direction 2.

Ash does not need a major syntax intervention yet. It does need better language. A more precise
prose vocabulary would make subsequent work on ad-hoc polymorphism, first-class workflows,
provider contracts, and effect integration much easier.

In particular, future notes should try to say:

- "capability declaration" when discussing source contracts
- "capability witness" when discussing `cap C` at workflow boundaries
- "provider" when discussing host-side implementation
- "effect" only when discussing computational classification

## Open Questions

1. Should `C` in the judgment be described primarily as an authority context, a capability
   environment, or something even more precise?
2. Is `cap C` the right long-term surface notation for a capability witness?
3. Which distinctions belong in normative specs, and which belong only in explanatory prose?
4. Should provider-level `effect()` remain deliberately coarse, or should future work relate it
   more explicitly to source-level effect reasoning?
5. How should future interface/typeclass explorations talk about capabilities without suggesting
   that capability witnesses are method dictionaries?
6. Is there a better umbrella term than "capability" for some of these facets?

## Related Explorations

- [TYPES-002 V2](TYPES-002-ad-hoc-polymorphism-v2.md)
- [MCE-003](../minimal-core/MCE-003-FUNCTIONS-VS-CAPS.md)
- [FIRST-CLASS-WORKFLOWS](../future/FIRST-CLASS-WORKFLOWS.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-01 | Exploration created | Ash needs sharper language for capability/effect discussions before more type-system exploration proceeds |
| 2026-04-01 | Judgment-first framing chosen | `Γ, C, P, Ω, π ⊢wf w ⇓ out` is the clearest current anchor for separating concerns |

## Next Steps

- [ ] Revisit capability prose in SPEC-017 and related notes using this vocabulary
- [ ] Check whether SPEC-003, SPEC-004, and SPEC-010 use "capability" and "effect" consistently
- [ ] Use this terminology in future revisions of TYPES-002 and MCE-003
- [ ] Decide whether a follow-on design note should focus specifically on the meaning of `C` in the workflow judgment
