# TASK-1629: Add end-to-end Core fixture tests

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add end-to-end tests that parse `.core` fixtures, validate them, lower them to CPS IR, and compare canonical output.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- [TASK-1624](TASK-1624-core-text-serializer.md)
- [TASK-1628](TASK-1628-core-to-cps-lowering-effects.md)

## Requirements

### Functional Requirements

1. Add `.core` fixture tests for the full Phase 161 subset.
2. Add expected `.cps` golden outputs or equivalent AST assertions.
3. Ensure fixture loading uses parser plus validator before lowering.
4. Include one intentionally invalid `.core` fixture that fails validation.

### Property Requirements

- Golden fixture diffs must be stable.
- End-to-end tests must not depend on `ash-interp` unless explicitly needed.

## TDD Steps

### Step 1: Write failing end-to-end tests

**Files:** `crates/ash-core/tests/task_1629_core_end_to_end.rs`

Run:

```bash
cargo test -p ash-core --test task_1629_core_end_to_end
```

Expected: fail until parser, serializer, validator, lowering, and fixtures are wired together.

### Step 2: Add fixtures and glue

**Files:**

- `crates/ash-core/tests/fixtures/core/*.core`
- `crates/ash-core/tests/fixtures/core/*.cps.golden`
- `crates/ash-core/tests/task_1629_core_end_to_end.rs`

Do not broaden the Core grammar while adding fixtures.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core
cargo fmt --check
```

Expected: end-to-end fixtures pass.

## Completion Evidence

- Added `task_1629_core_end_to_end.rs` to parse `.core` fixtures, validate them, lower to CPS, serialize canonical CPS terms, reparse the serialized term, and compare against stable `.cps.golden` files.
- Added end-to-end fixtures for let/value/jump, primitive conditional, non-tail call, `let-call`, raise/handle, and contract trap/discharge forms.
- Added `invalid_duplicate_row.core` to verify invalid Core fails validation before lowering.

Verified on 2026-06-20:

```bash
cargo test -p ash-core --test task_1629_core_end_to_end
```
