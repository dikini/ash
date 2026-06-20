# TASK-1627: Lower basic Core expressions to CPS IR

**Status:** Planned
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add the first Core-to-CPS lowering pass for pure/basic direct-style Core forms.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- [TASK-1625](TASK-1625-core-validator-basic-invariants.md)

## Requirements

### Functional Requirements

1. Create `crates/ash-core/src/core_ash_lower.rs`.
2. Lower Core atoms and values into existing `crate::cps` atoms/values where representable.
3. Lower `LetVal`, `LetRec`, `LetPrim`, `If`, `Call`, and `Jump`.
4. Synthesize CPS continuation fields according to SPEC-099 §12.
5. Introduce `LetCont` for non-tail calls where needed.

### Property Requirements

- `Call.row` is the callee body row union current continuation row.
- `If.row` is the local branch-row union.
- `Jump.row` is the target continuation row.

## TDD Steps

### Step 1: Write failing lowering tests

**Files:** `crates/ash-core/tests/task_1627_core_to_cps_basic.rs`

Cover:

- pure `let-prim` then `jump`;
- `if` branch lowering;
- tail call lowering;
- non-tail call introducing a continuation.

Run:

```bash
cargo test -p ash-core --test task_1627_core_to_cps_basic
```

Expected: fail because lowering APIs do not exist.

### Step 2: Implement basic lowering

**Files:** `crates/ash-core/src/core_ash_lower.rs`, `crates/ash-core/src/lib.rs`

Accept only `ValidCoreProgram` or call the validator at the public boundary.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo test -p ash-core --test task_1625_core_validator_basic
cargo fmt --check
```

Expected: basic lowering tests pass.
