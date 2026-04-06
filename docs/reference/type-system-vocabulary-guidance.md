# Type-System Vocabulary Guidance

## Status

Canonical prose guidance derived from `docs/ideas/type-system/TYPES-003-capabilities-effects-vocabulary.md`.

## Purpose

This document standardizes how the Ash docs corpus should talk about capabilities, effects,
authority, providers, obligations, provenance, and related type-system/runtime concepts.

It is a prose-guidance document, not a syntax-spec document. Its goal is to reduce ambiguity across
`SPEC-003`, `SPEC-004`, `SPEC-010`, `SPEC-017`, `SPEC-018`, `SPEC-019`, `SPEC-020`, planning docs,
and future type-system/interface explorations.

## Core Rule

Do not use the single word `capability` as if it meant all of the following at once.

Ash documentation should distinguish at least these facets:

1. capability declaration
2. capability identity
3. capability witness
4. provider
5. effect classification
6. policy context
7. obligation context
8. provenance context

## Canonical Terms

### Capability declaration

Use when referring to the source-level contract introduced by the `capability` keyword.

Example:
- "`Args` is a capability declaration."

Avoid:
- "the provider `Args`" when you mean the source declaration
- "the capability value `Args`" when you mean the declaration itself

### Capability identity

Use when referring to the resolved capability name used at invocation sites.

Example:
- "`observe Args 0` names the capability identity `Args` explicitly."

Avoid:
- describing the capability identity as an object receiver or method table

### Capability witness

Use when referring to a usage-site capability value such as `cap Args` at a workflow boundary.

Example:
- "`args: cap Args` binds a capability witness for the workflow boundary."

Canonical reminder:
- a capability witness is an authorization witness, not a method receiver and not a trait object by default

### Provider

Use when referring to the embedding/runtime implementation of a capability contract.

Example:
- "the runtime must register a provider that implements the `Args` capability declaration"

Avoid:
- equating provider and capability declaration

### Effect classification

Use when referring to Ash's computational effect layer (`Epistemic`, `Deliberative`,
`Evaluative`, `Operational`, and any future normalized bottom such as `Pure` if adopted).

In the current promoted contract, effect classification is computed from workflow forms and
source-level contracts. Embedding-side provider effect metadata is secondary compatibility and
validation metadata, not the primary source of source-level effect typing.

Example:
- "using this capability in this form incurs an epistemic effect classification"

Avoid:
- saying "the capability is epistemic" when you mean the use of the capability is epistemic

### Policy context

Use when referring to the governing policy environment that constrains or evaluates execution.

### Obligation context

Use when referring to duties introduced, discharged, or left pending.

Avoid:
- equating obligations with permissions or capability grants

### Provenance context

Use when referring to origin, lineage, and audit-trace concerns.

Avoid:
- collapsing provenance into effect classification or generic trace language

## Canonical Distinctions

### Capabilities are not effects

Preferred wording:
- "A capability use may incur an effect classification."
- "A capability declaration constrains governed operations; the effect system classifies computation."

Avoid:
- "A capability is an effect"
- "Providers define effect typing"

### Providers are not declarations

Preferred wording:
- "The provider implements the declared capability contract."

Avoid:
- "The capability provider is the capability"

### Capability witnesses are not interface dictionaries

Preferred wording:
- "`cap C` is a boundary authorization witness."

Avoid:
- "`cap C` is a receiver"
- "`cap C` is a method-dispatch object"

This distinction is especially important when discussing ad-hoc polymorphism or closed-world
interfaces.

## Writing Guidance for Existing Specs

### SPEC-017

Prefer:
- capability declaration
- capability identity
- capability witness
- provider

Do not let `cap C` read like a second declaration form or like object-style method dispatch.

### SPEC-003 and SPEC-004

Prefer:
- effect classification
- workflow effect grade
- policy context
- obligation context
- provenance context

Do not blur the semantic judgment dimensions into one overloaded type-system narrative.

### SPEC-010 and embedding/runtime docs

Prefer:
- provider
- provider metadata
- provider effect metadata

Be explicit that embedding-side provider metadata is not identical to source-level effect typing.

## Short Glossary Table

| Term | Use for | Do not use for |
|------|---------|----------------|
| capability declaration | source-level `capability` contract | runtime provider instance |
| capability identity | resolved invoked capability name | witness value |
| capability witness | usage-site `cap C` boundary authorization | trait object / method receiver |
| provider | runtime implementation of a capability contract | source declaration |
| effect classification | computation grade | capability identity |
| policy context | governing rules | capability declaration |
| obligation context | duties and discharge state | authority grant |
| provenance context | origin and lineage | generic effect label |

## Recommended Cleanup Targets

Near-term cleanup should use this guidance in:

- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/spec/SPEC-010-EMBEDDING.md`
- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- `docs/reference/type-to-runtime-contract.md`
- future interface/ad-hoc-polymorphism docs derived from `TYPES-002 V2`

## Relationship to Explorations

This guidance is promoted from:

- `docs/ideas/type-system/TYPES-003-capabilities-effects-vocabulary.md`

The narrow current effect-typing contract promoted from `TYPES-004` is recorded in:

- `docs/reference/type-to-runtime-contract.md`

That exploration remains the reasoning record. This file is the reusable cleanup guidance.
