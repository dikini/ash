# SPEC-039: Parser Tooling Infrastructure

## Status: Draft

## 1. Goal

Add the missing source-span and comment-trivia infrastructure to the Ash parser so that downstream tools (LSP, formatter, linter) can operate on precise locations and preserve user comments.

## 2. Scope

This spec covers two independent but related tracks:
1. **Binding spans** — Add `span` to `Expr::Variable` and `Pattern::Variable`.
2. **Comment trivia** — Preserve comments during lexing and make them retrievable for any AST node.

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
```

### 3.2 Required Change

Change `Variable(Name)` to `Variable(Name, Span)` in both `surface.rs` and `ast.rs`:

```rust
pub enum Expr {
    Literal(Literal),
    Variable(Name, Span),
    Call { ... },
    ...
}

pub enum Pattern {
    Wildcard,
    Variable(Name, Span),
    Literal(Literal),
    ...
}
```

### 3.3 Call Sites to Update

- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
- `crates/ash-parser/src/parse_pattern.rs` — pattern parsing
- `crates/ash-parser/src/lower.rs` — lower `Expr::Variable` and `Pattern::Variable`
- `crates/ash-typeck/src/check_expr.rs` — match arms for `Expr::Variable`
- `crates/ash-typeck/src/check_pattern.rs` — match arms for `Pattern::Variable`
- `crates/ash-typeck/src/names.rs` — any `Expr::Variable` destructuring
- `crates/ash-typeck/src/purity.rs` — any `Expr::Variable` destructuring
- `crates/ash-interp/src/eval.rs` — evaluation of `Expr::Variable` and `Pattern::Variable`
- `crates/ash-repl/src/ast.rs` — display/rendering of `Expr::Variable`
- All test files that construct `Expr::Variable` or `Pattern::Variable`

### 3.4 Migration Strategy

Because `Expr` and `Pattern` are widely matched, the change must be mechanical:
1. Update the enum definitions.
2. Fix the parser to capture `current_span()` when parsing identifiers.
3. Fix lowering to thread the span through.
4. Fix every match site.
5. Fix every test constructor.

## 4. Comment Trivia

### 4.1 Current State

The lexer discards comments in `skip_whitespace_and_comments()`:

```rust
fn skip_whitespace_and_comments(input: &mut Input) {
    // skips whitespace AND comments indiscriminately
}
```

There is no `Comment` token kind, and AST nodes do not carry comment data.

### 4.2 Design: Comment Side-Table

Rather than bloating every AST node with comment fields, store comments in a **side-table** keyed by the span of the token/declaration they precede or follow.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub text: String,
    pub kind: CommentKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentKind {
    Line,   // -- ...
    Block,  // /* ... */ (if supported in future)
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

### 4.2 Lexer Changes

Introduce a `Comment` token kind:

```rust
pub enum TokenKind {
    // ... existing kinds
    Comment(CommentKind, String),
}
```

The lexer's main tokenization loop should emit `Comment` tokens instead of skipping them. A higher-level filter (e.g., `filter_comments(tokens) -> Vec<Token>`) can strip comments when only semantic tokens are needed (e.g., for the interpreter).

> **Scope note:** `skip_whitespace_and_comments()` is currently used in many parser sub-modules (`parse_expr.rs`, `parse_pattern.rs`, `parse_policy.rs`, `parse_set.rs`, `parse_send.rs`, etc.). Emitting `Comment` tokens requires either removing all manual `skip_whitespace_and_comments` calls and driving the parser from a single token stream, or teaching every parser sub-module to skip `Comment` tokens in addition to whitespace. This is a parser-wide cleanup.

### 4.3 Comment Classification Heuristic

The post-lex pass assigns each `Comment` token to a non-comment token's span using the following rules:

1. **Trailing comment:** The comment appears on the same line as the preceding non-comment token, and there is no blank line between them. It is attached to the preceding token's span.
2. **Leading comment:** The comment appears on a line before the next non-comment token, or there is a blank line between the comment and the preceding token. It is attached to the next token's span.
3. **End-of-file comment:** A comment with no subsequent non-comment token is attached as a trailing comment of the last token in the file.

### 4.4 AST Integration

Add a `comments: CommentTable` field to `ModuleFile`:

```rust
pub struct ModuleFile {
    pub definitions: Vec<Definition>,
    pub comments: CommentTable,
    pub span: Span,
}
```

The comment table is populated during parsing by a post-lex pass that walks the token stream and assigns each `Comment` token according to the heuristic in §4.3.

### 4.5 `parse_module` Entry Point

Add a top-level public API to the parser crate:

```rust
pub fn parse_module(source: &str) -> Result<ash_parser::surface::ModuleFile, Vec<ash_parser::error::ParseError>>
```

This function lexes the input, builds the `CommentTable`, parses the module, and returns the fully populated `ModuleFile`.

### 4.6 Formatter Integration

The formatter (SPEC-042) will query `module.comments.leading_comments(span)` before emitting any declaration or expression, and insert the comment text verbatim.

### 4.7 LSP Integration

For hover and diagnostics, comment trivia is **not** required in the MVP. The side-table is primarily for the formatter and for future "generate documentation from comments" features.

## 5. Dependencies

None beyond the existing parser crate.

## 6. Testing Strategy

1. **Binding spans:** Property tests asserting that every parsed `Variable` carries a non-default span with correct line/column.
2. **Comment trivia:** Tests that lexing a file with comments produces `Comment` tokens; tests that the comment table correctly maps comments to tokens using the heuristic in §4.3.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (via binding spans), SPEC-041 (via `parse_module` API), SPEC-042 (via comment trivia), SPEC-043 (via `parse_module` API)
- **Blocked by:** None
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete

## 8. Affected Files Reference

The binding-span change (`Variable(Name, Span)`) affects:

- `crates/ash-parser/src/surface.rs` — enum definition
- `crates/ash-core/src/ast.rs` — enum definition
- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
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
- All test files that construct `Expr::Variable` or `Pattern::Variable`
