# SPEC-054: Generalized Typed Do-Notation

**Status:** Implemented MVP (Phase 105)
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

The target `K` is an explicit computation constructor. The block synthesizes `K<A>`, where `A` is the type of the final `return` expression after type checking. The original MVP implementation track was limited to compiler-known Act/Proc bridges; Phase 133 supersedes that present-tense boundary with selected `Monad<K>` evidence for implemented targets (`Act`, `Proc`, `Workflow`, `Option`, and supported `Result<_, E>` shapes), with any legacy bridge retained only as quarantined fallback when public evidence is unavailable.

This spec owns generalized do-notation syntax, target resolution, statement typing, typed elaboration, diagnostics, compatibility with legacy `act { ... }`, tower/purity behavior, and the rule that operational `fail` remains operational bottom rather than domain failure.

## 2. Motivation

Ash now has multiple computation-like layers:

- `Act<A>`: sequential effectful computation with hidden runtime-managed `ActEnv`.
- `Proc<A>`: process-capable computation over process identities, split/join policies, and explicit process handles.
- Pure data constructors such as `Option<A>` and supported `Result<A, E>` shapes that now have selected Phase 133 `Monad` evidence; `List<A>` has Functor/Monoid helper surfaces while full List Monad/comprehension semantics remain follow-up work.

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

This section records the implementation boundary that motivated the spec and the Phase 105 MVP now implemented.

Pre-Phase-105 baseline:

1. Expression-level `act { ... }` blocks parsed through `parse_act_block_expr` in `crates/ash-parser/src/parse_expr.rs`. Legacy syntax recognized `IDENTIFIER = expr;` and `ret expr;` only.
2. The surface AST had `Expr::ActBlock { stmts, span }` and `ActStmt::{Bind, Return}` in `crates/ash-parser/src/surface.rs`.
3. The parser lowerer handled `Expr::ActBlock` in `crates/ash-parser/src/lower.rs` by lowering to unqualified `unit`/`bind` calls and deciding whether a RHS was Act-like with syntactic heuristics.
4. The type checker handled `Expr::ActBlock` in `crates/ash-typeck/src/check_expr.rs` by always synthesizing `Act<A>`. It unwrapped `Act<A>` RHS values for binds and otherwise treated the RHS as pure.

Implemented Phase-105 MVP slices:

1. `do:K { ... }` parses into target-carrying `Expr::DoBlock` with `DoTarget` and `DoStmt::{Let, Bind, Return}`.
2. New-form expression `act { ... }` blocks that use generalized do grammar parse as `Expr::DoBlock` with target `Act`; legacy `act { x = ...; ret ...; }` remains an `Expr::ActBlock` compatibility carrier.
3. Raw parser-surface `lower_expr(Expr::DoBlock)` explicitly rejects until callers use typechecker-owned typed elaboration.
4. The type checker resolves MVP `Act` and `Proc` targets through hidden/builtin dictionary evidence, checks `let`/`<-`/`return` statements left-to-right, and exposes `elaborate_typed_do_block` for dictionary-directed core lowering.
5. Legacy `Expr::ActBlock` typechecking remains supported for compatibility and exposes a standalone migration-diagnostic carrier until a general warning pipeline is wired in.
6. Focused parser/typechecker tests cover the diagnostic families in §13, including target resolution errors, bind/return shape errors, migration hints, and the expression/workflow `act` ambiguity boundary.
Historical Phase-105 constraints:

1. Phase 105 was implemented before constructor-kinded interface/type-parameter syntax and impl-head support matured. Phase 133 adds the source-visible `std::algebra::Monad<M : * -> *>` surface and selected evidence for implemented targets, while arbitrary user Monad execution remains follow-up work.
2. `Act`, `Proc`, and `Workflow` remain builtin public carrier definitions in `TypeEnv`, and `act::unit`, `proc::unit`, `proc::bind`, `proc::from_act`, `proc::par`, `proc::await`, `proc::join`, and related operations exist as ordinary library/builtin values.
3. Phase 105 did not redefine Phase 104 capability implementation execution, authority admission, CLI binding configuration, or resource split/join work; Phase 133 likewise reuses those runtime substrates.

## 4. Scope

In scope for SPEC-054:

- `do:K { ... }` expression syntax.
- Statement forms: `let x = expr;`, `x <- expr;`, and final `return expr`.
- `act { ... }` compatibility as sugar or migration surface for `do:Act`.
- selected `Monad<K>` evidence for implemented targets, with the original Act/Proc builtin bridges treated as historical/quarantined fallback rather than ordinary hidden authority.
- Typed elaboration into target-specific `return`/`bind` operations.
- Tower/purity classification for `do:Act` and `do:Proc`.
- Diagnostics for target, kind, bind, return, migration, and tower mismatch errors.
- Explicit non-lifting between `Act`, `Proc`, `Result`, `Option`, `List`, and workflow contexts.

Out of scope or follow-up after Phase 133:

- Arbitrary user-defined `Monad<M>` execution beyond the selected stdlib/prelude evidence implemented for Act, Proc, Workflow, Option, and supported Result targets.
- General higher-kinded type lambdas beyond the current constructor-kinded interface surface.
- Pattern binding in `let` or `<-`.
- `Alternative`, `MonadFail`, `MonadPlus`, guard/empty syntax, or domain-failure sugar.
- Law declaration syntax and generated law-test tooling.
- Full List Monad/comprehension semantics until list bind semantics are chosen explicitly.
- Runtime scheduler, mailbox, resource, authority, or workflow-boundary changes beyond reusing existing Act/Proc/Workflow operations.

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

Original Phase-105 MVP restrictions, superseded where noted by Phase 133:

- Phase 105 accepted simple named unary constructors `Act` and `Proc`; Phase 133 adds selected evidence for `Workflow`, `Option`, and supported `Result<_, E>` targets.
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

### 5.2 Supported value-position hole targets

Phase 133 supports the single explicit value-position hole needed to view supported higher-arity constructors such as `Result<_, E>` as unary computation constructors:

```ash
do:Result<_, ParseError> {
    x <- parse_one(input);
    return x
}
```

The hole target elaborates to an effective unary constructor `λA. Result<A, ParseError>` and resolves through selected `Monad<Result<_, ParseError>>` evidence when that evidence is available. Unsupported hole shapes, extra holes, or ambiguous partial applications still fail closed with a targeted diagnostic rather than silently choosing an arbitrary partial application.

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

Historical Phase-105 MVP accepted targets:

```text
Act  : * -> *
Proc : * -> *
```

Phase 133 selected-evidence targets include:

```text
Act            : * -> *
Proc           : * -> *
Workflow       : * -> *
Option         : * -> *
Result<_, E>   : * -> * for supported one-hole targets
```

The target `K` is resolved by the type checker. The parser preserves the syntactic target as surface data; it must not lower `do:K` to `bind` calls before type checking.

Target requirements:

1. `K` must resolve to a known type constructor.
2. `K` must have effective kind `* -> *`.
3. The compiler must resolve selected `Monad<K>` evidence for `K`, with only quarantined legacy Act/Proc bridge fallback where public evidence is unavailable.
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

Phase 133 uses a compiler/prelude-known interface equivalent to:

```text
Monad<M> where M : * -> *
unit : A -> M<A>
bind : M<A> -> (A -> M<B>) -> M<B>
```

Selected evidence carries a target constructor, value constructor, `unit` operation, and `bind` operation. For source-visible stdlib evidence, those operations come from `std::algebra::Monad` impls or named prelude/tower shims. Any original Phase-105 Act/Proc hidden bridge is quarantined fallback for compatibility when public evidence is unavailable, not ordinary user authority and not imported into lexical scope.

Current selected evidence bindings include:

```text
Act      => act::unit / act::bind or named Act prelude shim evidence
Proc     => proc::unit / proc::bind
Workflow => workflow::unit / workflow::bind
Option   => stdlib/prelude Monad<Option> evidence
Result   => stdlib/prelude Monad<Result<_, E>> evidence for supported one-hole targets
```

Ordinary operations such as `proc::par`, `proc::await`, `proc::from_act`, `act::guard`, `Result::Err`, and Option/Result constructors remain ordinary names and must be in lexical scope or called qualified.

Monad laws are semantic obligations, not checker obligations in Phase 133:

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
| `Option` | Pure data | Phase 133 implements selected stdlib `Monad<Option>` evidence for explicit `do:Option`. |
| `Result<_, E>` | Pure data | Phase 133 implements selected stdlib/prelude `Monad<Result<_, E>>` evidence for explicit result do targets. |
| `List` | Pure data, partial follow-up | Phase 133 adds Functor/Monoid/List helper surfaces, but full List Applicative/Monad/comprehension execution remains follow-up work until list bind semantics are chosen explicitly. |
| `Workflow` | Workflow | Later phases added explicit `do:Workflow` selected evidence; fully self-hosted workflow runtime representation remains opaque. |

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

For `do:Result<_, E>`, `fail e` must not be interpreted as `Err(e)`. Domain failure syntax belongs to explicit constructors or future Alternative/MonadFail-like features.

## 13. Diagnostics

Diagnostics are part of the normative feature. Phase 105 must provide focused tests for at least these families:

1. unknown do target;
2. wrong target kind (`*` where `* -> *` is expected);
3. target has no selected `Monad<K>` evidence and no applicable quarantined bridge fallback;
4. `<-` RHS has the wrong constructor;
5. `<-` RHS is pure and should be `let`;
6. `let` binds a monadic value and does not sequence it;
7. missing final `return`;
8. `return` appears before the final statement;
9. trailing semicolon after final `return` in new grammar;
10. removed historical `ret`;
11. removed historical `x = effectful_expr;` inside `act {}`;
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
| [TASK-751](../plan/tasks/TASK-751-proc-do-integration-and-tower-behavior.md) | Validate `do:Proc`, explicit `from_act`, and tower behavior | Complete |
| [TASK-752](../plan/tasks/TASK-752-do-notation-diagnostics.md) | Implement targeted diagnostics and migration warnings | Complete |
| [TASK-753](../plan/tasks/TASK-753-do-notation-docs-examples-closeout.md) | Update examples/docs and close out Phase 105 | Complete |

## 16. Deferred Extensions

Deferred extensions after Phase 133 include:

- arbitrary user-defined unary constructor Monad execution beyond the implemented selected stdlib/prelude evidence paths;
- general type-lambda syntax beyond supported one-hole constructor targets;
- full `do:List` / List comprehension semantics;
- pattern binds;
- typed law declarations;
- generated property tests for law-bearing interfaces;
- `with`/reader-style environment do targets;
- additional workflow/runtime algebra beyond the current opaque Workflow evidence surface.

## 17. Changelog

### 2026-04-28

- Initial normative draft promoted from DESIGN-031.
- Historical Phase 105 scheduling limited the original MVP to Act/Proc builtin bridges; Phase 133 supersedes that with selected public/prelude `Monad<K>` evidence for implemented targets while leaving full arbitrary user Monad execution to follow-up work.
