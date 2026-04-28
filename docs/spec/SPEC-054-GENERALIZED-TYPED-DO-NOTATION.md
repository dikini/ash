# SPEC-054: Generalized Typed Do-Notation

**Status:** Draft
**Date:** 2026-04-28
**Promotes:** [DESIGN-031](../design/DESIGN-031-GENERALIZED-DO-NOTATION.md)
**Related:** [SPEC-002](SPEC-002-SURFACE.md), [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-004](SPEC-004-SEMANTICS.md), [SPEC-027](SPEC-027-PURE-FUNCTIONS.md), [SPEC-031](SPEC-031-FIRST-CLASS-FUNCTIONS.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md)
**Plan:** [PLAN-101](../plan/PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md)
**Implementation Tasks:** [TASK-746](../plan/tasks/TASK-746-generalized-do-notation-spec-plan-packet.md) through [TASK-753](../plan/tasks/TASK-753-do-notation-docs-examples-closeout.md)

## 1. Summary

Ash generalizes the Act-specific `act { ... }` sequencing form into explicit typed do-notation:

```ash
do:K {
    let x = pure_expr;
    y <- computation_expr;
    return result_expr
}
```

The target `K` is an explicit computation constructor. The block synthesizes `K<A>`, where `A` is the type of the final `return` expression after type checking. In the MVP implementation track, `K` is limited to compiler-known builtin dictionaries for `Act` and `Proc`; the normative design is intentionally shaped so the temporary bridge can later be replaced by a canonical `Monad<K>`-like interface over unary computation constructors.

This spec owns generalized do-notation syntax, target resolution, statement typing, typed elaboration, diagnostics, compatibility with legacy `act { ... }`, tower/purity behavior, and the rule that operational `fail` remains operational bottom rather than domain failure.

## 2. Motivation

Ash now has multiple computation-like layers:

- `Act<A>`: sequential effectful computation with hidden runtime-managed `ActEnv`.
- `Proc<A>`: process-capable computation over process identities, split/join policies, and explicit process handles.
- Pure data constructors such as `Option<A>`, `List<A>`, and `Result<A, E>` that may eventually have lawful monadic structure.

SPEC-047 introduced Act-specific block syntax:

```ash
act {
    x = read(path);
    ret x;
}
```

That syntax solved the first effectful-computation problem but mixed pure binding and monadic binding behind `=`, used `ret`, and lowered through Act-specific parser/lowering heuristics. As `Proc<A>` and other computation constructors mature, Ash needs a uniform, explicit sequencing form that does not silently cross tower boundaries or depend on implicit inference.

The explicit `do:K` annotation preserves Ash's design preference for minimal surprise: the user names the computation constructor, `<-` is the only monadic bind form, `let` is ordinary pure lexical binding, and `return` is the target's unit operation within the block rather than function-level control flow.

## 3. Implementation Baseline and Phase-105 Progress

This section records the implementation boundary that motivated the spec and the Phase 105 slices implemented so far.

Pre-Phase-105 baseline:

1. Expression-level `act { ... }` blocks parsed through `parse_act_block_expr` in `crates/ash-parser/src/parse_expr.rs`. Legacy syntax recognized `IDENTIFIER = expr;` and `ret expr;` only.
2. The surface AST had `Expr::ActBlock { stmts, span }` and `ActStmt::{Bind, Return}` in `crates/ash-parser/src/surface.rs`.
3. The parser lowerer handled `Expr::ActBlock` in `crates/ash-parser/src/lower.rs` by lowering to unqualified `unit`/`bind` calls and deciding whether a RHS was Act-like with syntactic heuristics.
4. The type checker handled `Expr::ActBlock` in `crates/ash-typeck/src/check_expr.rs` by always synthesizing `Act<A>`. It unwrapped `Act<A>` RHS values for binds and otherwise treated the RHS as pure.

Implemented Phase-105 slices:

1. `do:K { ... }` parses into target-carrying `Expr::DoBlock` with `DoTarget` and `DoStmt::{Let, Bind, Return}`.
2. New-form expression `act { ... }` blocks that use generalized do grammar parse as `Expr::DoBlock` with target `Act`; legacy `act { x = ...; ret ...; }` remains an `Expr::ActBlock` compatibility carrier.
3. Raw parser-surface `lower_expr(Expr::DoBlock)` explicitly rejects until callers use typechecker-owned typed elaboration.
4. The type checker resolves MVP `Act` and `Proc` targets through hidden/builtin dictionary evidence, checks `let`/`<-`/`return` statements left-to-right, and exposes `elaborate_typed_do_block` for dictionary-directed core lowering.
5. Legacy `Expr::ActBlock` typechecking remains supported for compatibility and exposes a standalone migration-diagnostic carrier until a general warning pipeline is wired in.
Remaining constraints:

1. The current kind system has `Kind::Type` and `Kind::Arrow`, but interface/type-parameter syntax and impl heads do not yet support constructor-kinded parameters such as `M : * -> *`.
2. `Act`, `Proc`, and `P` are registered as builtin public type definitions in `TypeEnv`, and `proc::unit`, `proc::bind`, `proc::from_act`, `proc::par`, `proc::await`, `proc::join`, and related operations exist as ordinary library/builtin values.
3. Phase 104 is active/in-flight for Ash-defined capability implementation bodies and pilot DX. Generalized do-notation implementation is scheduled as Phase 105 and must not redefine Phase 104 capability implementation execution, authority admission, CLI binding configuration, or resource split/join work.

## 4. Scope

In scope for SPEC-054:

- `do:K { ... }` expression syntax.
- Statement forms: `let x = expr;`, `x <- expr;`, and final `return expr`.
- `act { ... }` compatibility as sugar or migration surface for `do:Act`.
- MVP builtin dictionaries for `Act` and `Proc` shaped like future `Monad<K>` evidence.
- Typed elaboration into target-specific `return`/`bind` operations.
- Tower/purity classification for `do:Act` and `do:Proc`.
- Diagnostics for target, kind, bind, return, migration, and tower mismatch errors.
- Explicit non-lifting between `Act`, `Proc`, `Result`, `Option`, `List`, and workflow contexts.

Out of scope for the MVP implementation phase:

- Full user-defined constructor-kinded `Monad<M>` implementations.
- General higher-kinded type lambdas.
- `do:Result<_, E>` hole targets in the first implementation slice.
- Pattern binding in `let` or `<-`.
- `Alternative`, `MonadFail`, `MonadPlus`, guard/empty syntax, or domain-failure sugar.
- Law declaration syntax and generated law-test tooling.
- Workflow as a do target.
- Runtime scheduler, mailbox, resource, authority, or workflow-boundary changes beyond reusing existing `Act`/`Proc` operations.

## 5. Surface Grammar

### 5.1 Do block expression

Normative grammar sketch:

```text
do_expr      ::= "do" ":" do_target "{" do_stmt* do_return "}"
do_target    ::= type_constructor_target
do_stmt      ::= let_stmt | bind_stmt
let_stmt     ::= "let" IDENTIFIER "=" expr ";"
bind_stmt    ::= IDENTIFIER "<-" expr ";"
do_return    ::= "return" expr
```

MVP restrictions:

- The target must be a simple named unary constructor: `Act` or `Proc`.
- Binders are simple identifiers only.
- Non-final statements require semicolons.
- The final `return expr` has no trailing semicolon in the new grammar.
- No implicit final-expression return exists.
- Bare expression statements are rejected; use `_ <- computation;` to explicitly sequence and ignore a result.

Examples:

```ash
do:Act {
    raw <- read(path);
    let parsed = parse(raw);
    return parsed
}
```

```ash
do:Proc {
    ha <- proc::par(task_a(), task_b());
    joined <- proc::join(ha._0, ha._1);
    return joined
}
```

### 5.2 Future hole targets

A later extension may support one explicit value-position hole for higher-arity constructors:

```ash
do:Result<_, ParseError> {
    x <- parse_one(input);
    return x
}
```

The hole target elaborates to an effective unary constructor `λA. Result<A, ParseError>`. The MVP reserves this shape but does not require parsing or elaborating it. If parsed before implementation, it must produce a clear deferred-feature diagnostic rather than silently choosing an arbitrary partial application.

### 5.3 `act { ... }` compatibility

`act { ... }` remains source-level sugar for `do:Act { ... }` after migration:

```ash
act {
    x <- read(path);
    return x
}
```

The legacy SPEC-047 grammar:

```ash
act {
    x = read(path);
    ret x;
}
```

is deprecated by this spec but must not be removed without an explicit compatibility gate. The implementation plan must choose one of these migration modes:

1. accept both legacy and new Act forms temporarily, exposing migration diagnostics for legacy `ret` and `x = effectful_expr;` through a standalone carrier until TASK-752 wires general warning emission;
2. gate the new grammar behind a parser flag or phase branch while Phase 104 completes; or
3. perform a breaking grammar migration only after examples, stdlib, and tests have been updated in the same task.

Phase 105 chooses mode 1 unless implementation review finds dual parsing creates unacceptable ambiguity.

## 6. Computation Targets and Kinding

A do target denotes a computation constructor, not a module name.

MVP accepted targets:

```text
Act  : * -> *
Proc : * -> *
```

The target `K` is resolved by the type checker. The parser preserves the syntactic target as surface data; it must not lower `do:K` to `bind` calls before type checking.

Target requirements:

1. `K` must resolve to a known type constructor.
2. `K` must have effective kind `* -> *`.
3. The compiler must resolve `Monad<K>` evidence or an MVP builtin dictionary for `K`.
4. The resulting block type is `K<A>`, where `A` is the checked result type of the final `return` expression.
5. Expected type constraints may constrain `A`, but they must not change the selected target constructor.

Wrong-kind example:

```ash
do:Int {
    return 1
}
```

Diagnostic intent:

```text
error: do target Int has kind *, expected * -> *
hint: use a computation constructor such as Act or Proc
```

## 7. Monad Contract

The intended long-term mechanism is a compiler/prelude-known interface equivalent to:

```text
Monad<M> where M : * -> *
return : A -> M<A>
bind   : M<A> -> (A -> M<B>) -> M<B>
```

The MVP implementation may use hidden Rust dictionaries for `Act` and `Proc`, but those dictionaries must have this shape:

```text
DoDictionary<K> {
    target_name: K,
    return_op: A -> K<A>,
    bind_op: K<A> -> (A -> K<B>) -> K<B>,
    tower_level: Pure | Effectful | Proc | Workflow,
}
```

MVP dictionary bindings:

```text
Act  => act::unit / act::bind or the existing hidden Act bridge operations
Proc => proc::unit / proc::bind
```

These are not ordinary imports into user scope. They are compiler-known sequencing evidence. Ordinary operations such as `proc::par`, `proc::await`, `proc::from_act`, `act::guard`, and `Result::Err` remain ordinary names and must be in lexical scope or called qualified.

Monad laws are semantic obligations, not checker obligations in Phase 105:

```text
bind(return(a), f)  == f(a)
bind(m, return)     == m
bind(bind(m,f), g)  == bind(m, |x| bind(f(x), g))
```

Law syntax, proof, SMT/Z3 assistance, and generated law-test tooling are deferred.

## 8. Statement Typing

Given a resolved target constructor `K` and dictionary `D_K`:

### 8.1 Pure `let`

```ash
let x = expr;
```

Rules:

- `expr` is checked as an ordinary expression.
- `x` is bound to the type of `expr` in the lexical environment for later statements.
- `let` never calls `return`, never calls `bind`, and never lifts `expr` into `K`.
- If `expr` has type `K<A>`, the statement is legal as a pure binding of a computation value, but the compiler should warn that the computation is not sequenced.

### 8.2 Monadic bind

```ash
x <- expr;
```

Rules:

- `expr` must check as `K<A>` for the current target `K`.
- `x` is bound to `A` in later statements.
- If `expr` is pure `A`, this is an error; use `let x = expr;`.
- If `expr` is `K2<A>` for `K2 != K`, this is an error; use an explicit lift when one exists.
- `_ <- expr;` is the explicit ignored-result sequencing form.

### 8.3 Return

```ash
return expr
```

Rules:

- `expr` checks as ordinary type `A`.
- The do block synthesizes `K<A>`.
- `return` inside a do block is target-unit sugar, not function-level control flow.
- `return(expr)` may be supported later only if it can be parsed without ambiguity; MVP examples use keyword syntax.
- A final semicolon after `return expr` is rejected or warned as a legacy grammar error according to parser migration mode.

## 9. Typed Elaboration

Elaboration is type-directed and happens after the target and statements have been checked.

Source:

```ash
do:K {
    x <- mx;
    let y = f(x);
    z <- mz(y);
    return g(x, z)
}
```

Semantic elaboration:

```text
bind_K(mx, |x|
    let y = f(x) in
    bind_K(mz(y), |z|
        return_K(g(x, z))))
```

The parser must produce a surface carrier similar to:

```text
DoBlock {
    target: DoTarget,
    stmts: Vec<DoStmt>,
    span: Span,
}
```

A lowering pass may produce ordinary core calls only after typed elaboration has resolved:

- target constructor identity;
- target kind;
- dictionary/evidence operations;
- statement-local types;
- source spans for all diagnostics.

Parser-only lowering to unqualified `unit` and `bind` is not valid for generalized do-notation.

## 10. Scope and Name Resolution

`do:K` does not import target-specific operations.

Valid qualified operation use:

```ash
do:Proc {
    handles <- proc::par(task_a(), task_b());
    pair <- proc::join(handles._0, handles._1);
    return pair
}
```

Invalid unqualified use unless imported by ordinary `use` rules:

```ash
do:Proc {
    handles <- par(task_a(), task_b());
    return handles
}
```

The do target itself resolves through type-constructor resolution, not value/module lookup. A module named `Proc` does not make `do:Proc` valid unless a type constructor `Proc` of kind `* -> *` is also in scope.

## 11. Tower, Purity, and Lifting

Target tower classification:

| Target | Tower level | Notes |
| --- | --- | --- |
| `Act` | Effectful | `do:Act { return 1 }` is an effectful computation value of type `Act<Int>`. |
| `Proc` | Proc | `do:Proc { return 1 }` is a process-capable computation value of type `Proc<Int>`. |
| `Option`/`List`/`Result<_, E>` | Pure data, deferred | Reserved for later after general dictionaries. |
| `Workflow` | Workflow, deferred | Not an MVP do target. |

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

The inner block has type `Act<A>`, but `<-` in `do:Proc` expects `Proc<A>`. Use the explicit boundary:

```ash
do:Proc {
    x <- proc::from_act(do:Act {
        y <- read(path);
        return y
    });
    return x
}
```

`Proc<Act<A>>` remains a process computation whose normal result is a suspended `Act<A>`. It does not flatten implicitly.

Purity checking must be generalized from the current Act-only rule. A pure `fn -> A` cannot construct or execute `do:Act` or `do:Proc` blocks unless the type system explicitly permits carrying suspended computations as pure data in a later, narrower rule. For Phase 105, `do:Act` and `do:Proc` are rejected in pure-returning function bodies just as current `act {}` blocks are rejected.

## 12. Operational Failure

Operational `fail` is not monadic/domain failure.

Inside `do:K`, `fail e` remains tower-scoped operational bottom. It is routed according to the current tower/entity identity and must not be converted into:

- `None`;
- `Err(e)`;
- an empty list;
- any other domain-level failure value.

Examples:

```ash
do:Act {
    fail "missing authority"
}
```

raises an effectful operational failure.

```ash
do:Proc {
    fail "child failed"
}
```

raises a process/tower operational failure.

A future `do:Result<_, E>` must not interpret `fail e` as `Err(e)`. Domain failure syntax belongs to explicit constructors or future Alternative/MonadFail-like features.

## 13. Diagnostics

Diagnostics are part of the normative feature. Phase 105 must provide focused tests for at least these families:

1. unknown do target;
2. wrong target kind (`*` where `* -> *` is expected);
3. target has no `Monad` evidence or builtin dictionary;
4. `<-` RHS has the wrong constructor;
5. `<-` RHS is pure and should be `let`;
6. `let` binds a monadic value and does not sequence it;
7. missing final `return`;
8. `return` appears before the final statement;
9. trailing semicolon after final `return` in new grammar;
10. deprecated legacy `ret`;
11. legacy `x = effectful_expr;` inside `act {}`;
12. Act-to-Proc mismatch requiring `proc::from_act`;
13. parser ambiguity between expression-level `act { ... }` and workflow-level `act provider:action`.

Example:

```text
error: '<-' in do:Proc expects Proc<A>, found Act<String>
hint: use proc::from_act(...) to lift Act into Proc explicitly
```

Example:

```text
warning: 'let' binds an Act<String> value without sequencing it
hint: use 'x <- read(path);' if you intended to bind the action result
```

## 14. Phase 104 Coordination and Non-Interference

Phase 104 owns Ash-defined capability implementation execution and pilot DX:

- TASK-741: execution of Ash-defined capability implementation bodies;
- TASK-742: adapter/mock/replay examples;
- TASK-743: CLI/engine binding configuration;
- TASK-744: internal KV and test-clock pilots;
- TASK-745: final docs/examples/verification.

SPEC-054/Phase 105 must not redefine those semantics. In particular:

- do-notation does not change capability implementation conformance rules;
- do-notation does not add ambient authority or bypass `uses` admission;
- do-notation does not change resource binding ownership or split/join policy;
- do-notation does not change CLI binding configuration;
- do-notation may use the execution substrate produced by TASK-741, but it must depend on it rather than duplicating it.

Phase 105 implementation should start after Phase 104 is complete unless a user explicitly creates an isolated worktree and limits work to docs/spec review or parser-only experiments that do not touch capability implementation execution paths.

## 15. Implementation Plan Summary

The implementation plan is [PLAN-101](../plan/PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md).

Task summary:

| Task | Purpose | Status |
| --- | --- | --- |
| [TASK-746](../plan/tasks/TASK-746-generalized-do-notation-spec-plan-packet.md) | Promote DESIGN-031 into SPEC-054/PLAN-101 and establish Phase 105 traceability | Complete |
| [TASK-747](../plan/tasks/TASK-747-do-block-surface-ast-and-parser-substrate.md) | Add surface AST and parser substrate for `do:K` | Complete |
| [TASK-748](../plan/tasks/TASK-748-do-target-kinding-and-dictionary-resolution.md) | Add target kinding and MVP dictionary resolution | Complete |
| [TASK-749](../plan/tasks/TASK-749-typed-do-elaboration-and-lowering.md) | Type and elaborate do blocks to target `return`/`bind` calls | Complete |
| [TASK-750](../plan/tasks/TASK-750-act-block-compatibility-and-migration.md) | Migrate new-form `act {}` onto generalized grammar and preserve legacy carrier diagnostics | Complete |
| [TASK-751](../plan/tasks/TASK-751-proc-do-integration-and-tower-behavior.md) | Validate `do:Proc`, explicit `from_act`, and tower behavior | Planned |
| [TASK-752](../plan/tasks/TASK-752-do-notation-diagnostics.md) | Implement targeted diagnostics and migration warnings | Planned |
| [TASK-753](../plan/tasks/TASK-753-do-notation-docs-examples-closeout.md) | Update examples/docs and close out Phase 105 | Planned |

## 16. Deferred Extensions

Deferred extensions include:

- `interface Monad<M : * -> *>` in surface Ash;
- applying constructor parameters in user type syntax, e.g. `M<A>`;
- `impl Monad<Act>` and user-defined unary constructors;
- `do:Result<_, E>` with exactly one explicit value hole;
- pure `do:Option`/`do:List`;
- pattern binds;
- typed law declarations;
- generated property tests for law-bearing interfaces;
- `with`/reader-style environment do targets;
- workflow do targets.

## 17. Changelog

### 2026-04-28

- Initial normative draft promoted from DESIGN-031.
- Schedules Phase 105 after active Phase 104 and limits the MVP to Act/Proc builtin dictionaries before full constructor-kinded user-defined `Monad` support.
