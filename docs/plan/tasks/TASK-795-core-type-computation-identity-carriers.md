# TASK-795: Core Type-Computation Identity Carriers

## Status: 📝 Planned

## Description

Promote or add the `ash-core` identity carriers required for computation-grade canonical type IR, and move the single shared `Kind` definition into `ash-core`, without changing public syntax or module-summary computation export/import.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-794](TASK-794-type-expression-ir-and-kinding-audit-gate.md)

## Objective

Establish the canonical identity and shared-kind substrate that later canonical IR and rigid projections will use.

## Requirements

1. Reuse Phase 109 nominal identities instead of inventing parallel origin IDs.
2. Promote existing interface and associated-member identity carriers for internal computation-grade use.
3. Re-home the existing `Kind::Type` / `Kind::Arrow` model into `ash-core` so canonical IR can depend on one shared kind type.
4. Add any missing computation-head identity carrier needed by canonical IR.
5. Keep identity and shared-kind ownership in `ash-core`.
6. Do not add computation-summary export/import semantics.

## Files

- Modify: `crates/ash-core/src/semantic_summary.rs`
- Create or modify: `crates/ash-core/src/kind.rs`
- Create or modify: `crates/ash-core/src/type_ir.rs`
- Modify: `crates/ash-core/src/lib.rs`
- Modify: `crates/ash-typeck/src/kind.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Add tests in `ash-core` covering equality/hash/serde for new identities and the shared `Kind` carrier

## TDD Steps

1. Write focused failing tests for identity equality/hash/serde, shared `Kind` ownership/re-export behavior, and alias/re-export non-origin behavior.
2. Implement the minimal identity and shared-`Kind` changes to pass those tests.
3. Re-run focused `ash-core` and compatibility `ash-typeck` tests.
4. Stop before introducing canonical expression IR or lowering logic.

## Verification Steps

- [ ] `cargo test -p ash-core` for the new identity/shared-kind tests
- [ ] `cargo test -p ash-typeck` for the compatibility re-export/shim surface
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Cross-crate substrate task. Touch only `ash-core` and the minimal `ash-typeck` compatibility re-export/shim needed to keep downstream code compiling; do not touch parser or engine files here.
