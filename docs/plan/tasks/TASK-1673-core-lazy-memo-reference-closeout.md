# TASK-1673: Document and close out Core lazy/memo modes

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Document the implemented SPEC-101 Core lazy/memo behavior, reconcile tracking surfaces, and close out Phase 163.

## Specification Reference

- [SPEC-101](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)

## Dependencies

- [TASK-1671](TASK-1671-core-mode-end-to-end-fixtures.md)
- [TASK-1672](TASK-1672-core-mode-tracing-observability.md)

## Requirements

1. Add reference docs for Core mode syntax, type checking, lowering, and runtime behavior.
2. Document deferred surface lowering and optimizer behavior.
3. Add docs consistency tests.
4. Reconcile PLAN-163 task statuses, PLAN-INDEX, and CHANGELOG.
5. Record closeout review/audit and exact verification commands.

## TDD Steps

1. Add failing docs consistency test in `crates/ash-core/tests/task_1673_core_lazy_memo_docs_consistency.rs`.
2. Run focused test and confirm missing docs references.
3. Add/update reference docs and closeout audit.
4. Run focused Phase 163 gates, affected crate gates, and `cargo test -p spec_processor spec_links`.

## Completion Checklist

- [x] Reference docs explain implemented behavior and non-goals.
- [x] PLAN-INDEX and CHANGELOG are reconciled.
- [x] Closeout audit lists exact command evidence.

## Closeout Audit

- `cargo test -p ash-core --test task_1660_core_mode_ast`
- `cargo test -p ash-core --test task_1661_core_mode_text`
- `cargo test -p ash-core --test task_1662_core_mode_validation`
- `cargo test -p ash-core --test task_1665_core_mode_type_wellformedness`
- `cargo test -p ash-core --test task_1666_core_thunk_value_typing`
- `cargo test -p ash-core --test task_1667_core_letmode_force_typecheck`
- `cargo test -p ash-core --test task_1668_core_mode_public_summary`
- `cargo test -p ash-core --test task_1669_core_mode_lowering`
- `cargo test -p ash-core --test task_1670_core_thunk_capture_authority`
- `cargo test -p ash-core --test task_1671_core_mode_end_to_end`
- `cargo test -p ash-core --test task_1672_core_mode_tracing_docs_consistency`
- `cargo test -p ash-core --test task_1673_core_lazy_memo_docs_consistency`
- `cargo test -p ash-core --test task_1650_core_typecheck_integration`
- `cargo test -p ash-interp --test task_1664_cps_force_runtime`
- `cargo test -p ash-interp --test task_1663_cps_runtime_scaffold`
- `cargo test -p ash-interp --test task_1672_cps_thunk_trace_observability`
- `cargo test -p spec_processor spec_links`
