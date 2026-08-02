---
id: language.reference.forms.control-flow-and-patterns
title: Control Flow and Patterns
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/parse_expr.rs", "crates/ash-typeck/src/**", "crates/ash-cli/tests/**"]
---

# Control Flow and Patterns

[Forms index](index.md) · [Values, bindings, blocks, and calls](values-bindings-blocks-and-calls.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `if` expressions | accepted | checked | lowered | fixture-bounded | partial | tested | below_spec |
| `if let` expressions | accepted | partial | lowered | closed | partial | tested | below_spec |
| `match` expressions and patterns | accepted | partial | lowered | closed | partial | tested | below_spec |
| `check obligation_name` | accepted | rejected-after-parse | lowered | closed | partial | tested | below_spec |
| `panic` expression | accepted | checked | rejected | closed | partial | tested | below_spec |

The parser routes are `crates/ash-parser/src/{parse_expr.rs,parse_pattern.rs}` and the dedicated
function-body parser in `parse_module/fn_defs.rs`. Static evidence is
`crates/ash-typeck/src/check_expr/mod.rs`, `check_pattern.rs`, and `exhaustiveness.rs`; source
lowering is `crates/ash-parser/src/lower.rs`.

Focused tests include:

- `crates/ash-parser/tests/task_1007_if_let_parser_entrypoints.rs`
- `crates/ash-typeck/tests/task_916_pattern_canonicalization_diagnostics.rs`
- `crates/ash-cli/tests/task_1008_matching_diagnostics_surface.rs`
- `crates/ash-engine/tests/task_2003_pure_anf_normalizer.rs`
- `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`

The no-admitted-route cells are deliberately narrower than "unimplemented": `if let` and `match`
have parser, checker, and lowering paths, but this task found no general Engine execution evidence.

## Parser-context boundary

The ordinary `if`, `match`, and `panic` spellings documented here are dispatched by
`parse_module/fn_defs.rs::parse_fn_expr`, the expression parser used for a named function body.
They are therefore not evidenced as arbitrary general-expression entrypoints. By contrast,
`parse_expr.rs::expr_with_mode` explicitly dispatches `if let` before its ordinary expression
grammar, so `if let` has a general-expression parser route as well. All four forms become surface
expression carriers, but that AST fact does not widen their source parser contexts.

## `if`

Within a named function body, `if` is a value-producing expression. Its condition must typecheck
as `Bool`; with an `else`, the two branches must unify. Without an `else`, the then branch must
have type `Null`. Surface lowering represents an `if` as a Boolean match with `true` and `false`
arms.

**Fixture-bounded Engine example.** This exact pure shape is executed by the selected
pure-ANF route in `task_2003_pure_anf_normalizer.rs`. It is evidence for the displayed Boolean
and integer expression shape only, not arbitrary branch effects, calls, records, or patterns.

```ash
fn main() -> Int {
    if !(1 + 2 < 4) then 7 else 8
}
```

## `if let`

`if let pattern = expression then branch else branch` destructures a value in the then branch.
The `else` branch is mandatory. The checker canonicalizes the scrutinee type where possible,
rejects impossible patterns, binds the resulting names only in the then branch, and unifies branch
types. If a pattern is irrefutable, it reports that the else branch is unreachable as a non-fatal
diagnostic.

**Parser/static/lowering shape.** This complete function is the form exercised by the
parser entrypoint test. It is not a general runtime example.

```ash
fn unwrap_or_zero(value: Int) -> Int {
    if let x = value then { x } else { 0 }
}
```

Although the example uses an irrefutable variable pattern, an `if let` without `else` is rejected
as incomplete source. Structured patterns require the matching type/constructor information in
the checker; parser acceptance does not establish that information.

## `match` and patterns

Within a named function body, `match scrutinee { pattern => expression, ... }` selects an arm by
pattern. The parser supports variable, wildcard, tuple, list, record, variant, and literal
patterns. Uppercase-leading names are parsed as unit variants when bare, and as variant patterns
with record or tuple payloads; lowercase bare names are variable patterns.

The checker validates arm bindings against the scrutinee type and invokes canonicalized
exhaustiveness diagnostics where the type is matchable. Those diagnostics are intentionally
limited: an unresolved associated projection can block canonicalization, and a constructor that
does not match a primitive scrutinee must report its own error without fabricating a missing-arm
witness. A `_` arm is the portable way to cover an otherwise unsupported or blocked universe.

**Source/static shape; no runtime claim.** The variants and constructor definitions must be
checked in the surrounding module. This illustrates current arm spelling, not a complete Engine
execution route.

```ash
fn choose(answer: Answer) -> Int {
    match answer {
        Yes => 1,
        No => 0,
    }
}
```

Useful pattern fragments include `x`, `_`, `(left, right)`, `[head, ..tail]`, `{ name: value }`,
`Some { value: x }`, `Pair(left, right)`, and `42`. These fragments are only valid at a pattern
site; they are not interchangeable with expression constructors.

## `panic`

Within a named function body, `panic "message"` creates the `Expr::Panic` source carrier. The
current parser accepts a string-literal message only: despite the carrier being an expression, its
message is not an arbitrary expression. The ordinary expression checker gives the carrier a fresh
result type, and the purity check permits it in a pure function. Generic source lowering then
rejects `panic`, so it has no admitted execution route.

**Static-only complete named-function example.** The parser test
`fn_parser_tests::control_flow_and_blocks::parse_fn_panic` exercises this named-function-body
route. The static evidence is `check_expr/mod.rs`'s `Expr::Panic` branch and the
`purity::panic_in_pure_fn_is_ok` typechecker unit test. No Engine behavior is claimed.

```ash
fn require_value() -> Int {
    panic "value is required"
}
```

## The `check` carrier

`check obligation_name` is currently a source expression spelling. The parser builds
`Expr::CheckObligation`, and source lowering carries it to Core `CheckObligation`. The ordinary
expression typechecker then rejects it with `UnsupportedExpression`.

```ash
check audit_trail
```

This is a parser/lowering fragment, deliberately not a copyable checked program or a semantic
obligation-discharge example. Historical workflow-obligation material and the Rust obligation
helpers do not override the live source checker. Since the static route rejects this carrier,
there is no source typing or runtime sequent for `check`.

## Syntax

The grammar is limited to accepted source shapes. `if`, `match`, and `panic` below are
named-function-body forms; `if let` also has the general-expression parser dispatch identified
above. `expression`, `identifier`, and `literal` are shared parser domains; a valid pattern still
needs checker validation against its enclosing type.

```ebnf
if_expression = "if" expression "then" branch [ "else" branch ] ;
if_let_expression = "if" "let" pattern "=" expression "then" branch "else" branch ;
branch = block | expression ;
match_expression = "match" expression "{" [ match_arm { [ "," ] match_arm } [ "," ] ] "}" ;
match_arm = pattern "=>" expression ;
check_expression = "check" identifier ;
panic_expression = "panic" string_literal ;
pattern = variable_pattern | wildcard_pattern | tuple_pattern | list_pattern | record_pattern | variant_pattern | literal_pattern ;
variable_pattern = identifier ;
wildcard_pattern = "_" ;
tuple_pattern = "(" pattern { "," pattern } ")" ;
list_pattern = "[" [ pattern { "," pattern } ] [ "," ".." identifier ] "]" ;
record_pattern = "{" [ record_field { "," record_field } ] [ "," ".." ] "}" ;
record_field = identifier [ ":" pattern ] ;
variant_pattern = variant_name | variant_name "{" [ variant_field { "," variant_field } ] [ "," ".." ] "}" | variant_name "(" [ pattern { "," pattern } ] ")" ;
variant_field = identifier ":" pattern ;
literal_pattern = literal ;
```

The grammar shows source shape, not the type-directed distinction between variable names and
variants. In particular, a constructor name, its fields, and an exhaustiveness witness are not
validated by EBNF. The match parser permits no arms and permits commas between or after arms, but
an empty arm list may subsequently fail static exhaustiveness checking.

## What the checker does

The following sequents summarize the checker branches. They use `DELTA` for bindings introduced
by a checked pattern. Exact implementation side conditions remain important: `if` checks Bool,
branches must unify, and pattern canonicalization can report a blocked or impossible case instead
of producing `DELTA`.

```sequent
IfType :=
  [ GAMMA |- condition : Bool ] [ GAMMA |- then_branch : T ] [ GAMMA |- else_branch : T ]
  ===>
  GAMMA |- if condition then then_branch else else_branch : T
```

```sequent
IfLetType :=
  [ GAMMA |- source : S ] [ pattern : S |- bindings DELTA ] [ GAMMA, DELTA |- then_branch : T ] [ GAMMA |- else_branch : T ]
  ===>
  GAMMA |- if let pattern = source then then_branch else else_branch : T
```

```sequent
MatchType :=
  [ GAMMA |- source : S ] [ pattern_i : S |- bindings DELTA_i ] [ GAMMA, DELTA_i |- arm_i : T ]
  ===>
  GAMMA |- match source { pattern_i => arm_i } : T
```

These are static rules only. `lower_expr` carries `if let` and `match` to Core constructs and
lowers patterns recursively, but that is not evidence of generic Engine execution. There is no
sequent for `check` because its source static route is rejected, and none for `panic` because its
generic lowering path rejects it.

## Errors and limits

- `if` diagnoses a non-Boolean condition or branch-type mismatch. An `if` without `else` requires
  a `Null` then branch.
- `if let` requires `else`. Impossible or blocked patterns are rejected; an irrefutable pattern
  emits an unreachable-else diagnostic.
- Match diagnostics identify bad constructors and canonicalization limits. Do not infer complete
  exhaustiveness coverage from a parser-valid arm list.
- `panic` is a named-function-body form with a string-literal message. It may typecheck with a
  fresh result type, but `lower_expr` rejects generic panic lowering. It is not an Engine exception
  feature claim.
- `check obligation_name` parses and lowers as a carrier but is rejected after parsing by the
  ordinary expression checker; no admission/runtime route follows.
- Patterns, rows, contracts, and diagnostics never grant authority or install runtime frames.
- Workflow/tower control forms and source `raise` are excluded.

## Related evidence

- [AUDIT-206 LANG-006, LANG-007, and LANG-023](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2047](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- `cargo test -p ash-parser --test task_1007_if_let_parser_entrypoints`
- `cargo test -p ash-parser --test fn_parser_tests control_flow_and_blocks::parse_fn_panic`
- `cargo test -p ash-typeck --test task_916_pattern_canonicalization_diagnostics`
- `cargo test -p ash-typeck panic_in_pure_fn_is_ok`
- `cargo test -p ash-engine --test task_2003_pure_anf_normalizer`
