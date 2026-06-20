# TASK-1651: Document and close out Core type checking

**Status:** Planned
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
