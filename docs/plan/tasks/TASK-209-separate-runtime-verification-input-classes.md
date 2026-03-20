# TASK-209: Separate Runtime Verification Input Classes

## Status: ✅ Complete

## Description

Separate workflow-declared capability inputs from obligation-backed runtime requirements in
aggregate runtime verification.

This task resolves the current tension where aggregate verification restores enforcement by
deriving obligation requirements from `WorkflowCapabilities`, even though the reference contracts
distinguish capability availability from obligation satisfaction.

## Specification Reference

- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Reference Contract

- `docs/reference/type-to-runtime-contract.md`
- `docs/reference/runtime-verification-input-contract.md`

## Requirements

### Functional Requirements

1. Add an explicit aggregate-verification input path for obligation-backed runtime requirements
2. Stop treating workflow capability declarations as the canonical source of obligation
   requirements
3. Add tests proving capability availability and obligation-backed requirements can vary
   independently in aggregate verification

## Files

- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/runtime_verification_input_contracts.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving:

- capability availability can succeed while obligation-backed requirements fail,
- obligation-backed requirements can be satisfied without changing workflow capability
  declarations,
- aggregate verification does not conflate the two requirement classes.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-typeck --test runtime_verification_input_contracts -- --nocapture
```

Expected: fail because aggregate verification still derives obligation requirements implicitly
from workflow capability declarations.

### Step 3: Implement the minimal fix (Green)

Introduce the separate aggregate-verification requirement input and align the aggregate verifier
to use it.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-typeck --test runtime_verification_input_contracts -- --nocapture
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
git add crates/ash-typeck/src/runtime_verification.rs crates/ash-typeck/src/obligation_checker.rs crates/ash-typeck/tests/runtime_verification_input_contracts.rs CHANGELOG.md
git commit -m "fix: separate runtime verification input classes"
```

## Completion Checklist

- [x] failing runtime-verification input-contract tests added
- [x] failure verified
- [x] aggregate verification input classes separated
- [x] implicit capability-to-obligation substitution removed
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No interpreter execution changes
- No CLI/REPL changes
- No new policy features

## Dependencies

- Depends on: TASK-169
- Blocks: TASK-170, TASK-171
