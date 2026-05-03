# TASK-800: Associated Projection Canonicalization and Rigid Plumbing

## Status: ✅ Complete

## Description

Consume the registries landed by TASK-798 and replace every live stringly/sentinel associated-projection surface in `ash-typeck` with canonical identity-backed rigid projection elaboration, while owning projection-specific unresolved/ambiguous/unsupported-shape diagnostics and preserving the current simple associated-type compatibility path.

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

Make all active typechecker projection handling canonical and identity-backed instead of stringly and sentinel-based.

## Requirements

1. Replace all live projection carriers and consumers that still depend on interface strings or empty-string sentinels, including `Type::Associated { interface: String, ... }`, source/surface lowering handoff placeholders, projection comparison/unification paths, and projection-specific equality canonicalization inputs.
2. Elaborate current associated projections into canonical identity-backed projection IR for both unary `S::Assoc` and multi-parameter `Base<A, B>::Assoc` forms, preserving the declaring-interface argument order and using pre-registered `InterfaceIdentityId` / `AssociatedMemberIdentityId` entries from `TypeEnv`.
3. Own projection-specific error and diagnostic behavior: unresolved, ambiguous, unsupported-shape, and resolved states must be explicit and must not be represented by string or sentinel conventions.
4. Unsupported-shape diagnostics must cover syntactically admitted but unsupported projection bases such as `(S::Item)::Assoc` and `Map<K, V>::Entry::Assoc`.
5. Preserve current simple selected-impl associated-output substitution where already supported.
6. Keep unresolved generic projections rigid; do not add recursive family computation.
7. Do not add new parser rejection coverage; parser rejection-boundary evidence belongs to TASK-797.

## Files

- Modify: `crates/ash-typeck/src/types.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/error.rs`
- Modify: `crates/ash-typeck/src/diagnostic.rs`
- Add focused `ash-typeck` tests for canonical projection handling and projection diagnostics

## TDD Steps

1. Write failing tests for all remaining live projection surfaces: unary projection resolution, multi-parameter projection argument-spine ordering, unresolved-vs-ambiguous diagnostic separation, unsupported-shape diagnostics for `(S::Item)::Assoc` and `Map<K, V>::Entry::Assoc`, and rigid generic projections.
2. Implement the minimal canonicalization/elaboration changes.
3. Re-run focused projection tests.
4. Verify the current simple associated-output path still works.

## Verification Steps

- [x] `cargo test -p ash-typeck` for the new projection tests
- [x] `cargo fmt --check`
- [x] `git diff --check`

## Notes

This task assumes TASK-798 already emits/registers interface/member identities for both source and imported summaries, and TASK-799 already validates projection spines. It is the only Phase 110 task allowed to replace live stringly/sentinel projection surfaces or projection-specific diagnostics. Do not reinterpret this as general normalization.
