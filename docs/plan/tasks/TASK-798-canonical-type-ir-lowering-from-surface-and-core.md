# TASK-798: Canonical Type IR Lowering from Surface and Core

## Status: 📝 Planned

## Description

Introduce the main lowering boundary from current surface/core type syntax into the new canonical type IR, and plumb interface/member identities from source lowering and imported ordinary summaries into that boundary.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-796](TASK-796-core-canonical-type-expression-ir-and-neutral-carriers.md)
- ✅ [TASK-797](TASK-797-ordinary-type-parser-expression-parity-and-explicit-rejections.md)

## Objective

Make `ash-typeck` consume one canonical internal representation instead of parallel ad hoc shapes, with the same interface/member identities available for source-local and imported projections.

## Requirements

1. Lower `ash_core::ast::TypeExpr` into canonical IR.
2. Lower the current relevant surface type subset into the same canonical IR where needed.
3. Preserve existing ordinary nominal type behavior.
4. Do not add normalization, equality rollout, or engine-facing computation summaries.
5. Emit `InterfaceIdentitySummary` and `AssociatedMemberIdentitySummary` from source lowering where Phase 109 summaries are produced.
6. Register imported interface/member identities in `TypeEnv` so canonical lowering can resolve source-local and imported projections against the same identity space.
7. Treat this as ordinary summary identity plumbing, not computation-summary export/import.

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Create or modify: `crates/ash-typeck/src/type_ir_lower.rs`
- Add focused tests for source-local and imported identity-backed lowering

## TDD Steps

1. Write failing lowering tests for nominal applications, current associated projections, aliases, and source-vs-imported projection identity lookup.
2. Implement the canonical lowering path.
3. Re-run focused lowering tests.
4. Confirm ordinary ADT/type flows still work.

## Verification Steps

- [ ] `cargo test -p ash-parser` for source-summary lowering coverage
- [ ] `cargo test -p ash-typeck` for the new lowering tests
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Cross-crate lowering/plumbing task. Junior implementers should stay inside parser-lowering and typechecker-lowering boundaries and avoid unrelated unification or engine changes.
