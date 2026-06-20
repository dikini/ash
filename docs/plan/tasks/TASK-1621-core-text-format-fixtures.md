# TASK-1621: Freeze minimal Core text format and fixtures

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Define the first `.core` fixture grammar and add golden fixture files for the parser/serializer/lowering tasks.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- [TASK-1620](TASK-1620-core-ash-ast-carriers.md)

## Requirements

### Functional Requirements

1. Create `crates/ash-core/tests/fixtures/core/`.
2. Add small canonical `.core` fixture files for:
   - `let_val_jump.core`;
   - `let_prim_if.core`;
   - `call_non_tail.core`;
   - `raise_handle.core`;
   - `contract_trap.core`.
3. Create `docs/reference/core-ash-text-format.md`.
4. State that `.core` is a fixture/debug format, not surface Ash.

### Property Requirements

- Every fixture should be small enough to debug in one screen.
- The format should map directly to Core AST nodes and avoid surface sugar.

## TDD Steps

### Step 1: Write failing format tests

**Files:** `crates/ash-core/tests/task_1621_core_text_format.rs`

Add tests that assert fixture files exist and contain the expected top-level forms.

Run:

```bash
cargo test -p ash-core --test task_1621_core_text_format
```

Expected: fail because fixtures and docs do not exist.

### Step 2: Add fixtures and reference page

**Files:**

- `crates/ash-core/tests/fixtures/core/*.core`
- `docs/reference/core-ash-text-format.md`

Document only the forms needed by Phase 161.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1621_core_text_format
git diff --check -- crates/ash-core/tests/fixtures/core docs/reference/core-ash-text-format.md
```

Expected: focused fixture existence tests pass.

## Completion Evidence

- Added five canonical `.core` fixtures under `crates/ash-core/tests/fixtures/core/`.
- Added `docs/reference/core-ash-text-format.md`, explicitly documenting `.core` as a fixture/debug format, not surface Ash.
- Added `crates/ash-core/tests/task_1621_core_text_format.rs` to guard fixture presence, compactness, top-level forms, and reference boundary wording.
