---
status: drafting
created: 2026-08-03
last-revised: 2026-08-03
related-plan-tasks: []
tags: [research, type-system, interfaces, associated-types, type-families, visibility, abstraction, modules]
---

# TYPES-005: Component abstraction with interfaces and private types

## Purpose

This is a research idea, not an implementation proposal. It asks how far Ash can obtain the useful abstraction power of Standard ML signatures, structures, and functors by composing features Ash already has or is close to having. It does not propose replacing Ash modules, changing module lookup, or starting implementation work.

The starting point is simple. Ash modules already give names a public or private scope. Interfaces and implementations describe ad-hoc polymorphism. Associated types and associated type families describe type components. Parametric functions describe reuse over unknown types. The question is what small additions let these parts work together as a component system.

The aim is not to copy SML syntax. The aim is to express the same useful boundaries: a public contract, a private representation, declared type sharing, reusable parameterized components, and, where needed, fresh abstract identities.

## Scope

- **In scope:** type-level component identities, associated type families, equality constraints, public and private type equations, generic implementation families, and possible fresh component application.
- **Out of scope:** changing Ash modules from namespaces into values; a new import system; dynamic module loading; a second evaluator; or an implementation plan.
- **Related but separate:** executable interface dispatch, first-class packages, recursive components, and public constants with once-only initialization semantics.

All Ash code in this document is illustrative unless it is explicitly marked as current syntax. Proposed syntax is deliberately schematic.

## Static components and runtime realizations

This note owns the static half of the design. A component identity, its interface facts,
associated-type projections, and visible equations are compile-time objects. A component
application therefore does not allocate state, select a provider, or install runtime authority.

Resource kinds are also static descriptions, but concrete resource instances and admitted provider
frames are runtime identities. Repeated admission of one component recipe over different resource
instances creates distinct runtime bindings without creating distinct component identities.
Conversely, a fresh component application creates a distinct static identity without allocating a
resource instance. Runtime instance IDs must not participate in ordinary type equality.

The runtime-realization questions—resource slots, allocation/admission, provider frames, lifecycle,
sharing, and provenance—are explored separately in
[RESOURCES-001: Resource providers and runtime identity](../runtime/RESOURCES-001-resource-providers-and-runtime-identity.md).
The shared non-confusion contract is recorded in the
[Component-resource phase boundary](../architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md).

## Current understanding

Ash already has much of the required substrate.

- Ordinary associated types provide a type component selected by an implementation.
- Sealed associated type families provide checked, normalizable type computation through interface applications.
- The type-computation substrate has canonical projection identities, public summary transport, and normalizer-backed definitional equality.
- The proposition layer has equality, disequality, and interface-bound carriers. Its current source `where` syntax does not yet expose general equality clauses.
- `builtin type Name;` is a current bodyless opaque declaration. The target grammar proposes ordinary bodyless nominal types such as `type Name;`.
- Nominal `newtype` declarations already provide distinct type identities for inhabited representations.

These facts narrow the design problem. The difficult work is less about inventing type computation and more about deciding what Ash exports, what it keeps private, and when it creates a new identity.

## The basic encoding

A component can be represented by a nominal type used only as an identity. An interface is indexed by that identity. Its associated types are the component's public type members.

The following is **proposed surface syntax**. It uses the planned ordinary bodyless type spelling and a proposed equality predicate.

```ash
pub interface Ordered<O> {
    type Item;
    compare(O::Item, O::Item) -> Ordering;
}

pub type IntOrder;

pub impl Ordered<IntOrder> {
    type Item = Int;
    compare(left, right) = ...;
}
```

`IntOrder` is not a runtime object. It is the name of one ordered component. `IntOrder::Item` plays the same role as a type member such as `IntOrder.t` in an SML structure.

A set component can hide its state type while exposing the operations that use it.

```ash
pub interface Set<S> {
    type Element;
    type Handle;

    empty() -> S::Handle;
    insert(S::Handle, S::Element) -> S::Handle;
    contains(S::Handle, S::Element) -> Bool;
}

pub type IntSet;
type IntSetState = ...;  -- private

pub impl Set<IntSet>
    where IntSet::Element == IntOrder::Item
{
    type Element = Int;
    type Handle = IntSetState;

    empty() = ...;
    insert(set, element) = ...;
    contains(set, element) = ...;
}
```

The public client can use `IntSet::Handle`. It cannot name `IntSetState`, construct it, inspect it, or prove that the two names are equal. This is the central abstraction rule.

The example writes `IntSet::Element == IntOrder::Item` to show the intended sharing relation. In a final design, the equality might be inferred from the two explicit equations above, required as a checked declaration, or both. This note does not choose between those options.

## What type sharing needs to mean

An equality constraint must not cause the compiler to search for an implementation that makes the constraint true. It should compare the type expressions already in scope.

For example, this constraint should be checked by normalizing both sides and comparing the results:

```ash
where S::Element == O::Item
```

The compiler may reduce public, transparent equations. It must not cross a private equation boundary. Therefore these two declarations have different public meanings:

```ash
-- The public summary may state the equality.
pub impl Source<PublicSource> {
    type Output = Int;
}

-- The public summary names `SecureStore::Handle`, but omits this equation.
type StoreState = ...;
pub impl Store<SecureStore> {
    type Handle = StoreState;
}
```

Outside the second module, `SecureStore::Handle` remains a rigid abstract type. The fact that it is implemented by `StoreState` is available only inside the defining private scope.

This is a visibility rule, not a special form of unification. It gives private types their intended force.

## A small static semantics

This section gives an explanatory operational account. It is not proposed Core syntax.

Let a compile-time component environment be:

```text
K = (N, I, E, V)
```

where:

- `N` maps nominal type names to stable identities;
- `I` records interface declarations and their associated members;
- `E` records checked implementation evidence and associated-type equations;
- `V` records which identities and equations are visible at the present module boundary.

A type projection is written `Proj(C, A)`, meaning associated member `A` of component identity `C`.

### Projection reduction

A public, transparent associated equation may reduce:

```text
E(C, A) = T       T is visible in V
-----------------------------------  PROJ-PUBLIC
K |- Proj(C, A)  -->  T
```

A private equation does not reduce outside its module. The client keeps the projection:

```text
E(C, A) = T       T is not visible in V
----------------------------------------  PROJ-OPAQUE
K |- Proj(C, A)  -->  Proj(C, A)
```

The second rule is the abstraction boundary. It does not say that `Proj(C, A)` is unknown or ill-formed. It says that it is known only by its public identity.

### Equality checking

Equality checks normalize both operands only through visible equations:

```text
K |- T1 -->* N1       K |- T2 -->* N2       N1 = N2
----------------------------------------------------  EQ-TRUE
K |- T1 == T2
```

If the normal forms differ, the constraint is refuted. If either side is a rigid projection or neutral type-family application that cannot reduce, the checker defers the comparison or reports that the requested equality cannot be proved. It must not work backward from the requested output to invent an implementation or a family argument.

### Interface evidence

A checked implementation adds evidence for an interface application:

```text
K |- impl Set<C> { ... } checked
----------------------------  IMPL-ADD
K + Set<C>
```

A generic function may use that evidence through an ordinary constraint:

```ash
fn keep<S>(set: S::Handle) -> S::Handle
    where S: Set
{
    set
}
```

This says only that the function may rely on the `Set<S>` contract. It does not grant capability authority or make the component a runtime resource.

## Parameterized components without fresh identity

A generic implementation family can model applicative composition. The same inputs always produce the same type identity.

```text
MakeSet<IntOrder> == MakeSet<IntOrder>
```

In schematic Ash:

```ash
-- Proposed: `SetOf<O>` is a stable nominal family.
pub type SetOf<O>;
type SetState<O> = ...;

pub impl<O> Set<SetOf<O>>
    where O: Ordered
{
    type Element = O::Item;
    type Handle = SetState<O>;

    empty() = ...;
    insert(set, element) = ...;
    contains(set, element) = ...;
}
```

This is enough for many libraries. A parser family, a map family, a protocol adapter, or a resource wrapper can expose type members that depend on its input component while hiding its state.

It has one deliberate limit. Repeating `SetOf<IntOrder>` means the same component. That is the applicative choice.

## Fresh component application

SML can also make a new abstract identity each time a functor is applied. The same input may produce two components whose hidden types are distinct.

```text
A = fresh MakeSet(IntOrder)
B = fresh MakeSet(IntOrder)

A::Handle != B::Handle
A::Element == IntOrder::Item
B::Element == IntOrder::Item
```

Ash does not need a new module system to express this. It needs one static operation that does two things together:

1. creates a fresh nominal component identity; and
2. applies a checked implementation template to that identity.

A possible surface form, shown only to make the scope clear, is:

```ash
-- Proposed syntax, not a recommendation.
instantiate MakeSet(IntOrder) as A;
instantiate MakeSet(IntOrder) as B;
```

The compiler could elaborate the first declaration as follows:

```text
choose a nominal identity a not in N
copy the checked MakeSet template with Order = IntOrder and Self = a
add Set<a> and its associated equations to E
keep generated representation identities private unless exported explicitly
bind source spelling A to a in the current scope
```

The operation is compile-time only. It does not allocate a runtime module object. It does not change import resolution. It does not require dynamic loading.

A fresh type by itself is not enough. It would create `a`, but it would not say which operations, associated equations, and private state declarations belong to `a`. The template application supplies that missing connection.

## Design alternatives

### Alternative 1: stable families only

Ash could offer only named generic component families such as `SetOf<O>`.

**What it gives:**

- private representations behind public projections;
- associated type families;
- type sharing through equality predicates;
- reusable parameterized components;
- stable, easy-to-read type equality.

**What it does not give:**

- a fresh abstract result for each use site.

This is the smallest design. It covers the applicative fragment and avoids local implementation scope, fresh names, and questions about exporting a generated component.

### Alternative 2: explicit fresh template application

Ash could add a visible fresh-application form such as the illustrative `instantiate` form above.

**What it gives:**

- the stable family features from Alternative 1;
- SML-like generative abstraction when the author asks for it;
- clear identity behavior at the declaration site.

**What it costs:**

- a template declaration form or an equivalent way to declare what a fresh application installs;
- rules for the scope and export of generated identities;
- a coherence rule for evidence created by local applications;
- new diagnostic cases for escaping or comparing fresh components.

This is the most direct route if Ash needs generativity. It should remain explicit. A normal named generic family should not silently become generative.

### Alternative 3: local implementation blocks

Ash could allow a local nominal type and a local implementation block:

```ash
-- Proposed pseudo-code.
let type A;
let impl Set<A> = MakeSet(IntOrder);
...
```

This makes freshness and the resulting evidence visible as two separate declarations.

**What it gives:**

- a small core model: fresh type introduction plus local evidence introduction;
- a direct path to formalizing local scope.

**What it costs:**

- more surface machinery for a common operation;
- an awkward split between the fresh identity and the template that defines it;
- a stronger interaction with coherence, because implementations are no longer only module-level facts.

This is attractive as an internal elaboration model. It may be less attractive as user syntax.

### Alternative 4: existential packages

A package would hide a component identity inside a value-like container, roughly:

```text
exists S. Set<S>
```

A client could call the exported operations but could not name the hidden `S` outside the package.

**What it gives:**

- dynamic selection of an implementation;
- heterogeneous collections of components;
- returning an abstract component from a function.

**What it costs:**

- package types, unpacking rules, and value-level evidence representation;
- questions about dispatch and runtime representation;
- a larger language feature than static component abstraction needs.

This is useful later, but it is not needed to approximate ordinary static signatures and functors.

### Alternative 5: encode freshness with type families

A tempting approach is to make a type family produce a new result type:

```text
NewSet<O>
```

This does not provide generativity. A type-family application is defined by its inputs. Repeating the same application has the same result by design. It is useful for stable component families, but it cannot stand for a fresh allocation of nominal identity.

## Comparison

| Need | Stable family | Fresh template application | Local impl block | Existential package |
|---|---|---|---|---|
| Private representation | yes | yes | yes | yes |
| Associated type sharing | yes | yes | yes | yes |
| Reusable generic component | yes | yes | yes | limited |
| Same input, same identity | yes | optional | optional | not the main question |
| Same input, fresh identity | no | yes | yes | possible but indirect |
| Dynamic component choice | no | no | no | yes |
| New module semantics | no | no | no | no, but needs value packaging |
| Likely early complexity | low | medium | medium | high |

## Questions that decide the next step

1. Should the public summary of an associated member state whether its equation is transparent, opaque, or unavailable? A three-way state may improve diagnostics, but two states may be enough.
2. Should an equality constraint that reaches an opaque projection be deferred, rejected, or accepted only when both sides are the same projection? The conservative default is to accept only identical rigid projections.
3. Can current sealed associated families express all needed abstract type-constructor components through extra interface parameters, or is dedicated associated type-constructor syntax worth adding?
4. Does generic interface dispatch lower by monomorphization, explicit evidence passing, or a hybrid? This choice is separate from the type-level component design, but it determines which examples can run.
5. If fresh component application is added, may a generated component be exported? If so, what stable public spelling names its fresh identity?
6. Should local implementation evidence be allowed at all, or should fresh application be the only way to create local evidence?
7. Do public component contracts need immutable value members, or are pure zero-argument methods enough for the first design?

## Suggested research order

This note does not request implementation. If the idea is explored further, the lowest-risk questions are:

1. test whether existing type-family and proposition machinery can express the examples without new Core forms;
2. define the public/private export rule for associated equations and test it across imports and re-exports;
3. compare a stable component-family design with an explicit fresh-application design using the same set/map example;
4. decide whether generativity solves a real Ash workload before adding syntax or local evidence scope;
5. only then consider packages or higher-order component forms.

## Related explorations

- [RESOURCES-001: Resource providers and runtime identity](../runtime/RESOURCES-001-resource-providers-and-runtime-identity.md) explores the dynamic realization of resource-backed providers while consuming this note's static component vocabulary.
- [Component-resource phase boundary](../architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md) records the static/dynamic identity and admission invariants shared by the two explorations.
- [TYPES-002 V2: Ad-Hoc Polymorphism](TYPES-002-ad-hoc-polymorphism-v2.md) discusses coherence, interface evidence, and possible elaboration strategies.
- [TYPES-002 V2 MVP Cut](TYPES-002-v2-mvp-cut.md) records the narrower closed-world interface direction.
- [NOTE-026: Newtype and Phantom Types](../../notes/NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md) explains nominal identity and the distinction between aliases, bodyless types, and newtypes.
- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md) defines the existing family-computation substrate.
- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md) defines the existing proposition substrate.

## References

### Internal references

- [SPEC-035: Associated Types on Interfaces](../../spec/SPEC-035-ASSOCIATED-TYPES.md) — ordinary associated type declarations, selected implementation substitution, and rigid generic projections.
- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md) — sealed associated families, reduction, and public summary transport.
- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md) — equality propositions, conservative solving, and the non-inversion rule.
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md) — proposed ordinary bodyless nominal types and nominal `newtype`s.
- [Generics, kinds, interfaces, and implementations](../../reference/language/types/generics-kinds-interfaces-and-impls.md) — implementation-backed current interface and implementation boundary.

### External references

- Robin Milner, Mads Tofte, Robert Harper, and David MacQueen, *The Definition of Standard ML (Revised)*, 1997. The source definition of SML signatures, structures, functors, and sharing. <https://smlfamily.github.io/sml97-defn.pdf>
- Robert Harper, *Practical Foundations for Programming Languages*, 2nd ed., 2016, Chapter 48, “Modularity.” A clear account of module abstraction, sealing, and static versus dynamic structure. <https://www.cs.cmu.edu/~rwh/pfpl/2nded.pdf>
- Xavier Leroy, “A modular module system,” *Journal of Functional Programming* 10(3), 2000. A treatment of applicative and generative functors and their type-equality consequences. <https://doi.org/10.1017/S0956796800003683>

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-08-03 | Keep this document as a research idea. | The design has promising existing substrate, but it does not yet identify a selected surface, Core design, or implementation scope. |
| 2026-08-03 | Treat Ash modules as namespaces and visibility boundaries in this exploration. | The point is to compose existing mechanisms, not to introduce a second module language. |
| 2026-08-03 | Keep fresh application as an alternative, not a committed feature. | Stable generic families cover many use cases. Generativity needs a workload-based justification. |
| 2026-08-05 | Keep runtime resource/provider realization in a sibling exploration. | Components are a broader static abstraction; resource instances and provider frames require separately evolving admission and runtime semantics. |

## Changelog

| Date | Change |
|------|--------|
| 2026-08-05 | Added the explicit static-component versus runtime-resource/provider boundary and links to the sibling resource and bridge explorations. |
| 2026-08-03 | Created the research exploration of component abstraction from interfaces, associated types, visibility, equality constraints, and optional fresh application. |
