---
id: spec.ash.surface-grammar.target
title: Ash Surface Syntax Grammar — Target State
description: Target surface syntax for the Ash language with effect rows, unified do-notation, and structured effect items
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-07-03
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-096-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
---

# SPEC-095b: Ash Surface Syntax Grammar — Target State

**Status:** Draft — target surface syntax for the unified effect-row language
**Scope:** This document defines the grammar we want the parser to accept once the
unified effect system is implemented. It is a goal-state living document.
**Depends on:** SPEC-096 (Target Effect System), SPEC-097 (Target Type System)

## 1. Summary

The target grammar unifies Ash's computation-requirement surface into one coherent syntax:

- computation rows on function types and computation blocks;
- a single `do { ... }` form as direct-style sequencing sugar;
- structured row items for operations, resources, roles, policies, contracts, channels, process operations, failure, and evidence;
- transparent effect aliases and diagnostic groups;
- row variables for polymorphism.

This grammar replaces the separate `do:Act`, `do:Proc`, `do:Workflow`, `workflow`, `act`,
`ret`, and legacy workflow-statement syntax with a unified surface. During migration, legacy
forms remain accepted as compatibility aliases.

**Current implementation note (Phase 185/186/187/188/189/190/191/192).** Ordinary engine parsing now accepts a target entry
source with top-level `fn main(...) -> T { ... }` and no `workflow` block, and the CLI dry-run path
uses the same ordinary file-backed parse/check path for that source shape. The engine adapts that
entry to its existing runtime carrier internally; the source program remains a function-first module,
and `workflow` remains compatibility/runtime-profile syntax rather than the target core source path.
Target `do { ... }` accepts both `return expr` and the documented statement form `return expr;`.
The runtime also accepts field projection on named constructor payload values for the ordinary
record/ADT fixture shape accepted by the surface typechecker. Phase 187 adds bare structural record
expressions such as `{ name: "Ada", age: 41 }` in ordinary expression position, including `do` binding
and field projection. Phase 188 closes the next pattern-matching expression gap by allowing ADT
record-constructor expressions such as `Some { value: 41 }` directly as `match` scrutinees in
function bodies. Phase 189 extends that same match-scrutinee path to ordinary call, field-projection,
and binary expressions. Phase 190 adds the target `do` expression-statement form `expr;` to the
same direct-style sequencing path. Phase 191 extends ordinary expression parsing to nested block
expressions and block expression statements, using the same direct-style discard sequencing.
Phase 192 extends field projection to ordinary primary expressions such as record literals and
parenthesized constructor expressions. Phase 193 makes tuple-payload ADTs such as
`RuntimeError(2, "missing config")` usable in function-first match scrutinees and patterns on the
same ordinary expression path.

## 2. Lexical Structure

### 2.1 Tokens

The target grammar adds the following keywords to the current 99-keyword set:

| New keyword | Purpose |
|-------------|---------|
| `effect` | introduces row aliases and groups (operation declarations now use `interface` per NOTE-022) |
| `alias` | transparent alias introducer (within `effect`) |
| `group` | diagnostic group introducer (within `effect`) |
| `handle` | effect handler boundary (already reserved, now active) |
| `raise` | raise an effect (already reserved, now active) |
| `guard` | channel guard contract |
| `profile` | row profile constraint |
| `extern` | reserved for future host/FFI (no grammar production; see NOTE-024) |

The following keywords become deprecated compatibility aliases:

| Deprecated | Replacement |
|------------|-------------|
| `do:Act` | `do { ... }` with inferred row |
| `do:Proc` | `do { ... }` with inferred row |
| `do:Workflow` | `do { ... }` with inferred row |
| `ret` | `return` |
| `workflow` | `fn` with contract annotations and row |
| `capabilities` | operation items in effect row |
| `observes` | operation items in effect row |
| `receives` | `channel` items in effect row |
| `obligations` | `obligation` items in effect row |
| `owns` | `resource` items in effect row |
| `uses` | operation items in effect row |
| `plays role` | `role` items in effect row |

### 2.2 Operators and Punctuation

This target grammar adds no new built-in operators beyond the current parser surface. The existing
arrow `->` remains the function arrow, and the pipe operator `|>` remains reserved but not active as
a built-in operator in this document.

User-defined prefix, infix, suffix, and mixfix notation, including binary infix operator sections,
is reserved for [SPEC-095c](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md). Parser and AST work must
therefore preserve operator-like tokens and grouping at the surface layer instead of assuming a
closed operator set. Notation expands to ordinary callable syntax before Core lowering.

## 3. Module Structure

### 3.1 Crate Root and Module File

Unchanged from current grammar.

### 3.2 Definition List

The target definition list adds `effect_alias_definition`, `effect_group_definition`,
`handler_decl`, and `newtype_definition`:

```ebnf
definition = visibility (
    fn_definition
  | handler_decl          -- standalone handler at module level (NOTE-023 §7)
  | type_definition
  | newtype_definition    -- zero-cost nominal wrapper (NOTE-026)
  | role_definition
  | legacy_capability_definition
  | interface_definition
  | impl_definition
  | law_definition
  | proof_definition
  | proposition_definition
  | resource_type_definition
  | data_kind_definition
  | sealed_domain_definition
  | sealed_associated_family_decl
  | type_fn_definition
  | proxy_definition
  | yield_definition
  | use_decl
  | effect_alias_definition
  | effect_group_definition
) ;
```

Note: per NOTE-022, operation declarations are no longer a separate `effect_definition`
form; operations are declared via the existing `interface_definition` production. Per
NOTE-023 §7, `handler_decl` appears both at module level (standalone handler) and inside
impl bodies (co-located handler, see §8.4). In both positions it produces a handler-marked
function in the value namespace.

### 3.2.1 Target Entry Shape

The target executable entry shape is an ordinary function named `main`:

```ebnf
entry_definition = "fn" "main" "(" parameter_list? ")" "->" effect_row? type
                   proposition_tail? "{" expr "}" ;
```

The entry function follows the same row, type, expression, and lowering rules as any other `fn`.
Implementations may retain legacy `workflow main` as compatibility/runtime-profile syntax during
migration, but it must not be the only path for target entry sources.

### 3.3 Operation Interface Declarations

Per NOTE-022, operations are declared as `interface` members using ordinary `fn`
signatures. There is no longer a distinct `effect_definition` production for operation
declarations. The `effect_member` grammar shape is retained and applies to interface
methods:

```ebnf
effect_member = "fn" identifier type_params? "(" parameter_list? ")" "->" type
                contract_clause* ";" ;
```

Examples:

```ash
interface Fs {
    fn read(path: Path) -> String;
    fn write(path: Path, contents: String) -> Unit;
}
```

### 3.4 Effect Alias and Group Definitions

```ebnf
effect_alias_definition = "effect" "alias" identifier "=" effect_row ";" ;

effect_group_definition = "effect" "group" identifier "=" effect_row ";" ;
```

Examples:

```ash
effect alias IO = {PosixFs::read, PosixFs::write, StdoutLog::write};

effect group WorkflowIO = {
    PosixFs::read,
    StdoutLog::write,
    evidence audit_log,
};
```

## 4. Expressions

### 4.1 Expression Hierarchy

The target expression hierarchy is unchanged except for the addition of `do_block_expr` as a
primary expression and the removal of `act_block_expr` as a distinct form.

```ebnf
primary_expr = literal
             | identifier
             | qualified_identifier
             | "(" expr ")"
             | list_expr
             | record_constructor
             | tuple_constructor
             | do_block_expr
             | comprehension_expr
             | with_error_expr
             | field_access
             | index_access
             | call_expr
             | builtin_expr
             | "check" expr
             | "fail" [ string_literal ]
             | "raise" effect_item
             ;
```

`builtin fn` is not a target definition form. Trusted stdlib handler/provider methods may use
`builtin_expr` to delegate to a runtime primitive:

```ebnf
builtin_expr = "builtin" "(" runtime_primitive_symbol { "," expr } ")" ;

runtime_primitive_symbol = qualified_identifier ;
```

The `runtime_primitive_symbol` production is a placeholder for a symbol/key type or equivalent
typed literal. It is deliberately not a string literal: the compiler must validate the primitive
key and align the surrounding handler method signature with the runtime primitive descriptor.

### 4.2 Do Block Expression

```ebnf
do_block_expr = "do" "{" { do_stmt } "}"
              | "do" do_profile "{" { do_stmt } "}" ;
do_profile = ":" identifier ;  -- compatibility profile only

do_stmt = "let" identifier "=" expr ";"
        | identifier "<-" expr ";"
        | "return" expr ";"
        | expr ";"
        ;
```

Target `do { ... }` has no named tower target. It is checked as direct-style sequencing:
`let` binds an ordinary expression value, `<-` is direct binding sugar for the same checked
value in the current implementation slice, and `return` supplies the final expression type.
Rows remain callable requirement metadata; the `do` block does not install authority or choose
an `Act`, `Proc`, or `Workflow` runtime mode.

`handle effect_item with { ... }` is obsolete compatibility syntax, not target syntax. In target
Ash, handlers are ordinary handler-marked callables installed with `handle expr with handler_name`,
and default dynamic contract failure is structured bottom rather than a resumable handler arm.
Recoverable contract behavior must be modeled through an explicit `fail` row item and handler.

Examples:

```ash
fn read_config(path: String) -> {PosixFs::read} String {
    do {
        contents <- PosixFs::read(path);
        return contents
    }
}

fn safe_divide(a: Int, b: Int) -> {evidence nonzero_divisor} Int
where row {
    fact nonzero_divisor: requires { b != 0 };
    evidence nonzero_divisor;
} {
    do {
        return a / b
    }
}
```

The optional `do_profile` is a compatibility hint for migration:

```ash
do:Act { ... }   -- check against Act row profile
do:Proc { ... }  -- check against Proc row profile
do:Workflow { ... } -- check against Workflow row profile
```

During migration, `do:Act`, `do:Proc`, and `do:Workflow` are accepted as `do` with a profile
annotation. A future deprecation spec may remove them.

### 4.3 Handler Expressions

Per NOTE-023 (revised by NOTE-025), the target grammar adds first-class handler expressions and
handler-marked callables as eliminators for computation rows. Inline `handle effect_item with { ... }`
from older sketches is legacy compatibility syntax and is not part of the target grammar.

**`on` eliminator.** An `on` expression scrutinises a row-bearing computation and dispatches
on its operations:

```ebnf
on_expr = "on" expr "{" handler_clause+ "}" ;

handler_clause = impl_type_ref "::" operation_name "(" pattern "," identifier ")" "=>" expr
              | "done" "(" identifier ")" "=>" expr
              ;
```

- Each operation clause matches `ImplType::method(pattern, continuation) => expr`. The
  operation identity is impl-type-qualified (NOTE-025).
- The continuation parameter (`identifier`) is an ordinary function-typed parameter, **not** a
  keyword. It is the resume function passed by the handler runtime.
- A `done(value) => expr` clause completes handling when the computation returns normally.

Example:

```ash
on run(req) {
    PosixFs::read(path, k) => k(PosixFs::read(path)),
    done(v) => v,
}
```

**`handle ... with` sugar.** The `handle expr with identifier` form selects a named handler
function for the given computation. The identifier resolves through normal value-name
resolution — it must be a function whose first parameter accepts the thunk type:

```ebnf
handle_with_expr = "handle" expr "with" identifier ;
```

Example:

```ash
handle run(req) with posix_fs
```

This desugars to `posix_fs(fn () -> run(req))`. The identifier is always a value-namespace
function (per NOTE-025 §3), never a type.

**`handler` declaration and the handler marker.** Per NOTE-023 §7 (revised), `handler` is
**not** a pure keyword alias for `fn`. It is a declaration-site keyword that produces a
function whose type carries a **handler marker** — a type-level attribute identifying
handler intent, analogous to comp mode (eager/lazy/memo). The underlying function type is
structurally identical to the equivalent `fn` type; the marker is erased at runtime and
carries no data.

The handler marker serves two purposes:

1. **Derive filtering.** Inside an impl body, `derive handler` folds over operations only —
   members without the handler marker (operations) are folded; members with it (handlers) are
   skipped. See §8.4 for the `impl_member` grammar.
2. **`handle expr with` validation.** In the `handle...with` sugar, name resolution checks
   that the resolved identifier carries the handler marker. A plain `fn` with a compatible
   signature is rejected. See §6.4 for the marked `fn_type`.

The `handler` declaration production (`handler_decl`) is documented in §8.4 (Impl
Definitions) because it appears both at module level and as an impl member. Example:

```ash
handler posix_fs<A, r: Row>(
    comp: Unit -> {PosixFs::read, PosixFs::write | r} A
) -> {r} A
where requires host posix_fs
{
    on comp() {
        PosixFs::read(path, resume) => resume(PosixFs::read(path))
        PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
        done(value) => value
    }
}
```

**Subtyping.** A handler-marked function coerces to a plain function (`handler fn <: fn`).
The reverse is not true: a plain `fn` cannot be used where a handler is required. This means
a handler can be passed to any higher-order function expecting a plain function of the same
signature, but `handle expr with utility` fails if `utility` is a plain `fn`.

**Derive (compiler-synthesized handler from impl).** Per NOTE-025 §7.3, an impl body may
declare `derive handler <name>;` — the compiler synthesizes the total deep handler (the
identity fold over ALL interface operations) from the impl's method bodies. The generated
function is handler-marked and available in the value namespace. The handler marker is what
lets derive filter correctly: it folds over members without the marker, skipping those with
it. See §8.4 for the `derive_decl` grammar.

**Notes on continuation and multiplicity.**

- The continuation parameter is an ordinary function-typed parameter; there is no special
  `resume` keyword in the surface grammar.
- Multiplicity is derived from the function type: a handler is **affine** when the
  continuation's computation row is non-empty (the continuation may not be invoked more than
  once), and **multi-shot** when the row is pure (empty). The grammar does not encode this
  directly; it is enforced by the type system per NOTE-023.

### 4.4 Legacy Act Block Expression

`act { ... }` is accepted as a compatibility alias for `do { ... }`. It is not a distinct
syntactic form in the target grammar.

## 5. Patterns

Unchanged from current grammar.

## 6. Types

### 6.1 Type Hierarchy

The target type hierarchy adds `effect_row_type` as a type atom:

```ebnf
type = type_atom { "->" type } ;

type_atom = type_name
          | type_constructor
          | tuple_type
          | record_type
          | fn_type
          | effect_row_type
          | parenthesized_type
          | type_hole
          ;
```

### 6.2 Effect Row Type

```ebnf
effect_row_type = "{" [ row_contents ] "}" ;

row_contents = row_variable
             | effect_item { "," effect_item } [ "," ] [ "|" row_variable ]
             ;

row_variable = identifier ;
```

Examples:

```ash
{}                                      -- empty row
{PosixFs::read}                                -- closed row
{PosixFs::read, policy production_rate}         -- multiple requirements
{PosixFs::read | r}                             -- open row
{r}                                      -- whole-row variable
{IO}                                     -- transparent alias or group reference
```

### 6.3 Effect Items

```ebnf
effect_item = operation_effect
            | resource_effect
            | role_effect
            | policy_effect
            | contract_effect
            | channel_effect
            | process_effect
            | failure_effect
            | evidence_effect
            | effect_group_ref
            ;

operation_effect = operation_path [ "." operation_name ] ;
operation_path = identifier { "::" identifier } ;
operation_name = identifier ;

resource_effect = "resource" resource_path [ resource_mode ] ;
resource_mode = "own" | "read" | "write" | "split" | "join" ;

role_effect = "role" role_path ;

policy_effect = "policy" policy_path ;

contract_effect = requires_effect
                | ensures_effect
                | invariant_effect
                | law_effect
                | obligation_effect
                | guard_effect
                | trace_contract_effect
                ;

requires_effect = "requires" "{" predicate "}" ;
ensures_effect = "ensures" "{" predicate "}" ;
invariant_effect = "invariant" "{" predicate "}" ;
law_effect = "law" identifier "{" predicate "}" ;
obligation_effect = "obligation" obligation_path ;
guard_effect = "guard" "{" predicate "}" ;
trace_contract_effect = "trace" trace_contract_path ;
trace_contract_path = identifier { "::" identifier } ;

predicate          = predicate_or ;
predicate_or       = predicate_and { "||" predicate_and } ;
predicate_and      = predicate_not { "&&" predicate_not } ;
predicate_not      = [ "!" ] predicate_cmp ;
predicate_cmp      = predicate_add [ cmp_op predicate_add ] ;
predicate_add      = predicate_mul { ("+" | "-") predicate_mul } ;
predicate_mul      = predicate_unary { ("*" | "/" | "%") predicate_unary } ;
predicate_unary    = literal
                   | identifier
                   | "result"
                   | "message"
                   | "old" "(" snapshot_expr ")"
                   | predicate_call
                   | field_projection
                   | tuple_projection
                   | "(" predicate ")"
                   ;
predicate_call     = predicate_function "(" [ predicate_args ] ")" ;
predicate_function = identifier | qualified_identifier ;
predicate_args     = predicate { "," predicate } ;
snapshot_expr      = identifier { "." identifier } ;
cmp_op             = "==" | "!=" | "<" | "<=" | ">" | ">=" ;

channel_effect = channel_message_effect
               | channel_close_effect
               ;

channel_message_effect = "channel" channel_message_mode channel_path type [ channel_guard ] ;
channel_message_mode = "send" | "receive" | "select" ;
channel_close_effect = "channel" "close" channel_path ;
channel_guard = "where" "{" predicate "}" ;

process_effect = "proc" process_operation ;
process_operation = "spawn" | "await" | "join" | "cancel" | "yield" | identifier ;

failure_effect = "fail" [ failure_path ] ;

evidence_effect = "evidence" evidence_path
                | "report" report_path
                ;

effect_group_ref = identifier ;
```

Note (NOTE-021 §7): The `requires_effect`, `ensures_effect`, `invariant_effect`,
`law_effect`, and `guard_effect` row entries are convenience forms. Where a row
references such a predicate, the preferred spelling is an `evidence` reference pointing
at a named proof obligation, law, or fact binding — i.e., the canonical row item is
`evidence <path>` and the predicate form is sugar. This keeps the computation row
algebraic and avoids re-checking predicates at each row-position use. Direct predicate
row items are retained as a convenience form and do not change the surface grammar above.
Per NOTE-031, this grammar is a contract-position grammar, not permission to run arbitrary
Ash expressions while checking a contract. The type checker classifies parsed predicates as
static, dynamic, or rejected before lowering. Source-level quantifiers are not introduced by
this grammar; `forall`/`exists` in contract proofs remain internal proof metadata unless a
later grammar revision admits them explicitly.

### 6.4 Function Type

The target function type includes an effect row and an optional handler marker:

```ebnf
fn_type = [ "handler" ] [ effect_row_type ] "(" [ parameter_list ] ")" [ "->" type ] ;
```

The optional `handler` prefix is the **handler marker** (per NOTE-023 §7). It is a type-level
attribute that identifies handler intent — analogous to comp mode (eager/lazy/memo). The
underlying function type is structurally identical with or without the marker; the marker is
erased at runtime and carries no data.

- A plain function type has no marker: `(Unit -> {PosixFs::read | r} A) -> {r} A`.
- A handler-marked type carries the marker: `handler (Unit -> {PosixFs::read | r} A) -> {r} A`.

The `handler` keyword at declaration site produces the marker. It is **not** a pure alias for
`fn` — the marker is significant for derive filtering and `handle expr with` validation (see
§4.3). **Subtyping:** `handler fn <: fn` — a handler-marked function coerces to a plain
function (additive refinement). The reverse is not allowed: a plain `fn` cannot be used where
a handler is required.

Examples:

```ash
fn add(a: Int, b: Int) -> {} Int { a + b }
fn read_file(path: String) -> {PosixFs::read} String { ... }
fn map<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

NOTE-021 introduces an expanded `where row { ... }` alternate layout for the computation
row on function types. This is purely a layout alternative — the row's meaning is
unchanged. Inline row syntax and `where row { ... }` are mutually exclusive on a single
signature (per NOTE-021 §3):

```ash
fn process(req: Request) -> Response
where
    row {
        http.get,
        fail ProcessError,
    }
```

The `where row { ... }` form is preferred when the row is long or when other `where`
clauses (evidence rows, contract items) accompany it.

### 6.5 Type Parameters and Kinds

The target grammar adds `Row` as a kind atom (per NOTE-021, the source kind for
computation-row variables is `Row`; the older spellings `EffectRow` and `Effect` are
removed):

```ebnf
type_param = identifier [ ":" kind ] ;
kind = "Type" | "Row" | "Resource" ;
```

### 6.6 Bodyless Type Definitions

Per NOTE-025 §7.1, the `type_definition` production gains an optional body. The current
grammar (inherited from SPEC-095) requires `= type_body`:

```ebnf
-- SPEC-095 (current):
type_definition = "type" identifier [ type_params ] "=" type_body ";" ;
```

The target delta makes `= type_body` optional:

```ebnf
-- SPEC-095b (target):
type_definition = "type" identifier [ type_params ] [ "=" type_body ] ";" ;
```

Without `=`, the type is a **bodyless nominal type** — a new type with no constructors and no
representation. It has identity but cannot be constructed. This is the minimal form for
effect identity carriers (NOTE-025): an impl type that exists purely to distinguish effect
operation identities (e.g., `PosixFs::read` vs `MemoryFs::read`) needs no data.

**Critical distinction from transparent alias.** `type PosixFs = Unit;` is a transparent
alias — it canonicalizes to `Unit` at definitional equality (per SPEC-058/SPEC-100). This
collapses all identity-only types into one identity (`PosixFs ≡ Unit`, `MemoryFs ≡ Unit` →
identities collide). `type PosixFs;` is a nominal type that equals no other type, preserving
distinct identities.

With `= type_body` present, the existing forms are unchanged:

- `= alias_body` (a bare type): transparent alias — canonicalizes to origin head.
- `= enum_body` / `struct_body` / `record_type` / `tuple_type`: nominal data type with
  constructors.

Examples:

```ash
type PosixFs;                                      -- bodyless: identity-only, unconstructable
type ConfiguredFs = { root: Path, readonly: Bool }; -- data-carrying: nominal record
type List<T> = Nil | Cons { head: T, tail: List<T> }; -- nominal ADT
```

### 6.7 Newtype Definitions

Per NOTE-026, `newtype` is the zero-cost nominal wrapper form. It gives an existing inhabited
type a distinct identity and a constructor while sharing the representation of the wrapped
type at runtime.

```ebnf
newtype_definition = "newtype" identifier [ type_params ] "=" constructor "(" type ")" ";" ;
constructor = identifier ;
```

Examples:

```ash
newtype CustomFs = CustomFs(PosixFs);
newtype Tagged<Label> = Tagged(String);
newtype Edge<From, To> = Edge(Unit);
```

The constructor name is a value-level constructor. It commonly matches the type name, but the
grammar does not require that.

`newtype` differs from transparent aliasing:

```ash
type UserId = Int;            -- transparent alias: UserId ≡ Int
newtype OrderId = OrderId(Int); -- nominal wrapper: OrderId ≠ Int, runtime representation Int
```

Type parameters that do not occur in the representation type are phantom parameters. They
remain part of the type identity even though they do not affect runtime layout. This makes
phantom tagging a consequence of `newtype`, not a separate grammar form.

The wrapped representation type must be inhabited. A newtype over a bodyless nominal type is
ill-formed because there is no value to wrap.

## 7. Workflow Definitions

### 7.1 Target Workflow Definition

The target workflow definition is a `fn` with an effect row and contract annotations:

```ebnf
workflow_def = "fn" identifier [ type_params ] parameter_list [ "->" type ]
               [ effect_row_type ]
               [ fn_contract ]
               "{" { workflow_stmt } "}" ;
```

Example:

```ash
fn processor(req: Request) -> {role ai_agent, http.get} Response {
    do {
        result <- http.get(req.url);
        return result
    }
}
```

### 7.2 Legacy Workflow Definition

The current `workflow` keyword form is accepted as a compatibility alias during migration:

```ash
workflow processor
    plays role(ai_agent)
    capabilities: [network @ { hosts: ["*.example.com"] }]
{
    ...
}
```

This lowers to the same semantic representation as the `fn` form with a row and contract.

### 7.3 Workflow Statements

The target workflow statement set is a subset of the current 28 statement types, unified
through the `do` block and effect row:

```ebnf
workflow_stmt = let_stmt
              | expr ";"
              | "return" expr ";"
              | "if" expr "then" "{" { workflow_stmt } "}" [ "else" "{" { workflow_stmt } "}" ]
              | "match" expr "{" { match_arm } "}"
              | "for" pattern "in" expr "do" "{" { workflow_stmt } "}"
              | "done" ";"
              ;
```

Legacy statements such as `act`, `observe`, `send`, `receive`, `set`, `propose`, `decide`,
`check`, `oblige`, `yield`, `orient`, `with`, `maybe`, `must`, and `ret` are accepted as
compatibility aliases during migration. Each legacy statement lowers to an equivalent `do`
block expression or ordinary expression with the appropriate effect item in the row.

## 8. Declarations and Definitions

### 8.1 Role Definitions

The target role definition adds explicit operation entailment:

```ebnf
role_definition = "role" identifier [ "(" parameter_list ")" ] "{" { role_clause } "}" ;

role_clause = operation_ref ";"
            | obligation_ref ";"
            | "entails" effect_item ";"
            ;
```

Example:

```ash
role manager {
    entails approve_transfer;
    entails policy transfer_policy;
}
```

### 8.2 Legacy Capability Definitions

Target Ash has no separate capability declaration form. Current `capability` declarations are
legacy compatibility syntax and lower to effect operation declarations plus provider/admission
metadata during migration.

### 8.3 Interface Definitions

Interface methods may carry effect rows:

```ash
interface EffectfulMap<F> {
    map<A, B, r: Row>(F<A>, A -> {r} B) -> {r} F<B>;
}
```

### 8.4 Impl Definitions

Per NOTE-025 and NOTE-023 §7, an impl body contains three kinds of members: operations
(`fn`), handlers (`handler`), and derivations (`derive handler`). The `impl_member`
production replaces the single `impl_method` form inherited from SPEC-095:

```ebnf
impl_definition = "impl" type_name "for" type_name [ where_clause ]
                  "{" { impl_member } "}" ;

impl_member = impl_method | handler_decl | derive_decl ;

impl_method = "fn" identifier type_params? "(" parameter_list? ")" [ "->" type ]
              fn_body ;

handler_decl = "handler" identifier type_params? "(" parameter_list ")"
               "->" type [ where_clause ] handler_body ;

handler_body = "{" "on" expr "{" handler_clause+ "}" "}" ;
               -- handler_clause is defined in §4.3 (on_expr)

derive_decl = "derive" "handler" identifier ";" ;
```

**Operation methods (`impl_method`).** Each operation declared in the interface must have a
corresponding `fn` method body in the impl. The method body is the default deep-handler
behavior — what `derive handler` wraps as `resume(ImplType::op(args))`.

**Handler declarations (`handler_decl`).** A named handler function defined inside the impl
body, co-located with the operations it interprets. Its type carries the handler marker
(§6.4). Multiple handlers per impl are allowed (NOTE-025 §7.4) — they are distinct
value-namespace bindings with distinct names.

**Derive declarations (`derive_decl`).** `derive handler <name>;` synthesizes the total deep
handler — the identity fold over ALL interface operations (NOTE-025 §7.3). The compiler
generates a `handler_decl` with one clause per operation: `ImplType::op(args, resume) =>
resume(ImplType::op(args))`, plus `done(value) => value`. The generated function is
handler-marked.

**How derive filters.** Derive folds over `impl_method` members only (those without the
handler marker). `handler_decl` members are skipped — they have the handler marker on their
type, which distinguishes them from operations. This works across module boundaries because
the marker survives into module summaries.

Example:

```ash
type PosixFs;

impl Fs for PosixFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }

    derive handler posix_fs;       -- total fold — all default

    handler logging_fs<A, r: Row>(comp: Unit -> {PosixFs::read, PosixFs::write | r} A) -> {r} A {
        on comp() {
            PosixFs::read(path, resume) => {
                log("reading {}", path);          -- custom behavior in the clause
                resume(PosixFs::read(path))
            }
            PosixFs::write(path, contents, resume) => resume(PosixFs::write(path, contents))
            done(value) => value
        }
    }
}
```

Both `posix_fs` (derived) and `logging_fs` (explicit) are handler-marked values in the value
namespace. The caller chooses which to install via `handle expr with <name>`.

## 9. Policy Expressions

Unchanged from current grammar. Policy expressions remain combinators over named policy
bindings. Anonymous inline policy expressions in row position are deferred.

## 10. Migration Compatibility

### 10.1 Accepted Legacy Forms

During migration, the following legacy forms are accepted and lowered to the target
representation:

| Legacy | Target equivalent |
|--------|-------------------|
| `workflow X { ... }` | `fn X -> { ... }` with row profile |
| `do:Act { ... }` | `do { ... }` with Act profile |
| `do:Proc { ... }` | `do { ... }` with Proc profile |
| `do:Workflow { ... }` | `do { ... }` with Workflow profile |
| `act { ... }` | `do { ... }` |
| `ret expr` | `return expr` |
| `capabilities: [cap]` | operation items in row |
| `plays role(R)` | `role R` in row |
| `observes: [cap]` | operation items in row |
| `receives: [chan]` | `channel` items in row |
| `obligations: [obl]` | `obligation` items in row |
| `owns: [res]` | `resource` items in row |
| `uses: [cap]` | operation items in row |

### 10.2 Deprecated Forms

The following forms are deprecated with rewrite hints:

| Deprecated | Rewrite hint |
|------------|--------------|
| `workflow` keyword | Use `fn` with effect row and contract annotations |
| `ret` | Use `return` |
| `do:Act` / `do:Proc` / `do:Workflow` | Use `do` with inferred or explicit row |

### 10.3 Rejected Forms

The following forms are rejected in the target grammar:

| Rejected | Reason |
|----------|--------|
| Anonymous inline policy in row position | Policies must be named bindings (SPEC-006) |
| `effect alias` cycle | Cycles are rejected |
| `role` without admission context | Role effects require admission |
| `proc` effects in pure/Act profile | Requires Proc-capable profile |

## 11. See Also

- [SPEC-095a: Current Grammar](SPEC-095a-CURRENT-GRAMMAR.md) — what the parser accepts today
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR Changes](SPEC-098b-TARGET-IR.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)

## 12. Changelog

- 2026-06-29: Removed stale inline contract-handler target syntax, added trace-contract row syntax, and reserved user-defined notation/operator sections for SPEC-095c.
- 2026-06-18: Created as target-state grammar document. Defined effect row syntax, unified `do` form, effect aliases/groups, and migration compatibility.
- 2026-06-27: Reconciled with NOTE-021 (Row kind, where row layout, evidence rows), NOTE-022 (effects as interfaces, no effect keyword for operations), NOTE-023 (handler surface grammar: on, handle...with, named handler sugar).
- 2026-06-27: Reconciled with NOTE-025 (effect identity via sorts and impls). Handler clause identities changed from interface-qualified (`Fs.read`) to impl-type-qualified (`PosixFs::read`). Named handler sugar replaced by `handler`-as-alias-for-`fn`. Added derive mechanism. §4.3 revised.
- 2026-06-28: Handler marker reconciliation. §4.3: `handler` is no longer a pure alias for `fn` — it produces a handler-marked function type (type-level attribute). Added subtyping (`handler fn <: fn`), derive filtering via the marker, and `handle expr with` validation. Removed stale `handler_fn_decl` production (replaced by `handler_decl` in §8.4). §6.4: `fn_type` gains optional `handler` prefix marker. §6.6 (new): bodyless `type_definition` delta (`= type_body` optional) for identity-only nominal types. §8.4 (new): `impl_definition` with `impl_member` production (`impl_method`, `handler_decl`, `derive_decl`). §3.2: `handler_decl` added to top-level definition list (standalone + in-impl positions).
- 2026-06-28: Reconciled with NOTE-026. Added `newtype_definition` to the top-level definition list and §6.7 grammar for zero-cost nominal wrappers with explicit constructors, phantom parameters, inhabited representation requirement, and transparent-alias contrast.
- 2026-06-29: Reconciled with NOTE-031. Replaced the contract-position `predicate = expr` placeholder with a restricted predicate grammar including `old(snapshot_expr)`, predicate calls, projections, boolean/numeric operators, and explicit deferral of source-level quantifier syntax.
