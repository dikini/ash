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

1. Add residual typechecker-side diagnostic coverage and negative tests for ambiguous projections, unsupported projection shapes, wrong kind, wrong arity, and any remaining explicit Phase 110 failure boundaries after TASK-800/TASK-799. Limit code changes to the minimal diagnostic text or assertion updates required by those tests.
2. Add non-interference coverage for Phase 109 ordinary-type behavior.
3. Add non-interference coverage for current ADT/interface/workflow/capability/resource/do/comprehension behavior as relevant.
4. Do not broaden semantics while writing tests.

## Files

- Add or update `crates/ash-typeck/tests/task_803_projection_diagnostics.rs` for ambiguity, unsupported-shape, wrong-kind, wrong-arity, and multi-parameter projection-spine coverage
- Add or update `crates/ash-typeck/tests/task_803_phase110_non_interference.rs` for carried-forward Phase 109 ordinary-type behavior plus representative ADT/interface/workflow/capability/resource/do/comprehension regressions
- Update diagnostics text only where needed
- Do not create a new parser rejection suite; TASK-797 owns parser rejection-boundary evidence

## TDD Steps

1. Write failing tests first in `task_803_projection_diagnostics.rs` for unary ambiguity, unsupported projection shape, multi-parameter projection ordering/shape, wrong kind, and wrong arity.
2. Implement only the minimal fixes required to satisfy them.
3. Re-run `cargo test -p ash-typeck --test task_803_projection_diagnostics` and `cargo test -p ash-typeck --test task_803_phase110_non_interference`.
4. Review the resulting diff for accidental scope creep.

## Verification Steps

- [ ] `cargo test -p ash-typeck --test task_803_projection_diagnostics`
- [ ] `cargo test -p ash-typeck --test task_803_phase110_non_interference`
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

This task is test-heavy and should not invent later semantics merely to make a test green. Parser rejection-boundary evidence is owned by TASK-797; this task covers typechecker diagnostics and non-interference only.
