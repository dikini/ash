# TASK-1643: Type Core atoms and values

**Status:** Planned
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Add type synthesis/checking for Core atoms and inert Core values.

## Specification Reference

- [SPEC-100 §10](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#10-atom-and-value-typing)

## Dependencies

- [TASK-1641](TASK-1641-core-type-wellformedness.md)
- [TASK-1642](TASK-1642-core-row-normalization-solving.md)

## Requirements

### Functional Requirements

1. Synthesize types for variables, literals, primitive names, and constructor names.
2. Reject continuation references in atom/data positions.
3. Check lambda parameters, body type, and latent row annotation.
4. Type record values by field names and atom values.
5. Type tuple values by ordered elements.
6. Validate `DischargeMarker` metadata shape without exposing it as ordinary evidence data.

### Property Requirements

- Values have construction row `{}`.
- Lambda latent row is checked against the body row, not charged at construction.

## TDD Steps

### Step 1: Write failing atom/value tests

**Files:** `crates/ash-core/tests/task_1643_core_atom_value_typing.rs`

Cover:

- literals synthesize base types;
- unknown variable fails;
- record value synthesizes field-name keyed record type;
- tuple value preserves element order;
- lambda latent row mismatch fails;
- discharge marker is administrative.

Run:

```bash
cargo test -p ash-core --test task_1643_core_atom_value_typing
```

Expected: fail until atom/value typing exists.

### Step 2: Implement atom/value typing

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add helpers for atom synthesis and value checking.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1643_core_atom_value_typing
cargo test -p ash-core --test task_1642_core_row_normalization
cargo fmt --check
```

Expected: focused tests pass.
