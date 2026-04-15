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
    Literal(Literal),          // span work explicitly deferred; see §3.5
    Variable { name: Name, span: Span },
    Call { ... },
    ...
}

pub enum Pattern {
    Wildcard,
    Variable { name: Name, span: Span },
    Literal(Literal),          // span work explicitly deferred; see §3.5
    ...
}

pub enum PolicyExpr {
    Literal(Literal),          // span work explicitly deferred; see §3.5
    Var { name: Name, span: Span },
    ...
}
```

> **Prerequisite (cross-reference):** `ash_core::ast::Span` must derive `Hash` and `Eq` before `CommentTable` can be used in the core AST. This is tracked as a prerequisite for TASK-571. See §4.2 for details.

### 3.3 Call Sites to Update

- `crates/ash-parser/src/surface.rs` — enum definitions
- `crates/ash-core/src/ast.rs` — enum definitions
- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
- `crates/ash-parser/src/parse_pattern.rs` — pattern parsing
- `crates/ash-parser/src/parse_policy.rs` — policy variable parsing
- `crates/ash-parser/src/parse_module.rs` — `Expr::Variable` construction
- `crates/ash-parser/src/parse_send.rs` — `Expr::Variable` matches
- `crates/ash-parser/src/lower.rs` — lower `Expr::Variable`, `Pattern::Variable`, `PolicyExpr::Var`
- `crates/ash-typeck/src/check_expr.rs` — match arms for `Expr::Variable`
- `crates/ash-typeck/src/check_pattern.rs` — match arms for `Pattern::Variable`
- `crates/ash-typeck/src/lib.rs` — multiple match sites
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

> **Literal spans deferred:** Adding `span` to `Literal` (and therefore `Expr::Literal`, `Pattern::Literal`, `PolicyExpr::Literal`) is **explicitly deferred** to avoid scope creep. Only `Variable` / `Var` variants receive spans in this spec.

### 3.5 Literal Span Deferral

Adding `span` to `Literal` (and therefore `Expr::Literal`, `Pattern::Literal`, and `PolicyExpr::Literal`) is **explicitly out of scope** for this spec. Reasons:
1. `Literal` is a separate type shared across surface and core AST; changing it ripples into the lexer, parser, and every crate that constructs literals.
2. Binding spans are the immediate blocker for LSP hover/Go-to-Definition on variables.
3. Formatter and linter needs for literal spans can be addressed in a follow-up spec.

When literal spans are eventually added, the same `current_span()` capture pattern used for `Variable` can be applied at literal parse sites.

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
    // Span of the most recent non-whitespace token encountered during parsing.
    // Used to attach EOF trailing comments and to resolve trailing-comment classification.
    last_seen_token_span: Option<Span>,
}

impl CommentTable {
    pub fn leading_comments(&self, span: Span) -> &[Comment] { ... }
    pub fn trailing_comments(&self, span: Span) -> &[Comment] { ... }
}
```

> **Span type:** `CommentTable` uses `ash_parser::token::Span` because it derives `Hash` and `Eq`. `ash_core::ast::Span` does **not** currently derive `Hash`; if the core AST ever needs to own a `CommentTable`, `Hash` (and `Eq`) must be added to `ast::Span` first. This is a **hard prerequisite** for TASK-571 and is cross-referenced in §3.2.

> **Span::default() policy:** `Span::default()` (the zero/empty span) must **never** be used as a key in `CommentTable`. If a comment would otherwise be attached to `Span::default()`, it must be skipped (dropped) rather than inserted. Alternatively, a private sentinel span may be used internally for EOF handling, but it must not leak into the public query API.

### 4.3 Whitespace-Skipping Changes

Instead of discarding comments, a single consolidated `skip_whitespace_and_comments` helper appends discovered comments to a mutable `CommentTable` that is threaded through parsing:

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

> **Consolidation requirement (C2):** There are currently **nine copies** of `skip_whitespace_and_comments` scattered across parser sub-modules (`parse_expr.rs`, `parse_pattern.rs`, `parse_policy.rs`, `parse_set.rs`, `parse_send.rs`, etc.). Before adding `&mut CommentTable`, these must be **consolidated into a single shared helper** in `crates/ash-parser/src/parse_utils.rs` (or an equivalent shared module). The shared helper must have its own unit-test suite covering whitespace-only, comment-only, mixed, and edge-case inputs.

> **Backtracking and rollback (I5):** The Ash parser uses combinator-based backtracking. A mutable `CommentTable` side-table is not automatically rolled back when a parser branch fails. Therefore, either:
> 1. `CommentTable` must support state snapshotting (`save`/`restore` around backtracking points), or
> 2. Comments must be collected speculatively into a temporary buffer and only committed when a parser succeeds.
> The implementation must pick one strategy and document it in the module-level comments of `parse_utils.rs`.

> **Scope note:** After consolidation, every parser sub-module calls the shared helper. This is a parser-wide but mechanical cleanup.

### 4.4 Comment Classification Heuristic

The heuristic runs during whitespace skipping:

1. **Trailing comment:** The comment appears on the same line as the preceding non-comment token, and there is no blank line between them. It is attached to the preceding token's span.
2. **Leading comment:** The comment appears on a line before the next non-comment token, or there is a blank line between the comment and the preceding token. It is attached to the next token's span.
3. **End-of-file comment:** A comment with no subsequent non-comment token is attached as a trailing comment of the last token in the file (using `last_seen_token_span`).

> **Intra-expression comments:** Comments inside expressions (e.g., `foo(--c\n)(args)`) are **out of scope** for MVP. The side-table attaches comments to the nearest top-level or declaration span. Fine-grained expression-level comment attachment is deferred.

#### 4.4.1 Testing Strategy for the Heuristic

The shared `skip_whitespace_and_comments` helper (and `parse_surface_file` integration tests) must cover the following classification matrix:

| Case | Input Example | Expected Classification |
|------|---------------|------------------------|
| Simple trailing | `let x = 1 -- trailing\n` | Trailing on `1` |
| Simple leading | `-- leading\nlet x = 1` | Leading on `let` |
| Blank-line separator | `let x = 1\n\n-- leading\nlet y = 2` | Leading on `let y` |
| Consecutive comments (same line) | `let x = 1 -- a -- b\n` | Both trailing on `1` (order preserved) |
| Consecutive comments (multiline block) | `-- a\n-- b\nlet x = 1` | Both leading on `let` (order preserved) |
| Mixed block/line | `/* block */ -- line\nlet x = 1` | Both leading on `let` if newline separates; trailing on prior token if same line |
| EOF trailing | `let x = 1 -- eof` | Trailing on `1` (uses `last_seen_token_span`) |
| EOF leading-only (no prior token) | `-- eof\n` | Skipped (no valid span to attach to) |
| Comment after comma in list | `[1, --c\n 2]` | Trailing on `,` (or nearest list token) |
| Comment before first token | `-- header\nmodule M` | Leading on `module` |

All cases must be expressed as **property-style assertions** in the parser test suite:
- `assert_leading(span, expected_texts)`
- `assert_trailing(span, expected_texts)`
- `assert_comment_count(total)` to guard against dropped or duplicate comments.

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

### 4.6 `parse_surface_file` Design

Add a top-level public API to the parser crate:

```rust
pub fn parse_surface_file(source: &str)
    -> Result<ash_parser::surface::ModuleFile, Vec<ash_parser::error::ParseError>>
```

#### 4.6.1 Bootstrapping `CommentTable`

`parse_surface_file` performs the following steps:
1. Allocate an empty `CommentTable` on the stack.
2. Construct the parser `Input` from `source`.
3. Invoke the existing module parser combinator (e.g., `module_file.parse_next(&mut input)`) passing `&mut CommentTable` through every whitespace-skipping call.
4. On success, attach the populated `CommentTable` to the returned `ModuleFile`.
5. On failure, collect all `ParseError`s and return them as `Err(errors)`. The partially-filled `CommentTable` is discarded on error paths.

#### 4.6.2 Delegated Combinator

`parse_surface_file` does **not** reimplement parsing logic. It delegates to a combinator (or thin wrapper) that mirrors the current internal module-file parser. The wrapper's only additional responsibility is to:
- Thread `&mut CommentTable` into `skip_whitespace_and_comments`.
- Finalize EOF trailing comments by flushing any queued comments to `last_seen_token_span` after the module parser succeeds.

#### 4.6.3 Error Collection

Error handling follows the existing parser convention:
- Combinator failures are accumulated via `winnow` error recovery or early-return.
- `parse_surface_file` converts the final error state into `Vec<ParseError>`.
- Comment-table population is **best-effort** on the error path; do not attempt to return a partial `CommentTable` alongside errors in the MVP.

> **Naming:** The existing `parse_module` in `parse_module.rs` parses a single `Definition` from a module context. The new top-level entry point is named `parse_surface_file` to avoid collision.

### 4.7 Formatter Integration

The formatter (SPEC-042) will query `module.comments.leading_comments(span)` before emitting any declaration or expression, and insert the comment text verbatim.

### 4.8 LSP Integration

For hover and diagnostics, comment trivia is **not** required in the MVP. The side-table is primarily for the formatter and for future "generate documentation from comments" features.

## 5. Dependencies

None beyond the existing parser crate.

## 6. Testing Strategy

### 6.1 Binding Spans
- Property tests asserting that every parsed `Variable` and `Var` carries a non-default span with correct line/column.
- Regression tests for `Pattern::Variable` in match arms and let bindings.

### 6.2 Comment Trivia
- **Unit tests for the shared `skip_whitespace_and_comments` helper:**
  - Whitespace-only input produces empty maps.
  - Comment-only input is handled according to `Span::default()` policy (skipped or sentinel).
  - Mixed whitespace and comments trigger correct classification.
- **Integration tests via `parse_surface_file`:**
  - Parsing a file with comments produces a non-empty `CommentTable`.
  - All cases from the heuristic matrix in §4.4.1 are covered.
  - EOF comments are correctly attached via `last_seen_token_span`.
  - Consecutive comments preserve order.
- **Backtracking regression tests:**
  - A failing parser branch that consumes comment-containing input must not leak speculative comments into the final `CommentTable`.

## 7. Relationship to Other Specs

- **Blocks:** SPEC-038 LSP MVP (via binding spans), SPEC-041 (via `parse_surface_file` API), SPEC-042 (via comment trivia), SPEC-043 (via `parse_surface_file` API)
- **Blocked by:** None
- **Parallelizable with:** SPEC-040 (Diagnostic Infrastructure) after `TASK-570` binding-span changes are complete

## 8. Affected Files Reference

The binding-span change (`Variable { name, span }` / `Var { name, span }`) affects:

- `crates/ash-parser/src/surface.rs` — enum definitions
- `crates/ash-core/src/ast.rs` — enum definitions
- `crates/ash-parser/src/parse_expr.rs` — variable expression parsing
- `crates/ash-parser/src/parse_pattern.rs` — pattern parsing
- `crates/ash-parser/src/parse_policy.rs` — policy variable parsing
- `crates/ash-parser/src/parse_module.rs` — `Expr::Variable` construction
- `crates/ash-parser/src/parse_send.rs` — `Expr::Variable` matches
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

The comment-trivia change affects:

- `crates/ash-parser/src/parse_utils.rs` — new shared `skip_whitespace_and_comments` helper and its test suite
- `crates/ash-parser/src/surface.rs` — `ModuleFile` gains `comments: CommentTable`
- `crates/ash-parser/src/lib.rs` — new `parse_surface_file` public API
- All parser sub-modules that previously contained private copies of `skip_whitespace_and_comments`
