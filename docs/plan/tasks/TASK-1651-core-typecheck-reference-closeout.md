# TASK-1651: Document and close out Core type checking

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Document the implemented Core type-checking boundary, reconcile tracking surfaces, and close out Phase 162.

## Specification Reference

- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)

## Dependencies

- [TASK-1650](TASK-1650-core-typecheck-integration-fixtures.md)

## Requirements

### Functional Requirements

1. Add or update reference documentation for the Core type checker.
2. Document the initial algorithmic profile and deferred features.
3. Add docs consistency tests where feasible.
4. Reconcile PLAN-162 task statuses.
5. Update PLAN-INDEX and CHANGELOG.
6. Run focused Phase 162 gates and affected crate gates.
7. Record a closeout review/audit.

### Property Requirements

- Docs must not claim full inference, proof solving, typeclass solving, or `MultiShotPure` support.
- Closeout evidence must list exact commands run.

## TDD Steps

### Step 1: Write failing docs consistency test

**Files:** `crates/ash-core/tests/task_1651_core_typecheck_docs_consistency.rs`

Run:

```bash
cargo test -p ash-core --test task_1651_core_typecheck_docs_consistency
```

Expected: fail until reference docs name the implemented boundary and deferred features.

### Step 2: Add docs and closeout updates

**Files:**

- `docs/reference/core-ash-type-checking.md`
- `docs/plan/PLAN-162-CORE-ASH-TYPE-CHECKING.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`
- `docs/plan/audits/PHASE-162-CLOSEOUT-REVIEW.md`

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1651_core_typecheck_docs_consistency
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
cargo test -p spec_processor spec_links
```

Expected: all Phase 162 closeout gates pass.

## Completion Evidence

Implemented in Phase 162 worktree:

- Added `docs/reference/core-ash-type-checking.md` for the implemented checker boundary, algorithmic profile, checked lowering path, and deferred features.
- Added `crates/ash-core/tests/task_1651_core_typecheck_docs_consistency.rs` to keep the reference page honest about implemented APIs and non-goals.
- Added `docs/plan/audits/PHASE-162-CLOSEOUT-REVIEW.md`.
- Reconciled `PLAN-162`, `PLAN-INDEX`, and `CHANGELOG.md` to close Phase 162.

Verified:

```bash
cargo test -p ash-core --test task_1651_core_typecheck_docs_consistency
cargo test -p ash-core --test task_1640_core_typecheck_api
cargo test -p ash-core --test task_1641_core_type_wellformedness
cargo test -p ash-core --test task_1642_core_row_normalization
cargo test -p ash-core --test task_1643_core_atom_value_typing
cargo test -p ash-core --test task_1644_core_expression_basics_typecheck
cargo test -p ash-core --test task_1645_core_call_jump_row_accounting
cargo test -p ash-core --test task_1646_core_effect_operation_typing
cargo test -p ash-core --test task_1647_core_handle_affine_resume
cargo test -p ash-core --test task_1648_core_refinement_discharge
cargo test -p ash-core --test task_1649_core_public_summary
cargo test -p ash-core --test task_1650_core_typecheck_integration
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
cargo test -p spec_processor spec_links
```
