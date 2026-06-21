# TASK-1673: Document and close out Core lazy/memo modes

**Status:** Planned
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

- [ ] Reference docs explain implemented behavior and non-goals.
- [ ] PLAN-INDEX and CHANGELOG are reconciled.
- [ ] Closeout audit lists exact command evidence.
