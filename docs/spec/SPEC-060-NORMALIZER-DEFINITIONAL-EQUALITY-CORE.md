# SPEC-060: Normalizer and Definitional Equality Core

**Status:** Implemented MVP
**Date:** 2026-05-06
**Promotes:** [DESIGN-034 §16.4](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-044](SPEC-044-generic-builtin-fn.md)
**Plan:** [PLAN-108](../plan/PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
**Implementation Tasks:** [TASK-816](../plan/tasks/TASK-816-spec-d-spec-plan-packet.md) through [TASK-829](../plan/tasks/TASK-829-phase112-review-remediation.md)

## 1. Summary

SPEC-060 is DESIGN-034 SPEC-D. It defines the first implementation-grade contract for total type-expression normalization and normalize-and-compare definitional equality.

The phase establishes an internal normalizer before public `type fn` syntax exists. Tests therefore use internal fixture equation tables and hand-constructed canonical type-expression values instead of pretending source-level type functions are implemented.

The required end state is:

```text
CanonicalTypeExpr + sealed-domain metadata + fixture equations
  -> environment-aware normalizer
  -> canonical normal forms with neutral/stuck forms
  -> definitional equality API
  -> narrowly adopted forcing points in TypeEnv
```

This packet does not expose user-facing `type fn`, recursive associated type families, proposition solving, module-summary export/import of type-function equations, or new parser syntax.

## 2. Motivation

[SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) introduced computation-head identities, canonical projections, and rigid/neutral carriers. [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md) introduced closed domain constructor metadata. Later direct structural `type fn` work requires a normalizer and equality engine that can consume those substrates honestly.

Without SPEC-060, the compiler still has only ad-hoc equality around `TypeEnv::canonicalize_type_for_equality` and ordinary `unify(...)`. That is insufficient for future type computation because it cannot:

1. reduce closed type-level applications to constructor normal forms;
2. preserve neutral/stuck open applications canonically;
3. compare neutral and rigid projections structurally by canonical identity;
4. define where unification may solve top-level metas and where it must not solve underneath computation heads;
5. give diagnostics for neutrality-blocked equality instead of silently failing or inverting type functions.

## 3. Scope

In scope:

- a normal-form grammar / view over the Phase 110 canonical type-expression substrate;
- domain-constructor normal-form support for Phase 111 marker constructors;
- an internal type-function fixture/equation registry used only by tests and explicit compiler-internal test setup;
- total weak-head and full normalization APIs over canonical type expressions;
- neutral/stuck normal forms for open computation-head applications, neutral associated projections, and rigid associated projections;
- an environment-aware definitional equality API;
- non-inverting equality diagnostics for neutrality-blocked comparisons;
- narrow forcing-point adoption in current `ash-typeck::TypeEnv` equality/return/impl/projection/final-rendering seams;
- cycle/fuel detection as compiler robustness, not as the semantic reason accepted definitions terminate.

Out of scope:

- public `type fn` syntax or parser work;
- equation parsing/lowering from source;
- coverage, overlap, or structural-recursion validation for user definitions;
- public module-summary export/import of type-function equations;
- recursive associated type-family computation;
- generalized associated projection surface syntax beyond the current Phase 110 compatibility boundary;
- type-function inversion, injectivity, disequality solving, or arbitrary proof search;
- promoted data constructors and DataKinds-style runtime constructor promotion;
- global replacement of every typechecker comparison site beyond the named forcing points in this packet.

## 4. Live Baseline and Boundary

### 4.1 Core substrate baseline

`ash-core::type_ir::CanonicalTypeExpr` currently distinguishes primitives, variables, nominal applications, projections, and computation-head applications. It does not yet provide a normal-form view or a domain-constructor application form for sealed marker constructors.

### 4.2 TypeEnv equality baseline

`ash-typeck::TypeEnv` currently exposes:

- `lower_core_type_expr_to_canonical(...)`;
- `lower_surface_type_to_canonical(...)`;
- `canonicalize_transparent_aliases(...)`;
- `canonicalize_type_for_equality(...)`;
- `unify_types(...)`;
- `types_equivalent_for_equality(...)`.

SPEC-060 must preserve ordinary nominal constructor unification while moving computation-aware equality into a dedicated normalizer/definitional-equality API.

### 4.3 Sealed-domain baseline

`TypeEnv` already has dedicated sealed-domain summary registration and lookup methods from SPEC-059. SPEC-060 may consume those summaries to construct domain-constructor normal forms, but must not place marker constructors into ordinary constructor registries.

## 5. Normal-Form Grammar

The normalizer produces canonical normal forms. Implementations may encode this as a new `NormalTypeExpr` enum or an equivalent view, but it must distinguish at least:

```text
NormalTypeExpr ::= Primitive(name)
                 | Var(name)
                 | NominalApp(TypeDeclId, args, kind)
                 | DomainConstructorApp(DomainConstructorId, args, domain, kind)
                 | NeutralComputationApp(TypeComputationHeadId, args, kind, reason)
                 | Projection(InterfaceIdentityId, AssociatedMemberIdentityId, args, kind, rigidity, reason?)
```

Rules:

1. nominal applications and domain-constructor applications are data heads, not computation heads;
2. neutral computation applications compare structurally and are not decomposed for solving;
3. projection normal forms preserve the live Phase 110 `ProjectionRigidity` distinction: `Rigid` projections are blocked by lack of selected associated-family evidence, and `Neutral` projections are blocked by abstract type-variable bases;
4. both rigid and neutral projections compare structurally by canonical interface identity, associated member identity, arity, rigidity, and normalized argument spine;
5. arguments inside neutral computation applications and all projections are recursively normalized for comparison;
6. diagnostic rendering may preserve aliases/user spelling, but equality compares canonical identities.

## 6. Normalization Environment

Normalization is environment-aware. The environment contains:

- registered ordinary type identities and aliases from SPEC-057/058;
- registered sealed-domain summaries from SPEC-059;
- registered associated-member identities and current rigid projection metadata from SPEC-058;
- internal fixture type-function equations for tests in this packet;
- normalization options: weak-head vs full, demand-driven forcing reason, diagnostic fuel, and trace collection.

The first implementation SHOULD make this environment an `ash-typeck` type such as `Normalizer<'env>` or methods on `TypeEnv` backed by a `normalizer.rs` module.

## 7. Internal Fixture Equation Tables

Until SPEC-E exposes public `type fn` syntax, SPEC-060 tests MUST use internal fixtures. A fixture type function is not a source declaration and is not exported in module summaries.

A fixture equation table must support enough structure to test:

```text
Append<Nil, ys> = ys
Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>
```

where `Nil` and `Cons` are Phase 111 domain constructors represented by `DomainConstructorId`, not ordinary ADT constructors.

Fixture rules:

1. fixture tables are registered explicitly in test setup or explicit compiler-internal test setup;
2. fixture equations are closed and deterministic;
3. fixture matching may be limited to first-order constructor patterns and variables;
4. fixture equations are not user-visible, not parsed from source, and not transported in module summaries;
5. production compiler code must not depend on fixture declarations as a substitute for SPEC-E `type fn` declarations;
6. recursive fixture reduction must still terminate under the normalizer's robustness fuel and cycle guard.

## 8. Normalization Judgments

The core judgment is:

```text
Γ ⊢ τ ⇓ τ_norm
```

### 8.1 Weak-head normalization

Weak-head normalization reduces the outermost reducible computation head when its scrutinized shape is known. It may leave arguments unnormalized except where needed for equation selection.

### 8.2 Full normalization

Full normalization recursively normalizes every reachable subterm and repeatedly reduces computation heads until a constructor/neutral/rigid normal form remains.

### 8.3 Demand-driven normalization

Demand-driven normalization is a use-site policy that chooses weak-head or full normalization based on the question being asked. Equality must normalize enough to decide comparison or produce a precise neutrality-blocked diagnostic.

Phase 112 MVP note: the public config surface reserves `WeakHead` and `Demand` mode names, but the implemented normalizer uses the recursive full-normalization behavior for current comparisons. `Normalizer::definitional_equality` explicitly forces `Full` mode; future work may make weak-head/demand normalization operationally distinct without changing equality semantics.

### 8.4 Open terms

If equation selection is blocked by an abstract type variable, unresolved neutral computation head, neutral projection, or rigid projection, normalization returns a canonical neutral/projection form rather than an error.

Example:

```text
normalize(Append<Xs, Ys>) = NeutralComputationApp(Append, [Xs, Ys], reason = AbstractScrutinee)
```

### 8.5 Partial open reduction

Normalization should reduce known prefixes before stopping at neutral tails:

```text
normalize(Append<Cons<A, Xs>, Ys>)
  = Cons<A, NeutralComputationApp(Append, [Xs, Ys], reason = AbstractScrutinee)>
```

## 9. Definitional Equality

Definitional equality is normalize-and-compare:

```text
Γ ⊢ τ1 ≡ τ2  iff  normalize_full(τ1) == normalize_full(τ2)
```

The public API should return structured evidence rather than only `bool`, for example:

```text
Equal
NotEqual { lhs_norm, rhs_norm, mismatch }
BlockedByNeutrality { lhs_norm, rhs_norm, neutral_subterms, no_inversion_note }
```

A boolean convenience wrapper may exist, but diagnostics and tests must exercise the structured result.

## 10. Unification Boundary

SPEC-060 preserves the boundary from DESIGN-034 §8.2:

1. definitional equality normalizes and compares;
2. ordinary nominal constructors may continue to decompose under ordinary unification;
3. canonical abstract variables (`CanonicalTypeExpr::Var(String)`) and current inference metas (`Type::Var(TypeVar)`) are distinct; TASK-817 must map how each bridge participates before TASK-825 changes unification behavior;
4. top-level inference-meta solving remains owned by the existing `Type` unifier unless TASK-825 implements a concrete bridge, and any such bridge is limited to top-level metas subject to existing kind and occurs checks;
5. unification must not solve underneath neutral computation heads;
6. same-headed neutral computation applications compare equal only when their head identity, arity, kind, and normalized argument spines are equal;
7. neutral and rigid projections compare equal only by canonical projection identity, rigidity, and normalized arguments;
8. equality does not invert type functions and does not infer inputs from outputs.

## 11. Forcing Points

The first-slice forcing points are intentionally narrow and must be named in implementation tasks. TASK-817 must produce a forcing-point matrix with exact functions/callsites, and TASK-826 must consume that matrix rather than widening by search-and-replace.

1. `TypeEnv::unify_types` / `types_equivalent_for_equality`: route through definitional equality only where both sides can be lowered/canonicalized without losing current ordinary behavior.
2. Expression checking against expected type: use definitional equality for expected-vs-actual comparison, with fallback to existing unification for legacy shapes out of scope for canonical IR.
3. Declared function/workflow return checking: use structured normalized mismatch evidence when both sides canonicalize.
4. Impl overlap/coherence checks: normalize compatible canonical heads before comparison, but do not solve under neutral heads.
5. Associated projection resolution: normalize projection arguments and preserve unresolved projections as rigid/neutral according to Phase 110 rules; do not add associated-family computation.
6. Final inferred-type rendering: TASK-817 must identify exact callsites such as `TypeEnv::render_type_for_diagnostics` and direct `to_string()` diagnostics; TASK-826 may update only those selected callsites and must render the smallest relevant normalized slice, not a fully expanded diagnostic wall.

Any wider forcing-point rollout requires a later task/spec.

## 12. Diagnostics

Required diagnostic classes:

- neutral/stuck normalization note;
- concrete-normal-form-required error;
- equality blocked by neutral computation head;
- non-inverting equality note;
- cycle/fuel guard error with implementation-failure wording;
- normalized mismatch with user-written and normalized slices.

Diagnostics must state whether the term is neutral/stuck, rigid, mismatched, or unsupported, and must not imply type-function inversion occurred.

## 13. Acceptance Tests

SPEC-060 is complete when focused tests prove:

1. closed fixture reduction normalizes to domain-constructor normal form;
2. open fixture reduction produces canonical neutral forms;
3. partial open reduction preserves reduced prefixes and neutral tails;
4. rigid and neutral associated projection arguments normalize structurally without associated-family computation;
5. definitional equality succeeds after normalization;
6. equality blocked by neutrality returns a non-inverting diagnostic/evidence value;
7. ordinary constructor unification still works as before;
8. same-headed neutral computation apps compare structurally but do not solve under the head;
9. neutral associated projections compare structurally by canonical projection identity, rigidity, and normalized arguments;
10. cycle/fuel guard failures are classified separately from semantic stuckness;
11. existing Phase 109/110/111 type/module/domain behavior remains non-regressed.

## 14. Implementation Tasks

- [TASK-816](../plan/tasks/TASK-816-spec-d-spec-plan-packet.md): SPEC-D spec/plan packet.
- [TASK-817](../plan/tasks/TASK-817-normalizer-defeq-audit-gate.md): audit live normalizer/equality seams.
- [TASK-818](../plan/tasks/TASK-818-core-normal-form-and-domain-constructor-carriers.md): core normal-form and domain-constructor carriers.
- [TASK-819](../plan/tasks/TASK-819-typeck-normalizer-api-skeleton.md): typechecker normalizer API skeleton.
- [TASK-820](../plan/tasks/TASK-820-internal-fixture-equation-registry.md): internal fixture equation registry.
- [TASK-821](../plan/tasks/TASK-821-closed-computation-head-reduction.md): closed computation-head reduction.
- [TASK-822](../plan/tasks/TASK-822-open-neutral-and-partial-normalization.md): open neutral and partial normalization.
- [TASK-823](../plan/tasks/TASK-823-rigid-projection-and-alias-normalization.md): neutral/rigid projection and alias normalization.
- [TASK-824](../plan/tasks/TASK-824-definitional-equality-api.md): definitional equality API.
- [TASK-825](../plan/tasks/TASK-825-non-inverting-unification-boundary.md): non-inverting unification boundary and canonical-var/meta distinction.
- [TASK-826](../plan/tasks/TASK-826-typeenv-forcing-point-rollout.md): TypeEnv forcing-point rollout from the TASK-817 matrix.
- [TASK-827](../plan/tasks/TASK-827-normalizer-diagnostics-and-non-interference.md): diagnostics and non-interference.
- [TASK-828](../plan/tasks/TASK-828-spec-d-closeout-docs-and-verification.md): closeout docs and verification.
- [TASK-829](../plan/tasks/TASK-829-phase112-review-remediation.md): post-review remediation.

## 15. Non-Goals and Deferred Work

Deferred to SPEC-E/F/G/H or later:

- parsing and validating source `type fn` declarations;
- pattern-matrix coverage/overlap/termination checks for user equations;
- exporting public type-function equation summaries;
- associated type-family normalization through impl selection;
- proposition/disequality proof search;
- target inference, partial type-constructor application, and holes;
- public syntax for generalized associated-family projections.


## Implementation Status

Phase 112 implemented the internal SPEC-D normalizer and definitional equality core through TASK-829, with closeout verification recorded in TASK-828 and post-review remediation recorded in TASK-829. The implementation remains internal-only: public `type fn` syntax, source equation validation, associated-family solving, proof search, recursive user computation, type-function inversion, and fixture equation summary export/import are still deferred.
