# TASK-2074 Parenthesized Notation Import Design

**Status:** Approved design; not implementation or evidence
**Owner:** TASK-2074 — parser-owned syntax prepass
**Authority:** SPEC-103 §5, SPEC-095c §§7–10, and TASK-2071

## Decision

Extend the AST-only syntax prepass with explicit notation-import forms:

```ash
use crate::math::(<*>);
use crate::ranges::(_ between _ and _);
```

The parenthesized payload names one exact normalized token/hole pattern, rather than a callable;
it does not encode fixity, precedence, or associativity. `<*>` selects summaries with that token
pattern, while `_ between _ and _` selects summaries with that hole pattern. All public canonical
summaries for the selected pattern are transported with their full keys: normalized pattern,
fixity, associativity, and precedence. Consumer use-site context and notation-overlap rules select
a compatible variant or reject deterministically. Holes remain holes in the parsed pattern, and a
use of the activated mixfix binds its expression arguments left-to-right. Thus `x between lo and
hi` expands through the selected summary to its target with arguments `(x, lo, hi)`.

The provider supplies a public canonical notation summary. The prepass transports the summary's
normalized pattern, fixity, associativity, precedence, target identity, declaration provenance,
provider `ModuleKey`, visibility, and importing `Use::span`. The consumer's active notation table
uses those syntax facts only. The import does not bind or import the target callable's name.

## Boundary

Notation imports are syntax-phase dependencies only. They resolve from parsed AST summaries through
canonical module keys, participate in deterministic provider-before-consumer ordering, and fail the
whole expanded graph on a missing, private, conflicting, malformed, or cyclic dependency.

An ordinary callable import never activates notation, and importing notation never grants callable,
type, Core/CPS, runtime, admission, or Engine authority. The target callable's public reachability
and export closure remain separately checked; this prepass transports no proof that it is executable
or admitted. It reads no filesystem, raw source text, path cache, or direct evaluator.

There is no `as` spelling for notation imports: the normalized token/hole pattern is the selector.
An import transports every eligible public full-key variant for that selector. Local/imported
notation collisions are resolved or rejected deterministically by the existing notation-overlap
rules, not by silently choosing a target callable. Visibility failure reports the provider
declaration and consumer use anchors; missing and malformed patterns report the consumer use anchor;
cycles retain ordered provider/importer edges and their use spans.

## Rejected alternatives

- **Import the target function name.** This would turn syntax activation into callable binding and
  would lose the pattern/fixity/precedence identity required to resolve notation safely.
- **Module-wide notation glob.** This makes conflict detection and provenance ambiguous, broadens
  the activated syntax set and scope, and makes future re-export behaviour unclear.

## Testing and error plan

Focused parser RED tests should cover:

1. Public notation imports transporting every full-key variant for the selected normalized
   token/hole pattern, preserving pattern holes, left-to-right argument order,
   fixity/associativity/precedence, target identity, and provider/use provenance.
2. Exact-pattern selection: compatible prefix/infix variants sharing one pattern are selected by
   consumer use-site context; an overlapping variant rejects deterministically. A callable-only
   import with the same target does not activate notation.
3. Missing, private, malformed, and local/imported-conflicting forms, each with deterministic
   declaration/use diagnostics; reject every `as` form.
4. Two- and three-module syntax cycles containing notation dependencies, proving deterministic
   cycle edges and graph-wide atomic failure.
5. An authority fence over the dedicated prepass sources: no filesystem/raw-text/Engine/direct
   evaluation dependency, no callable-binding carrier, and no runtime/admission authority.

This is parser-stage expansion evidence only. File/inline expanded projection parity remains a
separate TASK-2074 obligation, and Type/runtime/client parity remains owned outside this design.
