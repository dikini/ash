# DESIGN-034: Total Compile-Time Type Computation

**Status:** Draft design note, SPEC-A/Phase 109 substrate implemented
**Date:** 2026-04-30
**Related:** DESIGN-031, DESIGN-032, SPEC-034, SPEC-035, SPEC-054, SPEC-055, SPEC-057

## 1. Summary

Ash should support a disciplined form of compile-time computation over types so
that Rust type-state and typelist patterns become fluent, explainable, and safe.
The target is not a Rust-like trait solver used accidentally as a Turing-complete
computation engine. If Ash adds compile-time type computation, it must be total,
terminating, and normalizing.

This design is adjacent to, but distinct from, the higher-kinded `Monad` /
generalized `do` work. Constructor-kinded parameters, type constructor
unification, generic impls, where bounds, and expression-general bidirectional
checking help with advanced generic programming. They are not sufficient by
themselves for type-level computation. The missing substrate is type-level
reduction: explicit normalization of type expressions, definitional equality
between normalized types, and a constrained proof/constraint layer around that
normalization.

Besedarium-style protocol encodings are a useful stress case because they expose
hard-to-express Rust type-state problems: phantom type witnesses, type-level
ASTs, typelists, projection-like transforms, well-formedness checks, and equality
of computed types. Protocols are not the specific product target of this design.
The broader target is natural expression of compile-time operations over
ADT-shaped phantom types, typelists, finite maps, state witnesses, and similar
static structures.

This document is a design anchor for a future **specification set**, not a
single implementation spec. Section 16 defines the intended spec packets and
ordering. Implementers MUST NOT interpret this document as authorization to jump
directly to `type fn` or recursive associated type-family implementation before
the Tier 0 substrate specs exist.

The most important review clarification is this:

```text
Total normalization does not mean every generic type expression reduces to
ordinary constructors. It means normalization always terminates and returns a
canonical normal form. For open generic terms, that normal form may contain
neutral/stuck type-function applications or rigid associated projections.
```

## 2. Terminology

This design uses the following terms consistently:

- **Type expression**: a type-level expression manipulated by the type checker,
  including ordinary types, type variables, constructor applications,
  type-function applications, and associated projections.
- **Type function**: a closed, compiler-checked set of equations that computes a
  type expression from type arguments.
- **Type family**: a type-level computation associated with an interface or impl
  family. This is the interface-integrated version of type computation.
- **Associated projection**: a reference to an associated type result, such as
  `T::Item` or a future explicit interface-qualified equivalent.
- **Normal form**: the canonical result of reducing all reducible type-level
  computation in the current context.
- **Neutral/stuck normal form**: a normal form that contains an unreduced
  application blocked by abstract type information rather than by an error.
- **Rigid projection**: an associated projection that cannot reduce in the
  current context because no unique concrete impl/family instance has been
  selected. Rigid projections are neutral forms.
- **Definitional equality**: equality after both sides are normalized and
  canonicalized.
- **Constraint/proof solving**: evidence search for propositions not reducible by
  normalization alone.
- **Closed type-level domain**: a sealed set of type-level constructors known to
  the compiler for coverage and structural-recursion checking.
- **Solver fuel**: an implementation robustness limit. Fuel is not the semantic
  reason accepted type computations terminate.

## 3. Design Principles

### 3.1 Total, terminating, normalizing

Compile-time type computation must have a normal form for every accepted input:

```text
normalize(type_expr) terminates and returns a canonical type expression
```

This is a semantic requirement, not just an implementation safeguard. A solver
fuel limit is useful for diagnostics and compiler robustness, but fuel exhaustion
should be treated as an implementation failure or rejected program, not as the
normal meaning of accepted type computation.

The normal form of an open expression may contain neutral/stuck forms. For
example, if `Xs` is an abstract type variable, the compiler cannot choose the
`Nil` or `Cons` equation for `Append<Xs, Ys>`. The normalized result is a
canonical neutral form:

```text
normalize(Append<Xs, Ys>) = Append<Xs, Ys>
  reason: Xs has abstract shape
```

This is still total: normalization terminates. The result is not a constructor
normal form, but it is a normal form.

### 3.2 Keep computation distinct from proof search

Ash should distinguish three related mechanisms:

1. **Type computation**: reducing type-level functions/families to normal forms.
2. **Definitional equality**: comparing normalized type expressions.
3. **Constraint/proof solving**: satisfying propositions such as membership,
   disjointness, well-formedness, interface bounds, or type disequality.

These should not collapse into one opaque trait solver. Computation should be
predictable and explainable before richer proof search is added.

### 3.3 Prefer direct type functions over accidental trait computation

Rust demonstrates that recursive traits and associated types can encode powerful
compile-time computation, but the result is often indirect and difficult to
explain. Ash should prefer direct, structurally recursive type functions or a
closed-world type-family subset as the first-class computation model.

Interfaces and associated types remain important, but the compiler should not
force all type-level computation to be expressed as impl-search side effects.

### 3.4 General substrate, not protocol-specific machinery

Protocol/session-type examples are valuable stress tests, not the primary target.
The substrate should also cover:

- typelists and type-level trees;
- finite maps and sets at the type level;
- type-state builders and state-machine witnesses;
- capability/resource state transitions;
- workflow/process static summaries;
- protocol-like AST transforms when needed.

## 4. Relationship to Monad, HKT, and Inference Work

The long-term generalized do/comprehension design target needs higher-kinded
generic programming:

- constructor-kinded parameters such as `M : * -> *`;
- partial constructor application / holes such as `Result<_, E>`;
- generic impl blocks;
- where bounds;
- constraint-based inference;
- expression-general bidirectional checking.

The implemented SPEC-054/SPEC-055 MVP deliberately does not yet provide all of
those features. It supports Act/Proc builtin dictionaries, rejects target
arguments/holes such as `Result<_, E>`, and defers user-defined `Monad<M>`, pure
List/Option/Result dictionaries, and target inference.

Future pseudocode for the intended abstraction looks like:

```text
interface Monad<M : * -> *> {
    pure : A -> M<A>
    bind : M<A>, (A -> M<B>) -> M<B>
}
```

This is not current Ash surface syntax. In particular, constructor-kinded
interface parameters such as `M : * -> *` remain future work.

Those features allow abstractions such as `Monad<Option>` and
`Monad<Result<_, E>>`. They do not by themselves compute new types such as:

```text
Append<Cons<A, Nil>, Cons<B, Nil>>
  == Cons<A, Cons<B, Nil>>
```

or:

```text
Project<GlobalProtocol, Role> == LocalProtocol
```

For those cases Ash needs type-level functions or type families plus
normalization and equality.

## 5. Motivating Shape: Type-Level Data and Typelists

A typical Rust type-state encoding uses zero-sized marker types and `PhantomData`
to build a type-level syntax tree:

```text
Nil
Cons<H, T>
State<Open>
State<Closed>
TChanSend<S, R, C, L, Msg, P, AIO>
TChanRecv<R, S, C, L, Msg, P, AIO>
```

The runtime values carry little or no data. The important information is in the
type parameters. Useful operations then transform or inspect those types at
compile time.

### 5.1 Marker types vs promoted data constructors

The examples above should not be read as assuming Haskell-style promoted ADT
constructors. Ash does not currently promote value constructors into type-level
constructors. Existing ADT variants are value constructors whose expression type
is the parent enum/type, not type-level data constructors.

For the first slice, this design chooses **nominal marker constructors grouped
by an explicit sealed type-level domain**. Examples such as:

```text
Nil
Cons<H, T>
Open
Closed
State<S>
```

are type-level marker constructors with canonical type-definition identities and
sealed-domain metadata. They are not promoted enum variants, and they do not
introduce runtime value constructors unless a later ordinary value-level type
declaration explicitly does so.

The first slice should treat a declaration like:

```text
sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}
```

as introducing or registering the type-level constructors `Nil` and `Cons` in a
closed domain named `TypeList`. The spec may decide whether those constructors
also have zero-sized runtime inhabitants, but runtime inhabitance is irrelevant
to type-level computation. Coverage, structural recursion, and type-level
pattern matching use the sealed-domain constructor metadata, not runtime value
constructors.

A later spec may choose to add promoted data constructors or full named data
kinds, but that is a separate feature. Until then, `Nil` and `Cons<H, T>` in this
document mean nominal marker constructors in a sealed domain.

### 5.2 Example computation

Example typelist computation, written as mathematical equations:

```text
Append(Nil, ys) = ys
Append(Cons(h, t), ys) = Cons(h, Append(t, ys))
```

After normalization:

```text
Append(Cons<A, Nil>, Cons<B, Nil>)
  ==> Cons<A, Cons<B, Nil>>
```

This normalized result should participate in ordinary type equality. A function
that expects `Cons<A, Cons<B, Nil>>` should accept a value whose type contains the
computed form, after normalization.

## 6. Proposed Feature Tiers

### Tier 0: Required substrate before type functions

Tier 0 is a **blocking prerequisite** for every later tier. Agents and humans
MUST NOT plan `type fn`, recursive associated type families, or type-level
normalization work as if this substrate already exists. It does not.

Tier 0 must specify and implement at least:

1. **A unified type-declaration pipeline.** Ordinary type declarations must flow
   through the normal parser/module/lowering/export path instead of relying on
   ad-hoc source-snippet extraction. Type functions and type-level domains MUST
   NOT be added on top of the current fragmented type-definition path.
2. **Closed type-level domains.** The compiler must have explicit metadata for
   domain identity, constructor sets, constructor arity, structural fields,
   field kinds/domains, visibility, and module-summary export.
3. **A distinct type-function application representation.** Reducible
   type-function calls and neutral/stuck type-function applications MUST NOT be
   encoded as ordinary nominal `Type::Constructor` nodes. Doing so would conflate
   nominal data constructors with computation heads and make equality, arity,
   normalization, and diagnostics unsound or unclear.
4. **A normal-form / neutral-form representation.** The typechecker needs a
   representation for reduced constructor normal forms, neutral type-function
   applications, and rigid associated projections.
5. **An environment-aware definitional-equality API.** Existing low-level
   unification is not enough because normalization needs access to type-function
   definitions, sealed domains, associated-family evidence, aliases, and module
   visibility.
6. **Module semantic summaries.** Public type-level domains, type-function
   equations, associated-family metadata, and any public normalized facts must be
   exported through stable module summaries. This should be a general semantic
   summary owned by the core/module pipeline, not bolted onto capability-specific
   export structures.

A future implementation should treat these as a hard gate. If Tier 0 is missing,
then Tier 1/Tier 2 plans are not implementation-grade.

Illustrative first-slice domain shape:

```text
sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}
```

The concrete syntax is not chosen here, but the metadata is mandatory. Ordinary
`Type::Constructor` applications of unrelated nominal types are insufficient for
coverage and structural recursion checks.

In type-level examples below, angle-bracket forms denote type expressions.
Parenthesized equations are mathematical notation only; they do not imply
runtime calls or promoted value constructors.

### Tier 1: Structural type functions

Tier 1 introduces direct type-level functions over closed type-level domains.

Illustrative syntax only:

```text
type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
{
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
}
```

The eventual spec must choose concrete surface syntax. The important semantics
are independent of whether equations are written with `Append(...)` or
`Append<...>` notation.

Normative intent:

- equations match on type constructors in a closed type-level domain;
- equation patterns are first-order constructor patterns over type expressions;
- pattern variables are linear unless a later equality-guard feature is added;
- recursive calls must be structurally smaller according to a declared or
  inferred decreasing parameter;
- every accepted application normalizes to either a reduced constructor form or
  a canonical neutral/stuck normal form;
- normalized forms participate in definitional equality;
- equation sets must be exhaustive for declared closed domains, or provide an
  explicit catch-all case;
- overlapping equations are rejected in the first slice unless a later spec adds
  a formal specificity/priority rule.

Initial Tier 1 should avoid:

- arbitrary equality/disequality guards;
- overlapping open-world equations;
- unrestricted recursion;
- mutual recursion;
- lexicographic or size-change termination unless explicitly specified;
- runtime value dependence;
- compile-time execution of ordinary Ash expressions.

### Tier 2: Associated type families integrated with interfaces

Tier 2 connects type computation to generic interfaces. This tier is useful, but
it is deliberately **not** the primitive first slice. Direct sealed-domain
`type fn` is the primitive normalization calculus. Associated type-family
computation is admitted only after the direct normalizer, neutral forms,
definitional equality, sealed domains, and module summaries are specified.

The following syntax is future pseudocode and intentionally extends the current
SPEC-034/SPEC-035 model:

```text
interface Append<Xs, Ys> {
    type Out;
}

impl<Ys> Append<Nil, Ys> {
    type Out = Ys;
}

impl<H, T, Ys> Append<Cons<H, T>, Ys>
where
    Append<T, Ys> is implemented
{
    type Out = Cons<H, <Append<T, Ys>>::Out>;
}
```

#### Relationship to SPEC-035

Current SPEC-035 projections are unary/base-shaped. The current implementation
conceptually models projections like:

```text
Type::Associated { interface, base, name }
```

and surface syntax such as `S::Ok` relies on a bound or selected impl to explain
which interface owns `Ok`. DESIGN-034 generalizes this to explicit
interface-application projections:

```text
Projection {
    interface: InterfaceId,
    args: Vec<Type>,
    assoc: Name,
}
```

A future Tier 2 spec MUST define an elaboration from current `T::Assoc` /
`S::Ok`-style syntax into this generalized projection form and choose at least
one disambiguated surface for multi-argument families, for example:

```text
<Append<Xs, Ys>>::Out
Append<Xs, Ys>::Out
Append::Out<Xs, Ys>
```

The exact spelling is deferred; the internal identity is not. Associated
projection identity must be based on canonical interface identity, canonical
argument spine, and associated name, not only on a textual base type.
SPEC-035's ambiguity rule for same-named associated types remains in force until
a future spec replaces it with the generalized projection syntax.

#### Simple associated types vs computable associated families

Ash must keep two cases separate:

1. **Simple associated types**: a selected impl supplies an associated output.
   SPEC-035-style substitution is enough for this case.
2. **Computable associated families**: a family of impls recursively computes an
   associated output. This requires the total normalization calculus, coverage,
   coherence, structural decreasingness, neutral forms, and module summaries.

A future implementation MUST NOT treat the current selected-scheme substitution
helper as the recursive associated-family normalizer. It is a local substitution
mechanism, not a normalization engine.

#### Projection reduction rules

An associated projection may reduce only when all of the following hold:

1. The projection elaborates to a canonical generalized projection:

   ```text
   Projection { interface, args, assoc }
   ```

2. The relevant associated member is declared as part of a computable sealed
   family, or the selected impl is a simple non-recursive associated-output case.
3. Impl/family selection is coherent and unique under the visible module summary.
4. Selection is not declaration-order dependent and does not rely on
   specialization unless a later spec defines specialization formally.
5. The selected family instance passes the same coverage, overlap, and
   structural-decreasingness checks as a direct type function.
6. Any recursive projection in the associated output decreases according to the
   family's declared decreasing parameter over a sealed domain.

"Sufficiently known inputs" means the projection arguments contain enough
canonical head-shape information to select exactly one family instance. The
arguments may still contain abstract variables that flow into the output.

Important examples:

```text
-- reducible under a unique generic impl
impl<X> Iterator<List<X>> {
    type Item = X;
}

normalize(<Iterator<List<A>>>::Item) = A
normalize(<Iterator<List<X>>>::Item) = X
```

The second reduction is allowed even though `X` is abstract, because the outer
argument shape `List<_>` uniquely selects the generic impl.

By contrast:

```text
fn f<T>(x: T) -> T::Item
where
    T: Iterator
```

keeps `T::Item` as a rigid projection in the generic body unless additional
information selects a concrete impl or computable family instance. A where-bound
provides evidence that an associated type exists; it is not, by itself, a
termination proof or a concrete equation.

#### Recursive associated families

Recursive associated-family computation is admitted only for sealed/coherent
family sets. A future Tier 2 spec must define:

- the family identity and sealed impl/equation set;
- the decreasing parameter(s) and their sealed domains;
- the pattern space used for coverage and overlap checking;
- the call graph used for recursive projection detection;
- the exact rule for recursive calls nested under constructors, projections, or
  other type-function applications;
- public/private equation export in module summaries;
- diagnostics for ambiguous selection, rigid projection, non-decreasing
  recursion, and non-exhaustive family coverage.

Current SPEC-034 generic impl resolution is not this substrate. Its recursion
limit is an implementation guard, not a semantic termination proof. Ordinary
open interface impl sets may continue to serve method dispatch and simple
associated outputs, but they are not automatically computable type families.

Preferred implementation model:

1. Specify and implement direct sealed-domain type functions first.
2. Represent associated projections using generalized `Projection` nodes.
3. For computable sealed families, elaborate selected family projections into
   the same normalization/equality engine used by direct type functions.
4. Leave unresolved or open-world projections as rigid neutral normal forms.

#### Associated-family equality

Rigid associated projections compare structurally by canonical identity:

```text
Projection(interface=I, args=[A, B], assoc=Out)
```

is equal to another rigid projection only when the canonical interface identity,
associated name, arity, and normalized argument spine are equal. The equality
judgment does not perform impl search under unrelated neutral heads and does not
invent evidence to make two projections equal.

### Tier 3: Constraint and proposition layer

Tier 3 adds proof/search predicates around normalized types:

```text
Contains<Role, Roles>
Disjoint<ChannelsA, ChannelsB>
WellFormed<P>
Project<G, R> == L
X != Y
```

This tier is intentionally separate from Tier 1 computation. Equality can use
normalization directly. Richer propositions may require proof evidence,
closed-world search, or domain-specific solvers.

Initial constraints should be conservative:

- type equality after normalization;
- interface bounds;
- explicit well-formedness predicates;
- no inversion of type functions to solve unknown inputs;
- disequality, if present, limited to closed normal forms or obvious
  constructor-disjointness cases.

For example, the first slice may know:

```text
Cons<A, T> != Nil
```

because `Cons` and `Nil` are distinct constructors of a closed domain. It should
not attempt to solve:

```text
X != Y
Append<Xs, Ys> != Nil
F<X> == Y
```

by guessing assignments for unknowns.

## 7. Kinding and Type Expression Model

The type system must distinguish ordinary types from type constructors and type
functions.

Useful kinds include:

```text
*                    ordinary inhabited/runtime type
* -> *               unary type constructor
* -> * -> *          binary type constructor
TypeList             optional domain kind for typelists, if introduced
TypeBool             optional type-level boolean kind, if introduced
```

Current Ash kind support is narrower than this design target. The current kind
substrate is centered on ordinary types and arrows. Named kinds such as
`TypeList`, type-level booleans, or promoted user data kinds require explicit
kind representation, parser syntax, diagnostics, and type-expression checking.
They are not just aliases.

Long-term design choice:

- keep only a small structural kind language (`*`, arrows, and sealed nominal
  marker domains), or
- later introduce promoted data kinds for user-defined type-level domains.

The first-slice decision is already fixed by this design: use sealed nominal
marker domains, not promoted data kinds. Any type function that pattern-matches
on constructors should declare a closed input domain. Plain kind `*` is not
sufficient for exhaustiveness or useful diagnostics unless the matched
constructors are known to form a closed family.

Although `Kind::Arrow` exists internally, Ash does not yet have a complete
source-level kind checker for user type parameters, constructor parameters,
partial application, type holes in all type positions, or general constructor
arity/kind validation. Existing constructor values mostly represent fully
applied proper types. Any spec derived from this design MUST explicitly add the
missing kind/arity machinery instead of assuming the internal `Kind` enum is
sufficient.

## 8. Normalization, Neutral Forms, and Equality

The compiler needs a normalization judgment:

```text
Γ ⊢ τ ⇓ τ_norm
```

and a definitional equality judgment:

```text
Γ ⊢ τ1 ≡ τ2
  iff canonical_normalize(τ1) == canonical_normalize(τ2)
```

For example:

```text
Γ ⊢ Append<Cons<A, Nil>, Cons<B, Nil>> ⇓ Cons<A, Cons<B, Nil>>
Γ ⊢ Append<Cons<A, Nil>, Cons<B, Nil>> ≡ Cons<A, Cons<B, Nil>>
```

### 8.1 Open terms and neutral normal forms

Normalization is defined for both closed and open type expressions. For open
expressions, reduction proceeds until no known equation applies. Irreducible
applications headed by type variables, abstract associated projections, or type
functions whose scrutinized argument is unknown are retained as neutral normal
forms.

Examples:

```text
normalize(Append<Xs, Ys>) = Append<Xs, Ys>
  when Xs is abstract

normalize(T::Out) = T::Out
  when T is generic and no concrete impl is selected
```

These neutral forms are canonical and may participate in definitional equality
by structural comparison. They are not errors. A later substitution, monomorphic
instantiation, or selected impl may unblock them.

### 8.2 Equality does not invert type functions

Definitional equality normalizes and compares. It does not solve equations by
inverting type functions.

For example:

```text
Append<Xs, Ys> == Cons<A, Nil>
```

should not infer `Xs` or `Ys` in the first slice. Similarly, for a **neutral
type-function application**:

```text
F<X> == F<Y>
```

definitional equality does not imply `X == Y` unless a later spec adds checked
injectivity or an explicit inversion rule for `F`. This does not change ordinary
nominal constructor unification: existing same-headed nominal constructors such
as `Pair<X, Z>` and `Pair<Y, Z>` may still decompose under the standard unifier.

The first spec should use the following boundary between definitional equality
and unification:

- definitional equality normalizes both sides and compares canonical forms;
- a top-level inference meta-variable may be bound to a normalized neutral
  type-function application, subject to kind and occurs checks;
- unification may solve ordinary meta-variables where the variable itself is the
  unknown;
- unification must not solve underneath neutral type-function heads;
- same-headed neutral type-function applications compare equal only when their
  function identity, arity, and normalized argument spines compare equal under a
  non-solving equality check;
- identical rigid projections compare equal only when their canonical interface
  identity, associated name, arity, and normalized argument spine are identical;
- occurs checks run after the normalization required by the current judgment;
- ordinary same-headed nominal constructor unification may continue to decompose
  constructor arguments, because nominal constructors are data heads rather than
  computation heads.

### 8.3 Canonical equality details

Definitional equality is kind-directed equality of canonical normal forms. The
first spec must decide the exact canonicalization policy, including:

- alpha-equivalence for type binders/type lambdas if they are admitted;
- alias expansion or alias preservation;
- equality of neutral/stuck forms;
- equality of rigid associated projections;
- whether normalization is weak-head, full, or demand-driven at each use site;
- occurs-check and cycle behavior.

Recommended default: normalize enough to decide the current equality question,
using canonical definition identities for comparison. Arguments inside neutral
applications and rigid projections should be recursively normalized to their
canonical comparison forms, even if diagnostics preserve the user-written shape.
Preserve user aliases in diagnostics where possible, and provide an expansion
trace on demand.

Alias handling is a correctness issue, not merely a display issue. The first spec
should resolve names to canonical definition identities before equality, expand
transparent aliases before type-function pattern matching, preserve aliases only
in diagnostic rendering where possible, and reject aliases as pattern
constructors unless they expand to a known domain constructor.

### 8.4 Forcing points

Normalization should be used by unification carefully. The typechecker should
avoid eagerly normalizing every type expression if that creates performance or
diagnostic problems, but equality and expected-type checks must be able to force
normalization at well-defined points.

Potential forcing points:

- checking an expression against an expected type;
- resolving associated type projections after concrete impl selection;
- comparing function return types;
- satisfying equality constraints;
- validating impl head overlap/coherence;
- producing final user-facing inferred types.

## 9. Termination Discipline

Accepted type computations must terminate. Candidate restrictions:

1. **Structural recursion**: each recursive call must use a syntactic subpart of
   a structurally decreasing argument.
2. **Declared decreasing parameters**: each recursive type function/family
   declares or infers one or more decreasing parameters.
3. **Closed equation sets**: a type function's equations are known together and
   can be checked as a unit.
4. **No arbitrary value-level computation**: type functions compute over type
   expressions, not runtime expressions.
5. **No unrestricted open recursion through impl search in the first slice**:
   recursive associated type families require a conservative decreasingness
   check or an explicit deferral.

First-slice structural recursion rule:

1. Each recursive type function declares or infers one decreasing parameter.
2. Recursive calls are accepted only when that argument is a direct structural
   subcomponent bound by the current equation pattern.
3. Calls on the same argument, reconstructed arguments, or arguments obtained
   from another type function are rejected.
4. Mutual recursion, lexicographic recursion, and size-change termination are
   deferred unless explicitly specified.

Example accepted shape:

```text
type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
{
    case Append<Nil, ys> = ys;
    case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
}
```

The recursive call `Append<t, ys>` is accepted because `t` is a direct structural
subcomponent of the current decreasing parameter `Cons<h, t>`.

Example rejected shapes:

```text
Bad(xs) = Bad(xs)                  -- same argument
Bad(Cons<h, t>) = Bad(Cons<h, t>)  -- rebuilt argument
Bad(Cons<h, t>) = Bad<Reverse<t>>  -- result of another type function
```

The compiler may still maintain solver fuel for robustness and diagnostics, but
fuel is not the semantic termination story.

## 10. Coverage, Partiality, and Equation Matching

Total type functions require coverage rules.

For closed input domains, a type-function equation set must cover every
constructor of each scrutinized domain, or provide an explicit catch-all case.
A non-exhaustive type function is rejected in the first slice.

Example rejected shape:

```text
type fn Head(xs: TypeList) -> Type {
    case Head<Cons<h, t>> = h;
}
```

The function does not define `Head<Nil>`, so the type-function definition is
rejected before any application of `Head` is accepted. A later design may allow
partial type functions under explicit proof constraints or a type-level `Option`
return, but partiality is deferred.

Pattern matching rules for the first slice:

- patterns are first-order constructor patterns;
- pattern variables bind type expressions;
- repeated pattern variables are rejected unless equality guards are added;
- wildcard/catch-all patterns may be allowed but must be explicit;
- catch-all patterns cover remaining **known constructors** of a closed domain;
  they do not reduce applications whose scrutinized argument is an abstract type
  variable unless a later spec deliberately adds open catch-all semantics;
- aliases are not silently treated as constructors unless the spec defines alias
  expansion before matching;
- overlapping explicit constructor rows are rejected;
- if default/catch-all rows are supported, they must be specified as residual
  rows after previous constructor rows are subtracted, not as implicit
  priority-based overlap.

For multi-argument type functions, coverage and overlap must be checked over a
pattern matrix. Per-argument coverage checks are insufficient. A future spec must
report uncovered constructor combinations, not merely uncovered constructors in
isolation.

For open inputs, failure to select an equation because the scrutinized shape is
abstract produces a neutral normal form, not a coverage error:

```text
Append<Xs, Ys>
```

where `Xs` is abstract is neutral. By contrast, the partial `Head` definition
above is rejected at definition time because `Head<Nil>` is a closed known
uncovered case. In any future partial-function mode, applying `Head<Nil>` would
need explicit proof/evidence or a partial result type.

Catch-all/default rows must not be mistaken for open-variable reduction in the
first slice. For example:

```text
type fn F(xs: TypeList) -> Type {
    case F<Nil> = A;
    case F<_> = B;
}
```

Then:

```text
normalize(F<Cons<X, Y>>) = B
normalize(F<Xs>) = F<Xs>   -- neutral when Xs is abstract
```

The `_` row covers the remaining **known constructors** of the sealed `TypeList`
domain after `Nil` is subtracted. It does not assert that every abstract `Xs`
normalizes to `B`. A later spec may add open catch-all semantics, but that would
be a separate, explicit extension.

## 11. Coherence, Overlap, and Closed-World Boundaries

Type computation must be coherent: the same type-level expression should not
normalize to different results depending on import order or unrelated impls.

For direct type functions:

- equations are owned by the type function definition;
- equation overlap is rejected in the first slice;
- external modules cannot add new equations to an existing closed type function
  unless an explicit extension mechanism is designed later.

For associated type families through interfaces:

- impl selection must be coherent;
- ambiguous overlapping impls are rejected;
- specialization is deferred unless a formal priority/coverage rule is added;
- ordinary interface method dispatch may continue to use SPEC-034's
  visible/imported impl-set model;
- computable associated type families require a sealed family equation/impl set
  fixed at the family definition site or exported in a stable module summary;
- downstream modules cannot extend a sealed computable family unless a later
  extension mechanism is explicitly designed;
- recursive associated family computation must not silently rely on the current
  depth-limited impl solver as a termination proof.

Module/import/export invariants to preserve:

- a closed type function's full public equation set is fixed at definition site;
- the constructor set of any scrutinized type-level domain is fixed at the
  domain definition site;
- exported module summaries must contain enough information for downstream
  normalization of public type expressions;
- private equations must not affect public type equality unless their public
  normalized results are represented in a stable exported summary;
- downstream modules cannot extend a sealed domain or closed type function by
  default.

Public/private rules that a Tier 0/Tier 1 spec must settle:

- a public type signature may mention a public type function or public sealed
  domain constructor;
- a public type signature MUST NOT require downstream modules to inspect private
  equations to decide equality;
- a private type function may appear behind an exported opaque type only if the
  exported summary marks the result opaque and no downstream definitional
  equality depends on reducing it;
- a public type function summary must include at least function identity,
  visibility, kind/arity, decreasing parameter metadata, referenced sealed
  domains, and either the public equation set or an explicit opaque marker;
- a public sealed domain summary must include domain identity, constructor
  identities, constructor arities, structural field metadata, field kinds/domains,
  and visibility;
- a public computable associated family summary must include family/interface
  identity, associated member identity, sealed impl/equation set identity,
  projection arity, decreasing parameter metadata, and public/private equation
  visibility.

Ordinary recursive ADTs remain separate from this machinery. Type-function
normalization does not unfold ordinary recursive ADTs. Recursive ADT equality
continues to use nominal definition identity and type arguments unless a future
spec explicitly connects an ADT to a sealed type-level domain.

## 12. Performance and Incrementality Risks

Total normalization can still be expensive. Common blow-up sources include:

- repeated `Append` / `Map` / `Reverse` reductions over large typelists;
- large protocol-like type-level AST transforms;
- nested associated projection chains;
- equality checks that repeatedly normalize the same subterms;
- final inferred types that expand aliases into large normal forms.

Implementation specs should consider:

- memoized normalization results;
- hash-consed or otherwise canonical type expressions;
- weak-head normalization first, with full normalization only when forced;
- sharing reductions across equality checks;
- cycle detection separate from semantic termination checks;
- diagnostic fuel as a compiler robustness limit with actionable output;
- module-summary caches for exported normalized forms.

The DX default should avoid dumping huge fully normalized terms. Diagnostics
should show the user-written form, the relevant normalized slice, and an optional
expansion trace.

## 13. Expression-General Bidirectional Typing Remains Separate

Expression-general bidirectional typing is still the right foundation for value
expressions:

```text
Γ ⊢ e ⇒ τ      synthesize
Γ ⊢ e ⇐ τ      check against expected type
```

It should serve `do`, comprehensions, lambdas, matches, constructors, empty
literals, and future expression forms uniformly.

Type-level normalization is a separate layer used by those judgments when they
compare or instantiate types. Do not implement a do-specific or
comprehension-specific type computation path.

## 14. Non-Goals and Deferred Work

This design note does not propose:

- dependent types over runtime values;
- compile-time execution of arbitrary Ash expressions;
- promoted data constructors / full DataKinds in the first slice;
- unrestricted recursive trait solving;
- Rust-compatible specialization semantics;
- type-function inversion or injectivity-based solving;
- protocol/session-type syntax as a first-class target;
- law proving for Monad or other algebraic interfaces;
- SMT-backed proof search in the first slice;
- defaulting rules for ambiguous type-level computations;
- partial type functions in the first slice;
- mutual recursive type functions in the first slice.

All of those may be explored later, but they should not be smuggled into the
first compile-time computation substrate.

## 15. Diagnostics and DX Requirements

Compile-time type computation needs teaching-oriented diagnostics. Errors should
state:

- the user-written type expression;
- the normalized form or relevant one-step reduction, when helpful;
- whether a term is neutral/stuck, blocked by missing evidence, ambiguous, or a
  real type mismatch;
- the decreasing parameter and offending recursive call for termination errors;
- the uncovered constructor for coverage errors;
- one likely fix.

### 15.1 Blocked neutral normalization

During ordinary generic normalization, blocked reduction should be rendered as a
note or trace, not as an error:

```text
note[type-normalization-neutral]: cannot reduce `Append<Xs, Ys>` yet
  reason: `Xs` is abstract, so no `Append` equation can be selected
  note: treating `Append<Xs, Ys>` as a neutral normal form in this generic context
```

It becomes an error only when the surrounding judgment requires a concrete
normalized form:

```text
error[type-normalization-blocked]: concrete normal form required for `Append<Xs, Ys>`
  reason: `Append<Xs, Ys>` is neutral because `Xs` is abstract
```

In ordinary diagnostics, prefer a note attached to the real failure, such as
"cannot prove equality because this subterm is neutral".

### 15.2 Non-decreasing recursion

```text
error[type-fn-nondecreasing-recursion]: type function `Bad` may not terminate
  recursive call `Bad(xs)` does not use a structurally smaller argument
  decreasing parameter: `xs`
  help: recurse on a constructor field, e.g. `t` in a `Cons(h, t)` case
```

### 15.3 Non-exhaustive type function

```text
error[type-fn-nonexhaustive]: `Head` does not cover `Nil`
  note: type function declared total over `TypeList`
  help: add a `Head<Nil>` case or return a type-level `Option`
```

### 15.4 Normalized mismatch

```text
error[type-mismatch]: expected `Cons<A, Cons<C, Nil>>`
  found `Append<Cons<A, Nil>, Cons<B, Nil>>`
  normalized found type: `Cons<A, Cons<B, Nil>>`
  note: mismatch occurs at second element: expected `C`, found `B`
```

### 15.5 Rigid associated projection

In generic code, a rigid associated projection should normally be silent or a
trace note:

```text
note[associated-type-rigid]: `T::Out` remains rigid in this generic context
  reason: `T` is generic and no concrete impl has been selected
```

It becomes an error only when a concrete associated result is required:

```text
error[associated-type-rigid]: cannot reduce `T::Out` where a concrete type is required
  reason: `T::Out` remains a rigid associated projection until `T` is known
```

### 15.6 Equality blocked by neutrality

```text
error[type-equality-neutral]: cannot prove `Append<Xs, Ys> == Cons<A, Nil>`
  reason: `Append<Xs, Ys>` is neutral because `Xs` is abstract
  note: Ash does not invert type functions to solve for `Xs` or `Ys`
  help: provide a more specific type, add evidence that determines `Xs`, or keep
        the result abstract
```

## 16. Spec Set and Planning Starting Points

DESIGN-034 should be promoted through a **specification set**, not a single
mega-spec. The packets below are the intended order. Each packet must be
self-contained, list its prerequisites, name crate ownership, define acceptance
tests, and state what it explicitly does not implement.

### 16.1 SPEC-A: Unified type/module pipeline and semantic summaries

Purpose: establish the Tier 0 carrier path for all later type computation.

Normative packet: [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
and [PLAN-105](../plan/PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md).

Primary ownership:

- `ash-parser`: surface top-level type declarations in the normal module parser;
- `ash-core`: canonical IDs and shared semantic summary carriers;
- `ash-engine`: module loading/export plumbing, without owning type semantics;
- `ash-typeck`: validation/registration of type metadata consumed by later specs.

Must specify:

- ordinary `type` declarations flowing through the normal parser `ModuleFile`,
  lowering, typechecking, and export path;
- replacement or containment of ad-hoc source-snippet type-definition scanning;
- concrete canonical identities for ordinary type declarations and ordinary ADT
  constructors/variants;
- reserved future identity namespaces for type functions, sealed domains,
  generalized projections, computation summaries, interfaces, and associated
  members, without giving those future namespaces computation semantics in
  SPEC-A;
- a core-owned module semantic summary format distinct from capability-specific
  export structures;
- visibility rules for public/private type metadata;
- cross-module import/export of ordinary type identity and public summaries.

Acceptance tests:

- public ordinary type declarations survive parser -> engine -> typechecker ->
  module summary;
- private type declarations can appear in private implementation details without
  leaking reducible private equality to downstream modules;
- public signatures containing public type identities import consistently across
  modules;
- top-level ordinary `type` declarations are parsed by the normal module-file
  parser rather than discovered only by source-snippet extraction;
- legacy fragmented type-definition paths are either removed or fenced behind a
  compatibility path with tests.

### 16.2 SPEC-B: Type-expression IR, canonical IDs, kind/arity substrate

Purpose: define the representation and kinding substrate that normalization will
consume.

Primary ownership:

- `ash-core`: shared canonical type-expression/ID data that crosses crate and
  module boundaries;
- `ash-typeck`: kinding, arity validation, canonicalization, and normal-form
  comparison support;
- `ash-parser`: only surface syntax for any new kind/type-expression forms that
  this packet explicitly admits.

Must specify:

- `TypeFnApp` or equivalent computation-head representation;
- neutral type-function applications;
- generalized associated projection representation:

  ```text
  Projection { interface: InterfaceId, args: Vec<Type>, assoc: Name }
  ```

- replacement or canonicalization of current stringly/base-shaped
  `Type::Associated { interface, base, name }`, including unresolved or
  empty-interface cases, into canonical projection identities before associated
  family computation;
- rigid projection normal forms;
- normal-form grammar or normal-form view;
- canonical IDs and alias canonicalization policy;
- source-level kind annotations and kind checking for type parameters,
  constructor parameters, type-function parameters, projections, and partial
  applications;
- arity checking for all relevant type constructors, not only current builtins;
- type holes/wildcards in type-expression positions only if the packet chooses
  to include them.

Acceptance tests:

- type-function apps are not represented as ordinary nominal constructors;
- same-headed nominal constructors still decompose under ordinary unification;
- same-headed neutral computation heads compare without inversion;
- wrong arity/kind is rejected before normalization;
- transparent aliases canonicalize before equality and pattern matching while
  preserving readable diagnostics.

### 16.3 SPEC-C: Sealed type-level domains

Purpose: define the closed constructor sets needed for coverage and structural
recursion.

Primary ownership:

- `ash-parser`: sealed-domain surface syntax, if exposed in this packet;
- `ash-core`: domain/constructor metadata and visibility-bearing IDs;
- `ash-typeck`: domain kinding, constructor validation, structural-field checks;
- `ash-engine`: import/export transport of public domain summaries.

First-slice decision: use nominal marker constructors plus explicit sealed
`type domain` metadata. Promoted data kinds are deferred.

Must specify:

- domain declaration syntax or internal declaration form;
- whether domain declarations introduce marker constructors or register existing
  marker constructors;
- constructor namespace and canonical identity;
- constructor field metadata, including field kind/domain and structural status;
- recursive-domain positivity or acyclicity restrictions, if any;
- public/private domain and constructor visibility;
- module-summary export/import for domain constructor sets.

Acceptance tests:

- `TypeList` exposes exactly `Nil` and `Cons` to coverage checking;
- unrelated nominal constructors cannot be matched as `TypeList` constructors;
- private domain constructors do not leak through public coverage/equality;
- malformed field domains/kinds are rejected with domain-aware diagnostics.

### 16.4 SPEC-D: Normalizer and definitional equality core

Purpose: implement total normalization and normalize-and-compare equality before
surface `type fn` syntax is exposed broadly.

Primary ownership:

- `ash-typeck`: normalizer, definitional equality API, forcing points,
  neutrality/rigidity diagnostics;
- `ash-core`: any normal-form or canonical type-expression carriers that must be
  shared outside type checking;
- `ash-parser`: no new public syntax required unless the packet chooses internal
  fixture declarations for tests.

Must specify:

- normalization judgment `Γ ⊢ τ ⇓ τ_norm`;
- weak-head, full, and demand-driven normalization boundaries;
- neutral/stuck form grammar;
- equality of constructor normal forms, neutral type-function apps, variables,
  aliases, and rigid projections;
- exact unification boundary: top-level metas may be solved, but no solving
  underneath neutral computation heads;
- forcing points in expression checking, return checking, impl overlap,
  associated projection resolution, and final inferred-type rendering;
- cycle detection as compiler robustness separate from semantic termination.

Acceptance tests:

Until SPEC-E exposes public `type fn` syntax, SPEC-D tests should use internal
fixture equation tables, compiler-internal declarations, or hand-constructed
semantic summaries rather than pretending source-level type functions already
exist.

- closed reduction normalizes to constructor form;
- open reduction produces canonical neutral forms;
- partial open reduction preserves reduced prefixes and neutral tails;
- equality succeeds after normalization;
- equality blocked by neutrality gives a non-inverting diagnostic;
- ordinary constructor unification still works as before.

### 16.5 SPEC-E: Direct structural type functions

Purpose: expose the first user-facing computation surface.

First-slice decision: direct structural `type fn` over sealed domains is the
normative first computation surface. Associated recursive families are deferred
until after this packet and SPEC-D are stable.

Prerequisites: SPEC-A through SPEC-D. Until SPEC-F is implemented, public
cross-module `type fn` normalization MUST be rejected or treated as
module-local/internal only. Exported/public type-function use is not complete
without semantic summaries.

Primary ownership:

- `ash-parser`: `type fn` surface syntax and source spans;
- `ash-core`: shared type-function/equation AST or semantic carriers;
- `ash-typeck`: registration, equation checking, coverage, overlap, termination,
  normalization integration, and diagnostics;
- `ash-engine`: module integration only, not normalization semantics.

Must specify:

- concrete `type fn` surface syntax;
- equation AST and lowering;
- type-level pattern grammar: constructor patterns, variables, wildcards;
- pattern linearity and repeated-variable rejection;
- coverage and overlap via pattern-matrix checking over sealed domains;
- catch-all/default semantics as residual known-constructor coverage, not open
  abstract-variable reduction;
- declared decreasing parameter(s);
- structural subcomponent relation;
- recursive call detection, including calls nested under constructors;
- diagnostics for non-exhaustiveness, overlap, kind/domain mismatch, and
  non-decreasing recursion.

Acceptance tests:

- `Append<Cons<A, Nil>, Cons<B, Nil>>` reduces to `Cons<A, Cons<B, Nil>>`;
- `Append<Xs, Ys>` remains neutral when `Xs` is abstract;
- catch-all reduces known residual constructors but not abstract variables;
- partial `Head`-style definitions are rejected at definition time when a closed
  constructor case such as `Nil` is uncovered;
- recursive calls on the same, rebuilt, or type-function-produced argument are
  rejected.

### 16.6 SPEC-F: Module-summary export/import for type computation

Purpose: make normalization coherent across module boundaries.

Primary ownership:

- `ash-core`: stable semantic summary data structures and canonical IDs;
- `ash-engine`: module loading/import/export transport and cache boundaries;
- `ash-typeck`: consumption of imported summaries during normalization/equality;
- `ash-parser`: no semantic ownership beyond preserving exported declarations.

Must specify:

- exported summaries for public sealed domains;
- exported summaries for public type functions;
- whether public equations are exported directly or through opaque normalized
  facts;
- private equation opacity rules;
- import-order independence;
- summary versioning/cache invalidation considerations;
- reconciliation or replacement of the current fragmented export carriers,
  including engine-private `ModuleExports`, parser capability/module export
  metadata, and core `ModuleGraph`; type-computation summaries MUST NOT remain
  engine-private or capability-specific;
- diagnostics for unavailable private reductions in downstream modules.

Acceptance tests:

- downstream module can normalize public type-function applications using public
  summaries;
- downstream module cannot depend on private equations;
- import order does not change normal forms;
- public opaque type-function results remain opaque but stable.

### 16.7 SPEC-G: Associated type-family computation

Purpose: integrate associated types with the total computation substrate without
turning ordinary impl search into a hidden Turing-complete solver.

Primary ownership:

- `ash-core`: generalized projection/family identities that appear in public
  summaries;
- `ash-typeck`: impl/family selection, rigid projection behavior, recursive
  family checking, and projection normalization;
- `ash-parser`: surface projection syntax and compatibility parsing for existing
  `T::Assoc` forms;
- `ash-engine`: transport of public family summaries only.

Prerequisites: SPEC-A through SPEC-F. This packet MUST NOT start before direct
normalization/equality and module summaries exist.

Must specify:

- generalized projection IR and chosen surface syntax;
- compatibility elaboration from SPEC-035 `T::Assoc` syntax;
- distinction between simple associated type substitution and computable
  associated families;
- family sealing/coherence rules;
- unique selected impl/family instance rules;
- reduction of uniquely selected generic impl schemes over abstract arguments;
- rigid projection behavior when only a generic bound exists;
- recursive associated-family coverage, overlap, and decreasingness;
- where-bound evidence versus family equation selection;
- public/private family equation summaries;
- associated-family diagnostics.

Acceptance tests:

- `<Iterator<List<A>>>::Item` reduces to `A` through a unique generic impl;
- `<Iterator<List<X>>>::Item` reduces to `X` even when `X` is abstract;
- `T::Item` under only `T: Iterator` remains rigid in generic code;
- ambiguous family impls are rejected or remain unreduced with a precise error,
  according to the chosen rule;
- recursive `Append`-style family computation passes only when sealed,
  exhaustive, coherent, and structurally decreasing;
- current SPEC-035 simple associated type substitution continues to work for
  non-recursive selected impl outputs.

### 16.8 SPEC-H: Constraint/proposition layer

Purpose: add proof/search predicates around normalized types after the core
calculus is stable.

Primary ownership:

- `ash-typeck`: constraint generation, conservative solving, and diagnostics;
- `ash-core`: shared proposition/evidence carriers only if they cross module or
  runtime boundaries;
- `ash-engine`: summary transport if public proposition evidence is exported.

Must specify:

- equality constraints after normalization;
- interface bounds;
- explicit proposition interfaces or predicates;
- conservative disequality over closed normal forms and obvious
  constructor-disjointness;
- no type-function inversion in the first slice;
- no unrestricted SMT/proof search in the first slice.

Acceptance tests:

- `Cons<A, T> != Nil` succeeds by closed-domain constructor disjointness;
- `Append<Xs, Ys> == Cons<A, Nil>` does not solve for `Xs` or `Ys`;
- unsupported propositions produce explicit deferred-feature diagnostics.

### 16.9 Cross-packet implementation gaps to plan explicitly

Future specs/plans must not assume the following already exist. These are
**missing substrate**, not minor TODOs:

- integrated ordinary `type` declarations in the normal `ModuleFile` / lowering /
  export path; current type-definition handling is fragmented and must be fixed
  before type functions or sealed domains are layered on top;
- top-level `type fn` parser or surface/core AST definitions;
- sealed type-level domain declarations and exported constructor-set metadata;
- promoted data constructors or named data kinds;
- explicit internal representation for `TypeFnApp`, neutral/stuck normal forms,
  and generalized associated-family projections;
- type holes/wildcards in all type-expression positions;
- constructor-kinded interface parameters such as `M : * -> *`;
- complete source-level kind checking for type variables, constructors, partial
  applications, and all type-expression arity checks;
- generalized interface-application where constraints such as `Append<T, Ys>`;
- canonical associated projection syntax for multi-argument interface families;
- replacement/canonicalization of current stringly `Type::Associated` values into
  generalized projection identities, including the unresolved empty-interface
  case;
- a recursive associated-family normalizer that selects evidence per projection;
  current SPEC-035 selected-impl substitution is not enough;
- environment-aware definitional equality integrated into checking/unification
  forcing points;
- recursive associated type-family termination checking;
- module-summary export/import of type-function equations, sealed domains, and
  associated family metadata;
- alias canonicalization policy for normalization and pattern matching;
- diagnostics for neutral/stuck normalization, rigid projections, non-exhaustive
  type functions, non-decreasing recursion, and neutral-blocked equality.

Generalized `do` and comprehension typed elaboration may consume normalized and
compared types, but they should not host the type-computation implementation.

A future implementation MUST NOT encode type-function applications as ordinary
`Type::Constructor` nodes without a separate computation-head/neutral marker.
That shortcut would make the design unsound or at least impossible to diagnose
cleanly.

#### 16.9.1 Status after SPEC-A through SPEC-H implementation

This section is historical design guidance plus a current ownership index. Phases
109 through 116 closed most of the original substrate gaps by promoting
DESIGN-034 SPEC-A through SPEC-H into SPEC-057 through SPEC-064. The remaining
items are not hidden TODOs inside those implemented specs; they are explicit
future packets owned by PLAN-113.

| §16.9 item | Current status | Owner |
|---|---|---|
| Ordinary `type` declarations in ModuleFile/lowering/export | Closed by SPEC-057 / PLAN-105 | None |
| Top-level `type fn` parser and carriers | Closed by SPEC-061 / PLAN-109 | None |
| Sealed type-level domains and constructor-set metadata | Closed by SPEC-059 / PLAN-107 | None |
| Promoted data constructors or named data kinds | Spec/plan packet created; marker constructors remain distinct until implementation | SPEC-065 / PLAN-114 |
| Type-function apps, neutral/stuck forms, associated-family projections | Closed by SPEC-058, SPEC-060, SPEC-063 | None |
| Type holes/wildcards in all type-expression positions | Spec/plan packet created outside type-function pattern wildcards | SPEC-066 / PLAN-115 |
| Constructor-kinded interface parameters such as `M : * -> *` | Spec/plan packet created; core `Kind` exists but source binders remain unimplemented | SPEC-067 / PLAN-116 |
| Complete source kinding including partial applications | Partial: nominal/projection/computation arity exists; holes/partial apps/HKT now have planned SPEC packets | SPEC-066 / SPEC-067 |
| Generalized interface-application constraints | Closed for SPEC-H MVP; multi-argument interface-bound proposition regression added | PLAN-113 / TASK-891 |
| Canonical associated projection syntax for multi-argument families | Closed by SPEC-063 / PLAN-111 | None |
| Stringly associated projection replacement | Closed at canonical lowering/equality boundaries; pattern/exhaustiveness rollout now has planned SPEC packet | SPEC-068 / PLAN-117 |
| Recursive associated-family normalizer | Closed by SPEC-063 / TASK-866 | None |
| Environment-aware definitional equality forcing points | Closed by SPEC-060 / TASK-826 | None |
| Recursive associated-family termination checking | Closed by SPEC-063 / TASK-865 | None |
| Module-summary export/import of computation facts | Closed by SPEC-059, SPEC-062, SPEC-063, SPEC-064 | None |
| Alias canonicalization for normalization and pattern matching | Closed for normalization/equality; pattern/exhaustiveness rollout now has planned SPEC packet | SPEC-068 / PLAN-117 |
| Neutral/stuck/rigid/non-exhaustive/non-decreasing diagnostics | Closed across SPEC-060, SPEC-061, SPEC-063, SPEC-064 | None |
| Separate computation-head/neutral marker, not `Type::Constructor` | Closed by canonical computation-head and neutral-computation carriers | None |

See [PLAN-113](../plan/PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)
for the active backlog and future packet gates.

### 16.10 Recommended implementation sequence

The first implementable specification set should be sliced in this order:

1. SPEC-A: pipeline cleanup / Tier 0 semantic summaries.
2. SPEC-B: internal type-expression IR, canonical IDs, kind/arity substrate.
3. SPEC-C: sealed domains and marker-constructor metadata.
4. SPEC-D: normalizer and env-aware definitional equality core, with internal
   tests before public `type fn` syntax.
5. SPEC-E: direct structural `type fn` syntax, coverage, overlap, termination,
   and diagnostics, restricted to same-module/internal use until SPEC-F exports
   coherent summaries.
6. SPEC-F: public module-summary export/import for domains and type functions.
7. SPEC-G: associated type-family computation, only after direct normalization
   and summaries are stable.
8. SPEC-H: constraint/proposition layer.

Agents MUST NOT collapse these into one implementation task. Each layer creates
new invariants consumed by the next.

## 17. Decisions and Spec Defaults

### 17.1 First user-facing computation surface

Decision: direct structural `type fn` is the first normative computation surface.
Associated type-family computation is later Tier 2 work and must reuse the same
normalization engine only for sealed/coherent impl sets that pass the same
coverage, overlap, and termination checks.

Residual spec work: choose concrete syntax and diagnostics for direct `type fn`.

### 17.2 Type-level domains

Decision: the first slice uses nominal marker constructors plus sealed type-level
domains. Promoted data constructors and full DataKinds-style promotion are
deferred.

Residual spec work: define exact declaration syntax, constructor namespace,
visibility, field metadata, and module-summary export.

### 17.3 Equality and disequality scope

Decision: first-slice equality is definitional equality by normalization only.
It does not invert type functions, solve under neutral computation heads, or use
general proof search. Disequality is limited to closed normal forms and obvious
constructor-disjointness.

Residual spec work: SPEC-D must formalize the equality/unification boundary;
SPEC-H must define any later proposition/disequality evidence without turning the
first constraint layer into proof search.

### 17.4 Closed-world boundary

Decision: closed-world type functions, sealed domains, and computable associated
family equation sets are sealed by definition site and exported through module
summaries when public.

Residual spec work: define opaque/public equation summary formats and private
reduction boundaries.

### 17.5 Diagnostic normal forms

Decision: diagnostics should show the user-written form and the smallest relevant
normalized slice. They should preserve aliases where possible and provide full
expansion traces only on demand.

Residual spec work: the diagnostic spec must define exact rendering limits,
trace toggles, and how much of a large normal form is shown by default.

### 17.6 Monomorphization and module summaries

Decision: normalize during type checking when equality demands it. Export enough
sealed-domain, equation, and projection metadata for downstream public
normalization. Monomorphization should consume already-checked normalized/equality
facts rather than rediscovering them.

Residual spec work: define exact summary serialization/cache invalidation and the
boundary between public normalized facts and private equations.

## 18. Current Position

The preferred direction is:

1. keep Monad/HKT/inference work as a separate generic-programming track;
2. define compile-time type computation as its own substrate with total,
   terminating normalization as a hard requirement;
3. define neutral/stuck normal forms for open generic type expressions;
4. start with sealed-domain structural type functions as the primitive
   normalization calculus;
5. use associated types to connect computation to interfaces only after the
   normalization/equality story is explicit;
6. keep recursive associated type-family computation inside the same totality,
   coverage, and coherence discipline as direct type functions;
7. treat equality as normalize-and-compare, not inversion/proof search;
8. use protocol-like examples as stress tests, not as the product definition.
