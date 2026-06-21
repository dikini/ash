# TASK-1642: Normalize and compare Core rows

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Implement row normalization, duplicate removal, row inclusion, and conservative structural row-variable solving.

## Specification Reference

- [SPEC-100 §7](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#7-row-normalization-and-compatibility)
- [SPEC-097b §5-§7](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#5-row-normalization)

## Dependencies

- [TASK-1640](TASK-1640-core-typecheck-api-and-environments.md)

## Requirements

### Functional Requirements

1. Normalize row item identities with effect-kind namespaces.
2. Remove exact duplicate row items before comparison.
3. Preserve row tails.
4. Compare closed rows by normalized set inclusion.
5. Solve explicit open-row tails by structural remainder.
6. Keep role entailment out of normalization and row solving.
7. Reject or defer ambiguous group/alias references.

### Property Requirements

- `cap fs.read` and `role fs.read` must remain distinct identities.
- Duplicate exact items normalize to one item, not an error.
- Solving must not invent requirements that are not used, expected, or constrained.

## TDD Steps

### Step 1: Write failing row tests

**Files:** `crates/ash-core/tests/task_1642_core_row_normalization.rs`

Cover:

- exact duplicate removal;
- namespace distinction;
- closed-row inclusion success/failure;
- open-row structural remainder binding;
- role items are not expanded into capabilities.

Run:

```bash
cargo test -p ash-core --test task_1642_core_row_normalization
```

Expected: fail until row normalization APIs exist.

### Step 2: Implement row normalization and comparison

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add normalized row/item carriers if useful. Keep them private unless later tasks need public summaries.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1642_core_row_normalization
cargo test -p ash-core --test task_1641_core_type_wellformedness
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added Core row normalization with exact duplicate removal, effect-kind namespace preservation, open-tail preservation, and ambiguous group-reference rejection.
- Added structural row inclusion comparison with closed-row subset checks and explicit open-row remainder solutions.
- Preserved role items structurally; normalization and solving do not expand roles into capabilities.
- Added `crates/ash-core/tests/task_1642_core_row_normalization.rs` covering duplicate removal, namespace distinction, closed inclusion success/failure, open-tail solving, role non-expansion, and group-reference rejection.
- Verified with:
  - `cargo test -p ash-core --test task_1642_core_row_normalization`
  - `cargo test -p ash-core --test task_1641_core_type_wellformedness`
  - `cargo test -p ash-core`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
