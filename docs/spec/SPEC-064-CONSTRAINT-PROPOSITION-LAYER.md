# SPEC-064: Constraint and Proposition Layer

**Status:** Draft
**Date:** 2026-05-13
**Promotes:** [DESIGN-034 §16.8](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md), [SPEC-062](SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md), [SPEC-063](SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-009](SPEC-009-MODULES.md), [SPEC-012](SPEC-012-IMPORTS.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md)
**Plan:** [PLAN-112](../plan/PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
**Implementation Tasks:** [TASK-871](../plan/tasks/TASK-871-spec-h-spec-plan-packet.md) through [TASK-884](../plan/tasks/TASK-884-phase116-review-remediation.md)

## 1. Summary

SPEC-064 is DESIGN-034 SPEC-H. It adds a conservative constraint/proposition layer around normalized type expressions. The layer can record, check, and explain type-level propositions without turning the type checker into an unrestricted theorem prover.

The required end state is:

```text
source proposition clauses / generated obligations
  -> raw parser carriers with spans
  -> core canonical proposition carriers when they cross crate/module boundaries
  -> TypeEnv proposition environment
  -> normalizer-backed equality/disequality checks
  -> conservative solver outcomes: satisfied, refuted, or deferred
  -> diagnostics/evidence that explain exactly why no proof search was attempted
```

This specification intentionally does not add type-function inversion, injectivity reasoning, unrestricted SMT, higher-kinded logic, holes, or proof terms. It is a first proof/search boundary for facts that are already structurally available after SPEC-057 through SPEC-063.

## 2. Motivation

The total type-computation substrate can now reduce ordinary public type functions and sealed associated families. The type checker still needs an explicit layer for propositions such as:

```text
Append<Xs, Ys> == Zs
Cons<A, T> != Nil
T: Iterator
SomeNamedPredicate<Xs>
```

Without a proposition layer, every consumer either uses ad-hoc equality calls or emits vague unsupported-feature errors. SPEC-064 gives those consumers one shared representation, one conservative solver, and one diagnostic vocabulary.

The key safety rule is non-inversion: a proposition may inspect normalized results, but it must not solve inputs from an output. For example, `Append<Xs, Ys> == Cons<A, Nil>` must not synthesize bindings for `Xs` or `Ys`.

## 3. Live Baseline

The live post-SPEC-063 substrate is:

- `ash-core::type_ir::CanonicalTypeExpr` represents canonical nominal, projection, and computation-head applications; sealed-domain constructor applications currently live in source-result/normal-form carriers such as `TypeFunctionResultExpr` and `NormalTypeExpr`. Phase 116 therefore adds or designates a proposition-term carrier that can represent both canonical type expressions and sealed-domain constructor applications before solving.
- `ash-typeck::normalizer::Normalizer` exposes normalize-and-compare `DefinitionalEqualityResult` with `Equal`, `NotEqual`, and `BlockedByNeutrality` outcomes.
- `ash-typeck::TypeEnv` already owns `WhereBound`/interface-bound data for current impl checking and selected associated-family tables, but it has no general proposition environment.
- `ash-parser::surface::WhereBound` currently represents only canonical `T: Interface`-style bounds in the existing impl-oriented surface. General type equality/disequality proposition clauses are not yet parser carriers.
- `ash-core::semantic_summary::ModuleSemanticSummary` has V4 public associated-family summaries, but no V5 proposition requirements/evidence surface.
- Existing workflow/capability constraints in `ash-core::ast` and `workflow_contract` are runtime/workflow-contract concepts. They are not the type-level proposition layer defined here.

## 4. Scope

In scope:

1. shared canonical proposition carriers for type-level equality, disequality, interface bounds, and named predicates;
2. source-level raw proposition clauses with spans where Phase 116 explicitly enables them;
3. TypeEnv proposition environment and generated obligation tracking;
4. normalization-backed equality propositions;
5. conservative disequality over normalized constructor heads and obvious sealed-domain constructor disjointness;
6. interface-bound propositions as first-class solver inputs;
7. named explicit proposition predicates as registrable but mostly deferred/opaque facts;
8. V5 semantic-summary transport for public proposition requirements/evidence when public APIs mention proposition clauses;
9. structured diagnostics for unsupported/deferred propositions, blocked neutral evidence, and non-inversion boundaries;
10. row-by-row acceptance and non-interference coverage.

Out of scope:

- solving under neutral computation heads;
- type-function or associated-family inversion;
- output-driven unification from proposition goals;
- unrestricted SMT, proof search, proof terms, tactics, or user-defined proof programs;
- higher-kinded type parameters, type lambdas, holes, partial type-constructor application, or implicit constructor currying;
- value-level contract/workflow predicates, runtime policy constraints, and capability-provider evaluation;
- changing SPEC-035 simple associated-type compatibility semantics.

## 5. Proposition Model

### 5.1 Canonical propositions

`ash-core` owns canonical proposition carriers when propositions cross crate, module, cache, summary, or stable diagnostic boundaries. Solver-private evidence details that never leave `ash-typeck` may remain in `ash-typeck`; shared evidence/refutation/deferred carriers belong in `ash-core` only for the boundary subset identified by TASK-872.

The proposition operand type must be able to represent all Phase 116 acceptance operands, including sealed-domain constructor applications such as `Cons<A, T>` and `Nil`. Implementations may either add a dedicated proposition-term carrier or extend an existing core term carrier, but they must not encode sealed-domain constructors as ordinary nominal types.

```rust
pub enum TypeProposition {
    Equality(TypeEqualityProposition),
    Disequality(TypeDisequalityProposition),
    InterfaceBound(InterfaceBoundProposition),
    NamedPredicate(NamedPredicateProposition),
}

pub enum TypePropositionTerm {
    Canonical(CanonicalTypeExpr),
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<TypePropositionTerm>,
        kind: Kind,
    },
}

pub struct TypeEqualityProposition {
    pub lhs: TypePropositionTerm,
    pub rhs: TypePropositionTerm,
}

pub struct TypeDisequalityProposition {
    pub lhs: TypePropositionTerm,
    pub rhs: TypePropositionTerm,
}

pub struct InterfaceBoundProposition {
    pub subject: TypePropositionTerm,
    pub interface: InterfaceIdentityId,
    pub interface_args: Vec<TypePropositionTerm>,
}

pub struct NamedPredicateProposition {
    pub predicate: PropositionPredicateId,
    pub args: Vec<TypePropositionTerm>,
}
```

Implementations may choose names that fit the existing `ash-core::type_ir` and `semantic_summary` modules, but they must preserve the four logical cases and the sealed-domain constructor operand case above without encoding them as debug strings or lossy `Vec<String>` event lists.

### 5.2 Source propositions

The parser owns raw surface syntax only. TypeEnv owns semantic resolution.

The MVP proposition-clause grammar is:

```text
proposition-clause  = type-expr proposition-op type-expr
                    | type-expr ":" interface-type-application
                    | predicate-name [ "<" type-expr ("," type-expr)* ">" ]
proposition-op      = "==" | "!="
proposition-list    = proposition-clause ("," proposition-clause)*
proposition-tail    = "where" proposition-list
```

Phase 116 enables proposition tails only at task-owned sites. The required first surface extends the live parser declarations rather than replacing their existing shape:

```text
type-fn-declaration = visibility? "type" "fn" name "(" type-fn-params ")"
                      "->" type-expr decreases-clause? proposition-tail?
                      "{" type-fn-equation* "}"
fn-declaration      = visibility? "fn" name type-params? "(" params? ")"
                      ("->" type-expr)? proposition-tail? contract-clauses? block
builtin-fn-declaration = visibility? "builtin" "fn" name type-params? "(" params? ")"
                         "->" type-expr proposition-tail? ";"
```

For ordinary `fn`, the proposition tail is parsed after the optional return type and before existing `requires:`/`ensures:` contract clauses. This keeps type-level propositions separate from value/runtime function contracts.

The existing impl/interface where-bound surface remains accepted. TASK-872 must decide, by live parser audit, whether generalized proposition lists can reuse that surface in Phase 116 or whether impl/interface proposition tails stay disabled with an explicit deferred-feature diagnostic. Task files must not broaden enabled sites by search-and-replace. If an implementation slice cannot safely add a listed site, it must patch SPEC-064/PLAN-112 first and record the scoped deferral before implementation proceeds.

### 5.3 Named predicates

Named proposition predicates are explicit, opaque proposition symbols. The MVP declaration grammar is:

```text
predicate-declaration = visibility? "prop" predicate-name [ "<" predicate-param ("," predicate-param)* ">" ] ";"
predicate-param       = identifier ":" type-expr
```

Examples:

```ash
prop NonEmpty<Xs: TypeList>;
pub prop Sorted<Xs: TypeList>;
```

Named predicate parameters require explicit type/domain annotations in the MVP; omitted predicate-parameter kinds are deferred. Named predicates can be imported/exported and can appear in proposition lists, but the first solver slice does not prove arbitrary named predicates. Unless a predicate is a compiler-known builtin registered by TypeEnv, solver outcome is `DeferredUnsupported` with a diagnostic that names the predicate.

## 6. Solver Outcomes and Evidence

The solver returns a typed outcome for every proposition:

```rust
pub enum PropositionOutcome {
    Satisfied(PropositionEvidence),
    Refuted(PropositionRefutation),
    Deferred(PropositionDeferredReason),
}
```

Evidence/refutation must include the following when the outcome crosses a boundary that requires shared evidence; solver-local outcomes may keep equivalent fields in `ash-typeck`-private structs:

- the original proposition;
- normalized lhs/rhs slices for equality/disequality when applicable;
- the solver rule used;
- source anchors when available;
- whether the decision is local-only or imported-summary-backed;
- the no-inversion note when neutrality blocks a proof.

Deferred is a successful conservative outcome for unsupported proof search. It is an error only at a checking site that requires the proposition to be discharged.

## 7. Equality Propositions

Equality propositions use the SPEC-060 normalizer and definitional equality API.

Rules:

1. Normalize both sides under the current `TypeEnv`.
2. `DefinitionalEqualityResult::Equal` satisfies the proposition.
3. `DefinitionalEqualityResult::NotEqual` refutes the proposition.
4. `DefinitionalEqualityResult::BlockedByNeutrality` defers the proposition unless the checking context explicitly allows a neutral assumption to be recorded.
5. The solver must not call legacy unification to solve underneath canonical computation variables, neutral computation heads, or associated-family projections.
6. Current inference metas remain owned by the existing top-level type unifier. SPEC-064 does not introduce a general bridge from proposition equality to meta solving.

Required examples:

```text
Append<Nil, Ys> == Ys                   // may satisfy after normalization
Append<Xs, Ys> == Cons<A, Nil>          // must defer; no Xs/Ys solving
<Iterator<List<A>>>::Item == A          // may satisfy after SPEC-063 reduction
T::Item == A where only T: Iterator     // must defer on rigid projection
```

## 8. Disequality Propositions

Disequality is conservative. It may succeed only when normal forms expose obviously disjoint heads.

Satisfied disequality cases:

1. Different sealed-domain constructor heads of the same sealed domain, even when their arguments contain open variables; for example `Cons<A, T> != Nil` succeeds because `Cons` and `Nil` are disjoint constructors of the closed `TypeList` domain.
2. Different closed nominal heads when both heads are fully known and the existing type system treats the constructors as disjoint.
3. Equality API returns `NotEqual` with no neutral blockers and the mismatch is at a solver-recognized disjoint head position.

Deferred disequality cases:

1. The head comparison depends on a neutral computation head or rigid projection.
2. The only possible proof would require inverting a type function or associated family.
3. Constructor disjointness would require an unimplemented domain-specific theorem or argument-level proof.

Refuted disequality cases:

1. The two sides normalize to the same normal form.
2. Equality evidence is imported or local and proves definitional equality.

Required example:

```text
Cons<A, T> != Nil       // satisfied by sealed-domain constructor-head disjointness
```

## 9. Interface Bounds

An interface bound proposition records that a subject type is known to implement an interface. It is satisfied only by evidence the type checker already has:

- a selected concrete impl;
- an in-scope generic where-bound for the exact subject variable/head;
- imported summary evidence that the audit task explicitly deems safe to trust/revalidate.

Interface-bound propositions do not trigger impl search beyond existing TypeEnv impl lookup. They must not solve type-function arguments or choose associated-family equations by expected output.

## 10. Public Summary Transport

If public APIs expose proposition clauses, summaries must preserve those clauses through a V5 semantic-summary schema.

V5 requirements:

- `SummaryVersion::SPEC064_PROPOSITIONS_V5` or equivalent is added after the SPEC-063/V4 associated-family version.
- V1/V2/V3/V4 summaries carrying non-empty proposition facts are malformed and rejected before partial registration.
- Public proposition requirements must use core-owned proposition carriers, source anchors, and dependency references.
- Imported proposition evidence is revalidated locally before it can discharge a checking obligation.
- Private helper type functions, private sealed domains, private associated families, or private predicates must not leak through public proposition summaries.

The MVP may transport requirements without transporting proof evidence if the audit finds evidence export too large. If evidence export is deferred, shared core carriers for proof evidence may be limited to requirement identities and deferred-evidence reasons; solver-private normalized traces remain in `ash-typeck`. The summary must say so explicitly and downstream TypeEnv must emit a precise deferred-evidence diagnostic rather than pretending the proposition was proved.

## 11. Diagnostics

Diagnostics must distinguish:

1. unsupported proposition syntax at a surface not enabled by this phase;
2. unknown named predicate;
3. unsupported named predicate solving;
4. equality blocked by neutral computation head;
5. equality blocked by rigid associated projection;
6. disequality unsupported because one side is open/neutral;
7. disequality refuted because both sides are equal;
8. interface bound not found;
9. public proposition summary malformed/unsupported version;
10. private proposition dependency leak;
11. no-inversion boundary: proposition would require solving inputs from outputs.

Diagnostics must include a stable code, span/source anchor when available, expected proposition shape, found proposition shape, and one likely next step. The no-inversion diagnostic must mention that Ash normalized both sides but did not solve under type functions or associated families.

## 12. Acceptance Tests

Phase 116 is accepted only when the following rows have focused evidence in the TASK-882 acceptance matrix:

| ID | Requirement | Expected outcome |
|----|-------------|------------------|
| H1 | `Cons<A, T> != Nil` over a sealed `TypeList` domain | Satisfied by sealed-domain constructor-head disjointness, even with open constructor arguments |
| H2 | `Append<Xs, Ys> == Cons<A, Nil>` with open `Xs`/`Ys` | Deferred/blocked with no substitution for `Xs` or `Ys` |
| H3 | unsupported named predicate in a proposition list | Explicit deferred-feature diagnostic |
| H4 | equality after direct type-function normalization | Satisfied without legacy unification fallback |
| H5 | associated-family projection equality from SPEC-063 | Satisfied when unique family reduction applies |
| H6 | rigid `T::Item` equality under only `T: Iterator` | Deferred on rigid projection, not solved |
| H7 | interface bound proposition for known impl | Satisfied by existing impl/bound evidence |
| H8 | interface bound proposition with no evidence | Refuted or checking-error diagnostic, not search |
| H9 | V5 summary with proposition requirements | Imported/revalidated or explicitly deferred |
| H10 | V4 summary carrying proposition facts | Rejected as malformed before partial registration |
| H11 | private predicate/helper leakage in public proposition summary | Rejected with private-dependency diagnostic |
| H12 | existing SPEC-035/SPEC-063 associated-type behavior | Non-interference: unchanged focused regressions |

## 13. Non-Interference Requirements

Phase 116 must not regress:

- SPEC-035 simple associated type substitution;
- SPEC-057 ordinary type summary import/export;
- SPEC-058 canonical projection identity and kind/arity boundaries;
- SPEC-059 sealed-domain registration and marker constructor metadata;
- SPEC-060 normalizer non-inversion and definitional equality results;
- SPEC-061 direct structural `type fn` validation;
- SPEC-062 public type-function summary import/export;
- SPEC-063 associated-family reduction, rigid where-bound behavior, and V4 summaries;
- workflow/capability runtime constraints, which remain separate from type-level propositions.

## 14. Implementation Tasks

- [TASK-871](../plan/tasks/TASK-871-spec-h-spec-plan-packet.md): SPEC-H spec/plan packet.
- [TASK-872](../plan/tasks/TASK-872-proposition-layer-audit-gate.md): Proposition layer audit gate.
- [TASK-873](../plan/tasks/TASK-873-core-proposition-carriers.md): Core proposition/evidence carriers and V5 summary schema.
- [TASK-874](../plan/tasks/TASK-874-parser-proposition-surface.md): Parser surface for proposition clauses.
- [TASK-875](../plan/tasks/TASK-875-typeenv-proposition-environment.md): TypeEnv proposition environment and obligation generation.
- [TASK-876](../plan/tasks/TASK-876-normalized-equality-disequality-solver.md): Normalized equality and constructor-head disequality solver.
- [TASK-877](../plan/tasks/TASK-877-interface-bound-proposition-solving.md): Interface-bound proposition solving.
- [TASK-878](../plan/tasks/TASK-878-named-predicate-registration-deferred-solving.md): Named predicate registration and deferred solving.
- [TASK-879](../plan/tasks/TASK-879-public-proposition-summary-transport.md): Public proposition summary transport.
- [TASK-880](../plan/tasks/TASK-880-checking-point-integration.md): Checking-point integration.
- [TASK-881](../plan/tasks/TASK-881-proposition-diagnostics.md): Proposition diagnostics.
- [TASK-882](../plan/tasks/TASK-882-spec-h-acceptance-non-interference-matrix.md): Acceptance/non-interference matrix.
- [TASK-883](../plan/tasks/TASK-883-spec-h-closeout-docs-and-verification.md): Closeout docs and verification.
- [TASK-884](../plan/tasks/TASK-884-phase116-review-remediation.md): Independent review remediation.

## 15. Changelog

### 2026-05-13

- Initial SPEC-H draft promoted from DESIGN-034 §16.8 for Phase 116 planning.
