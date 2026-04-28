# DESIGN-032: Monad Comprehension Syntax

**Status:** Implemented MVP
**Date:** 2026-04-28
**Related:** DESIGN-031, SPEC-054, SPEC-055, PLAN-101, PLAN-102

**Promoted by:** [SPEC-055](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) and [PLAN-102](../plan/PLAN-102-MONAD-COMPREHENSION-SYNTAX.md)

## 1. Summary

Ash should support bracket comprehension syntax as a container-view surface for generalized do-notation:

```ash
[result | qualifiers]
[result | qualifiers]: K
```

This feature has no independent semantics. It uses the same `Monad<K>` contract, target resolution, typed elaboration, tower rules, and nested `bind` / `return` lowering as `do` notation.

The distinction is visual and ergonomic:

- `do` presents the monad as a computation context.
- `[result | qualifiers]` presents the monad as a container/comprehension context.

The lowered artifacts should be identical modulo source-span/origin metadata.

## 2. Core Equivalence

A comprehension:

```ash
[f(x, y) | x <- xs, y <- ys]: List
```

is equivalent to:

```ash
do:List {
    x <- xs;
    y <- ys;
    return f(x, y)
}
```

If the target can be inferred:

```ash
[f(x) | x <- xs]
```

is a deferred extension. Phase 106 requires the explicit form:

```ash
[f(x) | x <- xs]: K
```

## 3. Target Resolution

Do notation supports:

```ash
do:K { ... }   // explicit target
do { ... }     // inferred target, if possible
```

Comprehension syntax supports:

```ash
[result | qualifiers]: K   // explicit target
[result | qualifiers]      // inferred target, if possible
```

The postfix annotation is comprehension-specific target syntax. It is intentionally shaped like a type ascription, but the current parser should not assume a general expression-level `expr: Type` annotation exists. The postfix slot is used because bracket comprehension is mixfix syntax and has no clean prefix target slot.

Target resolution uses the same mechanism as generalized do-notation. Phase 106 implements the explicit-target MVP only:

1. use the explicit target;
2. reject missing targets with a comprehension-specific diagnostic;
3. reserve expected-type/qualifier-driven inference for a later phase.

The target must resolve to a unary computation constructor:

```text
K : * -> *
```

and `Monad<K>` evidence must be available.

## 4. Qualifiers

MVP qualifiers mirror do-block statements.

Monadic bind:

```ash
x <- expr
```

Equivalent do statement:

```ash
x <- expr;
```

The RHS must have type `K<A>`.

Ignored monadic action:

```ash
_ <- expr
```

Equivalent do statement:

```ash
_ <- expr;
```

Pure lexical binding:

```ash
let x = expr
```

Equivalent do statement:

```ash
let x = expr;
```

The result expression becomes the final `return`.

General rewrite:

```ash
[r | q1, q2, q3]: K
```

normalizes to:

```ash
do:K {
    q1;
    q2;
    q3;
    return r
}
```

Example:

```ash
[f(x, z) | x <- xs, let y = g(x), z <- h(y)]
```

normalizes to:

```ash
do {
    x <- xs;
    let y = g(x);
    z <- h(y);
    return f(x, z)
}
```

## 5. Semantic Contract

MVP requires exactly:

```text
Monad<K>
```

Comprehension syntax does not require or imply separate Functor, Applicative, Monoid, Alternative, or MonadPlus semantics.

A map-like comprehension:

```ash
[f(x) | x <- xs]
```

still lowers through `bind` and `return`:

```text
bind(xs, |x| return(f(x)))
```

A multi-generator comprehension:

```ash
[f(x, y) | x <- xs, y <- ys]
```

lowers left-to-right through nested binds:

```text
bind(xs, |x|
    bind(ys, |y|
        return(f(x, y))))
```

It does not use applicative lowering.

## 6. Guards

Bare boolean guards are not MVP because they require more than plain `Monad<K>`.

Not MVP:

```ash
[x | x <- xs, x > 0]
```

MVP spelling uses an ordinary monadic operation in scope:

```ash
[x | x <- xs, _ <- guard(x > 0)]: List
```

Equivalent do form:

```ash
do:List {
    x <- xs;
    _ <- guard(x > 0);
    return x
}
```

`guard` is not imported or interpreted specially by comprehension syntax.

## 7. Tower Behavior

Comprehensions follow the same tower behavior as `do`.

No target is banned by syntax. This is valid if `Act` is a valid Monad target:

```ash
[parsed | raw <- read(path), parsed <- parse(raw)]: Act
```

It may be stylistically unusual, but the language should not reject it because the surface syntax has a container-view shape.

No implicit lifting occurs. If a comprehension target is `Proc`, then every `<-` qualifier must produce `Proc<A>`, not `Act<A>`, `Option<A>`, or another constructor. The user must call an explicit lift where needed.

Operational `fail` remains tower-scoped operational bottom. It is not converted into `None`, `Err`, empty list, or any domain-level failure.

## 8. Parser and Elaboration

The parser may keep a distinct surface node for source fidelity:

```text
Comprehension {
    result,
    qualifiers,
    target_annotation?
}
```

Typed elaboration must share the generalized do path. Comprehension syntax should normalize conceptually to a do block before or during typed elaboration, not to untyped parser-generated calls.

Typed elaboration owns:

- target inference and checking;
- kind check `K : * -> *`;
- `Monad<K>` evidence resolution;
- qualifier type checking;
- final `return` typing;
- lowering to nested `bind` / `return`.

## 9. MVP Exclusions

Out of scope for MVP:

- bare boolean guards;
- pattern binders;
- applicative lowering;
- functor-only lowering;
- monoid collection-builder semantics;
- implicit imports of `guard`, `empty`, `filter`, or related helpers;
- implicit tower lifts;
- parallel or zip comprehensions.

Parallel comprehensions may be considered later with distinct syntax and a separate interface requirement such as Applicative, Zip, or MonadZip.

## 10. Diagnostics

Reuse do-notation diagnostics with comprehension-specific wording.

Cannot infer target:

```text
error: cannot infer comprehension target
hint: add an explicit target annotation, e.g. `[expr | x <- xs]: List`
```

Pure expression used as generator:

```text
error: '<-' in comprehension expects K<A>, found Int
hint: use `let x = expr` for pure bindings
```

Target mismatch:

```text
error: comprehension target is List, but qualifier has type Option<Int>
hint: all '<-' qualifiers must produce the same target constructor
```

Suspicious `let` binding:

```text
warning: `let` binds a monadic value without comprehending over it
hint: use `x <- xs` to bind its contents
```

Bare boolean qualifier:

```text
error: bare boolean qualifiers are not part of Monad-only comprehensions
hint: use an explicit monadic operation, e.g. `_ <- guard(condition)`
```

## 11. Compact Rule

Ash comprehensions are a surface-only, container-view spelling of generalized do-notation:

```ash
[r | q1, q2, q3]: K
```

is equivalent to:

```ash
do:K {
    q1;
    q2;
    q3;
    return r
}
```

They require the same `Monad<K>` machinery and produce the same lowered artifacts as do-notation.
