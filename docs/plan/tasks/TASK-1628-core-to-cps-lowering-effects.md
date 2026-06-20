# TASK-1628: Lower Core effect and discharge forms to CPS IR

**Status:** Planned
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Extend Core-to-CPS lowering for raised operations, handlers, contract discharge, and traps.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)

## Dependencies

- [TASK-1627](TASK-1627-core-to-cps-lowering-basic.md)

## Requirements

### Functional Requirements

1. Lower `Raise` with `resume: current_cont` and local operation row only.
2. Lower `Handle` with `cont: current_cont` and local residual `Handle.row`.
3. Lower handler clauses with affine resume binding preserved.
4. Lower `RecordDischarge` to CPS `RecordDischarge`.
5. Lower `Trap` to CPS `Trap`.
6. Preserve `ContractViolation` only as a trap reason, never as an effect row item.

### Property Requirements

- `Handle.row` excludes the outer continuation row.
- Recoverable contract behavior must be represented only through explicit `fail`.

## TDD Steps

### Step 1: Write failing effect lowering tests

**Files:** `crates/ash-core/tests/task_1628_core_to_cps_effects.rs`

Cover:

- capability `Raise`;
- failure `Raise`;
- handler lowering;
- dynamic contract `RecordDischarge` plus `Trap`;
- negative assertion that `ContractViolation` is not an effect item.

Run:

```bash
cargo test -p ash-core --test task_1628_core_to_cps_effects
```

Expected: fail because effect lowering is missing.

### Step 2: Implement effect lowering

**Files:** `crates/ash-core/src/core_ash_lower.rs`

Keep row-field synthesis aligned with SPEC-098b.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1628_core_to_cps_effects
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo fmt --check
```

Expected: effect lowering tests pass.
