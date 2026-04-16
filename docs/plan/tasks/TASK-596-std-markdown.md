# TASK-596: std::markdown CommonMark AST MVP

## Status: 📝 Planned

## Description

Implement a CommonMark-compliant Markdown AST with Pandoc JSON filter compatibility, backed by a Rust parser.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track B
- DESIGN-SPEC-PROCESSOR.md §6 (D4 resolution)

## Dependencies

- TASK-597 (`std::json`) for `to_pandoc_json` implementation.

## Requirements

1. Define `Block`, `Inline`, `MarkdownDoc` ADTs with `Extension` escape hatches.
2. Implement Rust-backed `parse(text: String) -> Result<MarkdownDoc, MarkdownError>`.
3. Implement pure-Ash `to_pandoc_json(doc: MarkdownDoc) -> JsonValue`.

## TDD Steps

### Step 1: Write failing test

Parse a real `SPEC-*.md` file and assert non-empty `MarkdownDoc`.

### Step 2: Implement

- `std/src/markdown.ash`
- Rust capability backend using `pulldown-cmark`.

### Step 3: Verify

Round-trip through `json::stringify` produces valid Pandoc-like JSON.

## Verification Steps

- [ ] AST parses real spec files
- [ ] Pandoc JSON round-trips
- [ ] Codex verification: VERIFIED
