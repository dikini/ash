---
id: language.reference.lexical.notation-and-expression-macros
title: Notation, Expression Macros, and Operator Sections
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/**", "crates/ash-parser/tests/**"]
---

# Notation, Expression Macros, and Operator Sections

[Lexical and modules index](index.md) · [Source files and literals](source-files-names-and-literals.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Notation declaration and local table | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Expression macro declaration and invocation | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Parenthesized operator section | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |

Macro static summaries and operator-section resolution are both `partial`. A macro has no
independent admission/runtime route on this evidence. Operator sections are `bounded-only` for
lowering because elaboration must first remove the raw carrier; a raw section cannot cross the
lowering gate.

Primary evidence is `crates/ash-parser/src/{parse_module.rs,parse_expr.rs,surface.rs,lower.rs}`.
Focused tests are:

- `crates/ash-parser/tests/task_1730_notation_declaration_parser_ast.rs`
- `crates/ash-parser/tests/task_1732_local_notation_table_resolution.rs`
- `crates/ash-parser/tests/task_1754_macro_declaration_parse.rs`
- `crates/ash-parser/tests/task_1758_macro_lowering_boundaries.rs`
- `crates/ash-parser/tests/task_1768_binder_hygiene_metadata.rs`
- `crates/ash-parser/tests/task_1769_hygienic_binder_macros.rs`
- `crates/ash-parser/tests/task_1724_operator_section_boundary.rs`
- `crates/ash-parser/tests/task_1733_operator_section_elaboration.rs`

This page owns AUDIT-206 LANG-003 and LANG-024. It makes no generic runtime claim for notation,
macros, or sections.

## What it is and how to use it

A notation declaration maps a source pattern and fixity to a callable path for later local
notation-table construction and expansion. `prefix` and `suffix` may carry a precedence;
`infixl`, `infixr`, and `infix` require one; `mixfix` preserves its source pattern. A declaration
can be marked `pub`, but the tested local-table behavior still establishes only a local syntax
entry, not runtime callability or authority.

An expression macro has the form `macro name(parameters) => expression;`. Parameters may carry
source type annotations, which become syntax-phase signature summaries. An invocation is
unqualified and has the form `name!(...)`, `name![...]`, or `name!{...}`. Every delimiter form
preserves a token-tree body. A parenthesized body that completely parses as a comma-separated
expression list also receives the structured expression-argument carrier; this is the tested
expression-argument subset. Bracket and brace bodies have no such structured-argument carrier and
remain syntax-phase diagnostic carriers unless a bounded expansion route proves more.

Macro expansion is required before the expanded-surface lowering boundary. A missing local macro
and an unexpanded macro carrier are rejected before Core lowering. Macro declaration identity and
hygiene metadata are intentionally separate from callable identity: they do not grant effects,
provider authority, contracts, proof evidence, or runtime callability.

The currently evidenced hygiene behavior is deliberately narrow. Expansion records syntax-side
definition-site, call-site, and generated-binder metadata. The focused tests show binder renaming
for selected anonymous-function macro templates and that generated binders do not capture a
call-site argument. A macro template whose body contains an unsupported block binder fails
closed. This is hygiene/expansion evidence, not a claim that every expression-template binder is
supported or that hygiene metadata has runtime authority.

An operator section is a parenthesized partial operator expression: `(<op>)`, `(left <op>)`, or
`(<op> right)`. The parser preserves it as an `OperatorSection`; it must be elaborated into a
function/target expression before lowering. Built-in and declared local notation examples are
tested. An unresolved operator section fails closed rather than lowering as a raw section.

## Examples

**Parser-accepted notation declaration.** This is parser evidence from
`task_1730_notation_declaration_parser_ast.rs`; it does not establish runtime callability for
`combine`.

```ash
infixl 6 <+> = combine
```

**Non-copyable expression fragment fed to the local-table elaborator.** `(x <+>)` is the
parser-and-elaboration-only section shape exercised by
`task_1733_operator_section_elaboration.rs`. It is not a source file: the test supplies it in a
module with a local notation declaration before it builds the local table. It is neither a static
checking nor execution example.

```ash
(x <+>)
```

**Supported parenthesized macro subset.** This is a bounded syntax/lowering-only example from
`task_1758_macro_lowering_boundaries.rs`; the macro expands before the Core boundary only after a
matching local declaration is found. It is not a complete static-checking or runtime example.

```ash
macro inc(x) => add(x, 1);
fn add(x: Int, y: Int) -> Int { x + y }
fn use_macro(n: Int) -> Int { inc!(n) }
```

**Diagnostic token-tree carriers.** `inc![n]` and `inc!{n}` parse and preserve delimiter/token
tree shape, but the parser test labels them non-executable diagnostic carriers. Do not use them as
general macro execution examples.

```ash
fn inspect(n: Int) -> Int { inc![n] }
```

**Built-in operator section.** `(+)` is a parser-and-elaboration-only expression fragment. The
focused test parses it directly and elaborates it to an eta-expanded function. It is deliberately
not placed under a declared `Int` return type: this page does not establish a complete static form
for that generated function, and it has no independent admitted-runtime claim.

```ash
(+)
```

## Syntax

`notation_pattern`, `operator`, `expression`, and `token_tree_content` are source-preserving
parser domains whose detailed character rules are implemented in the named parser functions. The
EBNF deliberately avoids pretending that a display grammar validates their full contents.

```ebnf
notation_declaration = [ visibility ] notation_fixity notation_pattern "=" callable_path [ ";" ] ;
notation_fixity = "prefix" [ precedence ] | "suffix" [ precedence ] | "infixl" precedence | "infixr" precedence | "infix" precedence | "mixfix" ;
precedence = decimal_number ;
callable_path = callable_path_segment [ "::" callable_path_segment ] ;
callable_path_segment = ( ascii_alphanumeric | "_" ) { ascii_alphanumeric | "_" } ;
macro_declaration = [ visibility ] "macro" identifier "(" [ macro_parameter { "," macro_parameter } ] ")" [ "->" type ] "=>" expression ";" ;
macro_parameter = identifier [ ":" type ] ;
macro_invocation = identifier "!" macro_invocation_body ;
macro_invocation_body = parenthesized_token_tree | bracketed_token_tree | braced_token_tree ;
parenthesized_token_tree = "(" token_tree_content ")" ;
bracketed_token_tree = "[" token_tree_content "]" ;
braced_token_tree = "{" token_tree_content "}" ;
structured_expression_arguments = "(" [ expression { "," expression } ] ")" ;
operator_section = "(" operator ")" | "(" section_operand operator ")" | "(" operator section_operand ")" ;
```

### Reading the rules

- `notation_declaration` binds a source pattern to a callable path. It may start with `pub` and
  may end with `;`; the parser stores the chosen fixity, pattern, and target for local notation
  processing. `notation_pattern` is that source-preserved pattern, rather than a second grammar
  for its operator characters.
- `notation_fixity` selects how the pattern associates. `prefix` and `suffix` may omit their
  precedence, while `infixl`, `infixr`, and `infix` require one. `mixfix` has no precedence slot
  in this surface form.
- `precedence` is an unsigned decimal number. The parser stores it in a bounded numeric field, so
  a decimal spelling outside that field is rejected.
- `callable_path` is the notation target accepted by this grammar: one callable-path segment or
  two segments joined by `::`. `callable_path_segment` uses the parser's separate
  ASCII-alphanumeric-or-underscore rule, so it is not an ordinary identifier.
- `macro_declaration` defines an expression macro. It has a name, an optional parameter list and
  return-type summary, `=>`, an expression template, and a required `;`.
- `macro_parameter` names one macro parameter and may add a source type. These annotations form a
  syntax-phase summary; they do not typecheck an invocation by themselves.
- `macro_invocation` is an unqualified macro name followed by `!` and one delimited body.
  `macro_invocation_body` chooses parentheses, brackets, or braces.
- `parenthesized_token_tree`, `bracketed_token_tree`, and `braced_token_tree` preserve the body in
  the delimiter form used at the call site. `token_tree_content` stays abstract because the parser
  preserves nested token trees rather than imposing a display grammar for their contents.
- `structured_expression_arguments` is the narrower parenthesized case that also parses as a
  complete comma-separated expression list. Bracket and brace bodies do not use this carrier.
- `operator_section` describes a parenthesized operator with neither operand, only a left operand,
  or only a right operand. `operator` and `section_operand` are parser domains: the parser must
  recognize them before elaboration can turn the section into an expression.

## What the elaborator does

There is no implementation-backed source sequent for these syntax-phase mechanisms, so none is
invented here. The relevant checked transition is procedural:

1. Parse notation/macro/section source into preserved surface carriers.
2. Build the local notation table and expand local macro/section carriers.
3. Permit Core lowering only after the expanded-surface gate has no raw macro invocation or
   operator-section carrier.

The tests show that duplicate/conflicting notation and unresolved sections fail during expansion;
unknown macros and raw macro carriers fail at the lowering boundary. This boundary is not a
runtime evaluator and does not prove client parity.

## Errors and limits

- Qualified macro invocations such as `macros::inc!(n)` are rejected by the current MVP parser.
- A macro invocation with no matching local macro is rejected before lowering.
- Bracket and brace macro bodies are parser-preserved token trees, not a blanket executable macro
  form.
- Hygiene is syntax-side metadata. Selected generated function binders are renamed to avoid
  capture, while unsupported block-binder templates reject during expansion.
- Duplicate/conflicting local notation declarations fail when the local notation table is built;
  notation does not leak between parent and inline-module tables in the focused tests.
- `(_ + _)` is not a supported generalized mixfix section. Unresolved operators fail elaboration
  and no raw section may cross the lowering boundary.
- No notation, macro summary, expansion origin, or hygiene record grants provider/resource/role
  authority or runtime callability.
- Workflow/tower forms remain excluded.

## Related evidence

- [AUDIT-206 LANG-003 and LANG-024](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2047: function and expression forms](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`
- `cargo test -p ash-parser --test task_1754_macro_declaration_parse`
- `cargo test -p ash-parser --test task_1758_macro_lowering_boundaries`
- `cargo test -p ash-parser --test task_1724_operator_section_boundary`
- `cargo test -p ash-parser --test task_1733_operator_section_elaboration`
