# TASK-798: Canonical Type IR Lowering from Surface and Core

## Status: 📝 Planned

## Description

Introduce the main lowering boundary from current surface/core type syntax into the new canonical type IR, and make `TypeEnv` own the interface/member identity registries, storage, and source/import registration that later canonical projection elaboration will consume.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-796](TASK-796-core-canonical-type-expression-ir-and-neutral-carriers.md)
- ✅ [TASK-797](TASK-797-ordinary-type-parser-expression-parity-and-explicit-rejections.md)

## Objective

Make `ash-typeck` consume one canonical internal representation instead of parallel ad hoc shapes, with `TypeEnv` providing one shared interface/member identity space for source-local and imported projections.

## Requirements

1. Lower `ash_core::ast::TypeExpr` into canonical IR.
2. Lower the current relevant surface type subset into the same canonical IR where needed.
3. Define `TypeEnv` storage/registry structures for interface and associated-member identities.
4. Register source-local interface/member identities as ordinary summaries are produced.
5. Register imported interface/member identities into the same `TypeEnv` registries so later projection elaboration resolves source-local and imported code against one identity space.
6. Preserve existing ordinary nominal type behavior.
7. Do not replace live `Type::Associated { interface: String, ... }` / empty-sentinel projection consumers or projection-specific diagnostics; that boundary belongs to TASK-800.
8. Do not add normalization, equality rollout, or engine-facing computation summaries.
9. Treat this as ordinary summary identity plumbing, not computation-summary export/import.

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Create or modify: `crates/ash-typeck/src/type_ir_lower.rs`
- Add focused tests for source-local and imported identity registration plus canonical lowering
- Do not use this task to replace projection-specific string/sentinel consumers in `types.rs`, `error.rs`, or `diagnostic.rs`; that belongs to TASK-800

## TDD Steps

1. Write failing lowering tests for nominal applications, aliases, source-local identity registration, imported identity registration, and source-vs-imported canonical lookup through the same `TypeEnv` registry.
2. Implement the canonical lowering path.
3. Re-run focused lowering tests.
4. Confirm ordinary ADT/type flows still work.

## Verification Steps

- [ ] `cargo test -p ash-parser` for source-summary lowering coverage
- [ ] `cargo test -p ash-typeck` for the new lowering tests
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Cross-crate lowering/plumbing task. Junior implementers should stay inside parser-lowering, canonical-lowering, and `TypeEnv` registry/storage/registration boundaries and avoid replacing live projection consumers, unification behavior, or diagnostics in this task.
