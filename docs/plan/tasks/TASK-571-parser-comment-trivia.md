# TASK-571: Parser — Comment Trivia Preservation

**Phase:** 84
**Spec:** SPEC-039 §4
**Related:** TASK-570
**Estimate:** 10 hours
**Status:** 📝 Planned

## Description

Preserve comments during lexing and store them in a side-table attached to `ModuleFile`.

## Requirements

1. Add `Comment` token kind to `ash_parser::token::TokenKind`.
2. Lexer emits `Comment` tokens instead of discarding comments.
3. Define `CommentTable` with `leading_comments(span)` and `trailing_comments(span)` lookups.
4. Add `comments: CommentTable` field to `ModuleFile`.
5. Build a post-lex pass that assigns each `Comment` token to the nearest non-comment token's span.

## TDD Steps

### Red
- Tests for lexing commented source produce `Comment` tokens.

### Green
- Implement `CommentTable` and populate it during parsing.

## Completion Checklist

- [ ] `Comment` token kind exists
- [ ] Lexer emits comment tokens
- [ ] `CommentTable` defined and stored on `ModuleFile`
- [ ] Correct leading/trailing assignment for typical cases
- [ ] Edge cases handled (EOF comments, consecutive comments)
- [ ] Tests passing
- [ ] Clippy and fmt clean
