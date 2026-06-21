# Phase 162 Closeout Review

**Date:** 2026-06-21
**Scope:** PLAN-162 Core Ash Type Checking closeout review for TASK-1651.
**Result:** PASS

## Review Focus

- SPEC-100 implementation alignment.
- Core type-checking boundary clarity.
- Row-accounting facts consumed by checked lowering.
- Documentation overclaim prevention.
- Phase/task status reconciliation.

## Findings

No blocking or important findings.

## Evidence Reviewed

- `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`
- `docs/spec/SPEC-099-CORE-LANGUAGE.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/plan/PLAN-162-CORE-ASH-TYPE-CHECKING.md`
- `docs/plan/tasks/TASK-1640-*.md` through `TASK-1651-*.md`
- `docs/reference/core-ash-type-checking.md`
- `crates/ash-core/src/core_ash_typecheck.rs`
- `crates/ash-core/tests/task_1640_core_typecheck_api.rs` through `task_1651_core_typecheck_docs_consistency.rs`

## Review Notes

- The checker is documented and implemented as annotation-led, not as full Hindley-Milner inference.
- Row normalization removes exact duplicates for comparison while preserving namespaces and open-tail solving metadata.
- `Jump` local rows remain `{}` and checked continuation rows are preserved for CPS `Jump.row`.
- Checked lowering now also uses external function rows from `CoreTypeCheckEnv`, avoiding stale or missing lowering-context function rows.
- `Raise` rows remain operation-local, and handler residual rows preserve captured resume effects.
- Refinement obligations and discharge metadata are recorded as compiler facts; the implementation does not claim proof solving.
- Docs explicitly exclude typeclass solving, ad-hoc polymorphism, arbitrary user-defined algebraic effects, `MultiShotPure`, and surface-to-Core lowering.

## Verification

Focused Phase 162 tests passed:

```bash
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
cargo test -p ash-core --test task_1651_core_typecheck_docs_consistency
```

Affected crate and documentation gates passed:

```bash
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
cargo test -p spec_processor spec_links
```
