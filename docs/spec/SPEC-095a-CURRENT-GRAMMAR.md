---
id: spec.ash.surface-grammar.current
title: Ash Surface Syntax Grammar — Current State
description: EBNF grammar derived from the current parser implementation, as of main HEAD
code_commit: e61f2792
kind: spec
audience: [human, agent]
authority: derived-from-code
status: active
stability: beta
owner: language
last_verified: 2026-06-18
verified_against:
  git_commit: e61f2792
  code:
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/parse_module.rs
    - crates/ash-parser/src/parse_module/fn_defs.rs
    - crates/ash-parser/src/parse_pattern.rs
    - crates/ash-parser/src/parse_type_def.rs
    - crates/ash-parser/src/parse_workflow.rs
    - crates/ash-parser/src/parse_observe.rs
    - crates/ash-parser/src/parse_receive.rs
    - crates/ash-parser/src/parse_send.rs
    - crates/ash-parser/src/parse_set.rs
    - crates/ash-parser/src/parse_policy.rs
    - crates/ash-parser/src/parse_use.rs
    - crates/ash-parser/src/parse_crate_root.rs
    - crates/ash-parser/src/parse_visibility.rs
    - crates/ash-parser/src/parse_utils.rs
    - crates/ash-parser/src/lexer.rs
---

# SPEC-095a: Ash Surface Syntax Grammar — Current State

**Status:** Active — derived from live parser source code
**Scope:** This document records what the parser actually accepts today. It is the
authority for current syntax, not a proposal for future syntax.
**Frozen against:** `e61f2792` (main HEAD at time of verification)

## 1. Lexical Structure

### 1.1 Tokens

```ebnf
keyword = "act" | "always" | "analyze" | "as" | "authored" | "builtin" | "by"
        | "by_definition" | "cap" | "capabilities" | "capability" | "case"
        | "check" | "config" | "control" | "crate" | "data" | "decide"
        | "decreases" | "deliberative" | "dependency" | "do" | "domain"
        | "done" | "else" | "ensures" | "epistemic" | "evaluative" | "execute"
        | "exists" | "external" | "fail" | "false" | "family" | "fn" | "for"
        | "forall" | "from" | "handle" | "handles" | "if" | "impl" | "in"
        | "interface" | "kind" | "law" | "let" | "match" | "maybe" | "mod"
        | "must" | "never" | "not" | "null" | "obligations" | "oblige" | "observe"
        | "observes" | "operational" | "orient" | "owns" | "panic" | "plays"
        | "proof" | "prop" | "property" | "propose" | "pub" | "quickcheck"
        | "read" | "receive" | "receives" | "requires" | "resource" | "resume"
        | "ret" | "return" | "returns" | "role" | "sealed" | "self" | "small_world"
        | "super" | "test" | "then" | "true" | "type" | "under" | "use" | "uses"
        | "wait" | "where" | "with" | "with_error" | "workflow" | "write" | "yield" ;

identifier = [a-zA-Z_] [a-zA-Z0-9_-]* ;

string_literal = '"' { any_char - '"' } '"' ;
int_literal = [0-9]+ ;
float_literal = [0-9]+ "." [0-9]+ ;
bool_literal = "true" | "false" ;
null_literal = "null" ;

line_comment = "--" { any_char - newline } newline ;
block_comment = "/*" { any_char } "*/" ;
```

### 1.2 Operators and Punctuation

```ebnf
add_op = "+" | "-" ;
mul_op = "*" | "/" | "%" ;
cmp_op = "==" | "!=" | "<" | ">" | "<=" | ">=" | "in" ;
and_op = "&&" ;
or_op = "||" ;
not_op = "!" ;
pipe_op = "|>" ;
assign_op = "=" ;
arrow = "->" ;

lparen = "(" ;
rparen = ")" ;
lbrace = "{" ;
rbrace = "}" ;
lbracket = "[" ;
rbracket = "]" ;
comma = "," ;
semicolon = ";" ;
colon = ":" ;
double_colon = "::" ;
underscore = "_" ;
```

## 2. Module Structure

### 2.1 Crate Root

```ebnf
crate_root = crate_metadata { dependency_decl } ;
crate_metadata = "crate" identifier ";" ;
dependency_decl = "dependency" identifier "=" string_literal ";" ;
```

### 2.2 Module File

```ebnf
module_file = { module_decl } { definition } ;
module_decl = visibility "mod" identifier ";" ;

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
) ;
```

### 2.3 Function Definitions

```ebnf
fn_definition = "fn" identifier [ type_params ] parameter_list [ "->" type ]
                [ fn_contract ] fn_body ;

builtin_fn_definition = "builtin" "fn" identifier [ type_params ] parameter_list
                        [ "->" type ] [ fn_contract ] ";" ;

fn_contract = { requires_clause } { ensures_clause } ;
requires_clause = "requires" ":" expr ";" ;
ensures_clause = "ensures" ":" expr ";" ;

parameter_list = "(" [ parameter { "," parameter } [","] ] ")" ;
parameter = identifier ":" type ;

fn_body = "{" { fn_stmt } "}" ;
fn_stmt = let_stmt
        | fn_expr
        | "panic" [ string_literal ]
        ;
```

## 3. Expressions

### 3.1 Expression Hierarchy (Precedence)

```ebnf
expr = closure_expr
     | fn_expr
     | if_let_expr
     | pipe_expr ;

pipe_expr = ternary_expr { "|>" call_expr } ;
ternary_expr = or_expr [ "?" expr ":" expr ] ;
or_expr = and_expr { "||" and_expr } ;
and_expr = comparison_expr { "&&" comparison_expr } ;
comparison_expr = additive_expr { cmp_op additive_expr } ;
additive_expr = multiplicative_expr { add_op multiplicative_expr } ;
multiplicative_expr = unary_expr { mul_op unary_expr } ;
unary_expr = [ "-" | "!" ] primary_expr ;

primary_expr = literal
             | identifier
             | qualified_identifier
             | "(" expr ")"
             | list_expr
             | record_constructor
             | tuple_constructor
             | act_block_expr
             | do_block_expr
             | comprehension_expr
             | with_error_expr
             | field_access
             | index_access
             | call_expr
             | "check" expr
             | "fail" [ string_literal ]
             ;

call_expr = primary_expr [ "(" [ argument_list ] ")" ] ;
argument_list = expr { "," expr } [","] ;
```

### 3.2 Specific Expression Types

```ebnf
closure_expr = "|" [ closure_param { "," closure_param } ] "|" "->" expr ;
closure_param = identifier [ ":" type ] ;

fn_expr = "fn" [ identifier ] "(" [ parameter_list ] ")" [ "->" type ] fn_body ;

if_let_expr = "if" "let" pattern "=" expr "then" expr [ "else" expr ] ;

list_expr = "[" [ expr { "," expr } [","] ] "]" ;

record_constructor = identifier "{" [ constructor_field { "," constructor_field } [","] ] "}" ;
constructor_field = identifier [ ":" expr ] ;

tuple_constructor = identifier "(" [ expr { "," expr } [","] ] ")" ;

field_access = expr "." identifier ;
index_access = expr "." int_literal ;

qualified_identifier = identifier "." identifier ;

comprehension_expr = "[" expr "|" comprehension_qualifier { "," comprehension_qualifier } "]" ;
comprehension_qualifier = "let" pattern "=" expr
                         | "if" expr
                         | pattern "<-" expr
                         ;

with_error_expr = "with_error" [ identifier ] "{" expr "}" [ "handle" "{" expr "}" ] ;
```

### 3.3 Act and Do Block Expressions

```ebnf
act_block_expr = "act" [ capability_ref ] "{" { act_stmt } "}" ;

do_block_expr = "do" ":" do_target "{" { do_stmt } "}" ;
do_target = identifier [ "<" type_arg_list ">" ] ;
type_arg_list = type { "," type } ;

do_stmt = "let" identifier "=" expr ";"
        | "requires" ":" expr ";"
        | "ensures" ":" expr ";"
        | "return" expr ";"
        ;
```

## 4. Patterns

```ebnf
pattern = wildcard_pattern
        | variable_pattern
        | literal_pattern
        | record_pattern
        | tuple_pattern
        | list_pattern
        | variant_pattern
        ;

wildcard_pattern = "_" ;
variable_pattern = identifier ;
literal_pattern = int_literal | float_literal | string_literal | bool_literal | null_literal ;

record_pattern = "{" [ record_pattern_field { "," record_pattern_field } [","] ] "}" ;
record_pattern_field = identifier [ ":" pattern ] ;

tuple_pattern = "(" [ pattern { "," pattern } [","] ] ")" ;
list_pattern = "[" [ pattern { "," pattern } [","] ] "]" ;

variant_pattern = identifier [ variant_payload ] ;
variant_payload = record_pattern | tuple_pattern | simple_field_list ;
simple_field_list = identifier { "," identifier } ;
```

## 5. Types

```ebnf
type = type_atom { "->" type } ;

type_atom = type_name
          | type_constructor
          | tuple_type
          | record_type
          | fn_type
          | parenthesized_type
          | type_hole
          ;

type_name = identifier [ "::" identifier ] ;
type_constructor = identifier "<" type_arg_list ">" ;
tuple_type = "(" [ type { "," type } [","] ] ")" ;
record_type = "{" [ record_type_field { "," record_type_field } [","] ] "}" ;
record_type_field = identifier ":" type ;
fn_type = "Fn" "(" [ type { "," type } [","] ] ")" [ "->" type ] ;
parenthesized_type = "(" type ")" ;
type_hole = "_" ;

type_params = "<" type_param { "," type_param } [","] ">" ;
type_param = identifier [ ":" kind ] ;
kind = "Type" | "Effect" | "Capability" | "Resource" ;
```

## 6. Workflow Statements

```ebnf
workflow_def = "workflow" identifier [ parameter_list ] [ "->" type ]
               [ workflow_header_clauses ] "{" { workflow_stmt } "}" ;

workflow_header_clauses = { requires_clause }
                          { ensures_clause }
                          { capabilities_clause }
                          { observes_clause }
                          { receives_clause }
                          { obligations_clause }
                          { owns_clause }
                          { uses_clause }
                          { plays_roles }
                          ;

capabilities_clause = "capabilities" "{" capability_ref { "," capability_ref } "}" ;
observes_clause = "observes" "{" observe_spec { "," observe_spec } "}" ;
receives_clause = "receives" "{" receive_spec { "," receive_spec } "}" ;
obligations_clause = "obligations" "{" obligation_ref { "," obligation_ref } "}" ;
owns_clause = "owns" "{" resource_ref { "," resource_ref } "}" ;
uses_clause = "uses" "{" capability_ref { "," capability_ref } "}" ;
plays_roles = "plays" "role" role_ref { "," role_ref } ;

workflow_stmt = act_stmt
               | check_stmt
               | decide_stmt
               | done_stmt
               | for_stmt
               | if_stmt
               | let_stmt
               | maybe_stmt
               | must_stmt
               | observe_stmt
               | orient_stmt
               | propose_stmt
               | receive_stmt
               | ret_stmt
               | send_stmt
               | set_stmt
               | with_stmt
               | oblige_stmt
               | yield_stmt
               | expr ";"
               ;

act_stmt = "act" [ action_ref ] [ "where" "{" { constraint } "}" ] [ "then" workflow_stmt ] ;
action_ref = identifier | qualified_identifier | "(" expr ")" ;

check_stmt = "check" obligation_ref [ "under" "{" { guard } "}" ] ;
decide_stmt = "decide" "under" policy_ref "then" "{" { decision_arm } "}" ;
decision_arm = identifier "->" workflow_stmt ;
done_stmt = "done" ;
for_stmt = "for" pattern "in" expr "do" "{" { workflow_stmt } "}" ;
if_stmt = "if" expr "then" "{" { workflow_stmt } "}" [ "else" "{" { workflow_stmt } "}" ] ;
let_stmt = "let" pattern "=" expr ";" ;
maybe_stmt = "maybe" "{" { workflow_stmt } "}" [ "else" "{" { workflow_stmt } "}" ] ;
must_stmt = "must" expr ";" ;
observe_stmt = "observe" [ observe_index ] [ "as" pattern ] ";" ;
observe_index = identifier "[" expr "]" ;
orient_stmt = "orient" expr "as" type ";" ;
propose_stmt = "propose" expr "as" identifier ";" ;
receive_stmt = "receive" [ "wait" [ duration ] ] "{" { receive_arm } "}" ;
duration = int_literal ( "ms" | "s" | "m" | "h" | "d" ) ;
receive_arm = pattern [ "if" expr ] "->" "{" { workflow_stmt } "}" ;
ret_stmt = "ret" expr ";" ;
send_stmt = "send" expr [ "to" expr ] ";" ;
set_stmt = "set" identifier "=" expr ";" ;
with_stmt = "with" expr "do" "{" { workflow_stmt } "}" ;
oblige_stmt = "oblige" obligation_ref ";" ;
yield_stmt = "yield" "{" { yield_arm } "}" ;
yield_arm = pattern "->" "{" { workflow_stmt } "}" ;
```

## 7. Declarations and Definitions

### 7.1 Type Definitions

```ebnf
type_definition = "type" identifier [ type_params ] "=" type_body ";" ;
type_body = alias_body | enum_body | struct_body | record_type | tuple_type ;
alias_body = type ;
enum_body = "{" variant_def { "|" variant_def } "}" ;
variant_def = identifier [ variant_payload_type ] ;
variant_payload_type = record_type | tuple_type | identifier ;
struct_body = "{" [ field_def { "," field_def } [","] ] "}" ;
field_def = identifier ":" type ;
```

### 7.2 Capability Definitions

```ebnf
capability_definition = "capability" identifier "{" { capability_operation } "}" ;
capability_operation = identifier "(" [ parameter_list ] ")" [ "->" type ] [ operation_mode ] ";" ;
operation_mode = "read" | "write" | "execute" | "observe" | "analyze" | "control" ;

capability_interface_definition = "interface" identifier [ type_params ] "{" { interface_method } "}" ;
interface_method = identifier "(" [ parameter_list ] ")" [ "->" type ] ";" ;

capability_implementation_definition = "impl" identifier "for" identifier "{" { impl_method } "}" ;
impl_method = "fn" identifier "(" [ parameter_list ] ")" [ "->" type ] fn_body ;
```

### 7.3 Role Definitions

```ebnf
role_definition = "role" identifier [ "(" parameter_list ")" ] "{" { role_clause } "}" ;
role_clause = capability_ref ";" | obligation_ref ";" ;
```

### 7.4 Law and Proof Definitions

```ebnf
law_definition = "law" identifier [ parameter_list ] [ "->" type ] "by" proof_kind ";" ;
proof_definition = "proof" identifier [ parameter_list ] [ "->" type ] "by" proof_kind ";" ;
proof_kind = "test" | "property" | "quickcheck" | "small_world" | "definition" | "authored" | identifier ;
```

### 7.5 Proposition Definitions

```ebnf
proposition_definition = "prop" identifier [ parameter_list ] [ "->" type ] "{" { proposition_clause } "}" ;
proposition_clause = identifier "(" [ expr { "," expr } ] ")" ";" ;
```

### 7.6 Use Declarations

```ebnf
use_decl = "use" use_path ";" ;
use_path = simple_path | simple_path "::" "*" | simple_path "::" "{" use_item { "," use_item } [","] "}" ;
use_item = identifier [ "as" identifier ] ;
simple_path = identifier { "::" identifier } ;
```

### 7.7 Visibility

```ebnf
visibility = [ "pub" [ "(" restricted_body ")" ] ] ;
restricted_body = "crate" | "self" | "super" | path | "in" path ;
```

## 8. Policy Expressions

```ebnf
policy_expr = policy_or ;
policy_or = policy_and { "||" policy_and } ;
policy_and = policy_seq { "&&" policy_seq } ;
policy_seq = policy_unary [ ";" policy_unary ] ;
policy_unary = [ "!" | "forall" identifier [ ":" type ] | "exists" identifier [ ":" type ] ] policy_primary ;
policy_primary = identifier | policy_call | "(" policy_expr ")" | policy_method_chain ;
policy_call = identifier "(" [ expr { "," expr } ] ")" ;
policy_method_chain = policy_primary "." identifier [ "(" [ argument_list ] ")" ] ;
```

## 9. Known Deviations and Limitations

### 9.1 Parser Accepts More Than Specified

- The parser accepts `fn` expressions in more contexts than documented.
- Record shorthand patterns (`Cons { head, tail }`) are partially supported.
- Some reserved callable arrows (`-*>`, `=>`, `=*>`) are rejected by the parser but not consistently.

### 9.2 Specified But Not Implemented

- Deep destructuring (`let { a: { b } } = nested`) is not supported.
- Destructuring in workflow blocks (`observe`, `act`) is limited to simple `let name = value`.
- Arrow syntax in `fn` expressions (`fn(x) => expr`) is not supported.
- Pattern guards in `let` are not supported.

### 9.3 Reserved But Not Active

- Higher-stratum callable arrows: `-*>`, `=>`, `=*>`
- `extern fn` syntax
- Cross-stratum closure serialization

### 9.4 Language-Evolution Surfaces Not Owned Here

The following are future syntax targets defined in the goal-state specs. They are
**not** accepted by the current parser:

- effect rows such as `{cap fs.read, policy production_rate | r}`;
- transparent effect aliases and diagnostic groups;
- role effects and role-to-capability entailment syntax;
- policy effects as named policy-handler requirements;
- channel send/receive/select effects with guards as contracts;
- general user-defined effect-handler syntax.

## 10. Statistics

| Metric | Value |
|--------|-------|
| Parser source files | 15 |
| Parser functions | 327 |
| Keywords | 99 |
| Punctuation tokens | 37 |
| Expression precedence levels | 9 |
| Workflow statement types | 28 |
| Pattern types | 7 |
| Type constructs | 6 |

## 11. See Also

- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md) — future surface syntax direction
- [SPEC-002: Surface Language](SPEC-002-SURFACE.md) — older surface spec
- [SPEC-009: Modules](SPEC-009-MODULES.md)
- [SPEC-027: Pure Functions](SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-031: First-Class Functions](SPEC-031-FIRST-CLASS-FUNCTIONS.md)
- [SPEC-072: Tower Callable Type and Closure Syntax](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [SPEC-091: Let Destructors](SPEC-091-LET-DESTRUCTORS.md)

## 12. Changelog

- 2026-06-18: Split from combined SPEC-095 into current-state document. Frozen against `e61f2792`. Added scope note and cross-reference to target grammar.
- 2026-06-17: Initial draft.
