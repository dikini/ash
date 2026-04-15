# SPEC-039: Parser Tooling Infrastructure

## Status: Draft

## 1. Goal

Add the missing source-span and comment-trivia infrastructure to the Ash parser so that downstream tools (LSP, formatter, linter) can operate on precise locations and preserve user comments.

## 2. Scope

This spec covers two independent but related tracks:
1. **Binding spans** — Add `span` to `Expr::Variable`, `Pattern::Variable`, and `PolicyExpr::Var`.
2. **Comment trivia** — Preserve comments during parsing and make them retrievable for any AST node via a side-table.

## 3. Binding Spans

### 3.1 Current State

```rust
pub enum Expr {
    Literal(Literal),
    Variable(Name),          // no span
    Call { ... },
    ...
}

pub enum Pattern {
    Wildcard,
    Variable(Name),          // no span
    Literal(Literal),
    ...
}

pub enum PolicyExpr {
    Literal(Literal),
    Var(Name),               // no span
    ...
}
```

### 3.2 Required Change

Change to **struct variants** (consistent with every other spanned `Expr` / `Pattern` variant):

```rust
pub enum Expr {
    Literal(Literal),
    Variable { name: Name, span: Span },
    Call { ... },
    ...
}

pub enum Pattern {
    Wildcard,
    Variable { name: Name, span: Span },
    Literal(Literal),
    ...
}

pub enum PolicyExpr {
    Literal(Literal),
    Var { name: Name, span: Span },
    ...
}
```

### 3.3 Call Sites to Update

- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
- `crates/ash-parser/src/parse_pattern.rs` — pattern parsing
- `crates/ash-parser/src/parse_policy.rs` — policy variable parsing
- `crates/ash-parser/src/parse_module.rs` — `Expr::Variable` construction
- `crates/ash-parser/src/parse_send.rs` — `Expr::Variable` matches
- `crates/ash-parser/src/lower.rs` — lower `Expr::Variable`, `Pattern::Variable`, `PolicyExpr::Var`
- `crates/ash-typeck/src/check_expr.rs` — match arms for `Expr::Variable`
- `crates/ash-typeck/src/check_pattern.rs` — match arms for `Pattern::Variable`
- `crates/ash-typeck/src/names.rs` — any `Expr::Variable` destructuring
- `crates/ash-typeck/src/purity.rs` — any `Expr::Variable` destructuring
- `crates/ash-interp/src/eval.rs` — evaluation of `Expr::Variable` and `Pattern::Variable`
- `crates/ash-repl/src/ast.rs` — display/rendering of `Expr::Variable`
- `crates/ash-core/src/proptest_helpers.rs` — `Pattern::Variable` generation
- `crates/ash-fuzz/fuzz_targets/typeck.rs` — `Pattern::Variable` construction
- All test files that construct `Expr::Variable`, `Pattern::Variable`, or `PolicyExpr::Var`

### 3.4 Migration Strategy

Because `Expr` and `Pattern` are widely matched, the change must be mechanical:
1. Update the enum definitions.
2. Fix the parser to capture `current_span()` when parsing identifiers.
3. Fix lowering to thread the span through.
4. Fix every match site.
5. Fix every test constructor.

## 4. Comment Trivia

### 4.1 Current State

The parser discards comments in `skip_whitespace_and_comments()`:

```rust
fn skip_whitespace_and_comments(input: &mut Input) {
    // skips whitespace AND comments indiscriminately
}
```

There is no `Comment` token kind, and AST nodes do not carry comment data.

### 4.2 Design: Comment Side-Table (No Token-Stream Changes)

The Ash parser is string-slice / winnow-combinator based, not lexer-first. Emitting `Comment` tokens into a token stream would require a parser rewrite. Instead, comments are captured **inside the existing whitespace-skipping routine** and stored in a side-table keyed by the span of the next non-whitespace token.

```rust
use ash_parser::token::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub text: String,
    pub kind: CommentKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentKind {
    Line,   // -- ...
    Block,  // /* ... */ (already supported in the lexer)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommentTable {
    // Map from a token/node span to comments that immediately precede it.
    leading: HashMap<Span, Vec<Comment>>,
    // Map from a token/node span to comments that immediately follow it on the same line.
    trailing: HashMap<Span, Vec<Comment>>,
}

impl CommentTable {
    pub fn leading_comments(&self, span: Span) -> &[Comment] { ... }
    pub fn trailing_comments(&self, span: Span) -> &[Comment] { ... }
}
```

> **Span type:** `CommentTable` uses `ash_parser::token::Span` because it derives `Hash` and `Eq`. `ash_core::ast::Span` does **not** derive `Hash`; if the core AST ever needs to own a `CommentTable`, `Hash` must be added to `ast::Span` first.

### 4.3 Lexer Changes

Instead of discarding comments, `skip_whitespace_and_comments` (and any copies of it in parser sub-modules) appends discovered comments to a mutable `CommentTable` that is threaded through parsing:

```rust
pub fn skip_whitespace_and_comments(
    input: &mut Input,
    comments: &mut CommentTable,
) {
    // existing whitespace skipping
    // when a comment is encountered:
    //   1. Record it as a trailing comment of the previous non-whitespace token
    //      if it is on the same line.
    //   2. Otherwise, queue it as a leading comment of the next non-whitespace token.
}
```

> **Scope note:** `skip_whitespace_and_comments()` is currently used in many parser sub-modules (`parse_expr.rs`, `parse_pattern.rs`, `parse_policy.rs`, `parse_set.rs`, `parse_send.rs`, etc.). All copies must be updated to accept a `&mut CommentTable`. This is a parser-wide but mechanical cleanup.

### 4.4 Comment Classification Heuristic

The heuristic runs during whitespace skipping:

1. **Trailing comment:** The comment appears on the same line as the preceding non-comment token, and there is no blank line between them. It is attached to the preceding token's span.
2. **Leading comment:** The comment appears on a line before the next non-comment token, or there is a blank line between the comment and the preceding token. It is attached to the next token's span.
3. **End-of-file comment:** A comment with no subsequent non-comment token is attached as a trailing comment of the last token in the file.

> **Intra-expression comments:** Comments inside expressions (e.g., `foo(--c\n)(args)`) are **out of scope** for MVP. The side-table attaches comments to the nearest top-level or declaration span. Fine-grained expression-level comment attachment is deferred.

### 4.5 AST Integration

Add a `comments: CommentTable` field to `ModuleFile`:

```rust
pub struct ModuleFile {
    pub definitions: Vec<Definition>,
    pub module_decls: Vec<ModuleDecl>,
    pub workflow: Option<WorkflowDef>,
    pub comments: CommentTable,
    pub span: Span,
}
```

The comment table is populated during parsing by threading `&mut CommentTable` through all whitespace-skipping calls.

### 4.6 `parse_surface_file` Entry Point

Add a top-level public API to the parser crate:

```rust
pub fn parse_surface_file(source: &str)
    -> Result<ash_parser::surface::ModuleFile, Vec<ash_parser::error::ParseError>>
```

This function creates an empty `CommentTable`, parses the module with comment collection enabled, attaches the table to the `ModuleFile`, and returns it.

> **Naming:** The existing `parse_module` in `parse_module.rs` parses a single `Definition` from a module context. The new top-level entry point is named `parse_surface_file` to avoid collision.

### 4.7 Formatter Integration

The formatter (SPEC-042) will query `module.comments.leading_comments(span)` before emitting any declaration or expression, and insert the comment text verbatim.

### 4.8 LSP Integration

For hover and diagnostics, comment trivia is **not** required in the MVP. The side-table is primarily for the formatter and for future "generate documentation from comments" features.

## 5. Dependencies

None beyond the existing parser crate.

## 6. Testing Strategy

1. **Binding spans:** Property tests asserting that every parsed `Variable` and `Var` carries a non-default span with correct line/column.
2. **Comment trivia:** Tests that parsing a file with comments produces a non-empty `CommentTable`; tests that the classification heuristic in §4.4 correctly assigns leading and trailing comments.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (via binding spans), SPEC-041 (via `parse_surface_file` API), SPEC-042 (via comment trivia), SPEC-043 (via `parse_surface_file` API)
- **Blocked by:** None
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete

## 8. Affected Files Reference

The binding-span change (`Variable { name, span }` / `Var { name, span }`) affects:

- `crates/ash-parser/src/surface.rs` — enum definitions
- `crates/ash-core/src/ast.rs` — enum definitions
- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
- `crates/ash-parser/src/parse_policy.rs` — policy variable parsing
- `crates/ash-parser/src/parse_module.rs` — `Expr::Variable` construction
- `crates/ash-parser/src/parse_send.rs` — `Expr::Variable` matches
- `crates/ash-parser/src/parse_pattern.rs` — pattern parsing
- `crates/ash-parser/src/lower.rs` — lowering
- `crates/ash-typeck/src/check_expr.rs` — match arms
- `crates/ash-typeck/src/check_pattern.rs` — match arms
- `crates/ash-typeck/src/lib.rs` — multiple match sites
- `crates/ash-typeck/src/names.rs` — `resolve_expr` match
- `crates/ash-typeck/src/purity.rs` — match arms
- `crates/ash-interp/src/eval.rs` — evaluation and pattern destructuring
- `crates/ash-repl/src/ast.rs` — display/rendering
- `crates/ash-core/src/proptest_helpers.rs` — `Pattern::Variable` generation
- `crates/ash-fuzz/fuzz_targets/typeck.rs` — `Pattern::Variable` construction
- All test files that construct `Expr::Variable`, `Pattern::Variable`, or `PolicyExpr::Var`
