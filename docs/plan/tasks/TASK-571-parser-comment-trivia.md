# TASK-571: Parser — Comment Trivia Preservation

**Phase:** 84
**Spec:** SPEC-039 §4
**Related:** TASK-570
**Estimate:** 10 hours
**Status:** 📝 Planned

## Description

Preserve comments during parsing and store them in a side-table attached to `ModuleFile`, without changing the token stream architecture.

## Requirements

1. Define `Comment`, `CommentKind`, and `CommentTable` in `ash-parser`.
2. Update `skip_whitespace_and_comments` (and all copies in parser sub-modules) to accept `&mut CommentTable` and record comments instead of discarding them.
3. Apply the leading/trailing classification heuristic from SPEC-039 §4.4.
4. Add `comments: CommentTable` field to `ModuleFile`.
5. Implement `parse_surface_file(source: &str) -> Result<ModuleFile, Vec<ParseError>>` that builds and attaches the `CommentTable`.

## TDD Steps

### Red
- Tests for parsing commented source produce a non-empty `CommentTable`.

### Green
- Implement side-table population during whitespace skipping.

## Completion Checklist

- [ ] `CommentTable` defined using `ash_parser::token::Span`
- [ ] All `skip_whitespace_and_comments` call sites updated
- [ ] `CommentTable` stored on `ModuleFile`
- [ ] `parse_surface_file` entry point implemented
- [ ] Correct leading/trailing assignment for typical cases
- [ ] Edge cases handled (comments at EOF, consecutive comments)
- [ ] Tests passing
- [ ] Clippy and fmt clean
