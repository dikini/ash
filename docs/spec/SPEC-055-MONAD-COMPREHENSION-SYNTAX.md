# SPEC-055: Monad Comprehension Syntax

**Status:** Implemented MVP
**Date:** 2026-04-28
**Promotes:** [DESIGN-032](../design/DESIGN-032-MONAD-COMPREHENSION-SYNTAX.md)
**Builds on:** [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
**Related:** [SPEC-002](SPEC-002-SURFACE.md), [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)
**Plan:** [PLAN-102](../plan/PLAN-102-MONAD-COMPREHENSION-SYNTAX.md)
**Implementation Tasks:** [TASK-754](../plan/tasks/TASK-754-monad-comprehension-spec-plan-packet.md) through [TASK-759](../plan/tasks/TASK-759-monad-comprehension-docs-examples-closeout.md)

## 1. Summary

Ash comprehension syntax is a container-view spelling of generalized typed do-notation:

```ash
[result | qualifiers]: K
```

It has no independent runtime semantics. It parses to a source-fidelity comprehension surface form, then type-directed elaboration normalizes it through the same target resolution, statement checking, dictionary evidence, tower rules, and nested `bind` / `return` lowering as SPEC-054 `do:K { ... }`.

The core equivalence is:

```ash
[f(x, y) | x <- xs, y <- ys]: List
```

is semantically the same as:

```ash
do:List {
    x <- xs;
    y <- ys;
    return f(x, y)
}
```

Modulo span/origin metadata, both forms must elaborate to the same nested bind/return artifact.

## 2. Motivation

SPEC-054 gives Ash an explicit computation-context view:

```ash
do:K {
    x <- computation;
    return f(x)
}
```

Some lawful monadic targets are more naturally read as container comprehensions: `List`, `Option`, `Result<_, E>`, and similar future pure-data constructors. Bracket comprehension syntax gives those targets a compact expression form without introducing a second algebra or a second lowering path.

The feature is intentionally conservative:

1. `do:K` remains the canonical semantic substrate.
2. Comprehensions do not add implicit target imports, tower lifts, guards, filtering, collection builders, or applicative lowering.
3. The first implementation slice may be blocked by missing user-definable `Monad<K>`, pure `List` / `Option` / `Result` dictionaries, and constructor-hole support.
4. Act and Proc remain valid targets if their existing SPEC-054 dictionaries accept the qualifier shapes; the bracket syntax is visual, not semantic.

## 3. Scope

In scope:

- Bracket comprehension expression syntax.
- Optional explicit postfix target annotation.
- Qualifier forms that mirror SPEC-054 do statements.
- Parser surface node preserving result expression, qualifiers, target annotation, and spans.
- Typed normalization through the generalized do elaboration path.
- Diagnostics that reuse do-notation families with comprehension-specific wording.

Out of scope for the MVP:

- Bare boolean guards.
- Pattern binders.
- Applicative, zip, parallel, or functor-only lowering.
- Monoid collection-builder semantics.
- Implicit imports of `guard`, `empty`, `filter`, `par`, `zip`, or related helpers.
- Implicit lifting between `Act`, `Proc`, `Option`, `Result`, `List`, or workflow layers.
- Workflow as a comprehension target.
- General law declaration syntax or generated law tests.

## 4. Surface Grammar

Normative grammar sketch:

```text
comprehension_expr ::= "[" expr "|" comp_qualifier ("," comp_qualifier)* "]" target_annotation?
target_annotation  ::= ":" do_target    // comprehension-specific postfix target
comp_qualifier     ::= comp_bind | comp_let
comp_bind          ::= IDENTIFIER "<-" expr
comp_let           ::= "let" IDENTIFIER "=" expr
```

MVP restrictions:

1. Binders are simple identifiers only.
2. At least one qualifier is required.
3. Qualifiers are comma-separated inside brackets.
4. The result expression appears before `|` and becomes the final `return` expression of the normalized do block.
5. A target annotation is required for the first implementation slice unless target inference is explicitly implemented and tested. The annotation is comprehension-specific postfix target syntax; it must not rely on a nonexistent general expression-level type-ascription parser.
6. The explicit target uses the same `DoTarget` representation and target-resolution rules as SPEC-054.

Examples:

```ash
[f(x) | x <- xs]: List
```

```ash
[parsed | raw <- read(path), parsed <- parse(raw)]: Act
```

```ash
[result | p <- proc::from_act(read(path)), result <- process(p)]: Proc
```

## 5. Core Equivalence and Normalization

The comprehension:

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

Qualifier translation:

| Comprehension qualifier | Do statement |
|-------------------------|--------------|
| `x <- expr` | `x <- expr;` |
| `_ <- expr` | `_ <- expr;` |
| `let x = expr` | `let x = expr;` |

The normalized do block is conceptual. An implementation may either:

1. create a synthetic `Expr::DoBlock` during typed elaboration; or
2. feed comprehension qualifiers directly into the same internal typed-do checker.

It must not lower parser-surface comprehensions directly to untyped `bind` / `return` calls.

## 6. Target Resolution

A comprehension target denotes a computation constructor, not a module name.

```ash
[result | qualifiers]: K
```

uses the explicit `K` target. The target must satisfy the same requirements as SPEC-054:

1. resolve to a known type constructor;
2. have effective kind `* -> *`;
3. provide `Monad<K>` evidence, or an MVP builtin dictionary shaped like that evidence;
4. synthesize the result type `K<A>`, where `A` is the checked type of the result expression.

Target inference without an annotation is reserved but not required for the first implementation slice:

```ash
[result | qualifiers]
```

If target inference is not implemented, the compiler must report a clear deferred/ambiguous-target diagnostic rather than choosing a target from the first qualifier by syntactic heuristic.

## 7. Typing Rules

Given target constructor `K` and qualifiers `q1..qn`:

1. Type-check qualifiers left to right in an environment extended by previous binders.
2. For `x <- expr`, require `expr : K<A>` and bind `x : A` in subsequent qualifiers and the result expression.
3. For `_ <- expr`, require `expr : K<A>` and do not bind a value name.
4. For `let x = expr`, type-check `expr : A` normally and bind `x : A`; if `A` is syntactically or semantically `K<B>` for the current target, emit the same non-fatal diagnostic family as SPEC-054 monadic-value-in-`let`.
5. Type-check the result expression as `A` in the final environment.
6. Synthesize the whole comprehension as `K<A>`.
7. Expected type may constrain `A`, but must not silently change `K`.

No implicit conversion exists between different constructors. A `do:Proc` / `Proc` comprehension cannot bind an `Act<A>` RHS unless the RHS is explicitly lifted, e.g. through `proc::from_act`.

## 8. Elaboration

Typed elaboration must produce the same effective nested structure as SPEC-054 do notation:

```ash
[f(x, y) | x <- xs, y <- ys]: K
```

elaborates as:

```text
bind_K(xs, |x|
    bind_K(ys, |y|
        return_K(f(x, y))))
```

The implementation may preserve source-origin metadata that says the original source form was a comprehension. That metadata is for diagnostics, tooling, formatting, and source mapping only; it must not change semantics.

## 9. Guards and Filtering

Bare boolean guards are not part of the Monad-only MVP.

Rejected in MVP:

```ash
[x | x <- xs, x > 0]: List
```

Reason: boolean guard syntax requires an additional contract such as `Alternative`, `MonadPlus`, `MonadZero`, a target-specific `guard`, or collection filtering semantics. SPEC-055 deliberately does not choose that algebra.

MVP spelling uses an ordinary monadic operation, imported or qualified by normal name-resolution rules:

```ash
[x | x <- xs, _ <- guard(x > 0)]: List
```

`guard` is not imported or interpreted specially by comprehension syntax.

## 10. Tower Behavior and Operational Failure

Comprehensions inherit SPEC-054 tower behavior.

1. A comprehension with target `Act` is an `Act<A>` computation, not a pure `A` expression.
2. A comprehension with target `Proc` is a `Proc<A>` computation and must obey Proc resource/split/join and process-failure boundaries through ordinary Proc operations.
3. No implicit lifting occurs across `Pure < Act < Proc < Workflow`.
4. `fail` remains tower-scoped operational bottom. It is not converted into `None`, `Err`, empty list, or any domain-level failure by comprehension syntax.

## 11. Parser and AST Requirements

The parser should add a surface node analogous to:

```text
Expr::Comprehension {
    result: Box<Expr>,
    qualifiers: Vec<ComprehensionQualifier>,
    target: Option<DoTarget>,
    span: Span,
}
```

with qualifiers analogous to:

```text
ComprehensionQualifier::Bind { binder, expr, span }
ComprehensionQualifier::Let { binder, expr, span }
```

Exact Rust names are implementation choices, but the data must preserve:

- source span of the whole comprehension;
- result-expression span;
- each qualifier span;
- binder spans;
- optional target annotation span.

Lowering before type checking must reject or defer comprehension nodes, matching the SPEC-054 parser-only `DoBlock` boundary.

## 12. Diagnostics

Diagnostics should mirror SPEC-054 while naming the comprehension form.

Cannot infer or missing target:

```text
error: cannot infer comprehension target
hint: add an explicit target annotation, e.g. `[expr | x <- xs]: List`
```

Wrong target kind:

```text
error: comprehension target Int has kind *, expected * -> *
hint: use a computation constructor such as Act, Proc, or a registered Monad target
```

Pure expression used with `<-`:

```text
error: '<-' in comprehension expects K<A>, found Int
hint: use `let x = expr` for pure bindings
```

Wrong constructor in qualifier:

```text
error: comprehension target Proc, but qualifier has type Act<Int>
hint: use an explicit lift such as `proc::from_act(...)` when sequencing Act work in Proc
```

Suspicious `let` binding of a monadic value:

```text
warning: `let` binds a monadic value without comprehending over it
hint: use `x <- xs` to bind its contents
```

Bare boolean qualifier:

```text
error: bare boolean qualifiers are not part of Monad-only comprehensions
hint: use an explicit monadic operation, e.g. `_ <- guard(condition)`
```

## 13. Interaction with SPEC-054

SPEC-054 remains the normative owner of generalized do-notation. SPEC-055 depends on it and must not fork its semantics.

Shared rules:

- same `DoTarget` target representation where possible;
- same target/kind/dictionary resolution;
- same statement typing after qualifier normalization;
- same Act/Proc MVP dictionaries while user-defined `Monad<K>` is deferred;
- same parser-only lowering prohibition;
- same tower/no-implicit-lift/fail behavior;
- same diagnostics, with comprehension-specific wording.

If SPEC-054 evolves from builtin dictionaries to a canonical `Monad<K>` interface, SPEC-055 should consume that interface without adding comprehension-specific dictionary machinery.

## 14. Implementation Status

As of Phase 106 closeout:

- DESIGN-032 is implemented as an MVP over SPEC-054.
- Parser/surface support exists for bracket comprehensions with source-fidelity qualifiers and comprehension-specific postfix targets.
- Parser-only lowering rejects comprehension nodes; typed checking owns semantic normalization.
- Type checking and typed elaboration normalize comprehensions through the existing SPEC-054 generalized typed-do path for MVP `Act` and `Proc` targets.
- Diagnostics cover missing explicit targets, wrong-kind targets, missing MVP dictionaries, pure RHS values used with `<-`, wrong-constructor RHS values, suspicious `let` bindings of monadic values, and bare boolean qualifier rejection/non-acceptance.
- Phase 106 still requires explicit targets. Target inference is deferred.
- Pure `List`, `Option`, and `Result<_, E>` comprehension examples remain future semantic targets, not implementation claims, until their dictionaries and constructor-hole support exist.

## 15. Deferred Extensions

Deferred extensions include:

1. target inference for unannotated comprehensions;
2. one-hole constructor targets such as `Result<_, ParseError>`;
3. user-defined `Monad<K>` implementations;
4. pattern binders;
5. boolean guards through an explicitly chosen guard/empty algebra;
6. applicative or zip/parallel comprehensions with distinct syntax and interface requirements;
7. formatter support that preserves or canonicalizes comprehension source style;
8. generated law/property tests for monadic interfaces.
