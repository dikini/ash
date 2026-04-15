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

- `Expr::Variable(Name, Span)` and `Pattern::Variable(Name, Span)` in both surface and core AST
- `Comment` token kind in the lexer
- `CommentTable` stored on `ModuleFile`
- All parser/type-checker/interpreter match sites updated

## Timeline

1 week (~16 hours)

## Risks

- Widespread `Expr::Variable` / `Pattern::Variable` pattern matches across crates require careful mechanical refactoring.
- Comment-table population must correctly handle edge cases (comments at EOF, multiple consecutive comments).

## Parallelization

- `TASK-570` (binding spans) blocks `TASK-571` (comment trivia) because both touch `surface.rs` enum definitions.
- `TASK-570` also blocks Phase 86 (`ash-lint` AST visitors) and any work that pattern-matches on `Expr::Variable`.
- `TASK-571` can run in parallel with Phase 85 (`TASK-572` and `TASK-573`) after `TASK-570` is complete.
