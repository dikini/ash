# TASK-1622: Parse Core atoms, rows, types, and values

**Status:** Planned
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add the first parser slice for `.core` text: atoms, types, rows, row items, and values.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- [TASK-1621](TASK-1621-core-text-format-fixtures.md)

## Requirements

### Functional Requirements

1. Create `crates/ash-core/src/core_ash_text.rs`.
2. Add parser APIs for atoms, types, rows, row items, and values.
3. Return a typed `CoreTextError` with enough context for fixture debugging.
4. Reject unsupported operation/effect item spellings at parse time when they are unambiguously out of grammar.

### Property Requirements

- Literal and variable parsing must not conflate `Var(name)` with string literals.
- Row parsing must preserve order for diagnostics but compare duplicate-free rows through validation later.

## TDD Steps

### Step 1: Write failing parser tests

**Files:** `crates/ash-core/tests/task_1622_core_text_parser_atoms_values.rs`

Cover:

- integer/string/bool/unit atoms;
- variable atoms;
- function type with row;
- capability/failure row item;
- lambda, tuple, record, and discharge marker values.

Run:

```bash
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
```

Expected: fail because parser APIs do not exist.

### Step 2: Implement parser slice

**Files:** `crates/ash-core/src/core_ash_text.rs`, `crates/ash-core/src/lib.rs`

Keep the parser small and local. Do not parse full expressions yet.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core --lib core_ash
cargo fmt --check
```

Expected: focused parser tests pass.
