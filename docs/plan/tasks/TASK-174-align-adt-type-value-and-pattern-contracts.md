# TASK-174: Align ADT Type, Value, and Pattern Contracts

## Status: ✅ Complete

## Description

Align type definitions, runtime values, pattern typing, exhaustiveness, and pattern execution
to the canonical ADT contract.

This task is the main code-level convergence step for ADTs.

## Specification Reference

- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-020: ADT Types

## Reference Contract

- `docs/reference/parser-to-core-lowering-contract.md`
- `docs/reference/type-to-runtime-contract.md`

## Requirements

### Functional Requirements

1. Share one canonical ADT contract across type definitions, runtime values, pattern typing, and pattern execution
2. Align exhaustiveness analysis with the same ADT model
3. Add tests proving the shared contract end-to-end

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-core/src/value.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-typeck/src/check_pattern.rs`
- Modify: `crates/ash-typeck/src/exhaustiveness.rs`
- Modify: `crates/ash-interp/src/pattern.rs`
- Test: `crates/ash-typeck/tests/adt_contracts.rs`
- Test: `crates/ash-interp/tests/adt_contracts.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving the canonical ADT contract is shared by type definitions, runtime values, pattern typing, and pattern execution.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-typeck adt_contracts -- --nocapture
cargo test -p ash-interp adt_contracts -- --nocapture
```

Expected: fail due to current mismatches.

### Step 3: Implement the minimal fix (Green)

Align all ADT layers to the canonical contract.

### Step 4: Verify focused GREEN

Run the same commands again.

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-typeck
cargo test -p ash-interp
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-core/src/ast.rs crates/ash-core/src/value.rs crates/ash-parser/src/parse_type_def.rs crates/ash-typeck/src/check_pattern.rs crates/ash-typeck/src/exhaustiveness.rs crates/ash-interp/src/pattern.rs crates/ash-typeck/tests/adt_contracts.rs crates/ash-interp/tests/adt_contracts.rs CHANGELOG.md
git commit -m "fix: align adt type value and pattern contracts"
```

## Completion Checklist

- [x] failing ADT contract tests added
- [x] failure verified
- [x] ADT type/value/pattern layers aligned
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No stdlib helper expansion yet
- No new ADT feature families

## Dependencies

- Depends on: TASK-160, TASK-162, TASK-163
- Blocks: TASK-175, TASK-176
