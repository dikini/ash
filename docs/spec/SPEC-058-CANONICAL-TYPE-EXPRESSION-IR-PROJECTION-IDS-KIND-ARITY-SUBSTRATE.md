# SPEC-058: Canonical Type-Expression IR, Projection Identities, and Kind/Arity Substrate

**Status:** Draft
**Date:** 2026-05-02
**Promotes:** [DESIGN-034 §16.2](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Builds on:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-034](SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
**Related:** [SPEC-030](SPEC-030-MODULE-TYPE-RESOLUTION.md), [SPEC-042](SPEC-042-ASH-SOURCE-FORMATTER.md)
**Plan:** [PLAN-106](../plan/PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
**Implementation Tasks:** [TASK-793](../plan/tasks/TASK-793-spec-b-spec-plan-packet.md) through [TASK-805](../plan/tasks/TASK-805-phase110-review-remediation.md)

## 1. Summary

SPEC-058 is SPEC-B from DESIGN-034. It defines the internal type-expression substrate required before Ash can honestly implement sealed type-level domains, normalization, public `type fn` syntax, cross-module type-computation summaries, or recursive associated type-family computation.

The required end state is:

```text
surface/core type syntax
  -> canonical ash-core type-expression IR
  -> canonical nominal/origin identities
  -> canonical projection identities
  -> kind/arity validation
  -> rigid/neutral carriers suitable for later normalization/equality
```

This phase is intentionally substrate-only. It does not expose user-facing `type fn` syntax, sealed domains, generalized projection spellings, source-level kind binders, holes, partial type-constructor application, or module-summary export/import for computation facts.

## 2. Motivation

DESIGN-034 requires Ash to represent reducible type computation honestly. After Phase 109, ordinary type declarations already flow through `ModuleFile`, `ash-core` summary carriers, engine transport, and `TypeEnv`. That roadbed is necessary, but insufficient for total type computation.

The remaining blockers are structural:

1. `ash-typeck` still uses a stringly `Type::Associated { interface: String, base, name }` shape, including empty-string sentinels for unresolved projections.
2. the live parser/core/type carriers are still shaped around ordinary nominal types rather than a canonical computation-capable IR;
3. kind/arity checking exists only as a partial foothold and is not yet the explicit gate for all future computation heads;
4. the repo contains reserved interface/associated-member identity carriers from SPEC-057, but SPEC-057 intentionally leaves them uninterpreted for computation semantics.

SPEC-058 resolves those substrate gaps without pretending that later packets already exist.

## 3. Scope

In scope for SPEC-058:

- a shared canonical type-expression IR in `ash-core` for nominal applications, canonical projections, and future computation heads;
- promotion of computation-relevant canonical identities built on top of the Phase 109 identity substrate;
- an explicit distinction between nominal constructor application and computation-head application;
- canonical projection identity carriers that replace stringly interface-name matching;
- rigid/neutral carriers and a normal-form view suitable for later packets to consume;
- explicit kind/arity validation for nominal constructors, projections, and future computation heads;
- transparent-alias canonicalization policy and helper boundaries;
- elaboration from current surface/core associated projection syntax into canonical projection IR;
- diagnostics for ambiguous projections, unsupported shapes, and kind/arity failures;
- parser parity work only where required to keep the ordinary type-definition parser (`parse_type_def.rs`) and the surface/module type parser (`parse_module.rs`) aligned on the current supported subset and explicit rejection boundaries.

Out of scope:

- sealed type-level domains, marker constructors, or coverage checking;
- definitional equality, normalization, normalize-and-compare, or forcing-point rollout;
- public `type fn` syntax, equation checking, overlap, coverage, termination, or recursion analysis;
- cross-module export/import of computation-grade summaries;
- recursive associated type-family computation;
- proposition/disequality layers or proof search;
- new public projection spellings beyond the existing `base::Assoc` family already accepted by current specs;
- general user-facing kind annotation syntax, type holes, or partial type-constructor application.

## 4. Implementation Baseline

SPEC-058 assumes the following live substrate:

1. [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) already owns canonical `ModuleIdentity`, `TypeDeclId`, constructor identities, and reserved interface/associated-member identity slots.
2. [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md) already defines the current simple associated-type surface (`S::Ok`, `Map<K, V>::Entry`) and selected-impl substitution behavior.
3. [SPEC-003](SPEC-003-TYPE-SYSTEM.md) already defines `Kind::Type` and `Kind::Arrow`, and current nominal constructor applications carry a kind field.
4. current formatter/spec surfaces still treat `base::Assoc` as the canonical rendered associated projection spelling.

SPEC-058 refines these documents by introducing canonical internal representations and boundaries. It does not erase the current user-facing projection syntax or reinterpret Phase 109 summary transport as computation export/import.

## 5. Contradictions and Resolution Policy

SPEC-058 must not contradict existing language features. The live tensions are:

| Current contract | Tension against DESIGN-034 SPEC-B | SPEC-058 resolution |
|---|---|---|
| `ash-typeck::Type::Associated { interface: String, base, name }` and empty-string unresolved interface handling | Not canonical, unary/base-shaped, and unsuitable for later equality/normalization | Keep current surface syntax, but elaborate it into canonical projection identities backed by Phase 109 identity carriers. Empty-string sentinel state is replaced by explicit unresolved/ambiguous/resolved projection states. |
| SPEC-057 reserved interface/associated-member identity slots are explicitly uninterpreted for computation semantics | Later packets need canonical projection/member identity now | SPEC-058 formally promotes those identities for internal IR purposes only. Phase 109's ordinary-type identity contract remains unchanged. |
| Current surface/core projection syntax is `base::Assoc` | DESIGN-034 sketches more explicit generalized projection spellings | SPEC-058 keeps `base::Assoc` as the only normative user-visible spelling in this packet. Alternative spellings are deferred. |
| Existing kind support is partial and mostly implicit | DESIGN-034 wants general kind/arity substrate | SPEC-058 widens internal kind/arity validation but does not introduce new public kind syntax, holes, or partial applications. |
| Current associated-type normalization helper performs selected-impl substitution | Could be mistaken for the future normalizer | SPEC-058 treats the current helper as a compatibility path for simple associated outputs only. Full normalization remains deferred to SPEC-D / SPEC-G. |

### 5.1 Design authority

[DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) is the authoritative design source for this packet. SPEC-058 follows its ordered implementation sequence (§16.10) and its substrate constraints for honest type computation.

This packet intentionally does not depend on historical syntax-reduction notes as implementation authority. If later work introduces user-facing syntax in this area, that syntax must be grounded in the live parser/surface contracts at implementation time, because the real surface may have drifted since older design notes were written.

## 6. Required Invariants

1. `ash-core` owns the canonical computation-capable type-expression IR shared across crates.
2. Nominal constructor application and computation-head application are distinct internal forms.
3. Canonical projection equality keys use canonical interface/member identity plus an ordered argument spine, not string interface names.
4. Import aliases, re-export names, and diagnostic display names are never origin identities.
5. Transparent aliases canonicalize before the named current equality boundaries `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`, via `TypeEnv::canonicalize_type_for_equality`; Phase 110 does not require canonicalization rollout into pattern checking or exhaustiveness.
6. Wrong kind/arity is rejected before any later normalization/equality packet is allowed to consume the type expression.
7. Current simple associated-type behavior from SPEC-035 remains supported as a compatibility path.
8. This packet must not implement sealed domains, type functions, normalization, recursive associated-family computation, or cross-module computation summaries.

## 7. Canonical Identity Contract

SPEC-058 reuses and extends the Phase 109 identity substrate.

### 7.1 Nominal identities

`TypeDeclId` continues to be the canonical origin identity for ordinary nominal types. No new nominal-origin identity scheme is introduced in this packet.

### 7.2 Interface and associated-member identities

`InterfaceIdentityId` and `AssociatedMemberIdentityId` are promoted from reserved metadata into internal computation-grade identity carriers.

Promotion rules:

- the identity still originates in `ash-core`;
- promotion does not by itself expose new source syntax;
- promotion does not imply recursive associated-family computation;
- promoted identities may participate in canonical projection IR and rigid/neutral comparison keys.

These promoted identities must be available both for source-local declarations and through ordinary imported module summaries before canonical projection elaboration begins. This is ordinary identity plumbing, not computation-summary export/import.

### 7.3 Future computation-head identities

SPEC-058 requires a canonical identity concept for future type-computation heads (`TypeFnApp` or equivalent). The exact Rust name may vary, but the contract must distinguish:

- nominal heads backed by `TypeDeclId`;
- projection heads backed by canonical interface/member identities and argument spines;
- future explicit computation heads backed by a dedicated computation-head identity namespace.

Computation-head identities must not reuse `TypeDeclId` in a way that makes computation appear nominal.

## 8. Canonical Type-Expression IR

`ash-core` must define a canonical internal IR equivalent to the following schematic contract:

```text
CanonicalTypeExpr ::=
    Primitive
  | Var(TypeVar)
  | NominalApp {
        origin: TypeDeclId,
        visible_name: Name,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    }
  | Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
        rigidity: ProjectionRigidity,
        surface_anchor: ProjectionSurfaceAnchor,
    }
  | ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    }
```

This is a semantic contract, not a required exact Rust layout.

### 8.1 Nominal applications

Nominal applications remain the representation for ordinary ADTs, aliases, and builtin nominal type constructors.

They must continue to decompose under ordinary unification exactly as current nominal constructors do.

### 8.2 Computation-head applications

Computation-head applications represent future reducible or neutral computation forms. They must be distinct from nominal applications even before public `type fn` syntax exists.

SPEC-058 explicitly forbids encoding future type-function applications as ordinary `NominalApp` / `Type::Constructor` nodes.

### 8.3 Canonical projections

Canonical projections represent internal elaboration of current `base::Assoc` surface syntax.

They must carry:

- the declaring interface identity;
- the associated-member identity;
- the ordered interface argument spine;
- rigidity metadata;
- source/diagnostic anchor information sufficient to render current user-facing forms.

## 9. Projection Elaboration and Compatibility

### 9.1 Surface syntax preserved

The current projection spelling from SPEC-035 remains the only user-facing spelling in this packet:

```ash
S::Ok
Map<K, V>::Entry
```

No alternative public projection syntax becomes normative in SPEC-058.

### 9.2 Elaboration contract

Current surface/core projection forms are elaborated into canonical projections in the typechecker.

The elaboration must:

1. resolve the declaring interface/member when unique;
2. derive the ordered interface argument spine consumed by that interface;
3. produce a canonical projection IR node using promoted Phase 109 identities;
4. preserve enough source information to render current user-facing diagnostics.

For the currently supported SPEC-035 subset, unary `S::Assoc` elaborates to a one-argument projection spine. Multi-parameter projections use the interface argument order determined by the selected interface declaration and the current source/base shape.

### 9.3 Ambiguity and unresolved forms

If a projection name is ambiguous across in-scope interface bounds, elaboration must fail with a dedicated ambiguity diagnostic.

If the projection is not ambiguous but cannot reduce because only a generic/rigid form is available, elaboration must still produce a canonical rigid projection node rather than an empty-interface placeholder.

### 9.4 Compatibility boundary

Current selected-impl associated-type substitution remains allowed for the already-supported simple associated-output path, but SPEC-058 does not define that operation as general normalization.

## 10. Rigid / Neutral Carriers and Normal-Form View

SPEC-058 must define shared carriers sufficient for later packets to speak precisely about stuck computation without implementing reduction yet.

Required concepts:

- rigid projection forms;
- neutral computation-head applications;
- a normal-form view grammar or carrier shape that later equality/normalization packets can consume.

This packet does not define the normalization judgment. It only makes the relevant data model explicit and sharable. It also does not define comparison, decomposition, or solving rules under neutral computation-head applications; Phase 110 may carry these forms, but must not claim new reasoning through them.

## 11. Kind and Arity Substrate

### 11.1 Single kind model

`Kind` remains the single kind vocabulary for this packet. SPEC-058 must reuse the existing `Kind::Type` / `Kind::Arrow` model rather than inventing a second kind system.

Implementation note: because `ash-core` owns the canonical type IR and `ash-typeck` already depends on `ash-core`, Phase 110 must re-home the shared `Kind` definition into `ash-core` before canonical IR lands. `ash-typeck` may re-export that core-owned type for compatibility, but this phase must not introduce a second kind enum or a crate-local mirror.

### 11.2 Internal-first validation

SPEC-058 requires explicit kind/arity validation for:

- nominal constructor applications;
- canonical projection argument spines;
- future computation-head applications.

The validation boundary is internal-first:

- current source-level ordinary type parameters continue to default to kind `*` unless a future packet explicitly widens public syntax;
- user-facing kind binder syntax is not introduced by this packet;
- holes and partial type-constructor applications remain rejected or unsupported unless a later spec adopts them explicitly.

### 11.3 Rejection timing

Wrong kind or wrong arity must be rejected before any later normalization/equality packet consumes the type expression.

## 12. Transparent Alias Canonicalization Policy

SPEC-058 introduces a canonicalization policy for transparent aliases.

Rules:

1. transparent aliases canonicalize to their origin heads before the named current equality boundaries `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`, both routed through `TypeEnv::canonicalize_type_for_equality`;
2. diagnostics should preserve the user-written alias spelling where practical;
3. alias canonicalization in this packet is a helper/boundary substrate, not a full normalizer;
4. private or opaque representation facts remain subject to Phase 109 visibility/opacity rules.

### 12.1 Named Phase 110 canonicalization boundaries

For Phase 110, the named current comparison boundaries are `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`. Both consume canonicalized forms through `TypeEnv::canonicalize_type_for_equality`.

These boundaries are the Phase 110 adoption points for transparent-alias canonicalization from TASK-801 and canonical rigid projection forms produced by TASK-800.

Phase 110 does not name `check_pattern.rs` or `exhaustiveness.rs` as canonicalization boundaries. The live pattern checker uses a local structural compatibility check, and the live exhaustiveness checker operates over lowered pattern cells and `TypeDef`; neither currently consumes `TypeEnv` canonicalization state. Any rollout into those paths belongs to a later packet that explicitly unifies them with the canonical type environment.

## 13. Crate Ownership

- `ash-core` owns the single shared `Kind` type, canonical type-expression IR, promoted computation-grade identities, rigid/neutral carriers, and the summary-level interface/member identity records consumed across source and imports.
- `ash-parser` owns surface parsing plus source-summary emission for interface/member identity metadata. It does not resolve projections.
- `ash-typeck` owns elaboration from current surface/core type syntax into canonical IR, imported-summary registration of interface/member identities, kind/arity validation, ambiguity detection, and compatibility-path boundary adoption.
- `ash-engine` remains out of scope for computation-summary export/import in this packet.

## 14. Diagnostics

SPEC-058 requires dedicated diagnostics for:

- ambiguous associated projections;
- unsupported projection shapes admitted by current syntax but not resolvable to a unique canonical identity, distinct from ambiguity diagnostics;
- wrong kind;
- wrong arity;
- explicit rejection of holes/partial type-constructor application if parser parity work touches those paths.

Diagnostics should prefer current user spellings (`S::Ok`, aliases) while using canonical identities internally.

## 15. Acceptance Tests

Minimum acceptance criteria:

1. type-function applications are not represented internally as ordinary nominal constructors;
2. ordinary nominal constructors still decompose exactly as before under current unification;
3. rigid projections compare using canonical identity, not string interface names;
4. canonical projections preserve ordered interface argument spines for both unary `S::Assoc` and multi-parameter `Base<A, B>::Assoc` forms;
5. unsupported projection shapes fail with a dedicated unsupported-shape diagnostic rather than ambiguity fallback or placeholder state;
6. neutral computation-head carriers exist, but no new comparison, decomposition, or solving behavior under neutral heads is required or claimed by this packet;
7. wrong kind/arity is rejected before any later normalization/equality work;
8. transparent aliases and canonical rigid projections are consumed at `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`, via `TypeEnv::canonicalize_type_for_equality`, while diagnostics may continue to render readable source spellings;
9. current SPEC-035 simple associated-output substitution still works for the already-supported subset;
10. no new projection spelling becomes required or accepted beyond the current `base::Assoc` surface;
11. Phase 109 ordinary-type summary/import/export behavior remains unaffected.

## 16. Task Mapping

- [TASK-793](../plan/tasks/TASK-793-spec-b-spec-plan-packet.md): spec/plan packet and registration.
- [TASK-794](../plan/tasks/TASK-794-type-expression-ir-and-kinding-audit-gate.md): live audit and implementation gate.
- [TASK-795](../plan/tasks/TASK-795-core-type-computation-identity-carriers.md): promoted identity carriers and shared `Kind` ownership.
- [TASK-796](../plan/tasks/TASK-796-core-canonical-type-expression-ir-and-neutral-carriers.md): canonical IR and rigid/neutral carriers.
- [TASK-797](../plan/tasks/TASK-797-ordinary-type-parser-expression-parity-and-explicit-rejections.md): parser parity across `parse_type_def.rs` and `parse_module.rs`, plus explicit rejection boundaries.
- [TASK-798](../plan/tasks/TASK-798-canonical-type-ir-lowering-from-surface-and-core.md): canonical IR lowering plus source/import interface-member identity plumbing.
- [TASK-799](../plan/tasks/TASK-799-kind-and-arity-validation-hardening.md): kind/arity validation hardening.
- [TASK-800](../plan/tasks/TASK-800-associated-projection-canonicalization-and-rigid-plumbing.md): canonical projection elaboration and rigid-plumbing over pre-registered source/import interface/member identities.
- [TASK-801](../plan/tasks/TASK-801-transparent-alias-canonicalization-helper.md): alias canonicalization helpers.
- [TASK-802](../plan/tasks/TASK-802-canonicalization-boundary-adoption-for-current-equality-sites.md): boundary adoption at `TypeEnv::unify_types` / `TypeEnv::types_equivalent_for_equality` only; pattern/exhaustiveness rollout remains deferred.
- [TASK-803](../plan/tasks/TASK-803-spec-b-diagnostics-negative-tests-and-non-interference.md): diagnostics and non-interference.
- [TASK-804](../plan/tasks/TASK-804-spec-b-closeout-docs-and-verification.md): docs/status/changelog closeout.
- [TASK-805](../plan/tasks/TASK-805-phase110-review-remediation.md): post-review remediation.
