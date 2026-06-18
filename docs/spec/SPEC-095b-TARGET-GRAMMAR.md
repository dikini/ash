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
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-096-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097-TARGET-TYPE-SYSTEM.md
---

# SPEC-095b: Ash Surface Syntax Grammar — Target State

**Status:** Draft — target surface syntax for the unified effect-row language
**Scope:** This document defines the grammar we want the parser to accept once the
unified effect system is implemented. It is a goal-state living document.
**Depends on:** SPEC-096 (Target Effect System), SPEC-097 (Target Type System)

## 1. Summary

The target grammar unifies Ash's effect-accounting surface into one coherent syntax:

- effect rows on function types and computation blocks;
- a single `do { ... }` form with effect requirements inferred from the body;
- structured effect items for capabilities, resources, roles, policies, contracts, channels, process operations, failure, and evidence;
- transparent effect aliases and diagnostic groups;
- row variables for polymorphism.

This grammar replaces the separate `do:Act`, `do:Proc`, `do:Workflow`, `workflow`, `act`,
`ret`, and legacy workflow-statement syntax with a unified surface. During migration, legacy
forms remain accepted as compatibility aliases.

## 2. Lexical Structure

### 2.1 Tokens

The target grammar adds the following keywords to the current 99-keyword set:

| New keyword | Purpose |
|-------------|---------|
| `effect` | introduces effect aliases and groups |
| `alias` | transparent alias introducer (within `effect`) |
| `group` | diagnostic group introducer (within `effect`) |
| `handle` | effect handler boundary (already reserved, now active) |
| `raise` | raise an effect (already reserved, now active) |
| `guard` | channel guard contract |
| `profile` | row profile constraint |

The following keywords become deprecated compatibility aliases:

| Deprecated | Replacement |
|------------|-------------|
| `do:Act` | `do { ... }` with inferred row |
| `do:Proc` | `do { ... }` with inferred row |
| `do:Workflow` | `do { ... }` with inferred row |
| `ret` | `return` |
| `workflow` | `fn` with contract annotations and row |
| `capabilities` | `cap` items in effect row |
| `observes` | `cap` items in effect row |
| `receives` | `channel` items in effect row |
| `obligations` | `obligation` items in effect row |
| `owns` | `resource` items in effect row |
| `uses` | `cap` items in effect row |
| `plays role` | `role` items in effect row |

### 2.2 Operators and Punctuation

No new operators. The existing arrow `->` remains the function arrow. The pipe
operator `|>` remains reserved but not active.

## 3. Module Structure

### 3.1 Crate Root and Module File

Unchanged from current grammar.

### 3.2 Definition List

The target definition list adds `effect_alias_definition` and `effect_group_definition`:

```ebnf
definition = visibility (
    fn_definition
  | builtin_fn_definition
  | type_definition
  | role_definition
  | capability_definition
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

### 3.3 Effect Alias and Group Definitions

```ebnf
effect_alias_definition = "effect" "alias" identifier "=" effect_row ";" ;

effect_group_definition = "effect" "group" identifier "=" effect_row ";" ;
```

Examples:

```ash
effect alias IO = {cap fs.read, cap fs.write, cap log.write};

effect group WorkflowIO = {
    cap fs.read,
    cap log.write,
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
             | "check" expr
             | "fail" [ string_literal ]
             | "raise" effect_item
             ;
```

### 4.2 Do Block Expression

```ebnf
do_block_expr = "do" [ do_profile ] "{" { do_stmt } "}" ;
do_profile = ":" identifier ;

do_stmt = "let" identifier "=" expr ";"
        | identifier "<-" expr ";"
        | "return" expr ";"
        | "handle" effect_item "with" "{" { handler_arm } "}" ";"
        ;

handler_arm = identifier "->" expr ";" ;
```

Examples:

```ash
fn read_config(path: String) -> {cap fs.read} String {
    do {
        contents <- fs.read(path);
        return contents
    }
}

fn safe_divide(a: Int, b: Int) -> {} Int {
    do {
        handle requires {b != 0} with {
            requires -> if b != 0 then () else return 0
        };
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

### 4.3 Legacy Act Block Expression

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
{cap fs.read}                           -- closed row
{cap fs.read, policy production_rate}    -- multiple requirements
{cap fs.read | r}                        -- open row
{r}                                      -- whole-row variable
{IO}                                     -- transparent alias or group reference
```

### 6.3 Effect Items

```ebnf
effect_item = capability_effect
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

capability_effect = "cap" capability_path [ "." operation_name ] ;
capability_path = identifier { "::" identifier } ;
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
                ;

requires_effect = "requires" "{" predicate "}" ;
ensures_effect = "ensures" "{" predicate "}" ;
invariant_effect = "invariant" "{" predicate "}" ;
law_effect = "law" identifier "{" predicate "}" ;
obligation_effect = "obligation" obligation_path ;
guard_effect = "guard" "{" predicate "}" ;

predicate = expr ;

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

### 6.4 Function Type

The target function type includes an effect row:

```ebnf
fn_type = [ effect_row_type ] "(" [ parameter_list ] ")" [ "->" type ] ;
```

Examples:

```ash
fn add(a: Int, b: Int) -> {} Int { a + b }
fn read_file(path: String) -> {cap fs.read} String { ... }
fn map<A, B, r: EffectRow>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

### 6.5 Type Parameters and Kinds

The target grammar adds `EffectRow` as a kind atom:

```ebnf
type_param = identifier [ ":" kind ] ;
kind = "Type" | "EffectRow" | "Effect" | "Capability" | "Resource" ;
```

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
fn processor(req: Request) -> {role ai_agent, cap http.get} Response {
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
              | "handle" effect_item "with" "{" { handler_arm } "}" ";"
              ;
```

Legacy statements such as `act`, `observe`, `send`, `receive`, `set`, `propose`, `decide`,
`check`, `oblige`, `yield`, `orient`, `with`, `maybe`, `must`, and `ret` are accepted as
compatibility aliases during migration. Each legacy statement lowers to an equivalent `do`
block expression or ordinary expression with the appropriate effect item in the row.

## 8. Declarations and Definitions

### 8.1 Role Definitions

The target role definition adds explicit capability entailment:

```ebnf
role_definition = "role" identifier [ "(" parameter_list ")" ] "{" { role_clause } "}" ;

role_clause = capability_ref ";"
            | obligation_ref ";"
            | "entails" effect_item ";"
            ;
```

Example:

```ash
role manager {
    entails cap approve_transfer;
    entails policy transfer_policy;
}
```

### 8.2 Capability Definitions

Unchanged from current grammar, with the addition that capability interfaces may reference
effect rows in operation signatures.

### 8.3 Interface Definitions

Interface methods may carry effect rows:

```ash
interface EffectfulMap<F> {
    map<A, B, r: EffectRow>(F<A>, A -> {r} B) -> {r} F<B>;
}
```

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
| `capabilities: [cap]` | `cap` items in row |
| `plays role(R)` | `role R` in row |
| `observes: [cap]` | `cap` items in row |
| `receives: [chan]` | `channel` items in row |
| `obligations: [obl]` | `obligation` items in row |
| `owns: [res]` | `resource` items in row |
| `uses: [cap]` | `cap` items in row |

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
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)

## 12. Changelog

- 2026-06-18: Created as target-state grammar document. Defined effect row syntax, unified `do` form, effect aliases/groups, and migration compatibility.
