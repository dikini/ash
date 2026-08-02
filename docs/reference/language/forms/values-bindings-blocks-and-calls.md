---
id: language.reference.forms.values-bindings-blocks-and-calls
title: Values, Bindings, Blocks, and Calls
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/parse_expr.rs", "crates/ash-typeck/src/check_expr/**", "crates/ash-parser/tests/**"]
---

# Values, Bindings, Blocks, and Calls

[Forms index](index.md) · [Declarations and functions](declarations-and-functions.md) ·
[Control flow and patterns](control-flow-and-patterns.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Literals, variables, and constructor/record/list expressions | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Anonymous `fn` and pure closure expressions | accepted | checked | lowered | closed | partial | tested | below_spec |
| Named local functions | accepted | partial | lowered | closed | partial | tested | below_spec |
| Blocks, `let`, and expression statements | accepted | partial | lowered | fixture-bounded | partial | tested | below_spec |
| Direct calls and function application | accepted | partial | lowered | closed | partial | tested | below_spec |

Primary evidence is `crates/ash-parser/src/parse_expr.rs`,
`crates/ash-parser/src/parse_module/fn_defs.rs`, `crates/ash-parser/src/lower.rs`, and
`crates/ash-typeck/src/check_expr/mod.rs`. Focused evidence includes:

- `crates/ash-parser/tests/task_959_pure_closure_arrow.rs`
- `crates/ash-typeck/tests/task_959_pure_closure_arrow.rs`
- `crates/ash-parser/tests/fn_parser_tests.rs::closures::task556_named_fn_in_block_desugars_to_let`
- `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`
- `crates/ash-parser/tests/fn_parser_tests/contracts_and_types.rs`

The cells that show no admitted route mean this task did not find general Engine execution for
closures, rich application, or local functions. They do not say the parser or checker lacks those
forms.

## Values and expression position

Expressions include literals, variables, constructors, records, lists, field access, blocks,
function values, calls, applications, and control forms. This page focuses on the binding and
function-value portion of that set. Data constructors and their type declarations are documented
with ordinary types; handler/failure expressions and generic `do` remain outside this page's
runtime claims.

In source, an uppercase identifier followed by arguments may be parsed as a constructor while an
ordinary identifier followed by arguments is a named `Call`. If the callee itself is an expression,
the parser produces `FnApply`. The checker validates their different paths: a named call looks up
a callable target, while `FnApply` requires a function type and matching arity.

## Function values

An anonymous function starts with `fn(`, not `fn name(`. Its body is a block. The pure closure
shorthand uses bars with the mandatory `->` arrow and immediately becomes the same surface
`FnDef` carrier. The old `=>` closure arrow is rejected; `=>` remains meaningful for match arms.

**Checked expression fragment.** The parser and typechecker tests parse and check this expression
as `Fn(Int) -> Int`. It is an expression fragment, not a complete module file and not an Engine
execution claim.

```ash
|x: Int| -> x + 1
```

The equivalent explicit anonymous-function shape is:

```ash
fn(x: Int) -> Int { x + 1 }
```

The typed `FnDef` branch checks parameter annotations in a fresh environment, infers the body,
and unifies a written return annotation with that body type. A closure remains a pure `Fn` type
even in an ambient operational profile; that boundary is covered by the TASK-959 typechecker
test. It does not establish that the resulting closure crosses the Engine admission boundary.

## `let`, blocks, and named local functions

A block has zero or more statements and an optional tail expression. A `let` binds an irrefutable
pattern for the remainder of its block; the checker rejects impossible or refutable patterns at
that binding boundary. An expression followed by `;` is a statement whose result is discarded;
without a trailing `;`, the last expression is the block result.

The parser also accepts a named local function in a function block. It does not create a separate
local-declaration AST node: `fn add_one(x: Int) -> Int { x + 1 }` is desugared by the parser into
`let add_one = fn(x: Int) -> Int { x + 1 }`. The lowerer consequently has an ordinary nested
`Let`/`FnDef` route. This fresh source inspection replaces the stale legacy claim about a separate
runtime local-function feature. There is no current general Engine execution evidence for it.

**Local-function source shape; static/lowering only.** This is a complete module
example of the parser desugaring. It must not be read as a promise that Engine execution of local
functions is admitted.

```ash
fn use_local(n: Int) -> Int {
    fn add_one(x: Int) -> Int { x + 1 }
    add_one(n)
}
```

**Scoped `let` source shape.** This grammar and lowering route are shared by function and
anonymous-function blocks. The exact bounded `fn main` execution claim is in the declarations
page; this example is not a standalone runtime fixture.

```ash
fn add_after_binding(n: Int) -> Int {
    let next = n + 1;
    next + 1
}
```

## Calls and applications

`name(arguments)` is a direct named call. `expression(arguments)` is a function application, for
example an immediately applied anonymous function. The parser lowerer retains direct calls as
either built-in/module calls or a `FnApply` according to the target; arbitrary application has a
separate `FnApply` Core carrier.

**Checked expression fragment.** The static route requires `f` to have a function type and the
argument count/types to match. It is intentionally not a runnable top-level fragment.

```ash
(|x: Int| -> x + 1)(41)
```

No page in the manual should infer general closure application execution from this typechecking
route. `task_1865_surface_fn_main_entry.rs` demonstrates that source may parse/check/lower yet
still be rejected by typed Core/CPS admission.

## Syntax

This is the binding-and-call slice of the expression grammar. `expression`, `pattern`, `type`,
and `identifier` are shared parser domains; the grammar does not claim all their subforms here.

```ebnf
anonymous_function = "fn" "(" [ function_parameter { "," function_parameter } ] ")" [ "->" type ] block ;
pure_closure = "|" [ function_parameter { "," function_parameter } ] "|" "->" expression ;
function_parameter = identifier [ ":" type ] ;
block = "{" { block_statement } [ expression ] "}" ;
block_statement = let_statement | local_function_declaration | expression ";" ;
let_statement = "let" pattern "=" expression [ ";" ] ;
local_function_declaration = "fn" identifier "(" [ function_parameter { "," function_parameter } ] ")" [ "->" type ] block [ ";" ] ;
direct_call = identifier "(" [ expression { "," expression } ] ")" ;
function_application = callee_expression "(" [ expression { "," expression } ] ")" ;
```

`function_application` is the general expression-callee shape. Whether a particular parenthesized
spelling is parsed as a direct call, constructor, or application is resolved by the parser's
callee form and the callee's spelling.

## What the checker does

The next rules summarize concrete checker branches. They are typechecking rules, not a promise of
Core/CPS admission. In the block rule, `pat` must pass the checker's irrefutable-pattern boundary;
the environment extension contains exactly the pattern bindings. In the application rule, `n`
must equal the callable arity and the checker unifies each actual argument with its parameter.

```sequent
BlockLet :=
  [ GAMMA |- value : T ] [ GAMMA, pat : T |- body : R ]
  ===>
  GAMMA |- { let pat = value; body } : R
```

```sequent
FnApplyType :=
  [ GAMMA |- callee : Fn(T1, ..., Tn) -> R ] [ GAMMA |- arg1 : T1 ] [ GAMMA |- argn : Tn ]
  ===>
  GAMMA |- callee(arg1, ..., argn) : R
```

The lowerer maps blocks to nested Core `Let` expressions, maps surface `FnDef` to Core `FnDef`,
and maps surface `FnApply` to Core `FnApply`. The Engine's selected pure-ANF route accepts only a
narrow typed subset, so these lowering facts do not establish runtime parity.

## Errors and limits

- `fn name(...)` in a block is local-function syntax; `fn(...)` is anonymous-function syntax.
  They have different parser entries but the former desugars to a `let` binding.
- A closure must use `->`; legacy `=>` does not silently become a closure.
- A `let` pattern is checked for irrefutability. A structurally accepted pattern can still be
  rejected after parsing if it is impossible, refutable, or cannot be canonicalized.
- `FnApply` diagnoses non-function callees, wrong arity, and type mismatches. A named call can
  instead diagnose an unknown function or unsupported target.
- No binding, closure, function value, signature, or call creates runtime authority, provider
  frames, role admission, or a fallback evaluator.
- Workflow/tower callable syntax and their arrows are excluded.

## Related evidence

- [AUDIT-206 LANG-005 and LANG-006](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2047](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- `cargo test -p ash-parser --test task_959_pure_closure_arrow`
- `cargo test -p ash-parser --test fn_parser_tests closures::task556_named_fn_in_block_desugars_to_let`
- `cargo test -p ash-typeck --test task_959_pure_closure_arrow`
- `cargo test -p ash-engine --test task_1865_surface_fn_main_entry`
