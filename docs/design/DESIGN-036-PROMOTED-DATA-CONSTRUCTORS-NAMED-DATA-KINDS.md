# DESIGN-036: Promoted Data Constructors and Named Data Kinds

**Status:** Promoted to [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md) / [PLAN-114](../plan/PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
**Date:** 2026-05-14
**Origin:** [DESIGN-034 §16.9](DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly), [TASK-887](../plan/tasks/TASK-887-promoted-data-constructors-and-named-data-kinds-packet.md)
**Related:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)

## Summary

Ash already has two separate notions that can be confused if the next step is not explicit:

1. runtime algebraic data constructors from ordinary `type` declarations, owned by SPEC-020/SPEC-057;
2. sealed type-level marker constructors from SPEC-059, used by type functions, normal forms, associated families, and propositions.

This design chooses a conservative first packet: add an explicit promoted-constructor identity layer, rather than silently treating every runtime constructor as a type-level constructor. Promotion is opt-in and summary-backed. Existing sealed-domain marker constructors remain type-level-only, and existing runtime ADTs remain runtime constructors until a declaration opts into promotion.

## Core decisions

- Promoted constructors are distinct identities from runtime `ConstructorId` values even when derived from the same source constructor.
- Promotion must preserve a back-reference to the source ADT constructor for diagnostics and non-interference checks.
- Promoted constructor applications are type-level terms with checked kinds/domains; they are not ordinary nominal `Type::Constructor` applications.
- The MVP supports promotion of non-GADT ordinary ADT constructors only; dependent value promotion, singleton reflection, and term-level proof values are out of scope.
- Sealed-domain marker constructors are not retroactively reclassified as promoted runtime constructors.

## Why not reuse sealed domains

Sealed domains model closed type-level data first. DataKinds-style promotion models a source relationship between a runtime ADT and a type-level constructor family. Those relationships need different summaries, visibility rules, diagnostics, and pattern/exhaustiveness interaction.

## Implementation strategy

SPEC-065 should start with an audit of live ADT constructor metadata, semantic-summary transport, type-function RHS carriers, normal-form/domain-constructor carriers, and pattern/exhaustiveness callsites. The first implementation slice must prove non-interference: runtime construction and matching keep using runtime constructors, while promoted applications are accepted only in explicitly enabled type-level contexts.


## Non-goals

- Do not reopen SPEC-057 through SPEC-064 substrate decisions unless the new spec explicitly names an incompatibility.
- Do not encode type-level computation as ordinary `Type::Constructor` nodes.
- Do not broaden `do` notation or pattern behavior before the owning implementation phase lands.

## Handoff

The normative implementation contract is in [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). The task order is in [PLAN-114](../plan/PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md).
