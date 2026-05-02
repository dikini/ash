# SPEC-035: Associated Types on Interfaces

**Status:** Draft
**Date:** 2026-05-02
**Version:** 0.3
**Internal representation superseded by:** [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)

## 1. Overview

Allow interfaces to declare associated types — type-level outputs that are determined by each `impl` block. This enables interfaces to act as type families: a single interface maps a set of input types to a specific output type.

Associated types are the final extension needed for ergonomic generic libraries. They eliminate the "type parameter explosion" seen when output types must be threaded explicitly through every generic signature. Use cases include serialization (`Serializer::Ok`, `Serializer::Error`), collection traits (`Map::Key`, `Map::Value`), and besedarium's query builder (`QueryBuilder::Result`, `QueryBuilder::Error`).

## 2. Motivation

Without associated types, a generic serializer interface forces four type parameters into every function signature:

```ash
-- Verbose and leaky
interface Serialize<T, S, Out, Err> {
    serialize(T, S) -> Result<Out, Err>
}

fn to_json<T, Out, Err>(value: T) -> Result<Out, Err>
    where T: Serialize
{
    Serialize::serialize(value, JsonWriter)
}
```

With associated types, the caller only needs to know the input types:

```ash
interface Serializer<S> {
    type Ok
    type Error
    serialize_bool(S, Bool) -> Result<S::Ok, S::Error>
    serialize_string(S, String) -> Result<S::Ok, S::Error>
}

fn to_json<T>(value: T) -> Result<String, SerializeError>
    where T: Serialize
{
    Serialize::serialize(value, JsonWriter)
}
```

The output and error types are projected from the `impl` block for `JsonWriter`.

## 3. Semantics

### 3.1 Interface Declaration

Associated types are declared inside an `interface` block with the `type` keyword:

```ash
interface Iterator<I> {
    type Item
    next(I) -> Option<I::Item>
}

interface Serializer<S> {
    type Ok
    type Error
    serialize_bool(S, Bool) -> Result<S::Ok, S::Error>
}
```

Associated type names are in scope for all method signatures inside the interface. They are referenced via the interface's type parameter using `Param::AssocName` syntax (e.g., `S::Ok`).

### 3.2 Concrete Implementation

Each `impl` block must provide a concrete type for every associated type:

```ash
impl Serializer<JsonWriter> {
    type Ok = String
    type Error = SerializeError
    serialize_bool(writer, value) = ...
}

impl Iterator<ListIter<Int>> {
    type Item = Int
    next(iter) = list_iter::next(iter)
}
```

Generic `impl` blocks may use their own type parameters on the right-hand side:

```ash
impl<T> Iterator<ListIter<T>> {
    type Item = T
    next(iter) = ...
}
```

### 3.3 Projection Syntax

Associated types appear in user code and method signatures as:

```ash
S::Ok
S::Error
I::Item
Map<K, V>::Entry
```

At the surface level, associated projections use the existing `base::Assoc` spelling.

Current supported grammar:

```text
associated-projection = projection-base "::" identifier
projection-base       = identifier | nominal-type-application
```

Where `nominal-type-application` is the ordinary named type form with zero or more type arguments, for example `Map<K, V>`.

This grammar replaces earlier shorthand formulations such as `type "::" identifier` and `identifier "::" identifier`.

Notes:
1. `S::Ok` and `Map<K, V>::Entry` are both in this supported subset.
2. This section is intentionally surface-only. It does not define a canonical internal representation for projections.
3. The parser records the written base and associated-member name. Resolving the declaring interface/member remains the type checker’s job.

Ambiguity rule: if a type variable `T` has multiple interface bounds and two or more of those interfaces declare an associated type with the same name (for example, both `A` and `B` define `Ok`), then `T::Ok` is ambiguous and must be rejected with a dedicated ambiguity error. No alternative public disambiguation syntax is normative in this packet. If exactly one bound in scope defines the requested associated type name, `T::Ok` resolves to that interface’s associated type.

## 4. Implementation Boundary

SPEC-035 defines only:
- declaration syntax for associated types inside `interface` and `impl` blocks;
- the current projection surface forms `Base::Assoc` and `Base<A, B>::Assoc`;
- the ambiguity rule for bound-name lookup;
- the simple compatibility behavior in §5.

SPEC-035 does not define the canonical internal representation of projections, projection identities, elaboration states, kind/arity validation, alias canonicalization, or any general normalization judgment. Those internal and cross-crate contracts are owned by SPEC-058.

## 5. Current Compatibility Semantics

### 5.1 Concrete selected-impl substitution

After ordinary interface-method resolution selects a unique `impl` for a concrete call and computes the usual substitution from the impl head, each projection from that same interface is interpreted by:
1. looking up the named associated-type binding in the selected `impl`; and
2. applying the selected substitution to that binding’s right-hand side.

Example:

```ash
interface Serializer<S> {
    type Ok
    serialize_bool(S, Bool) -> S::Ok
}

impl Serializer<JsonWriter> {
    type Ok = String
    serialize_bool(writer, value) = ...
}
```

For `Serializer::serialize_bool(my_writer, true)`, selecting `impl Serializer<JsonWriter>` makes the projected result `S::Ok` compatible with `String` by substituting the selected impl binding.

When a concrete typing operation must compare a projected type against another type after unique impl selection, it uses this same selected-impl substitution result before the ordinary compatibility check.

For this packet, this selected-impl substitution path is the only required concrete associated-type reduction behavior. It is a compatibility rule for the current simple associated-type subset, not a general normalization, definitional equality, or recursive type-computation system.

### 5.2 Rigid projections in generic code

If no concrete `impl` has been selected, a projection remains rigid.

In particular:
- inside `fn<T: Serializer>(s: T) -> T::Ok`, the projection `T::Ok` is rigid;
- two rigid projections are compatible only when they refer to the same base or bound and the same associated member;
- a rigid projection does not become compatible with an arbitrary concrete type merely because some later `impl` could choose that type.

This rule preserves current generic associated-type behavior while leaving general projection comparison and normalization ownership to SPEC-058 and later packets.

## 6. Conformance

An implementation conforming to SPEC-035 must:
- parse `type Name` declarations inside `interface` blocks;
- parse `type Name = TypeExpr` bindings inside `impl` blocks;
- parse projection surface forms `Base::Assoc` and `Base<A1, ..., An>::Assoc` in type positions;
- reject ambiguous `T::Assoc` lookups when more than one in-scope bound defines `Assoc`;
- require each `impl` to provide exactly the associated type bindings declared by the interface, with no missing or extraneous bindings;
- apply the §5.1 selected-impl substitution rule for the current simple associated-output path;
- apply the §5.2 rigid-projection rule when no concrete `impl` has been selected;
- support generic `impl` blocks whose associated-type bindings mention the impl’s own type parameters.

All canonical internal representation details and generalized type-expression handling beyond these surface and compatibility requirements are specified by SPEC-058.
