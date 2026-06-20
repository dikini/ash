# TASK-1623: Parse Core expressions and effect forms

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Extend the `.core` parser to build full raw Core expressions.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- [TASK-1622](TASK-1622-core-text-parser-atoms-values.md)

## Requirements

### Functional Requirements

1. Parse `LetVal`, `LetRec`, `LetPrim`, `If`, `Call`, `Jump`, `Raise`, `Handle`, `RecordDischarge`, and `Trap`.
2. Parse handler clauses with explicit resume parameters.
3. Parse only supported `CoreEffectOp` operation kinds: capability, channel, process, and failure.
4. Add `parse_core_expr` and `parse_core_file` APIs.

### Property Requirements

- Parser errors should fail closed for unknown forms.
- The parser should not silently desugar surface-like syntax.

## TDD Steps

### Step 1: Write failing expression parser tests

**Files:** `crates/ash-core/tests/task_1623_core_text_parser_expressions.rs`

Use the fixtures from TASK-1621 and add at least one unknown-form negative test.

Run:

```bash
cargo test -p ash-core --test task_1623_core_text_parser_expressions
```

Expected: fail because expression parser APIs do not exist.

### Step 2: Implement expression parsing

**Files:** `crates/ash-core/src/core_ash_text.rs`

Implement expression parsing only. Do not add serializer or validation yet.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo fmt --check
```

Expected: expression and earlier parser tests pass.

## Completion Evidence

- Added `parse_core_expr` and `parse_core_file` APIs for full Core expression fixtures.
- Added support for `LetVal`, `LetRec`, `LetPrim`, `If`, `Call`, `Jump`, `Raise`, `Handle`, `RecordDischarge`, and `Trap`.
- Added parsing for handler clauses, affine resume parameters, supported effect operation kinds, continuation refs, and trap reasons.
- Added fail-closed tests for unknown Core forms and surface-like syntax.
- Verified:
  - `cargo test -p ash-core --test task_1623_core_text_parser_expressions`
  - `cargo test -p ash-core --test task_1622_core_text_parser_atoms_values`
