# TASK-1620: Add Core Ash AST carriers

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add the minimal Core Ash AST carriers from SPEC-099 in `ash-core`, separate from the existing CPS IR carriers.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- None.

## Requirements

### Functional Requirements

1. Create `crates/ash-core/src/core_ash.rs`.
2. Export the module from `crates/ash-core/src/lib.rs`.
3. Define `CoreAtom`, `CoreValue`, `CoreExpr`, `CoreType`, `CoreRow`, `CoreRowItem`, `CoreEffectOp`, `CoreHandlerClause`, `CoreContractDischarge`, and `CoreTrapReason`.
4. Keep Core AST names distinct from `crate::cps` names.
5. Represent only SPEC-099 operation-like effect ops: capability, channel, process, and failure.

### Property Requirements

- Core AST construction is deterministic and clone/equality friendly.
- `ContractViolation` is not representable as a row item or raised operation.

## TDD Steps

### Step 1: Write failing tests

**Files:** `crates/ash-core/tests/task_1620_core_ash_ast.rs`

Add tests that construct:

- a simple `LetVal`/`Jump` Core expression;
- a `Handle` with affine resume parameter metadata;
- a row with capability/failure items and no `ContractViolation` item.

Run:

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
```

Expected: fail because `core_ash` does not exist.

### Step 2: Implement minimal carriers

**Files:** `crates/ash-core/src/core_ash.rs`, `crates/ash-core/src/lib.rs`

Implement the AST data structures only. Do not add parsing, validation, or lowering in this task.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
cargo test -p ash-core
cargo fmt --check
```

Expected: focused test passes; affected crate remains green.

## Notes

If existing Phase 160 CPS variants differ from SPEC-098b, do not repair them here. This task only creates the Core layer.

## Completion Evidence

- Added `crates/ash-core/src/core_ash.rs` with SPEC-099 Core AST carriers and Core-specific type names.
- Exported `ash_core::core_ash` from `crates/ash-core/src/lib.rs`.
- Added `crates/ash-core/tests/task_1620_core_ash_ast.rs` covering direct-style expressions, affine handler resume metadata, representable raised operation kinds, and `ContractViolation` as trap-only.
