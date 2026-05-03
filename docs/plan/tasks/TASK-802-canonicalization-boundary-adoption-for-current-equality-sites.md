# TASK-802: Canonicalization Boundary Adoption for Current Equality Sites

## Status: ✅ Complete

## Description

Adopt the new canonicalization helpers only at the named current equality boundaries `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`, both routed through `TypeEnv::canonicalize_type_for_equality`, preserving current ordinary constructor behavior and stopping short of normalization.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-800](TASK-800-associated-projection-canonicalization-and-rigid-plumbing.md)
- ✅ [TASK-801](TASK-801-transparent-alias-canonicalization-helper.md)

## Objective

Use the TASK-800 projection plumbing and TASK-801 alias helper layer in the active TypeEnv equality hooks that Phase 110 must make canonical.

## Requirements

1. Route the named current equality boundaries `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality` through `TypeEnv::canonicalize_type_for_equality`.
2. Ensure those boundaries consume transparent-alias canonical heads from TASK-801 and canonical rigid projection forms from TASK-800.
3. Preserve ordinary nominal constructor decomposition behavior under the underlying unifier.
4. Do not widen this task into `check_pattern.rs` or `exhaustiveness.rs`; those modules are not current Phase 110 canonicalization boundaries.
5. Do not add full definitional equality or normalization, or any comparison, decomposition, or solving rule under neutral computation-head applications.

## Files

- Modify: `crates/ash-typeck/src/type_env.rs`
- Add focused `ash-typeck` equality-boundary tests

## TDD Steps

1. Write failing tests showing alias-aware and projection-aware equality through `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`.
2. Add negative tests proving unresolved/neutral forms still do not normalize, invert, compare, or solve under neutral computation heads.
3. Implement the minimal `type_env.rs` boundary adoption changes.
4. Reconfirm ordinary constructor cases are unchanged and that pattern/exhaustiveness paths are untouched by this task.

## Verification Steps

- [x] `cargo test -p ash-typeck` for the new boundary tests
- [x] `cargo fmt --check`
- [x] `git diff --check`

## Notes

Single-crate task. If the work starts to require changing `check_pattern.rs`, `exhaustiveness.rs`, or reasoning under a neutral computation head to pass, stop: that belongs to a later packet.
