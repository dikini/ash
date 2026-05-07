# SPEC-061: Direct Structural Type Functions

**Status:** Implemented MVP
**Date:** 2026-05-07
**Promotes:** [DESIGN-034 §16.5](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-020](SPEC-020-ADT-TYPES.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md)
**Plan:** [PLAN-109](../plan/PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
**Implementation Tasks:** [TASK-830](../plan/tasks/TASK-830-spec-e-spec-plan-packet.md) through [TASK-842](../plan/tasks/TASK-842-phase113-review-remediation.md)

## 1. Summary

SPEC-061 is DESIGN-034 SPEC-E. It defines Ash's first user-facing type-computation surface: direct structural `type fn` declarations over sealed type-level domains.

The required end state is:

```text
source `type fn` declaration
  -> surface AST with spans
  -> core type-function/equation carriers
  -> module-local TypeEnv registration
  -> totality checks: kind/domain, pattern linearity, coverage, overlap, structural recursion
  -> registered normalizer equation table
  -> module-local definitional equality reduction
```

SPEC-061 deliberately remains module-local. Until SPEC-F defines public semantic summaries for type functions, public/cross-module type-function normalization is rejected. Associated recursive type families remain SPEC-G work.

## 2. Motivation

SPEC-060 proved the internal normalizer and definitional-equality substrate using compiler-internal fixture equation tables. That kept the semantics honest before exposing source syntax, but it left users unable to define total type-level computations.

SPEC-061 replaces test fixtures with checked source declarations. It must not weaken the SPEC-060 non-inversion boundary: an open application such as `Append<Xs, Ys>` remains neutral when `Xs` is abstract, while a closed application such as `Append<Cons<A, Nil>, Cons<B, Nil>>` reduces.

## 3. Scope

In scope:

- module-level `type fn` parser surface and spans;
- shared surface/core carriers for type-function signatures, equations, patterns, decreasing parameter metadata, and source anchors;
- module-local type-function identity registration using `TypeComputationHeadId`;
- lowering from source equations to canonical computation-head applications;
- type-level pattern grammar: constructor patterns, variables, and wildcards;
- pattern linearity and repeated-variable rejection;
- coverage and overlap checking by finite, sealed-domain-directed pattern matrix analysis;
- explicit catch-all/default rows as ordered residual known-constructor coverage only;
- one declared decreasing parameter per recursive type function;
- structural subcomponent validation for recursive calls, including nested calls under constructors;
- normalizer integration by replacing SPEC-060 fixture-only registration with checked module-local source equation registration;
- diagnostics and acceptance tests for the DESIGN-034 §16.5 cases.

Out of scope:

- `pub type fn` export/import or public cross-module normalization;
- semantic-summary serialization of type-function equations;
- associated recursive type-family computation;
- mutual recursion, lexicographic recursion, size-change termination, or inferred decreasing parameters;
- equality/disequality guards in patterns;
- open catch-all reduction over abstract variables;
- type functions with no sealed-domain scrutinee in their parameter list;
- promoted runtime data constructors or DataKinds-style promotion;
- type-function inversion, injectivity, disequality solving, or proof search;
- holes, partial type-constructor application, or generalized type lambdas.

## 4. Live Baseline

The live substrates this spec builds on are:

- `ash-parser::surface::Definition` includes ordinary `Type(...)`, `Function(...)`, `BuiltinFn(...)`, `SealedDomain(...)`, and other module-level variants, but no `TypeFn(...)` variant.
- `ash-parser::surface::Type` already has constructor syntax via `Type::Constructor { name, args }` and associated projections via `Type::Associated { base, name }`.
- `ash-core::type_ir` owns `TypeComputationHeadId`, `CanonicalTypeExpr::ComputationHeadApp`, `NormalTypeExpr`, and `NormalFormBlockReason`.
- `ash-core::semantic_summary` owns sealed-domain and marker-constructor identities used by `NormalTypeExpr::DomainConstructorApp`.
- `ash-typeck::normalizer` owns fixture equation registration and reduction for SPEC-060 tests.
- `ash-typeck::TypeEnv` owns canonical lowering, sealed-domain registration, and guarded definitional equality forcing points.

SPEC-061 must extend these live carriers. It must not encode type-function applications or sealed-domain marker-constructor applications as ordinary nominal `Type::Constructor` / `CanonicalTypeExpr::NominalApp` nodes after semantic elaboration. Because live `CanonicalTypeExpr` has no sealed-domain constructor-application variant, this phase must add a dedicated source-equation result expression carrier or explicitly extend the canonical IR before source equations are lowered.

## 5. Surface Syntax

The first-slice syntax is:

```text
type-fn-def        = rejected-visibility? "type" "fn" name "(" params ")" "->" type decreases-clause? "{" equation* "}"
rejected-visibility = "pub" | "pub" "(" "crate" ")"
params             = name ":" type ("," name ":" type)*
decreases-clause   = "decreases" name
equation           = "case" name "<" type-patterns ">" "=" type ";"
type-patterns      = TypePattern ("," TypePattern)*
```

Example:

```ash
sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}

type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
{
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
}
```

Rules:

1. `type fn` is a top-level module definition.
2. `pub type fn` and `pub(crate) type fn` are parsed far enough to emit the SPEC-F handoff diagnostic, then rejected until SPEC-F.
3. Zero-parameter type functions are rejected in SPEC-E; at least one parameter must be present.
4. At least one parameter must be a sealed-domain scrutinee.
5. The function name in each `case` row must match the containing `type fn` name.
6. Equation arity must equal the parameter count.
7. The first slice requires an explicit `decreases <param>` clause when any recursive call to the same type function appears.
8. Non-recursive type functions may omit `decreases`.
9. Final semicolons are required on every `case` row.
10. No ordinary Ash expression body is accepted inside a `type fn`.

## 6. Type-Level Pattern Grammar

The first-slice pattern grammar is:

```text
TypePattern ::= ConstructorPattern
              | VarPattern
              | WildcardPattern

ConstructorPattern ::= Name | Name "<" TypePattern ("," TypePattern)* ">"
VarPattern         ::= lower_identifier
WildcardPattern    ::= "_"
```

Semantic rules:

1. Constructor names in patterns resolve only to visible sealed-domain marker constructors, never ordinary ADT/runtime constructors.
2. A constructor pattern is valid only when the corresponding parameter position has that constructor's sealed domain.
3. Pattern variables bind type expressions and are scoped over the equation RHS.
4. Repeated variables in one equation row are rejected. Equality guards are deferred; repeated names across different rows remain legal.
5. `_` does not bind a name.
6. Constructor field arity must match the sealed-domain constructor metadata.
7. Zero-field constructors are spelled as bare `Name`, not `Name<>`.
8. Nested constructor patterns are allowed only through explicitly sealed-domain-typed fields (`DomainRef(...)`) whose metadata makes the nested domain visible and structural for matching.
9. Nested constructor patterns are not allowed inside unconstrained `Type` slots in SPEC-E; a `Type` slot may bind a variable or wildcard but is not a closed-domain scrutinee.
10. Bare pattern names are disambiguated with the expected parameter/field domain. If the expected position is a sealed-domain slot and the bare name resolves to one of that domain's visible marker constructors, it is a constructor pattern even when the spelling is lowercase.
11. A lowercase name in a sealed-domain position binds a variable only when it does not resolve to a marker constructor for that expected domain. This avoids treating lowercase marker constructors as variables.
12. A bare name that resolves to both a visible marker constructor for the expected sealed domain and another type-level head that could be used in the same pattern position is rejected as an ambiguous marker-constructor pattern; implementations must not guess.
13. In unconstrained `Type` slots, marker-constructor matching is unavailable; bare lowercase names bind variables, `_` remains a wildcard, and uppercase/non-variable bare names must be rejected unless a later SPEC introduces open type-pattern matching.

## 7. Core Carriers

SPEC-061 adds shared carriers in `ash-core` or an equivalent core-owned module. The exact Rust names may differ, but the semantic shape is mandatory. These carriers are `[NEW]` unless already introduced by an adjacent implementation task.

`TypeFunctionEquation.result` must **not** be a bare live `CanonicalTypeExpr` unless `CanonicalTypeExpr` is first extended to represent sealed-domain marker-constructor applications. The Phase 112 carrier can normalize marker constructors with `NormalTypeExpr::DomainConstructorApp`, but live `CanonicalTypeExpr` has no equivalent source/canonical expression variant. SPEC-061 therefore requires a source-equation result expression carrier with this shape, or an explicit canonical-IR extension with the same information:

```text
TypeFunctionDef {
  visibility,
  head: TypeComputationHeadId,
  name,
  params: Vec<TypeFunctionParam>,
  return_type: CanonicalTypeExpr,
  decreases: Option<ParamName>,
  equations: Vec<TypeFunctionEquation>,
  source_anchor,
}

TypeFunctionEquation {
  head: TypeComputationHeadId,
  patterns: Vec<TypeFunctionPattern>,
  result: TypeFunctionResultExpr,
  source_anchor,
}

TypeFunctionPattern ::= DomainConstructor { constructor: DomainConstructorId, domain: SealedDomainId, fields }
                      | Var { name, kind_or_domain }
                      | Wildcard { kind_or_domain }

TypeFunctionResultExpr ::= Primitive(name)
                         | Var(name)
                         | NominalApp { origin: TypeDeclId, visible_name, args, kind }
                         | DomainConstructorApp { constructor: DomainConstructorId, domain: SealedDomainId, args, kind }
                         | Projection { interface, member, args, kind, rigidity }
                         | ComputationHeadApp { head: TypeComputationHeadId, args, kind }
```

Rules:

1. `DomainConstructorApp` uses sealed-domain `DomainConstructorId` / `SealedDomainId`, never ordinary runtime/ADT constructor identities.
2. `ComputationHeadApp` uses `TypeComputationHeadId` and remains distinct from nominal type constructors.
3. `TypeFunctionResultExpr` may share conversion helpers with `CanonicalTypeExpr`, but it must preserve marker-constructor apps until normalization.
4. If the implementation chooses to extend `CanonicalTypeExpr` instead, TASK-833 must add the domain-constructor variant and update existing consumers explicitly; silent encoding as `NominalApp` is forbidden.
5. Source-backed normalizer registration consumes checked `TypeFunctionResultExpr` values and converts marker constructors to `NormalTypeExpr::DomainConstructorApp` during reduction.

The carrier must preserve enough source anchors for diagnostics to point at:

- the `type fn` header;
- the declared decreasing parameter;
- the mismatched `case` head;
- each pattern variable or repeated variable;
- each ambiguous marker-constructor or type-function head;
- each uncovered, overlapping, unreachable, or empty-default row;
- each non-decreasing recursive call.

## 8. Source Type-Expression Resolution

Source type-function applications and sealed-domain marker-constructor applications use the existing angle-argument type-expression spelling, for example `Append<Nil, Ys>` and `Cons<h, Append<t, ys>>`. Semantic elaboration distinguishes them from nominal type constructors and from each other.

Resolution rules:

1. In `type fn` equation RHSs, name resolution first checks the equation's pattern-variable environment.
2. If the expected RHS position has a sealed-domain constraint, a name that resolves to a marker constructor of that domain lowers to `TypeFunctionResultExpr::DomainConstructorApp` (or the equivalent canonical-domain-constructor carrier), not to a nominal application.
3. After pattern variables and expected-domain marker constructors, resolution checks the current function's provisional self head, then already validated module-local type-function heads, then ordinary type names.
4. Outside a `type fn` body, local type-function heads are usable only after their definition has passed validation and only in non-public module-local type-expression positions in SPEC-E.
5. If a visible nominal type constructor and a visible type-function head share the same source name in a position where both could apply, SPEC-E rejects the reference as ambiguous rather than guessing.
6. If a visible sealed-domain marker constructor and a nominal/type-function head share the same source name in a RHS position where both could apply under the expected type/domain, SPEC-E rejects the reference as an ambiguous marker-constructor/type-head reference.
7. Type-function applications lower to `TypeFunctionResultExpr::ComputationHeadApp` or `CanonicalTypeExpr::ComputationHeadApp`; nominal constructors continue to lower to nominal applications.
8. Public ordinary type aliases, exported function signatures, exported interface surfaces, or other semantic summaries must not mention local type-function computation heads before SPEC-F. Such leakage is rejected with a SPEC-F handoff diagnostic.
9. Source-order rule: a type function may reference itself provisionally and may reference earlier same-module type functions that have already passed validation. It may not reference later same-module type functions in SPEC-E. Acyclic forward-reference topological validation is deferred; mutual recursion remains rejected.

## 9. Registration and Module Boundary

Registration is module-local in SPEC-061.

1. A local `type fn` declaration creates a `TypeComputationHeadId` in the current module identity.
2. Registration is two-phase for each declaration:
   - predeclare/provisionally allocate the current definition's `TypeComputationHeadId` from its header;
   - allow the current definition to resolve its own head while checking equations and recursive RHS calls;
   - allow references only to earlier same-module type functions that have already passed validation;
   - do not publish the current head to later declarations or normalizer tables until validation succeeds.
3. Later same-module type-function references are rejected in SPEC-E with a source-order diagnostic; topological validation for acyclic forward references is deferred.
4. Invalid recursive SCCs or mutually recursive type functions are rejected before publication.
5. The head is available while typechecking later declarations in the same module only after its definition has passed totality validation.
6. `pub type fn` declarations are rejected with a SPEC-F handoff diagnostic.
7. Public ordinary exports from the defining module must not expose local computation heads in signatures, aliases, interface surfaces, or summaries before SPEC-F.
8. Imports must not load type-function equations from dependency module summaries in this phase.
9. If an imported type mentions a computation head from another module before SPEC-F, typechecking must reject normalization with a precise unsupported-public-type-function diagnostic rather than silently treating it as a nominal type.
10. The engine may transport module files to TypeEnv, but it does not own normalization semantics.

## 10. Equation Checking

For each type function:

1. Parameter and return annotations lower to canonical type expressions and must be well-kinded.
2. A pattern in parameter position `i` is checked against parameter type `Pi`.
3. Constructor patterns require `Pi` to resolve to a sealed domain that contains the constructor.
4. Pattern variables and wildcards inherit the checked parameter position's kind/domain constraint.
5. Equation RHS typechecks as a type expression whose kind equals the declared return kind and whose domain/constraint conforms to the declared return type. For example, a `TypeList` return rejects an arbitrary unconstrained `Type` RHS even though both have kind `*`.
6. RHS name resolution first consults the equation's pattern-variable environment. Lowercase pattern variables such as `h`, `t`, and `ys` must not be lowered as nominal type names.
7. Unknown RHS variables are definition-time errors.
8. Recursive and non-recursive type-function applications in RHS positions lower to `TypeFunctionResultExpr::ComputationHeadApp` or an equivalent canonical computation-head carrier.
9. Wrong arity, wrong domain, unknown constructor, unknown pattern variable, result-kind mismatch, and result-domain mismatch are definition-time errors.

## 11. Coverage and Overlap

Coverage and overlap are checked over the full pattern matrix, not per parameter in isolation. For recursive domains, this is a finite symbolic pattern-space analysis with explicit nested refinement, not unbounded recursive-domain expansion.

Definitions:

- A row's **explicit constructor cells** are the sealed-domain positions where the row writes a concrete marker constructor.
- A row's **default cells** are sealed-domain positions where the row writes a variable or `_`.
- A **pattern space** is a finite symbolic tree rooted at each sealed-domain scrutinee. The root is split by the domain's known marker constructors. A constructor field is split further only when some row explicitly writes a nested constructor pattern for that sealed-domain-typed field.
- A **nested residual space** is the symbolic remainder of an explicitly inspected sealed-domain field after earlier rows have claimed some of that field's constructor cases.
- Explicit constructor rows are checked for ordinary overlap against earlier rows at every explicitly inspected path, including nested paths.
- A default row is interpreted as an ordered residual row: it covers only the finite known constructor spaces not already covered by earlier rows at the inspected depth.
- Variables in residual default rows bind the actual known residual constructor normal form selected at reduction time; they do not bind abstract scrutinees and do not make open catch-all reduction legal.
- Nested constructor patterns refine only explicitly written constructor fields using sealed-domain metadata; the checker never expands recursive domains without a written nested pattern.

Rules:

1. Every type function must have at least one sealed-domain parameter position that participates in the coverage matrix.
2. Every closed top-level constructor tuple induced by sealed-domain parameters must be covered by an explicit row or an ordered residual default row, or the definition is rejected.
3. If a row explicitly refines a sealed-domain field nested under a constructor, the checker must cover the symbolic nested residual for that field as well; a row such as `Cons<h, Nil>` covers only the `Cons` cases whose explicitly refined `tail` field is `Nil`.
4. A nested default such as `Cons<h, _>` covers only the nested residual under `Cons` after earlier `Cons<..., Ctor>` rows, and still does not reduce abstract tail variables.
5. Explicit duplicate constructor rows are rejected.
6. A later explicit row whose covered space is already consumed by an earlier default/residual row is rejected as unreachable/overlapping.
7. Wildcard/variable default rows are accepted only if their residual set is non-empty; otherwise emit the empty-default diagnostic.
8. Multiple default rows are legal only when each later default has a non-empty residual space after subtracting all earlier rows.
9. Wildcard/variable default rows do not reduce abstract-variable applications.
10. Nested constructor patterns inside unconstrained `Type` slots are rejected before coverage analysis.

Example accepted default behavior:

```ash
type fn F(xs: TypeList) -> Type {
    case F<Nil> = A;
    case F<_> = B;
}
```

Expected normalization:

```text
F<Cons<X, Y>> => B
F<Xs>         => NeutralComputationApp(F, [Xs], reason = AbstractScrutinee)
```

Example accepted multiple residual defaults over two sealed scrutinees:

```ash
type fn G(xs: TypeList, ys: TypeList) -> Type {
    case G<Nil, _> = A;      -- covers Nil × {Nil, Cons}
    case G<_, Nil> = B;      -- residual covers Cons × Nil
    case G<_, _> = C;        -- residual covers Cons × Cons
}
```

Each default row is accepted because it has a non-empty residual constructor space after subtracting earlier rows. An equivalent later row after the final `G<_, _>` default would be rejected as unreachable/empty.

## 12. Structural Recursion

The first slice uses one explicit decreasing parameter.

A recursive call to the same type function is accepted only if the recursive argument at the decreasing position is a direct structural subcomponent bound by the current equation pattern.

Accepted:

```text
Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>
```

Rejected:

```text
Bad<xs> = Bad<xs>                  -- same argument
Bad<Cons<h, t>> = Bad<Cons<h, t>>  -- rebuilt argument
Bad<Cons<h, t>> = Bad<Reverse<t>>  -- type-function-produced argument
```

Rules:

1. The decreasing parameter name must exist in the header.
2. The decreasing parameter must be a sealed-domain parameter with structural subcomponent metadata available from SPEC-059 domain fields.
3. Every recursive call must be found by recursively walking all canonical RHS children, including nominal application arguments, domain-constructor arguments, projection arguments/bases, and nested computation-head applications.
4. The decreasing argument in a recursive call must be exactly one variable bound as a direct structural subcomponent of the current row's decreasing-parameter pattern.
5. Passing the same variable, a reconstructed constructor, an alias of a subcomponent, or a result of another computation is rejected.
6. Mutual recursion is rejected by a call-graph SCC check or by rejecting unresolved cycles conservatively.
7. Fuel remains an implementation robustness guard only; it is not the semantic termination proof.

## 13. Normalizer Integration

A checked source `type fn` is registered into the SPEC-060 normalizer as a source-backed equation table.

The source-backed table must preserve all semantics of SPEC-060 fixture reduction:

- known-scrutinee `Append<Nil, Ys>`-style reduction, even when non-scrutinee arguments such as `Ys` remain abstract;
- closed recursive reduction;
- open neutral applications when equation selection is blocked by abstract scrutinees;
- partial prefix reduction before neutral tails;
- no inversion under neutral computation heads;
- structured definitional-equality evidence.

SPEC-060 fixture APIs may remain for tests/internal setup, but production source declarations must not be represented as ad-hoc test fixtures after validation.

## 14. Diagnostics

Required diagnostic families ([NEW] variants unless already provided by a future diagnostic umbrella):

- `TypeFunctionPublicExportDeferred`: `pub type fn` requires SPEC-F summaries.
- `TypeFunctionUnknownCaseHead`: `case Other<...>` inside `type fn Append`.
- `TypeFunctionWrongArity`: pattern or application arity mismatch.
- `TypePatternUnknownConstructor`: constructor pattern does not resolve to a sealed-domain marker constructor.
- `TypePatternWrongDomain`: constructor belongs to a different sealed domain.
- `TypePatternRepeatedVariable`: repeated variable in one equation.
- `TypeFunctionNonExhaustive`: uncovered closed constructor tuple(s).
- `TypeFunctionOverlappingEquation`: row overlaps a previous row.
- `TypeFunctionEmptyDefault`: wildcard/default row covers no residual known constructors.
- `TypeFunctionMissingDecreases`: recursive definition without `decreases`.
- `TypeFunctionInvalidDecreases`: decreasing parameter does not exist or is not structurally checkable.
- `TypeFunctionNonDecreasingRecursion`: same/rebuilt/computed recursive argument.
- `TypeFunctionResultKindMismatch`: RHS kind differs from declared return kind.
- `TypeFunctionResultDomainMismatch`: RHS kind matches but RHS violates the declared sealed-domain/domain constraint.
- `TypeFunctionNoSealedScrutinee`: definition has no sealed-domain parameter for SPEC-E structural coverage.
- `TypeFunctionPublicLeakageDeferred`: public ordinary export mentions a local computation head before SPEC-F.
- `TypeFunctionAmbiguousHead`: a name could resolve as both a nominal type constructor and a type-function computation head.
- `TypeFunctionAmbiguousMarkerConstructor`: a name could resolve as both a sealed-domain marker constructor and another type-level head in the expected RHS/pattern position.
- `TypeFunctionUnreachableEquation`: a row is fully covered by an earlier explicit/default row.
- `TypeFunctionForwardReferenceUnsupported`: a type function references a later same-module type function before SPEC-F-style summary/import or a future topo-validation slice.
- `TypeFunctionCrossModuleUnsupported`: imported/public type-function normalization attempted before SPEC-F.

Diagnostics must include the expected shape, the found shape, and one concrete fix where available.

## 15. Acceptance Tests

SPEC-061 is accepted only when tests prove:

1. parser accepts `type fn Append(...) decreases xs { case ... }` with accurate spans;
2. parser dispatches `type fn` before ordinary `type` definitions and does not let `starts_with_type_definition` consume it accidentally;
3. parser rejects malformed case heads, missing semicolons, and `pub type fn` / `pub(crate) type fn` with the SPEC-F handoff diagnostic;
4. core/lowering preserves `TypeComputationHeadId`, parameter metadata, equation order, result expressions, and source anchors;
5. source equation RHS carriers represent `Nil` / `Cons<...>` as sealed-domain marker-constructor apps, not nominal apps;
6. `Append<Cons<A, Nil>, Cons<B, Nil>>` reduces to `Cons<A, Cons<B, Nil>>`;
7. `Append<Xs, Ys>` remains neutral when `Xs` is abstract;
8. catch-all/default rows reduce known residual constructors but not abstract variables;
9. nested pattern definitions cover explicitly inspected nested residual spaces and reject missing nested cases;
10. explicit rows after an earlier default/residual row are rejected when unreachable;
11. positive multiple-default definitions are accepted when each default has a non-empty residual space, and empty default rows / duplicate defaults over empty residual spaces are rejected;
12. partial `Head`-style definitions are rejected when `Nil` is uncovered;
13. overlapping rows are rejected;
14. repeated pattern variables are rejected;
15. wrong-domain constructor patterns are rejected;
16. lowercase marker constructors are parsed/resolved as constructors in expected sealed-domain positions, while lowercase variables remain variables elsewhere;
17. marker-constructor-vs-nominal/type-function ambiguity in RHSs or patterns is rejected;
18. result-domain mismatch is rejected even when kind matches;
19. definitions with no sealed-domain scrutinee are rejected;
20. public ordinary exports that leak local computation heads are rejected before SPEC-F;
21. ambiguous nominal/type-function head names are rejected;
22. missing/invalid `decreases` clauses are rejected for recursive definitions;
23. recursive calls on the same, rebuilt, or type-function-produced argument are rejected;
24. recursive calls nested anywhere in canonical/source RHS children are detected;
25. mutual recursion and later same-module forward references are rejected in SPEC-E, while earlier validated same-module dependencies are accepted;
26. ordinary SPEC-057 type summaries, SPEC-059 sealed domains, and SPEC-060 fixture normalizer tests remain non-regressed;
27. no imported/cross-module type-function equation is normalized before SPEC-F.

## 16. Future Work

SPEC-F will define public module-summary export/import for type functions and domain computation metadata. SPEC-G will define associated recursive type-family computation on top of the same totality and normalizer substrate. SPEC-H will define any later proposition/constraint layer. None of those features are implied by SPEC-061.
