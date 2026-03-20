# SPEC-002: Surface Language

## Status: Draft

## 1. Overview

The surface language is designed to be:
- **Readable** by non-programmers (policy officers, auditors)
- **Writable** by LLMs (clear structure, predictable patterns)
- **Translatable** to IR for execution

## 2. Lexical Structure

### 2.1 Tokens

```
KEYWORD     ::= "workflow" | "capability" | "policy" | "role"
              | "observe" | "receive" | "orient" | "propose" | "decide" | "act"
              | "oblige" | "check" | "let" | "in" | "if" | "then" | "else"
              | "for" | "do" | "par" | "with" | "maybe" | "must"
              | "wait" | "control"
              | "exposes"
              | "timeout" | "done"
              | "epistemic" | "deliberative" | "evaluative" | "operational"
              | "authority" | "obligations" | "supervises"
              | "when" | "returns" | "where"

IDENTIFIER  ::= [a-zA-Z_][a-zA-Z0-9_-]*

STRING      ::= "\"" [^"]* "\""
NUMBER      ::= [0-9]+ ("." [0-9]+)?
BOOL        ::= "true" | "false"
NULL        ::= "null"

OPERATOR    ::= "+" | "-" | "*" | "/" | "=" | "!=" | "<" | ">" | "<=" | ">="
              | "and" | "or" | "not" | "in"

DELIMITER   ::= "(" | ")" | "{" | "}" | "[" | "]" | "," | ";" | ":" | "." | ".."
```

### 2.2 Comments

```
LINE_COMMENT    ::= "--" [^\n]*
BLOCK_COMMENT   ::= "/*" (!"*/" .)* "*/"
DOC_COMMENT     ::= "-- |" [^\n]*  (Documentation)
```

## 3. Grammar

### 3.1 Program Structure

```
program     ::= definition* workflow_def

definition  ::= capability_def | policy_def | role_def 
              | memory_def | datatype_def
```

### 3.2 Capability Definition

```
capability_def  ::= "capability" IDENTIFIER ":" effect_type
                    "(" param_list? ")"
                    ("returns" type)?
                    constraint_list?

effect_type     ::= "observe" | "read" | "analyze" | "decide" 
                  | "act" | "write" | "external"

param_list      ::= param ("," param)*
param           ::= IDENTIFIER ":" type

constraint_list ::= "where" constraint ("," constraint)*
constraint      ::= predicate
```

### 3.3 Policy Definition

```
policy_def  ::= "policy" IDENTIFIER ":"
                "when" expression
                "then" decision

decision    ::= "permit" | "deny" 
              | "require_approval" "(" "role:" IDENTIFIER ")"
              | "escalate"
```

### 3.4 Role Definition

```
role_def    ::= "role" IDENTIFIER "{"
                authority_clause
                obligations_clause?
                supervises_clause?
                "}"

authority_clause    ::= "authority:" "[" capability_ref* "]"
obligations_clause  ::= "obligations:" "[" obligation_ref* "]"
supervises_clause   ::= "supervises:" "[" IDENTIFIER* "]"

obligation_ref      ::= IDENTIFIER "must" predicate
                      | IDENTIFIER "may" action_ref
                      | IDENTIFIER "must-not" action_ref

workflow_obligation_ref ::= IDENTIFIER
```

### 3.5 Workflow Definition

```
workflow_def    ::= "workflow" IDENTIFIER workflow_clause* "{" workflow "}"

workflow_clause ::= observes_clause
                  | receives_clause
                  | sets_clause
                  | sends_clause
                  | exposes_clause

observes_clause ::= "observes" behaviour_ref ("," behaviour_ref)*
receives_clause ::= "receives" stream_ref ("," stream_ref)*
sets_clause     ::= "sets" settable_ref ("," settable_ref)*
sends_clause    ::= "sends" sendable_ref ("," sendable_ref)*
exposes_clause   ::= "exposes" "{" exposure_item ("," exposure_item)* "}"

exposure_item   ::= obligations_exposure
                  | behaviours_exposure
                  | values_exposure

obligations_exposure ::= "obligations:" "[" workflow_obligation_ref* "]"
behaviours_exposure   ::= "behaviours:" "[" behaviour_ref* "]"
values_exposure       ::= "values:" "[" IDENTIFIER* "]"

behaviour_ref   ::= capability_ref
settable_ref    ::= capability_ref
sendable_ref    ::= capability_ref
stream_ref      ::= IDENTIFIER (":" IDENTIFIER)?
                  | IDENTIFIER "{" IDENTIFIER ("," IDENTIFIER)+ "}"

workflow        ::= workflow_stmt (";" workflow_stmt)* ";"? "done"?

workflow_stmt   ::= observe_stmt | orient_stmt | propose_stmt
                  | decide_stmt | check_stmt | receive_stmt | act_stmt
                  | oblig_stmt | let_stmt | if_stmt
                  | for_stmt | par_stmt | with_stmt
                  | maybe_stmt | must_stmt

observe_stmt    ::= "observe" capability_ref ("as" pattern)? 
                    ("then" workflow)?

orient_stmt     ::= "orient" "{" expression "}" ("as" pattern)?
                    ("then" workflow)?

propose_stmt    ::= "propose" action_ref ("as" pattern)?
                    ("then" workflow)?

decide_stmt     ::= "decide" "{" expression "}" 
                    "under" IDENTIFIER
                    "then" workflow

check_stmt      ::= "check" workflow_obligation_ref ("then" workflow)?

receive_stmt    ::= "receive" ("control")? receive_mode?
                    "{" receive_arm ("," receive_arm)* "}"

receive_mode    ::= "wait" (duration)?

receive_arm     ::= receive_pattern ("if" expression)? "=>" workflow

receive_pattern ::= IDENTIFIER ":" IDENTIFIER "as" pattern
                  | STRING
                  | "_"

duration        ::= NUMBER ("ms" | "s" | "m" | "h")

act_stmt        ::= "act" action_ref ("where" guard)?

oblig_stmt      ::= "oblige" IDENTIFIER "to" check_ref
                    ("then" workflow)?

let_stmt        ::= "let" pattern "=" expression ("in" workflow)?

if_stmt         ::= "if" expression "then" workflow 
                    ("else" workflow)?

for_stmt        ::= "for" pattern "in" expression "do" workflow

par_stmt        ::= "par" "{" workflow ("|" workflow)* "}"

with_stmt       ::= "with" capability_ref "do" workflow

maybe_stmt      ::= "maybe" workflow "else" workflow

must_stmt       ::= "must" workflow
```

**Canonical workflow-form contracts**:
- `check` is reserved for obligation references. Policy instances are not valid `check` targets.
- `decide` is the policy gate, so `under <policy>` is required in the surface syntax.
- `receive` is the authoritative surface form for stream/mailbox intake in the core workflow language; neighboring specs should defer to this grammar when referring to workflow-level `receive`.
- Workflow clauses make input and output kinds explicit: `observes` declares behaviour inputs,
  `receives` declares stream inputs, and `sets` / `sends` declare output capabilities.
- `exposes` declares the externally monitorable workflow view. It does not imply control or
  messaging authority; it exposes only the named obligations, behaviours, and values.
- `workflow_obligation_ref` names a live workflow obligation state symbol exposed by the
  workflow. It is distinct from the role-level deontic `obligation_ref` syntax used inside
  `role_def`.
- `behaviour_ref`, `settable_ref`, and `sendable_ref` are intentionally distinct names even when
  they share the same token shape. The distinction is semantic: `observes` grants read access to
  behaviours, not write authority; write authority is declared separately with `sets` or `sends`.
- `exposure_item` is intentionally read-only. Monitor metadata such as `monitor_count` belongs in
  the exposed `values` set when it is meant to be visible.
- `if let` is surface sugar only. It is accepted for readability, but its canonical meaning is the
  same as a `match` with a wildcard fallback in the core language contract.
- Recoverable failures use explicit `Result` values and pattern matching for recoverable control
  flow.
- The current surface syntax does not yet standardize explicit `receive` scheduling syntax. Until
  it does, neighboring specs should use the terminology from
  [LANGUAGE-TERMINOLOGY](../design/LANGUAGE-TERMINOLOGY.md): the runtime implements a scheduler,
  and the current default behavior is the implicit `priority` source scheduling modifier defined
  in SPEC-013. No new receive scheduling syntax is introduced here.

### 3.6 Expressions

```
expression      ::= or_expr

or_expr         ::= and_expr ("or" and_expr)*
and_expr        ::= not_expr ("and" not_expr)*
not_expr        ::= "not" not_expr | comparison

comparison      ::= additive (("=" | "!=" | "<" | ">" | "<=" | ">=") additive)?
additive        ::= multiplicative (("+" | "-") multiplicative)*
multiplicative  ::= unary (("*" | "/") unary)*
unary           ::= ("-" | "#" | "not") unary | primary

primary         ::= literal
                  | IDENTIFIER
                  | "$" IDENTIFIER           -- Input reference
                  | primary "." IDENTIFIER   -- Field access
                  | primary "[" expression "]"  -- Index access
                  | primary "(" arguments ")"   -- Function call
                  | "(" expression ")"

arguments       ::= expression ("," expression)*

literal         ::= STRING | NUMBER | BOOL | NULL | list_literal

list_literal    ::= "[" (expression ("," expression)*)? "]"
```

### 3.7 Patterns

```
pattern         ::= IDENTIFIER
                  | "_"
                  | "(" pattern ("," pattern)* ")"
                  | "{" field_pattern ("," field_pattern)* "}"
                  | "[" pattern ("," pattern)* (".." IDENTIFIER)? "]"
                  | literal

field_pattern   ::= IDENTIFIER (":" pattern)?
```

### 3.8 Guards

```
guard           ::= "always" | "never" | predicate
                  | guard "and" guard
                  | guard "or" guard
                  | "not" guard
                  | "(" guard ")"

predicate       ::= IDENTIFIER "(" arguments? ")"
```

## 4. Semantic Sugar

### 4.1 Sequential Composition

```
w1; w2; w3
then done

-- Desugars to:
Seq(w1, Seq(w2, Seq(w3, Done)))
```

### 4.2 Optional Binding

```
observe cap as x
-- vs
observe cap  (continuation has no binding)
```

This is surface sugar only. The canonical core contract still has an explicit binding position for
the observation result; omitting the name in surface syntax does not create a new workflow form.
Lowering supplies the wildcard-style binding internally.

### 4.3 Implicit Done

```
workflow foo { act send_email(...) }

-- Equivalent to:
workflow foo { act send_email(...); done }
```

This is surface sugar only. The canonical core contract still ends in an explicit `Done` workflow
form; omitting `done` in surface syntax does not add a separate completion construct or change the
core workflow-form set.

## 5. Error Recovery

The parser should recover from common errors:
- Missing semicolons (insert and continue)
- Unclosed braces (report and skip to next top-level)
- Unknown keywords (suggest closest match)

## 6. Example Programs

### 6.1 Simple Observation

```ash
workflow simple {
  observe read_file with path: "/tmp/data.txt" as content;
  orient { parse_json(content) } as data;
  act print(data.message);
}
```

### 6.2 With Policy

```ash
capability delete_file : act(path: String)
  where file_exists(path)

policy destructive_actions:
  when action == "delete_file" and not in_trash(path)
  then require_approval(role: admin)

workflow cleanup {
  observe list_files with pattern: "*.tmp" as files;
  for file in files do {
    decide { file.age > 7_days } under destructive_actions then {
      act delete_file(file.path);
    }
  };
}
```

## 7. Pretty Printing

The surface language has canonical formatting:
- Indentation: 2 spaces
- Line length: 80 characters
- One statement per line
- Align `then` branches

## 8. Related Documents

- SPEC-001: IR
- SPEC-003: Type System
