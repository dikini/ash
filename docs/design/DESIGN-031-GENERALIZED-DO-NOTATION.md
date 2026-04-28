# DESIGN-031: Generalized Typed Do-Notation

**Status:** Promoted to [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) / [PLAN-101](../plan/PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md)
**Date:** 2026-04-28
**Related:** SPEC-047, SPEC-048, SPEC-049, SPEC-050, SPEC-054, PLAN-101, DESIGN-030

## 1. Summary

Ash should generalize the existing `act { ... }` block into explicit typed do-notation: `do:K { ... }`, where `K` is a computation constructor such as `Act`, `Proc`, or eventually `Result<_, E>`. The feature provides monadic sequencing through a `Monad<K>`-like contract while keeping pure bindings, monadic binds, operational failure, tower boundaries, and ordinary function scope explicit. The long-term semantic model is user-definable `Monad` implementations; Rust-side builtin dictionaries for `Act`/`Proc` are acceptable only as a temporary implementation bridge if constructor-kinded interfaces are not ready.

## 2. Rationale

Ash already has several monad-like structures:

- `Act<A>` for sequential effectful computation.
- `Proc<A>` for process-capable computation.
- `Option<A>`, `List<A>`, and `Result<A, E>` as pure data constructors with monadic or near-monadic behavior.

The current `act { ... }` block is effectively Act-specific do-notation. As `Proc<A>` becomes a first-class layer above `Act<A>`, Ash needs a uniform way to express sequential dependency in different computation contexts without duplicating bespoke block syntaxes.

The design should also support Ash's AI-facing goals. Syntax and diagnostics should be explicit enough for humans and LLMs to learn from compiler feedback: pure binding and monadic binding should not be ambiguous, tower crossings should be visible, and errors should explain expected computation shapes concisely.

## 3. Core Feature

A do-block has the form:

```ash
do:K {
    let x = pure_expr;
    y <- computation_expr;
    return result_expr
}
```

`K` is a computation constructor. For the MVP this means a unary constructor of effective kind `* -> *`, such as:

```ash
do:Act { ... }
do:Proc { ... }
do:Option { ... }
do:List { ... }
```

The block synthesizes type `K<A>`, where `A` is determined by the final `return` expression and surrounding expected type constraints.

`act { ... }` may remain as spelling sugar for `do:Act { ... }`, but it should use the same block grammar.

## 4. Computation Constructors

`do:K` is semantically about a computation constructor, not a module or namespace.

Required MVP shape:

```text
K : * -> *
```

Future target syntax should support one explicit value hole for higher-arity constructors:

```ash
do:Result<_, ParseError> { ... }
```

This elaborates to the unary constructor:

```text
λA. Result<A, ParseError>
```

Rules for future hole targets:

- Exactly one `_` marks the do-bound value position.
- The resulting target must elaborate to a unary computation constructor.
- Multiple holes are deferred.
- `do:Result { ... }` should produce a kind error with a hint such as: use `do:Result<_, E>` to fix the error type.

## 5. Monad Contract

The intended semantic mechanism is a `Monad<M>` interface over unary computation constructors:

```text
Monad<M> where M : * -> *
```

Required operations by type shape:

```text
return : A -> M<A>
bind   : M<A> -> (A -> M<B>) -> M<B>
```

The compiler/type checker enforces operation shapes only. Monad laws are semantic obligations, not type-checking obligations:

```text
bind(return(a), f)    == f(a)
bind(m, return)       == m
bind(bind(m, f), g)   == bind(m, |x| bind(f(x), g))
```

Law declaration syntax is deferred. Future tooling should be able to derive property tests for interface laws, not just Monad laws. SMT/Z3-assisted law checking is future research, not an MVP requirement.

## 6. Implementation Strategy

The design target is user-definable do targets via `Monad<M>` implementations.

Planning should first design the type-system extensions needed for constructor-kinded interfaces:

- kinding for constructor parameters such as `M : * -> *`;
- applying constructor parameters in types, e.g. `M<A>`;
- resolving `impl Monad<Act>`, `impl Monad<Proc>`, and user-defined unary constructors;
- using the resolved `Monad<K>` dictionary during typed do elaboration.

If that work is too large for the first implementation slice, Rust-side builtin dictionaries may temporarily provide `Monad<Act>` and `Monad<Proc>`. Such dictionaries must be shaped as hidden `Monad` implementations, not as unrelated do-specific magic, so they can later migrate to ordinary Ash-level interface resolution.

The `Monad` interface itself should be compiler/prelude-known for do-notation. Users should not need to import `Monad` for `do:K` to resolve the canonical do interface.

## 7. Block Grammar

MVP statement forms are deliberately explicit:

```ash
let x = expr;       // pure lexical binding
x <- expr;          // monadic bind; expr must have type K<A>
return expr         // terminal unit/return injection
```

Rules:

- `let` is pure only. It does not call `return`, `bind`, or lift into `K`.
- `<-` is the only monadic bind form. The RHS must have type `K<A>`.
- `return expr` is sugar for the target's `return(expr)` / `unit(expr)` operation.
- `return` in a do-block is not function-level control flow.
- `return(expr)` should also be valid call syntax where ordinary call syntax permits it.
- No implicit final-expression return.
- No bare monadic expression statements in MVP; use `_ <- action;` to sequence and ignore a value.
- `;` is sequencing/composition. Non-final statements use `;`; the final `return expr` has no trailing semicolon.

Canonical example:

```ash
do:Act {
    raw <- read(path);
    let parsed = parse(raw);
    return parsed
}
```

`ret` is an Act-era crutch and should not be part of the new grammar. If retained temporarily, it should be a deprecated alias only.

## 8. Desugaring Semantics

Do elaboration is recursive and equivalent to nested `bind` calls.

```ash
do:K {
    x <- mx;
    let y = f(x);
    z <- mz(y);
    return g(x, z)
}
```

elaborates semantically to:

```text
bind(mx, |x|
    let y = f(x) in
    bind(mz(y), |z|
        return(g(x, z))))
```

This is a typed elaboration, not parser lowering. The parser should preserve a surface node such as:

```text
DoBlock { target, stmts }
```

Typed elaboration must resolve `K`, check its kind, resolve `Monad<K>`, typecheck each statement, enforce purity/tower rules, and only then lower to calls.

## 9. Scope and Ordinary Operations

`do:K` does not import target-specific functions.

Only the monadic sequencing contract is implicit through the compiler/prelude-known `Monad` interface. Other operations remain ordinary names and require normal lexical scope or qualification.

Example:

```ash
use proc::{par, await};

do:Proc {
    handles <- par(task_a(), task_b());
    result <- await(handles.left);
    return result
}
```

or:

```ash
do:Proc {
    handles <- proc::par(task_a(), task_b());
    return handles
}
```

If `par` is not in scope and not qualified, it is not usable. Its compatibility with `do:Proc` is checked by its type, e.g. roughly:

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

Since `<-` expects `Proc<X>`, the result of `par` can be bound.

## 10. Tower, Purity, and Lifting

The do target determines the block's tower/purity level.

Examples:

- `do:List { ... }` is pure.
- `do:Option { ... }` is pure.
- `do:Act { ... }` is effectful, even if the body only says `return 1`.
- `do:Proc { ... }` is process-capable.

No implicit lifting occurs between computation constructors or tower levels.

Invalid without explicit lift:

```ash
do:Proc {
    x <- do:Act {
        y <- read(path);
        return y
    };
    return x
}
```

The inner block has type `Act<A>`, but `<-` in `do:Proc` expects `Proc<A>`. Use an explicit operation such as `proc::from_act(...)` when crossing the Act-to-Proc boundary.

`do:Proc` sequencing is sequential dependency. Parallelism and observation remain explicit:

```text
bind              = sequential dependency
par / scatter     = explicit concurrency
await / join / gather = explicit observation and joining
```

Applicative or MonadPlus structure may be useful later, but `do` means Monad sequencing in this design.

## 11. Operational Failure

Operational `fail` is not monadic/domain failure.

Inside `do:K`, `fail e` remains tower-scoped operational bottom. It is routed according to the current computation/tower context and is not converted into `None`, `Err`, an empty list, or any other domain-level failure value.

Examples:

- `do:Act { fail e }` raises Act/effect-level operational failure.
- `do:Proc { fail e }` raises Proc/process-level operational failure.
- `do:Result<_, E> { fail e }` should not mean `Err(e)`.

`with_error` should compose with do-blocks through the existing operational failure semantics. Domain failure helpers such as `guard`, `empty`, `Err`, or MonadFail/Alternative-style syntax are post-MVP.

## 12. Patterns

MVP binders are identifiers only:

```ash
x <- computation;
let y = pure;
```

Pattern binding is desirable long term but deferred:

```ash
(a, b) <- proc::join(ha, hb);
let Some { value } = maybe;
```

Pattern support requires separate decisions about refutable patterns, failure behavior, exhaustiveness, and interaction with operational bottom versus domain failure.

## 13. Diagnostics

Diagnostics are part of the feature. They should be concise, type-aware, and instructional for both humans and LLMs.

Required diagnostic families:

- Unknown do target.
- Target has wrong kind, e.g. `*` instead of `* -> *`.
- Missing `Monad<K>` implementation.
- `<-` RHS has wrong constructor.
- Pure expression used with `<-`; suggest `let`.
- Monadic value bound with `let`; warn that the computation is not run/bound.
- Missing explicit final `return`.
- Trailing semicolon after final `return`.
- Deprecated `ret`, if temporarily accepted.
- Cross-tower mismatch; suggest explicit lift such as `proc::from_act(...)` where known.

Example diagnostic style:

```text
error: '<-' in do:Proc expects Proc<A>, found Act<String>
hint: use proc::from_act(read(path)) to lift Act into Proc
```

```text
warning: 'let' binds an Act<String> value without running it
hint: use 'x <- read(path)' to bind the action result
```

## 14. Compatibility and Migration

This design prefers a clean break from the old Act-specific grammar.

Accepted new forms:

```ash
do:Act {
    x <- read(path);
    return x
}

act {
    x <- read(path);
    return x
}
```

Rejected legacy forms:

```ash
act {
    x = read(path);
    ret x
}
```

`act { ... }` may remain as spelling sugar for `do:Act { ... }`, but it should not preserve the old ambiguous bind/return grammar.

## 15. Deferred Work

Deferred from MVP:

- Full HKT and arbitrary type lambdas.
- Hole targets such as `do:Result<_, E>` if constructor holes are not ready.
- User-defined `Monad` implementations if constructor-kinded interfaces are not ready in the first slice.
- Pattern binding in `let` and `<-`.
- Guard/failure syntax based on Alternative, MonadPlus, or MonadFail.
- Applicative/parallel notation.
- Law declaration syntax and generated law-test tooling.
- SMT/Z3-assisted law checking.
- Workflow do-targets.

## 16. Promotion Notes

This design note is now promoted into:

- [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), which owns the normative `do:K` syntax, MVP Act/Proc target rules, typed elaboration semantics, diagnostics, tower/failure behavior, and legacy `act { ... }` migration contract.
- [PLAN-101](../plan/PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md), which schedules Phase 105 after the active Phase 104 capability/resource implementation work and breaks implementation into TASK-746 through TASK-753.

Resolved planning positions:

1. Full `M : * -> *` user-defined `Monad<M>` support remains the design target, but Phase 105 uses Act/Proc builtin dictionaries shaped like future Monad evidence.
2. The first implementation slice does not implement `do:Result<_, E>` holes, pure `Option`/`List` targets, pattern binds, law syntax, or workflow do-targets.
3. The parser must preserve a target-carrying `DoBlock` surface node until type-directed elaboration; parser-only lowering to unqualified `unit`/`bind` is explicitly non-normative for generalized do.
4. `act { ... }` migration is planned as compatibility sugar for `do:Act`, with legacy `x = ...; ret ...;` accepted only temporarily or migrated under explicit compatibility tests.
5. Phase 105 must not redefine Phase 104's Ash-defined capability implementation execution, authority admission, CLI binding configuration, or resource split/join semantics.
