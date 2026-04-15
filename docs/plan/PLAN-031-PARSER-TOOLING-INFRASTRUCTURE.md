# PLAN-031: Parser Tooling Infrastructure

## Phase: 84

## Goal

Add binding spans and comment-trivia preservation to the Ash parser so that downstream tools (LSP, formatter, linter) can operate on precise locations and preserve user comments.

## Specification

- [SPEC-039: Parser Tooling Infrastructure](../spec/SPEC-039-PARSER-TOOLING-INFRASTRUCTURE.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-570](../tasks/TASK-570-parser-binding-spans.md) | Add spans to `Expr::Variable` and `Pattern::Variable` | 6h | 📝 Planned |
| [TASK-571](../tasks/TASK-571-parser-comment-trivia.md) | Preserve comments in lexer and build `CommentTable` side-table | 10h | 📝 Planned |

## Deliverable

- `Expr::Variable { name: Name, span: Span }` and `Pattern::Variable { name: Name, span: Span }` in both surface and core AST
- `PolicyExpr::Var { name: Name, span: Span }` in surface AST only
- `impl Spanned for Expr` and `impl Spanned for PolicyExpr` updated in `surface.rs` to return the new spans
- `ast::Span` derives `Hash` and `Eq` (prerequisite for downstream Salsa usage in SPEC-043)
- `Literal` span work explicitly deferred
- `Comment` capture during whitespace skipping (side-table approach, no token-stream changes)
- 8 definitions of `skip_whitespace_and_comments` (7 duplicates + 1 shared in `parse_utils.rs`) consolidated into `parse_utils.rs` as a `pub(crate)` helper with dedicated test suite
- `CommentTable` attached to `ModuleFile`, including `last_seen_token_span` for EOF comments and a write API (`push_leading`, `push_trailing`, `set_last_token`) that enforces the `Span::default()` skip policy
- Backtracking-safe comment collection (snapshotting or speculative buffering; prefer speculative buffer if winnow provides a clean mechanism, else snapshotting)
- New `module_file` combinator created as part of the `parse_surface_file` implementation
- `parse_surface_file(source: &str)` top-level API per SPEC-039 §4.6, with gate criteria (populated `CommentTable` on all example files) that unblock SPEC-041
- All parser/type-checker/interpreter match sites updated (~400+ call sites, including `desugar.rs`, `constraints.rs`, `capability_check.rs`, `policy_check.rs`, `effect.rs`, `solver.rs`, `execute.rs`, `visualize.rs`, `stream.rs`, `test_helpers.rs`, benches, and integration tests)

## Timeline

1 week (~16 hours)

## Risks

- Widespread `Expr::Variable` / `Pattern::Variable` pattern matches across crates require careful mechanical refactoring (~400+ call sites, including `desugar.rs`, `constraints.rs`, `capability_check.rs`, `policy_check.rs`, `effect.rs`, `solver.rs`, `execute.rs`, `visualize.rs`, `stream.rs`, `test_helpers.rs`, benches, and integration tests).
- Consolidating 8 definitions of `skip_whitespace_and_comments` (7 duplicates + 1 shared in `parse_utils.rs`) may expose subtle behavioral differences that must be reconciled.
- Comment-table population must correctly handle edge cases (comments at EOF, multiple consecutive comments, blank-line separation).
- Mutable side-table + combinator backtracking requires an explicit rollback strategy (prefer speculative buffer if winnow provides a clean mechanism; fall back to snapshotting); failure to snapshot correctly will produce phantom comments.
- `parse_surface_file` gate criteria must be met before SPEC-041 can proceed; delays in comment-table edge-case handling will block downstream work.

## Parallelization

- `TASK-570` (binding spans) blocks `TASK-571` (comment trivia) because both touch `surface.rs` enum definitions.
- `TASK-570` also blocks Phase 86 (`ash-lint` AST visitors) and any work that pattern-matches on `Expr::Variable`.
- `TASK-571` can run in parallel with Phase 85 (`TASK-572` and `TASK-573`) after `TASK-570` is complete.
