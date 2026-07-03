# SPEC-072: Tower Callable Type and Closure Syntax

**Status:** Implemented MVP
**Date:** 2026-05-26
**Amends:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-027](SPEC-027-PURE-FUNCTIONS.md), [SPEC-031](SPEC-031-FIRST-CLASS-FUNCTIONS.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
**Builds on:** [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
**Plan:** [PLAN-121](../plan/PLAN-121-TOWER-CALLABLE-SYNTAX.md)
**Implementation Tasks:** [TASK-955](../plan/tasks/TASK-955-tower-callable-syntax-packet.md) through [TASK-963](../plan/tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md), with [TASK-962](../plan/tasks/TASK-962-tower-callable-syntax-closeout.md) as the final closeout gate

> **Target reconciliation.** This implemented callable syntax spec remains
> active for callable spelling and reserved tower arrows. References to
> capability/provider availability in Act-callable application are compatibility
> notes; target semantics should use computation-row requirements and
> provider/handler admission.

## 1. Summary

Ash callable types and closure literals use a tower-aligned arrow family:

```text
(A, B) ->  C    pure callable
(A, B) -*> C    Act callable, reserved until Act-callable application semantics land
(A, B) =>  C    Proc callable, reserved until Proc-callable application semantics land
(A, B) =*> C    Workflow callable, reserved until Workflow-callable application semantics land
```

For closures, the same arrow after the binder classifies the closure's application stratum:

```ash
|x, y| -> x + y      // pure closure
|x| -*> { ... }      // reserved Act closure
|x| => { ... }       // reserved Proc closure
|x| =*> { ... }      // reserved Workflow closure
```

The arrow classifies application power. The return type classifies the produced value. Therefore a pure callable may return `Act<A>`, `Proc<A>`, or `Workflow<A>` as an ordinary value without becoming an Act/Proc/Workflow callable.

## 2. Motivation

The older spelling `Fn(A, B) -> C` makes `Fn(...)` look like the function constructor even though the semantic constructor is the arrow from argument domain to result. It also creates avoidable confusion between a comma-separated function-argument list and ordinary tuple types.

This spec replaces that daily-use source spelling with arrow-first syntax and reserves distinct arrows for future callable values whose application itself requires Act, Proc, or Workflow environment power.

This keeps two axes separate:

1. **Application stratum:** what environment/power is required to apply the callable.
2. **Result type:** what value is produced by successful application.

Examples:

```ash
(Spec) -> Workflow<Result>    // pure smart constructor of a workflow value
(Spec) =*> Result             // workflow-level callable producing Result
(Path) -> Act<String>         // pure smart constructor of an Act value
(Path) -*> String             // Act-level callable producing String
(Job) -> Proc<Report>         // pure smart constructor of a Proc value
(Job) => Report               // Proc-level callable producing Report
```

## 3. Normative terms

- **Callable type:** a type whose values may be applied to arguments.
- **Callable arrow:** one of `->`, `-*>`, `=>`, or `=*>` in callable-type or closure-literal position.
- **Callable stratum:** the tower level required to apply a callable: Pure, Act, Proc, or Workflow.
- **Argument domain:** the source syntax left of a callable arrow. In callable-type position this is not automatically an ordinary tuple type.
- **Pure smart constructor:** a pure callable that constructs and returns an `Act<A>`, `Proc<A>`, or `Workflow<A>` value without entering that stratum during application.
- **Reserved syntax:** syntax the parser must not repurpose. It may either parse to a reserved marker and be rejected by typed elaboration, or be rejected by the parser with a targeted "reserved for future" diagnostic.

## 4. Scope

### 4.1 In scope for this spec

1. Source callable-type grammar for pure, Act, Proc, and Workflow arrows.
2. Source closure-literal grammar aligned with the same arrows.
3. Pure callable type semantics for the new `->` spelling.
4. Pure closure syntax `|args| -> body` as the preferred closure shorthand.
5. Compatibility handling for legacy `Fn(args...) -> ret` type syntax.
6. Reservation rules and diagnostics for Act/Proc/Workflow callable type arrows and closure arrows until their runtime semantics are implemented.
7. Internal representation guidance for future callable strata without forcing immediate runtime semantics.

### 4.2 Out of scope for the first implementation slice

1. Implementing Act callable application semantics for `-*>` values.
2. Implementing Proc callable application semantics for `=>` values.
3. Implementing Workflow callable application semantics for `=*>` values.
4. Serializing closures or sending closures across process/workflow boundaries.
5. Partial application or automatic currying.
6. Fully inferred closure return annotations beyond existing inference boundaries.
7. Replacing generalized `do:K` syntax or tower algebra operations.

The syntax in items 1-3 is still reserved by this spec.

## 5. Callable type grammar

### 5.1 Arrow family

```text
callable-arrow ::= "->" | "-*>" | "=>" | "=*>"
```

Spacing is permitted around the arrow. Lexing MUST use maximal munch so `=*>` is one token in callable contexts, not `=` followed by `*>`, and `-*>` is one token, not `-` followed by `*>`.

### 5.2 Callable type production

```text
callable-type ::= callable-domain callable-arrow type

callable-domain ::=
    type                                      // unary callable
  | "(" ")"                                 // nullary callable, if nullary values are admitted
  | "(" type ")"                            // unary callable with grouping
  | "(" type "," type ("," type)* ","? ")" // n-ary callable, n >= 2
```

The callable arrow is right-associative unless parentheses force grouping:

```text
A -> B -> C       == A -> (B -> C)
(A, B) -> C       == one callable with two arguments
((A, B) -> C)     == grouped callable type
```

Implementations MAY initially accept only the forms needed by existing function positions, but any accepted form must obey this associativity.

### 5.3 Argument domain is not a tuple by default

In callable-type position, `(A, B)` immediately to the left of a callable arrow denotes a two-argument domain, not a single tuple argument.

```text
(A, B) -> C       // two arguments: A and B
```

A unary callable whose single argument is a tuple must be written in a non-ambiguous form. First-slice implementations MUST support at least one of:

```ash
type Pair = (A, B);
Pair -> C
```

or, if parenthesized type grouping is implemented:

```text
((A, B)) -> C
```

The parser/typechecker task MUST audit the live tuple-type grammar and choose the supported unary-tuple spelling before implementation. It must not silently treat `(A, B) -> C` as a unary tuple-argument callable.

Implementation requirement: callable-domain parsing MUST be a separate syntactic path from tuple-type parsing. For `(T1, T2, ..., Tn) -> R`, `n >= 2`, the parser/lowering path must produce a callable domain with argument list `[T1, T2, ..., Tn]` directly. It must not first produce `Type::Tuple([T1, T2, ..., Tn])` or `TypeExpr::Tuple(...)` and then wrap that tuple as a unary callable argument. A tuple may appear as a single argument only through an explicitly supported unary tuple spelling such as a type alias or a separately audited `((A, B)) -> C` form.

### 5.4 Pure callable type

```text
(A, B) -> C
```

This is the preferred source spelling for a pure callable with two arguments and result `C`.

The canonical semantic shape is:

```rust
CallableType {
    stratum: CallableStratum::Pure,
    args: vec![A, B],
    ret: C,
}
```

For compatibility with current internals, first-slice implementations MAY lower pure callable types to the existing `Type::Fn(Vec<Type>, Box<Type>)`. If a future shared callable carrier is introduced, it MUST preserve the stratum explicitly rather than inferring it from the return type.

### 5.5 Act callable type

```text
(A, B) -*> C
```

This syntax is reserved for a callable whose application occurs in the Act/effectful stratum. Applying such a callable requires the Act environment: capability/provider availability, sequential effect state, policy/capability admissibility, and provenance/effect recording.

It is not equivalent to:

```text
(A, B) -> Act<C>
```

The latter is a pure callable that constructs an `Act<C>` value.

Until Act callable values are implemented, source using `-*>` in callable-type position MUST be rejected with a reserved-feature diagnostic.

### 5.6 Proc callable type

```text
(A, B) => C
```

This syntax is reserved for a callable whose application occurs in the Proc stratum. Applying such a callable requires process identity/scope, process environment, scheduler/process-composition context, and later process observation/failure routing as defined by SPEC-048/SPEC-049/SPEC-050.

It is not equivalent to:

```text
(A, B) -> Proc<C>
```

The latter is a pure callable that constructs a `Proc<C>` value.

Until Proc callable values are implemented, source using `=>` in callable-type position MUST be rejected with a reserved-feature diagnostic.

### 5.7 Workflow callable type

```text
(A, B) =*> C
```

This syntax is reserved for a callable whose application occurs in the Workflow stratum. Applying such a callable requires workflow governance: admission, roles/capabilities/resources, contract obligations, reporting, and workflow-boundary failure behavior.

It is not equivalent to:

```text
(A, B) -> Workflow<C>
```

The latter is a pure callable that constructs a `Workflow<C>` value.

Until Workflow callable values are implemented, source using `=*>` in callable-type position MUST be rejected with a reserved-feature diagnostic.

## 6. Closure literal grammar

### 6.1 Unified closure form

```text
closure-literal ::= "|" closure-param-list? "|" callable-arrow closure-body
closure-body    ::= expr | block
```

Examples:

```ash
|x| -> x + 1
|x, y| -> x + y
|x: Int, y: Int| -> x + y
|x| -> { x + 1 }
```

The closure arrow classifies the closure's application stratum. It is not a generic body separator independent of tower semantics.

### 6.2 Pure closure

```ash
|x, y| -> x + y
```

This is the preferred shorthand for a pure closure. It desugars to the equivalent anonymous pure function expression, preserving capture and arity rules from SPEC-031.

A first-slice implementation MUST implement or preserve pure closure semantics only if the current closure runtime is otherwise enabled for the containing context. This spec changes the spelling and classification, not the closure capture lifetime model.

A pure closure remains Pure-stratum even when written inside an Act, Proc, or Workflow context. The enclosing context does not upgrade `|args| -> body` into an effectful/process/workflow closure. Its captures and body must satisfy pure-closure rules; higher-stratum captures/body operations require the reserved higher-stratum closure arrows and remain rejected until those arrows are implemented.

### 6.3 Act/Proc/Workflow closures are reserved

The following forms are reserved:

```ash
|x| -*> { ... }
|x| => { ... }
|x| =*> { ... }
```

Until corresponding callable strata are implemented, these forms MUST be rejected with targeted diagnostics:

```text
Act closures are reserved but not implemented yet; use `|x| -> ...` to build an Act value purely, or use `do:Act`/`act { ... }` inside existing supported syntax.
Proc closures are reserved but not implemented yet; use `|x| -> ...` to build a Proc value purely, or use `do:Proc` inside existing supported syntax.
Workflow closures are reserved but not implemented yet; use `|x| -> ...` to build a Workflow value purely, or use `do:Workflow` inside existing supported syntax.
```

### 6.4 Old fat-arrow pure closure syntax

Older SPEC-031 text used:

```ash
|x, y| => x + y
```

as pure closure sugar. This spec supersedes that interpretation. In closure-literal position, `=>` is reserved for Proc closures.

Migration rule:

1. If the implementation currently accepts `|args| => body` as pure closure syntax, it SHOULD emit a deprecation diagnostic and suggest `|args| -> body` during a migration window.
2. After the migration window, `|args| => body` MUST NOT mean a pure closure. It is either a reserved Proc closure diagnostic or a real Proc closure if Proc closures are implemented.

### 6.5 Return type annotations

The preferred first-slice return annotation for closures is an outer binding annotation:

```ash
let add: (Int, Int) -> Int = |x, y| -> x + y;
```

Inline closure return-type syntax is deferred. A later spec may choose one of:

```ash
|x: Int, y: Int| -> Int { x + y }
|x: Int, y: Int| : Int -> x + y
```

but this spec does not require either form.

## 7. Internal representation guidance

### 7.1 Source-level carrier

Implementations SHOULD represent parsed source arrows with an explicit stratum discriminant:

```rust
pub enum CallableStratum {
    Pure,
    Act,
    Proc,
    Workflow,
}

pub struct CallableTypeSyntax<T> {
    pub stratum: CallableStratum,
    pub args: Vec<T>,
    pub ret: Box<T>,
}
```

The exact type names are not normative, but the discriminant is. Implementations MUST NOT encode the stratum solely by looking at the return type.

### 7.2 Typechecker representation

Current `ash-typeck` has:

```rust
Type::Fn(Vec<Type>, Box<Type>)
Type::Fun(Vec<Type>, Box<Type>, Effect)
```

This spec changes the intended source model:

- Pure callable syntax maps to `Type::Fn(args, ret)` for the first slice.
- Act/Proc/Workflow callable syntax is reserved until a stratum-aware callable representation exists or the existing representation is explicitly extended.
- `Type::Fun(args, ret, effect)` MUST NOT be used as a catch-all encoding for every non-pure tower callable merely by choosing an effect grade. Proc and Workflow carry environment requirements that are not reducible to the legacy `Effect` lattice.

A future implementation SHOULD move toward:

```rust
Type::Callable {
    stratum: CallableStratum,
    args: Vec<Type>,
    ret: Box<Type>,
}
```

or an equivalent representation. Until then, reserved diagnostics are preferable to overloading `Type::Fun`.

### 7.3 Return type does not classify the callable

These pairs are distinct:

```text
A -> Act<B>       // pure callable returning an Act value
A -*> B           // Act callable returning B
A -> Proc<B>      // pure callable returning a Proc value
A => B            // Proc callable returning B
A -> Workflow<B>  // pure callable returning a Workflow value
A =*> B           // Workflow callable returning B
```

The typechecker MUST classify application by callable stratum, not by the outer constructor of the return type.

## 8. Parsing and precedence requirements

### 8.1 Maximal-munch tokenization

The lexer/parser MUST recognize callable arrows before their prefixes:

1. `=*>` before `=>` or `=`.
2. `-*>` before `->` or `-`.
3. `=>` before `=`.
4. `->` before `-`.

### 8.2 Contextual use of `=>`

`=>` may continue to exist as a separator in other syntactic contexts, such as match arms, if those contexts are unambiguous. This spec reserves `=>` specifically as the Proc arrow in callable-type and closure-literal contexts. It does not require rewriting match-arm syntax.

### 8.3 Legacy `Fn(...) -> ...`

Legacy pure function type syntax remains a compatibility spelling:

```text
Fn(A, B) -> C
```

A first-slice implementation SHOULD continue to parse it as the same semantic pure callable type as:

```text
(A, B) -> C
```

but diagnostics and generated/reference output SHOULD prefer the new arrow-domain spelling. After an explicit migration window, legacy `Fn(...) -> ...` may be deprecated or removed by a later spec.

### 8.4 Existing unary `A -> B`

Existing parser paths that already treat `A -> B` as a unary function type remain compatible. This spec clarifies that `A -> B` is the unary form of the same callable arrow family, not a separate named-constructor shorthand.

## 9. Typechecking and application rules

### 9.1 Arity

Callable application is not partial application. A call must provide exactly the number of arguments in the callable domain.

Live-code caution: current `ash-typeck` has historically permitted partial application through `instantiate_fn_call`-style helper paths. The SPEC-072 implementation tasks must audit those paths and either remove that behavior for callable application or explicitly fence any pre-existing compatibility behavior behind a separate migration decision. As written, this spec requires exact arity for the new callable syntax.

```ash
let f: (Int, Int) -> Int = |x, y| -> x + y;
f(1, 2);   // valid
f(1);      // arity error, not partial application
```

### 9.2 Pure application

Applying a pure callable is permitted in Pure and higher strata, subject to ordinary name/capture/visibility rules. Pure application does not require Act/Proc/Workflow environment access.

### 9.3 Reserved higher-stratum application

Until higher-stratum callables are implemented:

- A source type containing `-*>`/`=>`/`=*>` MUST be rejected or reserved before successful typechecking.
- A closure literal using `-*>`/`=>`/`=*>` MUST be rejected or reserved before lowering to executable closure/runtime terms.
- No runtime fallback may reinterpret a reserved higher-stratum callable as a pure closure returning `Act`/`Proc`/`Workflow`.

### 9.4 Future higher-stratum application

When implemented, applying a higher-stratum callable must be legal only in contexts that admit that stratum or a higher one:

| Callable stratum | Minimum application context | Notes |
| --- | --- | --- |
| Pure | Pure | Also callable from Act/Proc/Workflow as ordinary pure computation. |
| Act | Act | May be embedded into Proc/Workflow through explicit existing tower rules. |
| Proc | Proc | Requires process identity/scheduling context. |
| Workflow | Workflow | Requires workflow admission/governance context. |

No implicit lift is introduced by this spec. Existing explicit lifts such as `proc::from_act` and workflow `from_act`/`from_proc` remain the bridge between returned computation values and higher contexts.

## 10. Diagnostics

Implementations should provide these diagnostics:

| Case | Diagnostic intent | Suggested fix |
| --- | --- | --- |
| `Fn(A, B) -> C` accepted | Legacy syntax note/warning if warnings are enabled | Use `(A, B) -> C`. |
| `(A, B) -> C` parsed | None | This is preferred syntax. |
| `((A, B)) -> C` unsupported | Explain unary tuple-argument spelling is not implemented | Use a type alias for the tuple argument. |
| `|x| => x + 1` in pure context | `=>` is reserved for Proc closures, not pure closures | Use `|x| -> x + 1`. |
| `(A) -*> B` | Act callable syntax is reserved | Use `A -> Act<B>` for a pure smart constructor, or wait for Act callables. |
| `(A) => B` | Proc callable syntax is reserved | Use `A -> Proc<B>` for a pure smart constructor, or wait for Proc callables. |
| `(A) =*> B` | Workflow callable syntax is reserved | Use `A -> Workflow<B>` for a pure smart constructor, or wait for Workflow callables. |
| Applying wrong arity | Not partial application | Provide all arguments or construct an explicit unary closure. |

## 11. Acceptance criteria

### C72-1: Preferred pure callable type syntax

The parser and typechecker accept preferred pure callable type syntax such as:

```ash
let f: (Int, Int) -> Int = |x, y| -> x + y;
```

or the equivalent context currently supported by Ash for function-value annotations.

### C72-2: Legacy compatibility

Legacy `Fn(Int, Int) -> Int` remains accepted during the compatibility window and normalizes to the same pure callable type as `(Int, Int) -> Int`, with preferred rendering using `(Int, Int) -> Int`.

### C72-3: Tuple ambiguity is resolved explicitly

Tests prove `(Int, Int) -> Bool` is treated as a two-argument callable, not a unary tuple-argument callable. A unary tuple-argument callable has an explicit accepted spelling or a targeted diagnostic.

### C72-4: Pure closure arrow

Pure closures use `|args| -> body`. Old `|args| => body` is not silently accepted as pure closure syntax after this spec's parser slice.

### C72-5: Reserved higher-stratum callable type arrows

`-*>`/`=>`/`=*>` in callable-type position are reserved and rejected with targeted diagnostics unless their semantics are implemented by the same task.

### C72-6: Reserved higher-stratum closure arrows

`|args| -*>`, `|args| =>`, and `|args| =*>` are reserved and rejected with targeted diagnostics unless their semantics are implemented by the same task.

### C72-7: Smart constructor distinction

Tests or documentation examples prove that `A -> Workflow<B>` remains a pure callable returning a workflow value and is distinct from reserved `A =*> B`.

### C72-8: Reference and stale-spec reconciliation

SPEC-027, SPEC-031, the functions reference chapter, agent cards, and generated/diagnostic rendering are updated or explicitly marked as amended so readers do not copy stale `Fn(...) -> ...` or pure `|x| => ...` examples.

## 12. Implementation notes by crate

### ash-parser

Known live seams to audit before implementation:

- `crates/ash-parser/src/surface.rs`: `Type::Tuple(Vec<Type>)` and `Type::Fn(Vec<Type>, Box<Type>)`.
- `crates/ash-parser/src/parse_module.rs`: `parse_surface_type_with_holes`, `parse_surface_type_atom_with_holes`, the legacy explicit `Fn(...) -> ...` branch, the current unary `lhs -> rhs` branch, and `convert_type_expr`.
- `crates/ash-parser/src/parse_type_def.rs`: `parse_type_expr`, `parse_fn_type`, `parse_tuple_type`, and current `lhs -> rhs` lowering through synthetic `Constructor { name: "Fn", ... }`.

Requirements:

- Add parser support for n-ary parenthesized callable domains before callable arrows.
- Preserve source spans for the full callable type and arrow token.
- Add/reserve arrow token parsing for `-*>`, `=>`, and `=*>` in callable-type and closure-literal contexts.
- Decide and test unary tuple-argument spelling.
- Do not parse `(A, B) -> C` by first producing a tuple type and then wrapping it as a unary `Fn` argument.
- Audit whether `parse_type_def::TypeExpr` needs a dedicated function/callable carrier instead of overloading synthetic `Constructor { name: "Fn", args }` before conversion to `surface::Type`.
- Update closure parsing so `|args| -> body` is pure closure syntax and `|args| => body` no longer means pure closure.
- Locate and name the live closure/fn-expression parser and lowering/runtime carrier before changing closure syntax. If closure syntax is not currently implemented in the live parser, TASK-959 must fail closed rather than inventing runtime semantics.

### ash-core

- If a source-preserving carrier is needed across crate boundaries, define a shared `CallableStratum`/callable syntax carrier here rather than parser-private semantic structs.
- Do not expose runtime environment carriers for Act/Proc/Workflow callables in this syntax slice.

### ash-typeck

Known live seams to audit before implementation:

- `crates/ash-typeck/src/types.rs`: `Type::Fn`, `Type::Fun`, `Display for Type`, and call-instantiation helpers such as `instantiate_fn_call`.
- `crates/ash-typeck/src/check_expr.rs`: callable application checking and exact-arity diagnostics.
- `crates/ash-typeck/src/lib.rs`: `workflow_surface_type_to_type`, `fn_signature_type`, `builtin_fn_signature_type`, and function signature registration/refinement paths.

Requirements:

- Map pure callable syntax to the existing pure callable type representation for the first slice.
- Keep higher-stratum callable arrows fail-closed/reserved until a stratum-aware callable type representation is implemented.
- Update type rendering and diagnostics to prefer `(A, B) -> C` for pure callables.
- Add focused exact-arity tests: `f(1, 2)` succeeds for `f : (Int, Int) -> Int`, while `f(1)` and `f(1, 2, 3)` fail.
- Add rendering tests for unary, n-ary, nested-return callable types, and the chosen unary tuple-argument spelling.

### ash-engine

- Audit module export/import summaries that carry function signatures. Ensure preferred rendering and compatibility syntax do not lose argument lists or return types across module boundaries.
- No runtime execution changes are required for reserved higher-stratum callables.

### ash-interp / runtime crates

- Preserve existing pure closure runtime behavior where it already exists.
- Do not implement Act/Proc/Workflow closure runtime behavior in the first syntax slice unless a later task explicitly expands scope.

### reference/docs

- Update daily-use function reference pages and agent cards to prefer `(A, B) -> C` and `|args| -> body`.
- Mark old specs as amended by SPEC-072 where full rewriting is out of scope.

## 13. Changelog

### 2026-05-26

- Initial draft. Defines tower callable arrows `->`, `-*>`, `=>`, `=*>`; switches pure closure shorthand to `|args| -> body`; reserves Act/Proc/Workflow callable and closure syntax; and separates callable application stratum from returned computation value type.
- Implemented MVP closeout via TASK-957 through TASK-963, with TASK-962 as the final closeout gate. The first slice accepts preferred pure callable syntax, preserves legacy `Fn(...) -> ...` compatibility, enforces exact callable arity, implements pure closure `|args| -> body`, reserves higher-stratum arrows with fail-closed diagnostics, migrates std/reference daily-use surfaces, and records C72-1 through C72-8 evidence in `docs/plan/audits/TASK-962-tower-callable-syntax-acceptance-matrix.md`.
