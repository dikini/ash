# TASK-210: Clear Workspace Clippy Warnings

## Status: ✅ Complete

## Description

Clear the remaining workspace clippy warnings that still break CI after the runtime convergence work.

This task is limited to the currently-reported warnings in `ash-core` and `ash-repl` and should
not expand into broader refactoring.

## Specification Reference

- SPEC-021: Runtime Observable Behavior

## Reference Contract

- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Remove the `clippy::box_default` warning in `ash-core`
2. Remove the `clippy::while_let_on_iterator` warning in `ash-repl`
3. Preserve existing runtime and test behavior
4. Restore a clean `cargo clippy --all-targets --all-features` run

## Files

- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-repl/src/error.rs`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Verify the failing check (Red)

Run:

```bash
cargo clippy --all-targets --all-features
```

Expected: warnings for `clippy::box_default` and `clippy::while_let_on_iterator`.

### Step 2: Implement the minimal fix (Green)

Update the warned code paths without changing observable behavior.

### Step 3: Verify focused GREEN

Run:

```bash
cargo clippy --all-targets --all-features
cargo test --all
```

Expected: clean clippy run and passing tests.

## Completion Checklist

- [x] failing clippy run verified
- [x] warnings removed with minimal code changes
- [x] `cargo clippy --all-targets --all-features` clean
- [x] `cargo test --all` passing
- [x] `CHANGELOG.md` updated

## Completion Notes

- Replaced a test-only `Box::new(vec![])` construction in `ash-core` with `Box::default()` to satisfy `clippy::box_default`.
- Rewrote the ANSI-skip loop in `ash-repl` test helpers from `while let` to `for ... in chars.by_ref()` to satisfy `clippy::while_let_on_iterator`.
- Verified the repository-level gates with `cargo clippy --all-targets --all-features` and `cargo test --all`.

## Non-goals

- No broader clippy cleanup
- No unrelated refactoring
