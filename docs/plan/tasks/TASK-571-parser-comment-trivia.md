# TASK-571: Parser — Comment Trivia Preservation

**Phase:** 84
**Spec:** SPEC-039 §4
**Related:** TASK-570
**Estimate:** 10 hours
**Status:** 📝 Planned

## Description

Preserve comments during parsing and store them in a side-table attached to `ModuleFile`, without changing the token stream architecture. Before adding `&mut CommentTable`, consolidate the nine copies of `skip_whitespace_and_comments` into a single shared helper with a dedicated test suite.

## Requirements

1. Define `Comment`, `CommentKind`, and `CommentTable` in `ash-parser`.
   - `CommentTable` must include a `last_seen_token_span: Option<Span>` field for EOF trailing comments.
   - `Span::default()` must never be used as a lookup key; such comments are skipped.
2. **Consolidation:** Merge all nine copies of `skip_whitespace_and_comments` into `crates/ash-parser/src/parse_utils.rs` (or equivalent). The shared helper must have a unit-test suite covering whitespace-only, comment-only, mixed, and edge-case inputs.
3. Update the shared `skip_whitespace_and_comments` to accept `&mut CommentTable` and record comments instead of discarding them.
4. Apply the leading/trailing classification heuristic from SPEC-039 §4.4.
5. Handle backtracking: implement state snapshotting or speculative buffering so that failed parser branches do not leak comments into the final table.
6. Add `comments: CommentTable` field to `ModuleFile`.
7. Implement `parse_surface_file(source: &str) -> Result<ModuleFile, Vec<ParseError>>` that:
   - Bootstraps an empty `CommentTable`,
   - Delegates to the existing module combinator,
   - Flushes EOF trailing comments via `last_seen_token_span`,
   - Attaches the table on success and returns errors on failure.

## TDD Steps

### Red
- Tests for the shared `skip_whitespace_and_comments` helper (classification matrix from SPEC-039 §4.4.1).
- Tests for parsing commented source produce a non-empty `CommentTable`.
- Tests for backtracking branches with comments do not pollute the final table.

### Green
- Implement shared helper in `parse_utils.rs`.
- Implement side-table population with rollback support.
- Implement `parse_surface_file` entry point.

## Completion Checklist

- [ ] `CommentTable` defined using `ash_parser::token::Span` with `last_seen_token_span`
- [ ] `Span::default()` policy documented and enforced
- [ ] All `skip_whitespace_and_comments` copies consolidated into `parse_utils.rs`
- [ ] Shared helper has standalone unit-test suite
- [ ] Backtracking/rollback strategy chosen and implemented
- [ ] `CommentTable` stored on `ModuleFile`
- [ ] `parse_surface_file` entry point implemented per SPEC-039 §4.6
- [ ] Correct leading/trailing assignment for typical cases
- [ ] Edge cases handled (comments at EOF, consecutive comments)
- [ ] Tests passing
- [ ] Clippy and fmt clean
