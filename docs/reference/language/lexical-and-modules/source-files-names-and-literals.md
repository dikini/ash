---
id: language.reference.lexical.source-files-names-and-literals
title: Source Files, Names, Comments, and Literals
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-parser/tests/**"]
---

# Source Files, Names, Comments, and Literals

[Lexical and modules index](index.md) · [Modules and imports](modules-imports-and-visibility.md) ·
[Notation and macros](notation-and-expression-macros.md) · [Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Complete source file and comment trivia | accepted | not-applicable | not-applicable | not-applicable | implemented | tested | not-applicable |
| Ordinary identifiers | accepted | partial | not-applicable | not-applicable | implemented | tested | not-applicable |
| Expression literals | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |

Identifier static behavior is `partial` because the surrounding context decides whether a name
resolves. Literal static behavior is likewise `partial`, and literal lowering is `bounded-only`
because their enclosing expression owns those routes. No standalone lexical runtime route is
claimed.

The current complete-file route is
`crates/ash-parser/src/lib.rs::parse_surface_file`, which calls
`crates/ash-parser/src/parse_module.rs::module_file` and returns a
`surface::ModuleFile` with a comment table. The primary source evidence is
`crates/ash-parser/src/{lib.rs,parse_module.rs,parse_utils.rs,parse_expr.rs}`.
Focused test evidence is `crates/ash-parser/tests/comment_syntax.rs`, the
`test_parse_surface_file_populates_comment_table` unit test in
`crates/ash-parser/src/lib.rs`, and the literal-parser tests in
`crates/ash-parser/src/parse_expr/tests.rs`.

This page owns AUDIT-206 LANG-001's lexical portion. It makes no claim that a parsed file, name,
or literal is a runnable program; entry/admission evidence belongs to TASK-2052.

## What it is and how to use it

An Ash source file is parsed as a sequence of module declarations and top-level definitions with
whitespace and comment trivia skipped between them. `parse_surface_file` is the supported whole
file route: it records comments for tooling as well as parsing the structure. A file has no
implicit execution meaning merely because it parses.

For ordinary source identifiers, the first character is an ASCII letter or `_`; later characters
can also be ASCII digits or `-`. Reserved words cannot be ordinary identifiers. Individual grammar
positions can be narrower: for example, the direct `use` parser uses a path-segment parser that
does not admit `-`; see [Modules, imports, and visibility](modules-imports-and-visibility.md).
Visibility and import paths likewise use route-specific path segments, whose implementation admits
ASCII alphanumeric characters and `_` rather than applying the ordinary-identifier first-character
rule.

The expression parser accepts integer, decimal float, string, boolean, `null`, and nested literal
list forms. Use them only where the expression grammar permits them. Their typing, lowering, and
execution depend on their enclosing form and are documented by the forms and execution tasks, not
by this lexical page.

Comments are trivia, not expressions or declarations. Line comments begin with `--` or `//`;
block comments use `/*` and `*/` and the shared trivia skipper supports nesting. The parser stores
the skipped text and spans in `CommentTable`; comments do not create runtime behavior.

## Examples

**Parser-only source-file and comment example.** This is covered by
`crates/ash-parser/tests/comment_syntax.rs`: comments are skipped and retained as trivia, while
the function is merely parsed here. It is not an execution example.

```ash
-- a line comment
/* a block comment */
fn sample() -> Int { 1 }
```

**Expression-literal syntax example.** `1`, `3.5`, `"text"`, `true`, `false`, `null`, and
`[1, 2]` are parser-accepted literal shapes in expression positions. This is parser evidence only;
whether a surrounding function checks or runs is outside this page.

```ash
fn literal_shapes() -> Int { 1 }
```

The second snippet deliberately avoids claiming that every listed literal has the shown return
type. It provides an accepted enclosing expression position, not a type or runtime tutorial.

## Syntax

This EBNF is a compact description of the accepted lexical forms. `definition` and `expression`
refer to the dedicated form grammars; they are not expanded here. `comment_text` and
`string_source_text` mean the corresponding parser-recognized source text and are explained in
prose, not regular-expression syntax.

The string parser begins and ends a string with the source double-quote character. Railroad EBNF
does not offer an escaped double-quote terminal that would faithfully display that delimiter, so
`string_source_text` is deliberately an abstract nonterminal below instead of the misleading
literal words `double quote`.

```ebnf
source_file = { source_item } ;
source_item = module_declaration | definition ;
module_declaration = [ visibility ] "mod" identifier ( ";" | "{" { definition } "}" ) ;
visibility = "pub" | "pub" "(" visibility_scope ")" ;
visibility_scope = "crate" | "super" | "self" | "in" visibility_path ;
visibility_path = path_segment { "::" path_segment } ;
path_segment = ( ascii_alphanumeric | "_" ) { ascii_alphanumeric | "_" } ;
identifier = identifier_start { identifier_continue } ;
identifier_start = ascii_letter | "_" ;
identifier_continue = ascii_letter | ascii_digit | "_" | "-" ;
comment = line_comment | block_comment ;
line_comment = "--" comment_text | "//" comment_text ;
block_comment = "/*" comment_text "*/" ;
literal = integer_literal | float_literal | string_literal | boolean_literal | null_literal | list_literal ;
integer_literal = decimal_digit { decimal_digit } ;
float_literal = integer_literal "." integer_literal ;
string_literal = string_source_text ;
boolean_literal = "true" | "false" ;
null_literal = "null" ;
list_literal = "[" [ literal { "," literal } [ "," ] ] "]" ;
```

### Reading the rules

- `source_file` is the complete-file parser input. It accepts zero or more module items, with
  whitespace and comments skipped between items.
- `source_item` divides those items into module declarations and the other definitions documented
  in their owning chapters.
- `module_declaration` names either a file-based module, which ends with `;`, or an inline module,
  which contains definitions between `{` and `}`. The parser accepts only the definition forms it
  recognizes in that position.
- `visibility` is the optional `pub` modifier on a module declaration. `visibility_scope` gives
  the four restricted forms allowed inside `pub(...)`.
- `visibility_path` is a `::`-separated sequence of path segments used only by `pub(in ...)`.
  `path_segment` uses the restricted-visibility parser's ASCII-alphanumeric-or-underscore rule,
  which differs from an ordinary identifier.
- `identifier` describes ordinary source names. `identifier_start` gives the first-character rule;
  `identifier_continue` gives the characters that may follow. The parser rejects reserved words
  after matching this shape.
- `comment` covers source trivia. `line_comment` runs from either line-comment prefix to the end
  of its line. `block_comment` uses `/*` and `*/`; the trivia skipper also tracks nested block
  comments.
- `literal` selects one expression-literal form. `integer_literal` and `float_literal` describe
  decimal numbers; a float needs digits on both sides of `.`.
- `string_literal` names the quote-delimited form represented by `string_source_text`. The current
  parser reads characters up to the next source double quote; this grammar does not promise escape
  sequences.
- `boolean_literal` and `null_literal` are the three reserved literal words.
- `list_literal` is a bracketed, comma-separated list of literals. It may be empty and may end
  with a trailing comma; its elements may themselves be lists.

## What the parser keeps

There is no implemented standalone lexical typing or transition rule to state as a sequent. The
implemented behavior is a parser route: source is accepted into `ModuleFile`, comment trivia is
recorded, and later layers consume particular declarations or expressions. The literal parser is
`parse_expr::literal`; it does not itself prove a static or runtime result for an enclosing
expression.

## Errors and limits

- `parse_surface_file` rejects a file when `module_file` cannot consume a supported module item;
  an AST carrier elsewhere is not evidence of accepted file syntax.
- A keyword is not an ordinary identifier. Some narrowly scoped parser helpers admit contextual
  callable names; that is not a general escape from the reserved-word rule.
- The documented string parser is quote-delimited; this page does not promise a general escape
  language beyond the source implementation.
- Comments are preserved as parser trivia, not as typechecking, authority, lowering, or runtime
  instructions.
- `workflow` is removed and excluded from current examples.

## Related evidence

- [AUDIT-206 LANG-001](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [Modules, imports, and visibility](modules-imports-and-visibility.md)
- [TASK-2047: forms and expressions](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- `cargo test -p ash-parser --test comment_syntax`
