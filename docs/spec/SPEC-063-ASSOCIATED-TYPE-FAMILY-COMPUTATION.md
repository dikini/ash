# SPEC-063: Associated Type-Family Computation

**Status:** Implemented MVP
**Date:** 2026-05-12
**Promotes:** [DESIGN-034 §16.7](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md), [SPEC-062](SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-009](SPEC-009-MODULES.md), [SPEC-012](SPEC-012-IMPORTS.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
**Plan:** [PLAN-111](../plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
**Implementation Tasks:** [TASK-857](../plan/tasks/TASK-857-spec-g-spec-plan-packet.md) through [TASK-870](../plan/tasks/TASK-870-phase115-review-remediation.md)

## 1. Summary

SPEC-063 is DESIGN-034 SPEC-G. It integrates associated types with the total type-computation substrate by making selected associated outputs reducible through explicit, sealed, coherent associated type-family tables rather than through open-ended ordinary impl search.

The required end state is:

```text
interface associated member declaration
  -> compatibility parser preserves existing Base::Assoc forms
  -> explicit family projection syntax can name an interface/member/argument spine
  -> TypeEnv registers sealed family heads and validated impl-family equations
  -> normalizer reduces uniquely selected family equations over concrete or abstract arguments
  -> generic bounds alone produce rigid projections, not hidden solver work
  -> public family summaries travel through V4 semantic summaries only when export-closed
```

This specification preserves the SPEC-035 simple associated-type substitution path while adding a normalizer-owned family computation path for declarations that opt into sealed family semantics.

## 2. Motivation

SPEC-035 made associated types useful for selected concrete impls, and SPEC-058 gave projections canonical identities and rigidity. That is not enough for type-level libraries where associated outputs must compute in purely type-position code.

Required examples from DESIGN-034 §16.7:

```text
<Iterator<List<A>>>::Item  ==> A
<Iterator<List<X>>>::Item  ==> X
T::Item where T: Iterator  ==> rigid projection, not arbitrary impl search
```

The key risk is accidentally turning every associated projection into a search problem over all possible impls. SPEC-063 avoids that by requiring explicit family sealing/coherence, finite coverage, and structural decreasingness before a projection becomes reducible.

## 3. Live Baseline

The live post-SPEC-062 substrate is:

- `ash-core::type_ir::CanonicalTypeExpr::Projection` already carries `InterfaceIdentityId`, `AssociatedMemberIdentityId`, argument spines, kind, and `ProjectionRigidity`.
- `ash-core::type_ir::NormalTypeExpr` already preserves neutral and rigid associated projections with blocker reasons.
- `ash-core::semantic_summary` already has interface/member identity summaries and V3 public type-function summaries, but no associated-family summary schema.
- `ash-typeck::TypeEnv` already stores interface definitions, associated type names, impl information, canonical interface/member identity registries, and imported public type-function summaries, but it does not own a sealed family selection table.
- The live parser/typechecker associated-member substrate is name-only: interface type parameters and associated type declarations do not yet preserve result-domain annotations, `sealed type family` status, `decreases` clauses, or module ownership context. Phase 115 tasks must add those carriers before semantic validation.
- The live ordinary impl path is not the final family-equation carrier. Family schemes need checked pattern/result carriers for sealed-domain constructors and recursive associated projections rather than lossy lowering through ordinary nominal `Type` values.
- `SPEC-035` simple associated type behavior is compatibility-only: selected concrete impl substitution plus rigid projections when no concrete impl is selected.
- `SPEC-061`/`SPEC-062` provide total source equation validation and public type-computation summary export/import for direct `type fn`, but they do not define associated recursive families.

## 4. Scope

In scope:

1. explicit associated-family projection syntax plus compatibility elaboration for existing `Base::Assoc` forms;
2. a core-owned associated family head identity that combines an interface identity and associated member identity;
3. a distinction between ordinary associated types and sealed computable associated families;
4. family sealing/coherence rules that close the equation set before reduction;
5. unique selected impl/family instance rules over canonical interface argument spines;
6. reduction of uniquely selected generic impl schemes over abstract arguments without solving under neutral computation heads;
7. rigid projection behavior when only generic where-bound evidence exists;
8. recursive associated-family coverage, overlap, and decreasingness validation over sealed domains;
9. a strict split between where-bound evidence and family equation selection;
10. V4 public/private associated-family summary export/import rules;
11. associated-family diagnostics and acceptance/non-interference tests.

Out of scope:

- general proposition solving, constraint implication, or proof search (SPEC-H);
- type-function inversion, injectivity, or solving arguments from projected outputs;
- higher-kinded type parameters, type lambdas, holes, and partial type-constructor application;
- open-world/orphan family equation extension for sealed families;
- runtime ADT behavior, value-level method dispatch, capability/provider semantics, and workflow runtime semantics;
- replacing SPEC-035 simple associated type substitution for non-family associated types.

## 5. Surface Syntax and Compatibility

### 5.1 Projection syntax

SPEC-063 chooses this explicit projection syntax for computation-grade associated families:

```text
associated-family-projection = "<" interface-type-application ">" "::" associated-member
interface-type-application   = interface-name [ "<" type-expr ("," type-expr)* ">" ]
interface-name               = identifier
associated-member            = identifier
```

Examples:

```ash
<Iterator<List<A>>>::Item
<Append<Cons<H, T>, Ys>>::Out
```

The Phase 115 MVP intentionally keeps the explicit projection head unqualified at the parser surface. Imported or re-exported interfaces are used through source-visible local names/aliases established by the existing module system; adding path-qualified type/interface names inside type expressions is a later parser/type-name substrate task, not an implicit part of SPEC-063.

The existing SPEC-035 compatibility spelling remains accepted in type positions:

```text
associated-projection = projection-base "::" identifier
projection-base       = identifier | nominal-type-application
```

Compatibility elaboration rules:

0. Parser work for SPEC-063 includes projection syntax, typed interface/impl type parameters such as `Xs: TypeList`, and the raw declaration surface for `sealed type family Name: ResultDomain [decreases Param]` members inside interface declarations. `: ResultDomain` is mandatory in the MVP; omitting it is a parser/typecheck error rather than an implicit default. The parser records raw tokens, result-domain type syntax, optional `decreases` clause, parameter annotations, and spans only; semantic validation remains in `ash-typeck`.
1. Existing compatibility forms such as `Iterator<List<A>>::Item` may elaborate to the same associated-family projection as `<Iterator<List<A>>>::Item` when the base resolves unambiguously to an interface application. Imported names and aliases are resolved by the type checker from source-visible bindings, not by path-qualified type parsing.
2. `T::Assoc` under exactly one in-scope bound declaring `Assoc` elaborates to a rigid projection keyed by that bound unless a concrete impl/family instance has already been selected by the current typing operation.
3. Ambiguous `T::Assoc` remains the SPEC-035 ambiguity error; SPEC-063 does not add an implicit disambiguation search.
4. Parser output remains raw surface syntax plus spans. Interface/member resolution is owned by `ash-typeck`.

### 5.2 Associated member declarations

Ordinary associated types keep the SPEC-035 spelling:

```ash
interface Serializer<S> {
    type Ok
}
```

Computable associated families opt in explicitly. The result domain after `:` is mandatory for Phase 115 so kind/domain validation has an explicit source contract:

```ash
interface Iterator<I> {
    sealed type family Item: Type
}

interface Append<Xs: TypeList, Ys: TypeList> {
    sealed type family Out: TypeList decreases Xs
}
```

Normative meaning:

- `type Name` is a simple associated type. It supports SPEC-035 selected-impl substitution and rigid projections only.
- `sealed type family Name: ResultDomain ...` is a closed associated family. It may reduce in the normalizer after this spec's totality/coherence checks pass.
- Interface and impl type parameters may carry domain annotations such as `Xs: TypeList`; `ash-typeck` owns checking that a declared `decreases Param` names one of those sealed-domain-constrained interface arguments.
- `sealed` is mandatory for reducible families in the MVP. Non-sealed/open associated families may be reserved by future specs but do not reduce here.

## 6. Core IR and Identities

`ash-core` must own the shared family identity and summary carriers. Conceptually:

```rust
pub struct AssociatedFamilyHeadId {
    pub interface: InterfaceIdentityId,
    pub member: AssociatedMemberIdentityId,
}

pub struct AssociatedFamilyProjection {
    pub head: AssociatedFamilyHeadId,
    pub interface_args: Vec<CanonicalTypeExpr>,
    pub kind: Kind,
    pub rigidity: ProjectionRigidity,
}
```

Implementations may reuse the existing canonical projection variant if it can represent this information without lossy reconstruction. If reuse is chosen, the implementation must still expose named helper APIs so downstream code distinguishes:

- simple associated projection identity;
- reducible sealed associated-family head;
- rigid projection from generic bound evidence;
- neutral projection because family reduction is blocked or unavailable.

The same core slice must also provide family-specific scheme/result carriers or reusable checked carriers that can represent sealed-domain constructor patterns/results and recursive associated-family projections. Ordinary `ImplScheme`/`Type` lowering is not sufficient when it would encode marker constructors as nominal types or lose recursive projection structure.

Public summaries must use core-owned identities. `ash-parser` must not own semantic family identities, and `ash-engine` must not define engine-private family semantics.

## 7. Family Sealing and Coherence

A sealed associated family has a closed equation set determined at the interface/family definition boundary.

Rules:

1. All reducible equations for a sealed family must be known before the family is published to the normalizer.
2. The MVP accepts equations from impl schemes that are in the same module as the family declaration, or from imported public family summaries validated by SPEC-063 V4 import rules. Family declarations and impl-family schemes therefore carry defining module identity; TypeEnv registration APIs must receive enough module/owner context to reject unauthorized extensions.
3. Downstream modules may still define ordinary impls for interfaces that contain no sealed associated-family members where existing coherence permits it.
4. If an interface contains a sealed associated-family member, any downstream impl that would provide or alter that member's equation is rejected in the MVP as an unauthorized sealed-family extension. A future spec may add explicit extension hooks, but Phase 115 does not.
5. Every family impl scheme must target exactly one associated family head and provide exactly one RHS for the associated member being reduced.
6. Overlapping schemes are rejected unless ordered residual subtraction proves the later scheme covers a non-overlapping residual space.
7. A family is normalizer-available only after all schemes pass signature, coverage, overlap, result-domain, and recursion validation.

This is stricter than ordinary method dispatch. Ordinary where-bound evidence establishes obligations; it does not add equations to the sealed family table. Public export is also closed: a reducible public family summary may be exported only when the entire validated closed equation set and every dependency are public-summary-visible. Phase 115 does not export partial private equation tables; private families/equations remain same-module only, and any attempted public export with private dependencies is rejected rather than represented as a downstream-reducible family.

## 8. Selection Semantics

A family reduction candidate is selected by matching the canonical interface argument spine against validated family impl scheme heads.

Selection must be:

1. **unique** — exactly one residual scheme applies after coherence validation;
2. **structural** — matching decomposes only known nominal/sealed-domain constructors and canonical variables;
3. **one-way** — family selection is first-order pattern matching from a validated scheme head to the queried projection argument spine. The matcher may bind only scheme-owned variables. Caller/environment variables and inference metas in the queried projection are opaque inputs and are never assigned;
4. **non-inverting** — matching never solves for variables by inspecting the desired projected output, and expected output types are never consulted during scheme selection;
5. **neutral-safe** — matching never solves underneath neutral computation heads or rigid projections;
6. **summary-stable** — imported public summaries select the same scheme as the defining module.

Positive examples:

```text
<Iterator<List<A>>>::Item  ==> A
<Iterator<List<X>>>::Item  ==> X
```

Here `Iterator<List<T>>` is a unique generic family impl scheme, and matching `List<X>` against `List<T>` yields the direct substitution `T := X` without solving under a neutral head.

Generic bound example:

```text
fn head<T: Iterator>(x: T) -> T::Item
```

`T::Item` remains rigid because the bound proves an obligation but does not identify a unique impl/family equation. No speculative search over possible `Iterator<_>` impls is allowed.

## 9. Normalization and Equality

The normalizer must extend its projection handling with a family lookup path:

1. normalize projection interface arguments according to the current demand mode;
2. consult the local associated-family table only when the projection head is a sealed family and local validated equations are available; imported table availability is added by the V4 summary import task after import validation succeeds;
3. reduce through the unique selected scheme by substituting matched scheme-owned arguments into the associated RHS;
4. recursively normalize the RHS under existing SPEC-060 fuel/cycle controls;
5. return a neutral or rigid projection with a precise blocker when reduction is unavailable, ambiguous, not sealed, blocked by generic-bound rigidity, blocked by private opacity, unsupported by current summary/import visibility, or fuel/cycle exhausted.

Definitional equality remains normalize-and-compare. SPEC-063 does not add projection inversion. A comparison such as `<Append<Xs, Ys>>::Out == Cons<A, Nil>` must not solve `Xs` or `Ys` from the output.

## 10. Recursive Associated Families

Recursive associated-family computation is permitted only for sealed families that pass totality checks analogous to SPEC-061 direct structural `type fn`, adapted to impl-family heads.

Required validation:

1. every recursive associated family must explicitly declare `decreases Param`, and that parameter must be a sealed-domain-constrained interface argument;
2. scheme heads form a finite residual coverage matrix over the inspected sealed-domain spaces;
3. overlap/unreachable/default rows are handled by the same ordered residual semantics as SPEC-061;
4. recursive RHS projections target the same family head only with a direct structural subcomponent of the declared decreasing parameter;
5. mutual recursion is rejected in the MVP;
6. RHS result expressions conform to the declared result kind/domain;
7. recursive calls through imported summaries are allowed only when the imported V4 summary records the same validated decreasingness facts.

Example shape:

```ash
interface Append<Xs: TypeList, Ys: TypeList> {
    sealed type family Out: TypeList decreases Xs
}

impl<Ys: TypeList> Append<Nil, Ys> {
    type Out = Ys
}

impl<H, T: TypeList, Ys: TypeList> Append<Cons<H, T>, Ys> {
    type Out = Cons<H, <Append<T, Ys>>::Out>
}
```

This passes only if `TypeList`, `Nil`, and `Cons` are sealed-domain facts available to the validator, coverage is exhaustive, schemes are coherent, and recursion decreases from `Cons<H, T>` to `T`.

## 11. Public/Private Summary Export and Import

SPEC-063 advances semantic summaries to a V4 computation-family boundary. The V4 schema must be concrete enough for import-side revalidation; opaque or partial equation-table exports are not reducible in the MVP.

Required core-owned summary payloads:

```rust
pub struct AssociatedFamilySummary {
    pub head: AssociatedFamilyHeadId,
    pub interface_identity: InterfaceIdentityId,
    pub member_identity: AssociatedMemberIdentityId,
    pub visible_name: String,
    pub result_domain: CanonicalTypeExpr,
    pub result_kind: Kind,
    pub export_mode: AssociatedFamilyExportMode,
    pub schemes: Vec<AssociatedFamilySchemeSummary>,
    pub dependency_closure: AssociatedFamilyDependencyClosure,
    pub source_anchor: SummarySourceAnchor,
}

pub struct AssociatedFamilySchemeSummary {
    pub interface_arg_patterns: Vec<FamilyPatternSummary>,
    pub result: FamilyResultExprSummary,
    pub decreases: Option<ValidatedDecreasesSummary>,
    pub source_anchor: SummarySourceAnchor,
}
```

Names are conceptual; implementations may use existing core carriers when they preserve the same data. The schema must carry or reference:

- family head identity, interface/member identities, and visible/exported names;
- result kind/domain and sealed public export mode;
- ordered scheme heads/patterns and RHS result expressions;
- decreases parameter and validated decreasingness metadata for recursive families;
- enough coverage/overlap/coherence data to revalidate, or raw checked schemes from which those facts are revalidated;
- source anchors for family declarations and impl schemes;
- dependency closure for ordinary types, sealed domains/marker constructors, direct type functions, associated projections, and other family heads;
- whether helper family heads are source-visible or only normalizer-available through dependency closure.

Version and visibility rules:

- V1/V2/V3 summaries carrying non-empty associated-family facts are malformed and must be rejected.
- V4 summaries may carry public associated-family facts only when the full validated closed equation set and all dependencies are public-summary-visible.
- Private family equations are not exported, and public summaries with private family equations or private dependencies are rejected rather than partially exported.
- Imported V4 family summaries must be batch-declared before validation, alongside ordinary types, sealed domains, interface/member identities, type-function heads, and other associated-family heads.
- Importers must revalidate kind/domain, coverage/overlap, coherence, selected-scheme uniqueness, recursion/decreases metadata, and public dependency closure before registering imported equations with the normalizer.

Named imports, glob imports, and `pub use` re-exports preserve canonical family head identities. Dependency-closure helper family heads may be normalizer-available without becoming source-visible names, matching SPEC-062's helper-head rule for direct type functions.

## 12. Diagnostics

SPEC-063 introduces these diagnostic families:

- `AssociatedFamilySyntaxUnsupported` — projection or declaration shape is parsed but outside the MVP.
- `AssociatedFamilyNotSealed` — a reducible family projection targets an ordinary/open associated type.
- `AssociatedFamilyAmbiguousMember` — compatibility `T::Assoc` resolves to multiple bounds or members.
- `AssociatedFamilyImplNotInSealedSet` — an impl attempts to extend a sealed family from an unauthorized module or phase.
- `AssociatedFamilyMissingBinding` / `AssociatedFamilyExtraBinding` — impl bindings do not match the family/member contract.
- `AssociatedFamilyOverlap` / `AssociatedFamilyUnreachableRow` / `AssociatedFamilyNonExhaustive` — coherence or coverage failure.
- `AssociatedFamilyMissingDecreases` / `AssociatedFamilyInvalidDecreases` — recursive family declarations omit `decreases`, name an unknown parameter, or name a parameter that is not sealed-domain-constrained.
- `AssociatedFamilyNotDecreasing` — recursive RHS does not structurally decrease on the declared parameter.
- `AssociatedFamilyResultKindMismatch` / `AssociatedFamilyResultDomainMismatch` — family declaration annotations or impl RHSs do not conform to the declared result kind/domain.
- `AssociatedFamilyMutualRecursionUnsupported` — a family SCC contains mutual recursion, which is rejected in the MVP.
- `AssociatedFamilySelectionAmbiguous` — a forcing point observes more than one applicable family scheme despite validation safeguards.
- `AssociatedFamilyRigidProjection` — note/hint that a projection remains rigid because only where-bound evidence is available.
- `AssociatedFamilyPrivateReductionUnavailable` — reduction requires private equations not available across the module boundary.
- `AssociatedFamilyExportPrivateDependency` / `AssociatedFamilyExportNotClosed` — a public family summary would require private or incomplete dependencies.
- `AssociatedFamilyImportOrderConflict` / `AssociatedFamilyDependencyClosureConflict` — imported V4 summaries cannot be batch-declared/revalidated consistently.
- `AssociatedFamilySummaryMalformed` / `AssociatedFamilySummaryUnsupportedVersion` — imported summary version or content is invalid.

Diagnostics must include the family/member name, interface identity/path, projection source span, selected or candidate impl source anchors when available, and one likely fix.

## 13. Acceptance Tests

Required acceptance matrix:

1. `<Iterator<List<A>>>::Item` reduces to `A` through a unique generic impl.
2. `<Iterator<List<X>>>::Item` reduces to `X` even when `X` is abstract.
3. `T::Item` under only `T: Iterator` remains rigid in generic code.
4. Existing SPEC-035 selected concrete impl substitution continues to work for non-family associated types.
5. Existing SPEC-035 compatibility projection spelling elaborates to the same canonical family projection as explicit `<Interface<...>>::Assoc` when unambiguous.
6. Ambiguous family impls are rejected before normalizer registration, or at a forcing point with a precise ambiguity diagnostic if a malformed imported summary reaches the boundary.
7. Recursive `Append`-style associated family computation passes only when sealed, exhaustive, coherent, and structurally decreasing.
8. Non-decreasing recursive family equations are rejected and never registered.
9. Public family summaries reduce downstream through V4 semantic summaries, independent of import order.
10. Private family equations remain opaque downstream and produce unavailable-reduction diagnostics when a boundary requires reduction.
11. Where-bound evidence and family equation selection remain separate: adding a bound must not make a projection reduce unless a family head/argument spine selects a unique sealed scheme.
12. `<Append<Xs, Ys>>::Out == Cons<A, Nil>` or an equivalent associated-family output comparison does not solve `Xs` or `Ys`; it remains non-inverting evidence.
13. Existing SPEC-057 ordinary summaries, SPEC-058 projection identity behavior, SPEC-060 non-inversion, SPEC-061 direct `type fn`, and SPEC-062 public type-function summaries remain non-regressed.
14. Family selection binds only scheme-owned variables; queried projection variables/metas remain opaque and are never solved by expected output shape.
15. Public summary export rejects private or incomplete family equation/dependency closures instead of exporting a partial reducible table.
16. V4 import rejects malformed decreases metadata, result-domain mismatches, selected-scheme ambiguity, and dependency-closure conflicts before normalizer registration.
17. `sealed type family Name` without `: ResultDomain` is rejected or diagnosed according to the mandatory-result-domain MVP rule.

TASK-868 must produce a row-by-row acceptance/non-interference artifact mapping every item above to focused tests or recorded evidence.

## 14. Implementation Tasks

- [TASK-857](../plan/tasks/TASK-857-spec-g-spec-plan-packet.md): SPEC-G spec/plan packet.
- [TASK-858](../plan/tasks/TASK-858-associated-family-audit-gate.md): live substrate audit gate and downstream task-file hardening gate.
- [TASK-859](../plan/tasks/TASK-859-associated-family-surface-and-compat-parser.md): surface syntax and compatibility parser.
- [TASK-860](../plan/tasks/TASK-860-core-associated-family-identity-carriers.md): core identities and summary carriers.
- [TASK-861](../plan/tasks/TASK-861-typeck-family-declaration-registration-coherence.md): family declaration, registration, sealing, and coherence.
- [TASK-862](../plan/tasks/TASK-862-spec035-substitution-compatibility-bridge.md): SPEC-035 substitution compatibility bridge.
- [TASK-863](../plan/tasks/TASK-863-unique-generic-impl-family-selection.md): unique generic impl-family selection over abstract arguments.
- [TASK-864](../plan/tasks/TASK-864-rigid-where-bound-projection-boundary.md): rigid where-bound projection boundary.
- [TASK-865](../plan/tasks/TASK-865-recursive-associated-family-totality.md): recursive family totality and decreasingness.
- [TASK-866](../plan/tasks/TASK-866-normalizer-projection-family-integration.md): local-family normalizer and equality integration.
- [TASK-867](../plan/tasks/TASK-867-associated-family-summary-export-import.md): public/private family summary export/import plus imported-family normalizer availability.
- [TASK-868](../plan/tasks/TASK-868-associated-family-diagnostics-acceptance-matrix.md): diagnostics and acceptance matrix.
- [TASK-869](../plan/tasks/TASK-869-spec-g-closeout-docs-and-verification.md): closeout docs and verification.
- [TASK-870](../plan/tasks/TASK-870-phase115-review-remediation.md): independent review remediation.

## 15. Implementation Status

Phase 115 implemented the SPEC-G MVP through TASK-870, including TASK-869 closeout verification and TASK-870 independent post-closeout review remediation. The implemented slice includes parser surface support for explicit associated-family projections and sealed family declarations, core associated-family identities and V4 summaries, TypeEnv family declaration/coherence/selection/recursive-totality validation, normalizer integration for local and imported reducible families, public summary export/import with helper-family opacity, public/source type-position lowering for explicit associated-family projections, diagnostics, and acceptance/non-interference evidence.

The MVP remains intentionally bounded: SPEC-035 ordinary associated-type substitution is preserved for non-family members; where-bound-only projections remain rigid; definitional equality remains normalize-and-compare and non-inverting; and SPEC-H proposition solving, proof search, type-function inversion, HKT, holes, and open-world/orphan family extension are still deferred.

## 16. Non-Goals and Handoff

SPEC-063 completes associated type-family computation over the total type-computation substrate. SPEC-H owns propositions, constraint solving, disequality reasoning, and proof/search predicates over normalized types. Later HKT/Monad work may add constructor-kinded abstractions and holes; this spec deliberately does not depend on them.
