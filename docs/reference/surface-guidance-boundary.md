# Surface Guidance Boundary

## Status

TASK-194 boundary note.

## Purpose

This note defines what human-facing workflow guidance belongs in surface-language documentation for:

- advisory stages
- gated stages
- committed stages

It exists to prevent surface-language documentation from drifting into premature syntax design or
from overloading runtime-only constructs such as monitor views, `exposes`, or workflow
observability.

This note does not add new surface syntax.
It does not redefine the canonical workflow grammar.
It defines the documentation boundary that later `SPEC-002` edits must respect.

## Decision

At this stage, the required human-facing stage guidance is **explanatory only**, not normative.

That means:

- the guidance may explain how humans should read workflow phases such as advisory, gated, and
  committed work
- the guidance may explain how these phases relate to runtime authority and reasoner participation
- the guidance must not introduce new required syntax, annotations, or grammar forms unless a later
  separate task explicitly opens that design space

The current need is documentation clarity, not new language surface.

## Why the Boundary Is Explanatory First

The follow-up review established:

- runtime-only constructs are already semantically stable
- the interaction contract now exists as a separate reference
- terminology around projection and monitorability has been tightened

What remains is helping humans understand the workflow at a larger-grain stage level.
That is a presentation and reading concern first.

If surface syntax is changed too early, the project risks:

- encoding unstable stage concepts directly in grammar,
- conflating human guidance with canonical operational meaning,
- and reusing existing runtime-only features for unrelated interaction-layer purposes.

So the correct next move is explanatory guidance first, syntax later only if still needed.

## What Belongs in Human-Facing Surface Guidance

Later `SPEC-002` documentation may explain:

- that some workflow forms are typically advisory in role, such as `orient` and `propose`
- that some workflow forms are runtime-owned gates, such as `decide` and `check`
- that some workflow forms are runtime-owned commitment points, such as `act`
- that workflow authors may think in larger human-facing stages even when the canonical language is
  expressed in finer-grained workflow forms
- that runtime visibility and monitorability are separate from reasoner projection

This guidance should help readers answer:

- what kind of work is happening here?
- who owns authority at this point?
- is this step interpretive, gated, or committed?

## What Does Not Belong in This Guidance

The following do not belong in the explanatory stage-guidance layer:

- new required syntax for stages
- new grammar productions or keywords
- implicit reinterpretation of `exposes`
- implicit reinterpretation of monitor views as reasoner projection
- changes to the canonical semantics of `observe`, `orient`, `propose`, `decide`, `check`, or `act`

The stage-guidance layer explains the existing language. It does not silently extend it.

## Protected Runtime-Only Constructs

The following remain outside surface stage guidance except as explicit examples of what not to
conflate:

- monitor views
- `exposes` clauses
- workflow observability
- `MonitorLink`
- runtime tracing
- capability verification

These are runtime-facing visibility or enforcement constructs, not human-facing stage markers.

## Recommended Placement in SPEC-002

When `SPEC-002` is revised later, the preferred placement is:

- a short explanatory subsection near the overview or workflow-definition discussion

That subsection should:

- explain how readers may interpret workflow forms as advisory, gated, or committed stages
- explicitly state that this is reading guidance, not additional grammar
- defer canonical meaning to `SPEC-001`, `SPEC-004`, and the runtime-to-reasoner interaction
  contract where appropriate

The grammar sections themselves should remain grammar-focused.

## Relationship to Other Documents

This note depends on:

- [Runtime-to-Reasoner Interaction Contract](runtime-to-reasoner-interaction-contract.md)
- [Runtime-Reasoner Separation Rules](runtime-reasoner-separation-rules.md)
- [Ash Language Terminology Guide](../design/LANGUAGE-TERMINOLOGY.md)
- [SPEC-002: Surface Language](../spec/SPEC-002-SURFACE.md)

It should be used before editing `SPEC-002` for human-facing workflow-stage explanation.

## Reopen Conditions

The boundary should be revisited only if a later task concludes that explanatory guidance is not
enough and one of the following is truly needed:

- syntax-level stage annotations
- workflow-state grouping syntax
- explicit runtime/reasoner placement markers in the surface language

If any of those are proposed, they must be handled by a separate design task rather than by
silently extending this note.
