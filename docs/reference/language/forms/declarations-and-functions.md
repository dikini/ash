---
id: language.reference.forms.declarations-and-functions
title: Declarations and Functions
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/parse_module/**", "crates/ash-typeck/src/**", "crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs"]
---

# Declarations and Functions

[Forms index](index.md) · [Language reference](../index.md) ·
[Source of truth](../source-of-truth.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Ordinary `fn` declaration | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| `builtin fn` declaration | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| `requires:` and `ensures:` function contracts | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| `handler` declaration spelling | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| `law` and `proof` authoring | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |

Primary parser evidence is `crates/ash-parser/src/parse_module.rs::module_file` and
`crates/ash-parser/src/parse_module/fn_defs.rs::{parse_fn_definition,parse_builtin_fn_definition,
parse_handler_declaration}`. Static and expression-lowering evidence is
`crates/ash-typeck/src/{lib.rs,check_expr/mod.rs}` and `crates/ash-parser/src/lower.rs`.
The bounded entry evidence is `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`.

Focused tests include:

- `crates/ash-parser/tests/fn_parser_tests/contracts_and_types.rs`
- `crates/ash-typeck/tests/pure_function_contracts_task_505.rs`
- `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`
- `crates/ash-engine/tests/builtin_fn_e2e_import.rs`
- `crates/ash-parser/tests/task_1361_law_keyword_module_scope.rs`
- `crates/ash-parser/tests/task_1363_proof_keyword_module_scope.rs`
- `crates/ash-typeck/tests/task_1364_law_name_checking.rs`
- `crates/ash-typeck/tests/task_1365_proof_name_checking.rs`

The builtin Engine test is negative admission evidence: imported builtin source can parse and
typecheck yet is rejected before direct dispatch when validated Core/CPS lowering is absent.
Law/proof registration evidence is static: `type_check_program` invokes
`register_module_laws` and `register_module_proofs_with_fuel`; it is not a module-lowering or
application-runtime route.

## What declarations are current

At module level the current parser recognizes `fn`, `builtin fn`, `handler`, `law`, and `proof`.
The broader active-declaration inventory also includes type, effect, interface, resource, role,
and type-level families; those families have their own TASK-2048 through TASK-2051 pages. An AST
carrier alone is not a declaration: in particular, retained top-level capability and policy
carriers have no active `module_file` branch and are excluded from this manual.

Use `fn` for a named function with a body. Parameters have source type annotations; the return
annotation is optional for ordinary functions. A `builtin fn` is a declaration for a host-provided
callable: it has no Ash body, requires a return type, and ends in `;`. It can be imported and
typechecked from its declared signature, but an imported source call is rejected at the checked
Core/CPS admission boundary until validated lowering exists. A declaration neither installs a host
implementation nor admits a call.

`handler` has a function-shaped declaration and retains an explicit handler marker. It is listed
here only to establish its current declaration spelling. This page makes no handler execution
claim: `handle … with …`, handler-body forms, and the sealed fixture-bounded handler cases belong
to TASK-2051. Do not infer them from an ordinary function contract or from this page's examples.

`law` declares a pure proposition and `proof` provides a named proof/evidence body. The checker
registers laws and module/implementation proofs during program checking, including proof-totality
and matching-law checks. This is authoring and static evidence machinery, not a module-lowering,
ordinary application result, or Engine execution route.

## Functions and bounded entry execution

An ordinary function declaration introduces a named callable and a block body. The current
checker first records callable signatures, refines them, and checks bodies. Surface lowering can
represent selected functions and calls, but the Engine's admitted route is deliberately narrower
than parser or checker acceptance.

The only function execution claim on this page is the exact Engine fixture below. It is accepted,
checked, lowered/admitted through the selected route, and returns `42`. It does **not** establish
that arbitrary helpers, closures, contracts, records, matches, or generic applications execute.

```ash
fn main() -> Int {
    do {
        return 42;
    }
}
```

`task_1865_surface_fn_main_entry.rs` executes that exact shape. A richer source containing helper
calls, records, ADTs, `match`, and `do` is deliberately rejected at the checked Core/CPS admission
boundary in the same test file. See TASK-2052 for the full entry and terminal contract.

## Contracts on functions and handlers

`requires:` clauses are parsed before `ensures:` clauses and before the function body. A clause
contains one or more comma-separated expressions; multiple clauses normalize into one contract
carrier. The checker records/refines current function-contract facts and retains sidecar evidence
in selected routes. This page has no admitted source execution route for contract clauses and does
not establish general postcondition enforcement for function calls.

This static/lowering example is exercised by the parser contract tests and
`pure_function_contracts_task_505.rs`. It is not offered as a general runtime contract example.

```ash
fn nonnegative(value: Int) -> Int
    requires: value >= 0
    ensures: result >= 0
{
    value
}
```

The parser accepts the same clause position on `handler` declarations, but there is no tested
general handler-contract semantics. Do not attach a contract to a handler expecting this page to
establish its behavior.

## Builtins, laws, and proofs

**Parser-only builtin declaration fragment.** A builtin is bodyless and needs a return type. This
is a spelling example, not proof that `current_tick` is registered or runnable.

```ash
builtin fn current_tick() -> Int;
```

**Checked law/proof pair.** TASK-1365 verifies this matching module-scope law and proof pair
through the program checker. It is not an application program and has no module-lowering or Engine
execution claim.

```ash
law reflexive(x: Int): x == x

proof reflexive(x: Int) {
    by_definition
}
```

Proof bodies currently include `by_definition`, named authored tests, property/quickcheck tests
with optional strategies, small-world tests, and an expression-body carrier. Their detailed
evidence-runner behavior is owned by the library/documentation task; a proof body is never a
general-purpose expression evaluator claim here.

## Syntax

The grammar below records the accepted declaration shape. Its visibility grammar is the same as
the [module visibility grammar](../lexical-and-modules/modules-imports-and-visibility.md#syntax).
`type`, `expression`, `constraint`, and the handler body belong to their owning chapters.

```ebnf
function_declaration = [ visibility ] "fn" callable_name [ type_parameters ] "(" [ parameter { "," parameter } ] ")" [ "->" type ] [ proposition_tail ] { requires_clause } { ensures_clause } function_body ;
builtin_function_declaration = [ visibility ] "builtin" "fn" callable_name [ type_parameters ] "(" [ parameter { "," parameter } ] ")" "->" type [ proposition_tail ] ";" ;
handler_declaration = [ visibility ] "handler" callable_name [ type_parameters ] "(" [ parameter { "," parameter } ] ")" "->" type [ proposition_tail ] { requires_clause } { ensures_clause } function_body ;
requires_clause = "requires" ":" expression { "," expression } ;
ensures_clause = "ensures" ":" expression { "," expression } ;
law_declaration = "law" identifier "(" [ parameter { "," parameter } ] ")" [ "where" constraint { "," constraint } ] ":" expression ;
proof_declaration = "proof" identifier "(" [ parameter { "," parameter } ] ")" [ "where" constraint { "," constraint } ] "{" proof_body "}" ;
proof_body = "by_definition" | "by" "test" proof_test_mode | expression ;
proof_test_mode = string_literal | "authored" string_literal | "property" [ "with" "{" strategy_binding { "," strategy_binding } "}" ] | "quickcheck" [ "with" "{" strategy_binding { "," strategy_binding } "}" ] | "small_world" ;
strategy_binding = identifier "<-" expression ;
function_body = "{" block_contents "}" ;
parameter = identifier ":" type ;
visibility = "pub" | "pub" "(" visibility_scope ")" ;
visibility_scope = "crate" | "super" | "self" | "in" visibility_path ;
visibility_path = path_segment { "::" path_segment } ;
```

## What the checker does

The source checker has a concrete rule-shaped path for ordinary function expressions and their
typed bodies; named declarations add registration/refinement around that expression rule. The
following schematic is limited to that checked expression behavior. `annotation(x)` is optional;
when present it must resolve and agree with the inferred body result. The actual implementation
reports diagnostics rather than asserting a total formal calculus.

```sequent
FnDefType :=
  [ GAMMA, x1 : T1, ..., xn : Tn |- body : R ]
  ===>
  GAMMA |- fn(x1 : T1, ..., xn : Tn) -> R { body } : Fn(T1, ..., Tn) -> R
```

This rule corresponds to `check_expr`'s `Expr::FnDef` branch and does not imply that a function
value is admitted for Engine execution. Contracts, laws, proofs, handler markers, and module
signature registration carry extra data outside this expression rule. There is no evidence-backed
sequent here for universal contract discharge, handler contracts, proof execution, or builtin
host dispatch, so none is stated.

## Errors and limits

- `builtin fn` requires a return type and a semicolon; a body is rejected by the parser.
- Contracts use current `requires:`/`ensures:` clauses, not removed workflow headers. A raw
  operator section in a contract can parse as surface syntax but fails at the expanded-surface
  boundary.
- `fn main` is not a blanket runtime escape hatch. The selected pure/admitted fragment is small;
  use Engine admission evidence before claiming execution.
- Host builtin availability is catalog- and admission-dependent. Imported source calls remain
  rejected until their validated Core/CPS lowering exists; a declaration neither installs a
  provider nor grants authority.
- Laws/proofs are checked evidence declarations. They are outside module lowering and do not
  generate a callable value or establish a theorem prover/runtime route from their surface spelling.
- Workflow/tower declarations, their headers, and removed callable-arrow spellings are excluded.

## Related evidence

- [AUDIT-206 LANG-004, LANG-005, LANG-015, and LANG-019](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2047](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- `cargo test -p ash-parser --test fn_parser_tests`
- `cargo test -p ash-typeck --test pure_function_contracts_task_505`
- `cargo test -p ash-typeck --test task_1365_proof_name_checking`
- `cargo test -p ash-engine --test task_1865_surface_fn_main_entry`
- `cargo test -p ash-engine --test builtin_fn_e2e_import builtin_fn_runtime_rejects_without_validated_core_cps_lowering`
