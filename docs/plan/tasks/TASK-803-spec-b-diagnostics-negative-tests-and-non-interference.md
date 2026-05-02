# TASK-803: SPEC-B Diagnostics, Negative Tests, and Non-Interference

## Status: 📝 Planned

## Description

Add diagnostics, negative tests, and non-interference coverage proving the SPEC-B substrate is tight and does not leak later semantics or regress earlier phases.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-799](TASK-799-kind-and-arity-validation-hardening.md)
- ✅ [TASK-800](TASK-800-associated-projection-canonicalization-and-rigid-plumbing.md)
- ✅ [TASK-801](TASK-801-transparent-alias-canonicalization-helper.md)
- ✅ [TASK-802](TASK-802-canonicalization-boundary-adoption-for-current-equality-sites.md)

## Objective

Prove the Phase 110 substrate is correct, explicit about failures, and non-interfering with existing language behavior.

## Requirements

1. Add diagnostics and negative tests for ambiguous projections, unsupported projection shapes, wrong kind, wrong arity, and any remaining explicit rejection boundaries that Phase 110 still owns.
2. Add non-interference coverage for Phase 109 ordinary-type behavior.
3. Add non-interference coverage for current ADT/interface/workflow/capability/resource/do/comprehension behavior as relevant.
4. Do not broaden semantics while writing tests.

## Files

- Add or update `crates/ash-typeck/tests/task_803_projection_diagnostics.rs` for ambiguity, unsupported-shape, wrong-kind, wrong-arity, and multi-parameter projection-spine coverage
- Add or update `crates/ash-typeck/tests/task_803_phase110_non_interference.rs` for carried-forward Phase 109 ordinary-type behavior plus representative ADT/interface/workflow/capability/resource/do/comprehension regressions
- If parser rejection-boundary evidence is still required here rather than only via TASK-797, add or update `crates/ash-parser/tests/task_803_phase110_rejection_boundaries.rs`
- Update diagnostics text only where needed

## TDD Steps

1. Write failing tests first in `task_803_projection_diagnostics.rs` for unary ambiguity, unsupported projection shape, multi-parameter projection ordering/shape, wrong kind, and wrong arity.
2. Implement only the minimal fixes required to satisfy them.
3. Re-run `cargo test -p ash-typeck --test task_803_projection_diagnostics`, `cargo test -p ash-typeck --test task_803_phase110_non_interference`, and the exact parser rejection suite carried forward from TASK-797 or added here.
4. Review the resulting diff for accidental scope creep.

## Verification Steps

- [ ] `cargo test -p ash-typeck --test task_803_projection_diagnostics`
- [ ] `cargo test -p ash-typeck --test task_803_phase110_non_interference`
- [ ] `cargo test -p ash-parser --test task_803_phase110_rejection_boundaries` or the exact carried-forward TASK-797 rejection suite name, recorded in TASK-804
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

This task is test-heavy and should not invent later semantics merely to make a test green.
