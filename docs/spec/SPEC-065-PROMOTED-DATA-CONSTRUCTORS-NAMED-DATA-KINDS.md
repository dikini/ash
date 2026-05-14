# SPEC-065: Promoted Data Constructors and Named Data Kinds

**Status:** Draft
**Date:** 2026-05-14
**Promotes:** [DESIGN-036](../design/DESIGN-036-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
**Origin:** [TASK-887](../plan/tasks/TASK-887-promoted-data-constructors-and-named-data-kinds-packet.md)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md), [SPEC-062](SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md), [SPEC-064](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
**Related:** [SPEC-020](SPEC-020-ADT-TYPES.md), [SPEC-003](SPEC-003-TYPE-SYSTEM.md)
**Plan:** [PLAN-114](../plan/PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
**Implementation Tasks:** [TASK-892](../plan/tasks/TASK-892-promoted-constructor-audit-gate.md) through [TASK-897](../plan/tasks/TASK-897-promoted-constructor-closeout.md)

## 1. Summary

SPEC-065 defines an opt-in promoted-constructor layer for using ordinary ADT constructors as type-level constructors and for naming data kinds derived from ADT shapes. It fills the DESIGN-034 gap that was intentionally left outside sealed-domain marker constructors.

The MVP adds explicit promotion metadata and canonical promoted-constructor applications. It does not silently treat every runtime constructor as type-level data, and it does not reuse sealed-domain marker constructors as runtime ADT constructors.

## 2. Baseline

Live substrate before this spec:

- ordinary ADTs are parsed/lowered/exported by SPEC-020 and SPEC-057;
- sealed domains from SPEC-059 have marker constructors that are type-level only;
- normal forms from SPEC-060 can carry domain constructor applications;
- type functions and propositions can consume closed type-level constructor metadata;
- no source rule says that an ADT constructor is automatically promoted.

## 3. Scope

In scope:

1. opt-in source syntax for promoted data kinds or promoted ADT constructor sets;
2. core identities for promoted data kinds and promoted constructors;
3. canonical promoted-constructor applications distinct from ordinary nominal type applications;
4. semantic-summary transport for public promoted identities;
5. TypeEnv registration, kind/domain validation, and normalizer term construction;
6. integration with type-function RHSs and proposition operands;
7. non-interference with runtime ADT construction, runtime pattern matching, and sealed-domain marker constructors.

Out of scope:

- automatic global DataKinds promotion for every ADT;
- term-level singleton reflection, dependent pattern matching, or proof values;
- promotion of arbitrary runtime values;
- GADT-style constructors or equality constraints introduced by constructors;
- changing ordinary ADT runtime layout or matching semantics.

## 4. Source Model

The implementation phase must choose exact syntax in TASK-892 before parser work begins. The preferred MVP is an explicit attribute or declaration rather than implicit promotion.

Acceptable design shapes include:

```ash
@promote
pub type Nat = Z | S(Nat);
```

or:

```ash
pub data kind Nat from type Nat;
```

The chosen syntax must state:

- whether the promoted kind name is the ADT type name or a separate name;
- whether every constructor is promoted or only an explicit subset;
- how constructor visibility follows or overrides runtime constructor visibility;
- whether recursive ADTs can be promoted in the first slice.

## 5. Identity and IR

`ash-core` owns promoted identities when they cross crate/module/cache boundaries.

Required logical carriers:

```rust
pub struct PromotedDataKindId { /* module + source ADT + promoted name */ }
pub struct PromotedConstructorId { /* kind + source constructor + promoted name */ }

pub enum TypeLevelConstructorApp {
    SealedDomainConstructor(DomainConstructorApp),
    PromotedDataConstructor(PromotedConstructorApp),
}
```

The exact Rust names may differ, but implementations must preserve the distinction between:

- runtime ADT constructor identity;
- sealed-domain marker constructor identity;
- promoted data constructor identity.

## 6. Kinding and Normalization

A promoted data kind is a closed type-level domain. A promoted constructor has an explicit kind derived from the source constructor payload shape and recursive positions.

Rules:

1. zero-argument constructors have kind equal to the promoted kind;
2. payload-bearing constructors have arrow kinds from promoted field kinds to the promoted kind;
3. fields whose source type is not promotable are rejected in the MVP;
4. recursive fields may be accepted only if the audit proves existing structural recursion checks can consume the promoted kind safely;
5. promoted constructor applications may appear in type-function RHSs and proposition operands only after TypeEnv validates the promoted kind.

## 7. Summary Transport

Public promoted identities require semantic-summary metadata:

- promoted kind identity;
- source ADT identity;
- promoted constructor identities in source order;
- constructor field kind/domain metadata;
- visibility and source anchors;
- version guard rejecting promoted facts in older summaries.

Engine remains transport-only. TypeEnv owns semantic validation.

## 8. Diagnostics

Required diagnostics:

- attempted promotion of an unsupported runtime field type;
- promoted constructor used as ordinary runtime constructor or vice versa;
- sealed-domain marker constructor confused with promoted data constructor;
- private promoted constructor leakage through public type functions or propositions;
- ambiguous promoted/runtime constructor name in a type-level context.

## 9. Acceptance Matrix

| ID | Case | Expected result |
|----|------|-----------------|
| PDC-1 | promoted zero-argument constructor in a type-function RHS | accepted and normalized as promoted constructor |
| PDC-2 | promoted recursive constructor with explicit supported recursive field | accepted only if audit enables recursive promotion |
| PDC-3 | ordinary runtime constructor in type position without promotion | rejected with promotion-required diagnostic |
| PDC-4 | sealed-domain marker treated as runtime ADT constructor | rejected |
| PDC-5 | public type function leaks private promoted constructor | rejected before summary export |
| PDC-6 | existing runtime ADT construction/pattern tests | unchanged |

## 10. Implementation Tasks

See [PLAN-114](../plan/PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md).
