# TASK-1649: Add Core public summary scaffold

**Status:** Planned
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Add the first public type/row summary scaffold needed for future Core export/import checking.

## Specification Reference

- [SPEC-100 §13](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#13-public-summaries)
- [SPEC-097b §9.4](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#94-exportimport)

## Dependencies

- [TASK-1648](TASK-1648-core-refinement-obligations-discharge.md)

## Requirements

### Functional Requirements

1. Define summary carriers for normalized public function types and rows.
2. Preserve public type constructor identities and arities.
3. Preserve public contract/refinement obligation identities.
4. Reject or expand private aliases/groups that would leak into public rows.
5. Keep the scaffold separate from surface module-summary formats unless a local adapter already exists.

### Property Requirements

- Public summaries must not lose row item namespaces.
- Private diagnostic names must not leak into public summaries.

## TDD Steps

### Step 1: Write failing public summary tests

**Files:** `crates/ash-core/tests/task_1649_core_public_summary.rs`

Cover:

- public row summary preserves capability namespace;
- private alias in public row fails or expands;
- obligation identity is preserved in summary metadata.

Run:

```bash
cargo test -p ash-core --test task_1649_core_public_summary
```

Expected: fail until summary scaffold exists.

### Step 2: Implement summary scaffold

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add minimal summary types and extraction helpers.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1649_core_public_summary
cargo test -p ash-core --test task_1648_core_refinement_discharge
cargo fmt --check
```

Expected: focused tests pass.
