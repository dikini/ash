# TASK-2078: Notional Existential Types Exploration Note

**Status:** Complete
**Track:** Incubating Future Type-System Explorations
**Semantic task classification:** non-semantic-workflow-enforcement

**Task nature:** exploratory documentation only; no semantic rule or implementation change

## Description

Capture the notional existential-type design explored alongside Ash's SPEC-104-frozen rank-1
generics, minimal coherent interfaces and associated types, nominal effects, ordinary structural
modules, and opaque nominal marker types. The note is a future-direction aid only: it records a
possible compositional route to scoped abstract types and generative type identities without
adding SML signatures, SML functors, or first-class modules to Ash.

The note must keep proposed pseudo-Ash syntax visibly non-normative and must distinguish shared
declaration shapes from shared semantics. In particular, similarities among Ash interfaces, Ash
effect declarations, and notional existential schemas do not authorize collapsing their typing,
selection, dispatch, handling, row, continuation, admission, or runtime behavior.

## References

- **Lead current scope authority:**
  [SPEC-104: Language Scope Freeze](../../spec/SPEC-104-LANGUAGE-SCOPE-FREEZE.md). Existential
  types and generative type identities are not selected by its P1, P2, or P3+ sets. Promotion
  therefore requires an explicit SPEC-104 amendment before a target spec or implementation task.
- **Current module authority:**
  [SPEC-103: Module Realization and Operational Semantics](../../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md).
  It supplies stable canonical module identity and conventional structural modules, and excludes
  user-visible runtime module values, dynamic imports, and first-class module computation.
- **Current associated-type and projection substrate:**
  [SPEC-035: Associated Types](../../spec/SPEC-035-ASSOCIATED-TYPES.md) owns the current
  interface-associated surface and selected-implementation compatibility rule, while
  [SPEC-058: Canonical Type-Expression IR](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
  owns canonical projection identities and rigid IR plumbing. Neither authorizes existential
  syntax or general type normalization.
- [NOTE-INDEX](../../notes/NOTE-INDEX.md)
- **Related exploratory context:**
  [NOTE-026: Newtype and Phantom Types](../../notes/NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md). It is
  useful comparative material, but SPEC-104 controls which nominal/phantom facilities are current.
- **Historical/pre-freeze conflict context only:**
  [NOTE-022](../../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md),
  [NOTE-023](../../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md),
  [NOTE-025](../../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md),
  [SPEC-095b](../../spec/SPEC-095b-TARGET-GRAMMAR.md),
  [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md), and
  [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md). These documents may explain design
  history or detailed retained rules only where they do not conflict with SPEC-104; they do not
  authorize the notional forms in this task.

## Requirements

1. Lead with the authority boundary: SPEC-104 controls feature inclusion and phase. State that
   existential types and generative type identities are absent from its P1, P2, and P3+ sets, so
   the note neither reserves syntax nor creates representation compatibility, and any promotion
   first requires a SPEC-104 amendment.
2. Add one note under `docs/notes/` with explicit metadata and status identifying it as a
   non-normative, exploratory future-direction document.
3. Present `exists` as a notional ordinary type-expression constructor, with explicit type
   binders and block-scoped `pack`/`unpack` examples. Explain that unpacking introduces a local
   rigid abstract type name and does not reveal its representation equation.
4. Cover nesting and composition of already-defined existential type expressions, including the
   binder-aware composition problem, hygienic alpha-renaming, member namespaces, and explicit
   bridge requirements that the compiler cannot invent.
5. Separate the following dimensions and show how they may compose without implicit coupling:
   universal versus existential quantification; schema shape; abstract/associated type members;
   equation transparency or opacity; stable/applicative versus fresh/generative identity;
   nominal and phantom markers; and module-name visibility.
6. Compare the possible schema-level resemblance among Ash interfaces, Ash effect declarations,
   and notional existential schemas while preserving their distinct meanings and ownership rules.
   The current SPEC-104 baseline keeps interfaces and effects distinct: interfaces may declare
   associated types and methods, while effects contain bodyless operation signatures only and do
   not have associated type members. Do not present a shared internal schema substrate as current
   syntax, semantics, or implementation.
7. Keep provider concepts outside existential evidence/package examples. Under SPEC-104, provider
   declarations, manifest recipe selections, and Engine-authorized provider bindings are distinct
   and none is a first-class Ash value. Existential packaging must not imply a provider value,
   provider selection, installed binding, handler frame, or authority grant.
8. Include the practical phantom/generative-witness examples discussed: currency-safe arithmetic,
   explicit currency conversion, typestate or construction-history witnesses, and session-local
   brands that prevent values from different runtime sessions from being mixed. Label these as
   notional illustrations rather than claims about current phantom/generative surface support.
9. Explain the relationship to SML applicative and generative functors as inspiration, while
   retaining SPEC-103's stable conventional structural modules and exclusion of user-visible
   runtime module values rather than importing an SML module calculus. Include the
   closure-conversion/defunctionalization analogy as context, not semantics.
10. Record soundness constraints and open questions honestly, especially existential escape,
   runtime calls versus static fresh identities, handler/effect type stability, evidence and
   authority separation, and whether anonymous schemas or binder-aware composition syntax earn
   their complexity.
11. Update `docs/notes/NOTE-INDEX.md` in the same change with a type-system read path, document-table
   row, status, role, tags, and useful related-document links.
12. Add a Common Changelog entry under `[Unreleased]` for the exploratory note.

## TDD / Verification Steps

1. **RED — orientation absence:** verify that `NOTE-INDEX.md` has no current read path or document
   row for the notional existential design and that no existing note already owns the full scope.
2. **GREEN — minimal documentation:** add only the exploratory note, its orientation records, and
   the matching changelog entry. Keep every proposed form labelled non-normative.
3. **Boundary review:** search the new material for claims that would make `exists`, `pack`,
   `unpack`, generativity, sealing, or schema composition part of current Ash. Rewrite any such
   claim as a possibility, question, or future promotion requirement, and verify that promotion
   is explicitly gated on amending SPEC-104.
4. **Distinct-semantics review:** verify that interface evidence selection, effect raising and
   handling, existential packaging, and authority/admission are described separately even where
   their declarations have superficial operation-signature similarities. Verify specifically that
   the note does not give current effects associated type members or make provider declarations,
   recipe selections, or bindings first-class.
5. **Orientation validation:** run
   `python3 tools/docs/validate_orientation_indexes.py --self-test`.
6. **Documentation gate:** run `bash scripts/check-docs-gate.sh`.
7. **Diff hygiene:** run `git diff --check` and inspect the final diff for scope creep.

## Completion Checklist

- [x] The non-normative existential exploration note exists and carries explicit authority/status
      metadata.
- [x] `exists`, `pack`, `unpack`, nesting, composition, and non-escape are illustrated with
      clearly labelled pseudo-Ash.
- [x] Independent design dimensions are identified without assigning one construct hidden side
      effects in another dimension.
- [x] Interface, effect, and existential similarities and distinctions are recorded accurately.
- [x] SPEC-104 leads the authority statement; SPEC-103 supplies the stable-module boundary; older
      notes/specs are labelled historical/pre-freeze conflict context rather than current
      authority.
- [x] SML functor inspiration, phantom/generative witness use cases, and the
      closure-conversion/defunctionalization analogy are bounded appropriately.
- [x] Open questions and promotion prerequisites are explicit.
- [x] `NOTE-INDEX.md` and `CHANGELOG.md` are updated.
- [x] Orientation-index validation, the documentation gate, and diff hygiene pass.

## Completion Evidence

- Independent authority review found no blocking authority or soundness issue and confirmed the
  SPEC-104 amendment gate, SPEC-103 module boundary, interface/effect/existential separation,
  skolem-versus-generative distinction, and authority non-interference.
- `python3 tools/docs/validate_orientation_indexes.py --self-test` passed.
- `bash scripts/check-docs-gate.sh` passed with no missing Markdown links or semantic-traceability
  errors.
- `git diff --check` passed.

## Non-goals

- No Rust or Ash implementation, parser/AST/checker/lowering/Core/CPS/runtime change, test fixture,
  or public API addition.
- No target-spec amendment, accepted grammar, semantic rule, proof claim, or semantic-rule coverage
  update.
- No placement of existentials or generativity into P1, P2, or P3+; promotion is impossible without
  a separate SPEC-104 amendment.
- No decision to add existential types, SML signatures, SML applicative or generative functors,
  first-class Ash modules, anonymous schemas, higher-rank types, or a general type-level
  programming language.
- No reinterpretation or unification of Ash interfaces and Ash effects; their present separation
  remains intact, and effects acquire no associated type members.
- No first-class provider declaration, provider recipe selection, provider binding, handler, or
  runtime module value.
- No claim that a phantom marker grants runtime authority or that existential packaging installs an
  effect handler, provider, capability, admission decision, or runtime frame.
- No requirement that current simplification, module realization, effect/interface separation, or
  runnable semantic-realization work accommodate this exploration now.

## Current-work non-interference

This task is deliberately off the implementation critical path. The note may preserve future design
space and identify questions, but it must not add acceptance criteria, dependencies, syntax
reservations, or implementation obligations to an active or completed Ash phase. Any promotion
requires a separate target-spec decision, plan/task packet, semantic-rule coverage update where
applicable, TDD implementation, conformance evidence, and ordinary review gates.
