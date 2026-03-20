# TASK-169: Unify Runtime Verification Context and Obligation Enforcement

## Status: ✅ Complete

## Description

Unify runtime verification context shape and restore required aggregate obligation enforcement.

This task aligns the verification layer with the canonical runtime context contract and the
documented aggregate checks.

## Specification Reference

- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Reference Contract

- `docs/reference/type-to-runtime-contract.md`
- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Align runtime verification context fields with the canonical contract
2. Restore required aggregate obligation and capability checks
3. Add tests proving canonical verification behavior

## Files

- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-typeck/src/capability_typecheck.rs`
- Test: `crates/ash-typeck/tests/runtime_verification_contracts.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving canonical runtime context shape and aggregate obligation enforcement behavior.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-typeck runtime_verification_contracts -- --nocapture
```

Expected: fail because the aggregate path does not fully enforce the intended contract.

### Step 3: Implement the minimal fix (Green)

Unify the runtime context contract and restore the required aggregate checks.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-typeck runtime_verification_contracts -- --nocapture
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
git add crates/ash-typeck/src/runtime_verification.rs crates/ash-typeck/src/capability_typecheck.rs crates/ash-typeck/tests/runtime_verification_contracts.rs CHANGELOG.md
git commit -m "fix: unify runtime verification context"
```

## Completion Checklist

- [x] failing runtime-verification tests added
- [x] failure verified
- [x] context contract aligned
- [x] aggregate obligation enforcement restored
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No CLI/REPL changes
- No provider API redesign

## Dependencies

- Depends on: TASK-163, TASK-168
- Blocks: TASK-170, TASK-171
