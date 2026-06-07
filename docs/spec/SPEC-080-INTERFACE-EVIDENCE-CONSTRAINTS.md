# SPEC-080: Interface Evidence Constraints

**Status:** Draft
**Date:** 2026-06-08
**Builds on:** [SPEC-033](SPEC-033-MULTI-PARAMETER-INTERFACES.md), [SPEC-034](SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-064](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-078](SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
**Plan:** [PLAN-130](../plan/PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)
**Implementation Tasks:** [TASK-1038](../plan/tasks/TASK-1038-interface-evidence-constraints-packet.md) through [TASK-1048](../plan/tasks/TASK-1048-interface-evidence-constraints-closeout.md)

## 1. Summary

SPEC-080 adds interface-level evidence constraints. An interface may declare evidence that must be available for each valid application of that interface.

The motivating example is the algebra relation between `Monad` and `Applicative`:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

The clause `where M: Applicative` means that `Monad<M>` evidence is valid only when `Applicative<M>` evidence is also available. In other words, `M: Monad` entails `M: Applicative`; `Monad` requires `Applicative`; or `Monad<M>` has an `Applicative<M>` constraint.

This is not an object hierarchy. The constraint does not copy methods, synthesize implementations, or create a subtype relation.

## 2. Motivation

Generic impl constraints are the wrong semantic home for this requirement:

```ash
impl<M : * -> *> Monad<M> where M: Applicative {
    ...
}
```

That form defines a blanket implementation scheme. It can overlap with concrete implementations, implies method bodies can be derived from the weaker evidence, and does not make every `Monad<M>` evidence item require `Applicative<M>` unless an extra global rule is added elsewhere.

The desired rule belongs to the interface itself:

```text
valid evidence Monad<M> requires valid evidence Applicative<M>
```

The type checker must verify this requirement. Ash does not automatically derive `Monad<M>` from `Applicative<M>`, and it does not derive `Applicative<M>` implementation bodies from `Monad<M>`. It only checks that required evidence exists and makes verified required evidence available where the stronger evidence is in scope.

## 3. Surface syntax

### 3.1 Interface declarations

The grammar extends interface declarations with an optional evidence-constraint tail before the body:

```text
interface-def = visibility? "interface" name interface-params?
                interface-constraint-tail?
                "{" interface-item* "}"

interface-constraint-tail = "where" interface-constraint-list
interface-constraint-list = interface-constraint ("," interface-constraint)*
interface-constraint = identifier ":" interface-type-application
interface-type-application = identifier [ "<" type-expr ("," type-expr)* ">" ]
```

Accepted examples:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}

interface Traversable<T : * -> *> where T: Functor, T: Foldable {
    traverse(T<Int>) -> List<Int>
}

interface RichMap<M, K, V> where K: Eq, V: Clone {
    get(M, K) -> Option<V>
}
```

The MVP reuses the existing `T: Interface` shape from impl `where` clauses. The subject must name an interface parameter introduced by the same interface declaration. The right-hand side names an interface evidence application. For single-parameter interfaces, `M: Applicative` is shorthand for `Applicative<M>` evidence.

### 3.2 Rejected syntax

The MVP does not add general proposition syntax to interface declarations. These forms remain rejected unless a later spec explicitly enables them:

```ash
interface Bad<T> where T == U { ... }
interface Bad<T> where T != U { ... }
interface Bad<T> where NonEmpty<T> { ... }
interface Bad<T> where T: Applicative + Monad { ... }
```

The MVP also does not add object-style extension syntax:

```ash
interface Monad<M> : Applicative<M> { ... }     -- rejected
interface Monad<M> extends Applicative<M> { ... } -- rejected
```

Ash documentation must use evidence vocabulary for this feature: “requires”, “entails”, “evidence constraint”, and “required evidence”. It must not describe the relation as object hierarchy or subtype behavior.

## 4. Semantic model

### 4.1 Evidence validity

An interface application is a requested evidence item:

```text
I<A1, ..., An>
```

If the interface declaration has constraints, the type checker computes the required evidence applications by substituting the interface arguments into each constraint.

Example:

```ash
interface Monad<M : * -> *> where M: Applicative { ... }
```

For requested evidence `Monad<Option>`, the required evidence set is:

```text
Applicative<Option>
```

For requested evidence `Monad<Result<_, E>>`, the required evidence set is:

```text
Applicative<Result<_, E>>
```

A requested evidence item is well formed only if all of its required evidence is available in the same evidence environment.

### 4.2 Directionality

The entailment direction is from the constrained interface to its required evidence:

```text
M: Monad entails M: Applicative
```

The reverse direction is never inferred:

```text
M: Applicative does not entail M: Monad
```

The type checker verifies constraints; it does not synthesize implementations.

### 4.3 Registration of impl evidence

When registering an `impl` for an interface with evidence constraints, the type checker must verify every substituted constraint.

This succeeds only if `Applicative<Option>` evidence is already registered or registered as part of the same checked module batch before the `Monad<Option>` evidence is accepted:

```ash
impl Applicative<Option> { ... }
impl Monad<Option> { ... }
```

This fails:

```ash
interface Monad<M : * -> *> where M: Applicative { ... }
impl Monad<Option> { ... }  -- error: missing Applicative<Option> evidence
```

For generic impl schemes, the constraint may be discharged by the impl's own `where` clause or by an in-scope stronger bound that entails the required evidence:

```ash
impl<M : * -> *> Monad<M> where M: Applicative {
    ...
}
```

This generic impl form remains a normal implementation scheme. It is not created by the interface constraint, and it is not required for ordinary concrete impls. It should not be used to express the interface-level requirement itself.

### 4.4 Generic environments

When a generic context has a bound for a constrained interface, the type checker may use the required evidence inside the same context.

```ash
fn lift_one<M : * -> *>() -> M<Int>
    where M: Monad
{
    Applicative::pure(1)
}
```

The `M: Monad` bound gives the context verified access to `M: Applicative` because `Monad` declares that evidence constraint. This is still verification, not derivation: the `M: Monad` evidence is accepted only if the required `M: Applicative` evidence is known to hold.

### 4.5 Cycles

Evidence constraints must not form unguarded cycles.

Rejected examples:

```ash
interface A<T> where T: A { ... }

interface A<T> where T: B { ... }
interface B<T> where T: A { ... }
```

The initial implementation may reject any cycle in the interface evidence-constraint graph. A later spec may distinguish benign recursive evidence if a sound model is needed.

### 4.6 Overlap and coherence

Interface-level constraints do not create impl schemes. Therefore they do not by themselves introduce overlap.

Overlap checking for impl schemes remains owned by SPEC-034 and SPEC-067. Constraint verification runs after candidate evidence is identified and before evidence is accepted or used.

## 5. Type checker requirements

The type checker must:

1. store interface-level evidence constraints in `InterfaceInfo` or an equivalent evidence metadata carrier;
2. validate that each constraint subject names an interface parameter;
3. validate that each referenced interface exists and has compatible kind/arity;
4. substitute concrete or generic interface arguments into constraints during evidence registration and lookup;
5. verify required evidence for concrete impl registration;
6. verify or defer through explicit where-bound evidence for generic impl schemes;
7. expose required evidence to generic bodies when the stronger constrained evidence is in scope;
8. reject missing required evidence with a diagnostic naming the constrained interface, requested evidence, missing required evidence, and source span;
9. reject cycles in the interface evidence-constraint graph;
10. never synthesize `Monad` evidence from `Applicative` evidence or method bodies from required evidence.

## 6. Parser and AST requirements

`ash-parser` must parse the interface `where` tail and preserve spans.

The parser must accept final surface examples such as:

```ash
interface Applicative<F : * -> *> {
    pure(Int) -> F<Int>
}

interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

The parser must reject unsupported proposition forms at the interface declaration site with an explicit diagnostic or parse error. Parser tests must include non-zero positive and negative cases.

Surface and core carriers may reuse or generalize existing `WhereBound`/`InterfaceBound` structures, but they must preserve enough information to distinguish:

- impl `where` constraints, which constrain an implementation scheme; and
- interface evidence constraints, which constrain validity of every evidence item for that interface.

## 7. Standard algebra requirement

After this feature lands, standard algebra interfaces must use interface-level evidence constraints for accepted algebra requirements. The first required relation is `Monad` requiring `Applicative`:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

Additional accepted algebra requirements in this phase are:

```ash
interface Applicative<F : * -> *> where F: Functor {
    pure(Int) -> F<Int>
    apply(F<(Int) -> Int>, F<Int>) -> F<Int>
}

interface Monoid<A> where A: Semigroup {
    empty() -> A
    append(A, A) -> A
}
```

The implementation must then prove:

1. every accepted stdlib `Monad<K>` evidence has corresponding `Applicative<K>` evidence;
2. every accepted stdlib `Applicative<K>` evidence has corresponding `Functor<K>` evidence;
3. every accepted stdlib `Monoid<T>` evidence has corresponding `Semigroup<T>` evidence;
4. attempting to register constrained evidence without its required evidence fails;
5. generic contexts may use required evidence from stronger constrained evidence;
6. reverse entailment is rejected for every relation;
7. no blanket generic impl is introduced as part of these migrations.

No separate `Functor`/`Monoid` evidence constraint is part of this specification. The monoid-in-endofunctors interpretation belongs to the `Monad` relation itself; it is not modeled as an extra Ash interface constraint between `Functor<F>` and scalar `Monoid<A>` evidence.

## 8. Diagnostics

Required diagnostic families:

- `InterfaceEvidenceConstraintUnknownSubject` — the constraint subject is not an interface parameter.
- `InterfaceEvidenceConstraintUnknownInterface` — the required evidence interface does not exist.
- `InterfaceEvidenceConstraintWrongKind` — the substituted subject does not match the required interface parameter kind.
- `InterfaceEvidenceConstraintWrongArity` — the required evidence application has the wrong arity.
- `InterfaceEvidenceConstraintMissingEvidence` — an impl or lookup attempts to accept constrained evidence without required evidence.
- `InterfaceEvidenceConstraintCycle` — interface evidence constraints form a cycle.
- `InterfaceEvidenceConstraintUnsupportedSyntax` — a generalized proposition or object-style extension form appears at the interface declaration site.

Implementations may choose exact enum names, but diagnostics must include stable enough structured information for focused tests to assert the failing interface, requested evidence, and missing required evidence.

## 9. Acceptance matrix

| ID | Case | Expected result |
|----|------|-----------------|
| IEC-1 | Parse `interface Monad<M : * -> *> where M: Applicative { ... }` | Surface AST preserves one interface evidence constraint with span |
| IEC-2 | Parse multiple constraints `where T: Functor, T: Foldable` | Both constraints preserved in source order |
| IEC-3 | Parse `where T == U` on interface | Rejected as unsupported interface constraint syntax |
| IEC-4 | Register `interface Monad<M : * -> *> where M: Applicative` before `Applicative` exists | Rejected or deferred until batch validation, but cannot silently accept an unknown required interface |
| IEC-5 | Register `impl Monad<Option>` without `impl Applicative<Option>` | Rejected with missing required evidence |
| IEC-6 | Register `impl Applicative<Option>` then `impl Monad<Option>` | Accepted if method conformance passes |
| IEC-7 | Generic context `where M: Monad` calls `Applicative::pure` | Accepted through verified entailment |
| IEC-8 | Context `where M: Applicative` calls `Monad::bind` | Rejected; no reverse entailment |
| IEC-9 | Cyclic constraints `A requires B`, `B requires A` | Rejected before evidence use |
| IEC-10 | `std::algebra::Monad` declares `where M: Applicative` | Final stdlib source parses/checks through module import path |
| IEC-11 | No blanket generic `Monad` impl is created by the constraint | Impl scheme inventory has no synthesized blanket impl |
| IEC-12 | Existing SPEC-034 impl `where` constraints continue to parse/check | Non-interference with generic impl schemes |

## 10. Non-goals

This spec does not add:

- object hierarchy, subtype relations, or method copying;
- default method bodies;
- automatic implementation derivation;
- proof search for missing evidence;
- generalized proposition syntax in interface declarations beyond interface-bound evidence constraints;
- `+` bound composition;
- specialization or overlapping impl resolution changes;
- law proving for algebra interfaces.

## 11. Implementation tasks

- [TASK-1038](../plan/tasks/TASK-1038-interface-evidence-constraints-packet.md): SPEC/PLAN packet.
- [TASK-1039](../plan/tasks/TASK-1039-interface-evidence-constraints-audit-gate.md): audit parser/typechecker/stdlib seams and freeze exact commands.
- [TASK-1040](../plan/tasks/TASK-1040-interface-constraint-parser-surface.md): parser/surface support.
- [TASK-1041](../plan/tasks/TASK-1041-interface-constraint-core-lowering-and-summaries.md): core/lowering/summary carriers if needed.
- [TASK-1042](../plan/tasks/TASK-1042-typeenv-interface-constraint-registration.md): TypeEnv registration and concrete evidence verification.
- [TASK-1043](../plan/tasks/TASK-1043-generic-entailment-and-evidence-lookup.md): generic entailment and lookup integration.
- [TASK-1044](../plan/tasks/TASK-1044-stdlib-monad-applicative-constraint.md): stdlib Monad -> Applicative migration and corpus updates.
- [TASK-1045](../plan/tasks/TASK-1045-stdlib-applicative-functor-constraint.md): stdlib Applicative -> Functor migration and corpus updates.
- [TASK-1046](../plan/tasks/TASK-1046-stdlib-monoid-semigroup-constraint.md): stdlib Monoid -> Semigroup migration and corpus updates.
- [TASK-1048](../plan/tasks/TASK-1048-interface-evidence-constraints-closeout.md): diagnostics, broad verification, independent review, and closeout.

## 12. Changelog

### 2026-06-08

- Initial draft for interface-level evidence constraints, centered on `Monad<M>` requiring `Applicative<M>` without object-hierarchy wording or automatic derivation.
