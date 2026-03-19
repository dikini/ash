# TASK-168: Align Type Checking for Policies and Receive

## Status: 📝 Planned

## Description

Align type checking and declaration checking with the canonical policy and `receive` contracts.

This task ensures the type layer enforces the same policy model and stream declaration rules
that the stabilized docs and lowering layers require.

## Specification Reference

- SPEC-003: Type System
- SPEC-006: Policy Definitions
- SPEC-013: Streams and Event Processing
- SPEC-017: Capability Integration

## Reference Contract

- `docs/reference/type-to-runtime-contract.md`
- `docs/reference/parser-to-core-lowering-contract.md`

## Requirements

### Functional Requirements

1. Type-check canonical policy forms consistently
2. Enforce canonical `receive` declaration and typing rules
3. Add tests proving policy and `receive` contracts are enforced together

## Files

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/policy_check.rs`
- Modify: `crates/ash-typeck/src/capability_check.rs`
- Test: `crates/ash-typeck/tests/policy_contracts.rs`
- Test: `crates/ash-typeck/tests/receive_contracts.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving canonical policy forms and `receive` declarations are type-checked consistently.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-typeck policy_contracts receive_contracts -- --nocapture
```

Expected: fail due to current contract mismatch.

### Step 3: Implement the minimal fix (Green)

Update type checking and declaration checking to match the canonical contracts.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-typeck policy_contracts receive_contracts -- --nocapture
```

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-typeck
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-typeck/src/check_expr.rs crates/ash-typeck/src/policy_check.rs crates/ash-typeck/src/capability_check.rs crates/ash-typeck/tests/policy_contracts.rs crates/ash-typeck/tests/receive_contracts.rs CHANGELOG.md
git commit -m "fix: align type checking for policies and receive"
```

## Completion Checklist

- [ ] failing type-check tests added
- [ ] failure verified
- [ ] policy type checking aligned
- [ ] `receive` type/declaration checking aligned
- [ ] focused and broader verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No interpreter/runtime execution changes
- No CLI/REPL changes

## Dependencies

- Depends on: TASK-163, TASK-166, TASK-167
- Blocks: TASK-169, TASK-170, TASK-171
