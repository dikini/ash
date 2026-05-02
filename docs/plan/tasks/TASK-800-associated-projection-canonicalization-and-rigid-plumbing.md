# TASK-800: Associated Projection Canonicalization and Rigid Plumbing

## Status: 📝 Planned

## Description

Replace stringly associated-projection handling in `ash-typeck` with canonical identity-backed rigid projection elaboration while preserving the current simple associated-type compatibility path.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-795](TASK-795-core-type-computation-identity-carriers.md)
- ✅ [TASK-796](TASK-796-core-canonical-type-expression-ir-and-neutral-carriers.md)
- ✅ [TASK-798](TASK-798-canonical-type-ir-lowering-from-surface-and-core.md)
- ✅ [TASK-799](TASK-799-kind-and-arity-validation-hardening.md)

## Objective

Make active typechecker projection handling canonical and identity-backed instead of stringly and sentinel-based.

## Requirements

1. Elaborate current associated projections into canonical identity-backed projection IR for both unary `S::Assoc` and multi-parameter `Base<A, B>::Assoc` forms, preserving the declaring-interface argument order and using source-local or imported `InterfaceIdentityId` / `AssociatedMemberIdentityId` entries already registered in `TypeEnv`.
2. Replace empty-string unresolved interface handling with explicit unresolved/ambiguous/resolved states.
3. Preserve current simple selected-impl associated-output substitution where already supported.
4. Keep unresolved generic projections rigid; do not add recursive family computation.

## Files

- Modify: `crates/ash-typeck/src/types.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Add focused `ash-typeck` tests for canonical projection handling

## TDD Steps

1. Write failing tests for unary projection resolution, multi-parameter projection argument-spine ordering, ambiguous projection rejection, and rigid generic projections.
2. Implement the minimal canonicalization/elaboration changes.
3. Re-run focused projection tests.
4. Verify the current simple associated-output path still works.

## Verification Steps

- [ ] `cargo test -p ash-typeck` for the new projection tests
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

This task assumes TASK-798 already emits/registers interface/member identities for both source and imported summaries, and TASK-799 already validates projection spines. Do not reinterpret this as general normalization.
