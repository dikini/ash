# TASK-1624: Add canonical Core text serializer

**Status:** Planned
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add a canonical serializer from Core AST to `.core` text and round-trip tests through the parser.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- [TASK-1623](TASK-1623-core-text-parser-expressions.md)

## Requirements

### Functional Requirements

1. Add `core_expr_to_string` and `write_core_expr_to_file` APIs.
2. Use one canonical spelling and stable field ordering.
3. Round-trip representative Core expressions through `serialize -> parse`.
4. Do not require the serializer to preserve comments or whitespace.

### Property Requirements

- `parse(serialize(ast)) == ast` for representative ASTs.
- Serializing the same AST twice produces identical text.

## TDD Steps

### Step 1: Write failing serializer tests

**Files:** `crates/ash-core/tests/task_1624_core_text_serializer.rs`

Cover:

- simple let/jump expression;
- nested let/if expression;
- handler expression;
- contract trap expression.

Run:

```bash
cargo test -p ash-core --test task_1624_core_text_serializer
```

Expected: fail because serializer APIs do not exist.

### Step 2: Implement serializer

**Files:** `crates/ash-core/src/core_ash_text.rs`

Keep output canonical and explicit.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo fmt --check
```

Expected: parser/serializer round-trip tests pass.
