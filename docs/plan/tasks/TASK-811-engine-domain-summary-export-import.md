# TASK-811: Engine Domain Summary Export/Import

## Status: 📝 Planned

## Description

Transport public domain summaries through engine export/import, alias, and re-export paths with explicit exposed-versus-opaque constructor-set behavior.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)

## Dependencies

- [TASK-810](TASK-810-domain-lowering-and-summary-versioning.md)

## Objective

Make public domain metadata available to downstream modules without leaking hidden constructors or collapsing origin identities.

## Requirements

1. Extend engine export collection to include domain summaries alongside existing ordinary summaries.
2. Preserve origin domain identity across named imports, aliases, glob imports where applicable, and `pub use` re-exports.
3. Respect exposed-versus-opaque constructor-set policy; hidden constructors must never appear in imported public summaries.
4. Keep ordinary type/module summary transport and workflow-summary transport non-regressed.
5. Add focused tests for public exposed domains, opaque exports, alias identity preservation, and re-export visibility boundaries.
6. Do not add typechecker registration or semantic evaluation in this task.

## Files

- Modify: `crates/ash-engine/src/module_loader.rs`
- Modify: `crates/ash-engine/src/lib.rs` if required by the exported carrier surface
- Add focused tests under `crates/ash-engine/tests/`

## TDD Steps

1. Write failing engine tests for exposed imports, opaque imports, alias/re-export identity preservation, and hidden-constructor non-leakage.
2. Implement the minimal export/import transport changes.
3. Re-run focused engine tests.
4. Confirm ordinary type/workflow summary transport still works.

## Verification Steps

- [ ] `cargo test -p ash-engine --test task_811_domain_summary_transport`
- [ ] `cargo test -p ash-engine`
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Engine transport task only. Do not add `TypeEnv` registration or domain evaluation here.
