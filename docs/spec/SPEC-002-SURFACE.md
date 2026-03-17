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
              | "observe" | "orient" | "propose" | "decide" | "act"
              | "oblige" | "check" | "let" | "in" | "if" | "then" | "else"
              | "for" | "do" | "par" | "with" | "maybe" | "must"
              | "attempt" | "retry" | "timeout" | "done"
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
```

### 3.5 Workflow Definition

```
workflow_def    ::= "workflow" IDENTIFIER "{" workflow "}"

workflow        ::= workflow_stmt (";" workflow_stmt)* ";"? "done"?

workflow_stmt   ::= observe_stmt | orient_stmt | propose_stmt
                  | decide_stmt | check_stmt | act_stmt
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
                    ("under" IDENTIFIER)?
                    "then" workflow
                    ("else" workflow)?

check_stmt      ::= "check" obligation_ref ("then" workflow)?

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

literal         ::= STRING | NUMBER | BOOL | NULL
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

### 4.3 Implicit Done

```
workflow foo { act send_email(...) }

-- Equivalent to:
workflow foo { act send_email(...); done }
```

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
