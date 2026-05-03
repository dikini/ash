# TASK-794: Type-Expression IR and Kinding Audit Gate

## Status: ✅ Complete

## Description

Audit the live parser/core/typechecker representation stack before Phase 110 implementation begins. Freeze the exact contradictions, scope boundaries, and junior-safe implementation gate for SPEC-B.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-793](TASK-793-spec-b-spec-plan-packet.md)

## Objective

Produce the authoritative audit artifact that says what SPEC-B may change now, what must remain compatible, and what is explicitly deferred to SPEC-C/D/E/F/G.

## Requirements

1. Audit `ash-parser`, `ash-core`, and `ash-typeck` for current type-expression shapes, kind/arity entry points, alias canonicalization, and stringly associated projections.
2. Cross-reference live code against DESIGN-034, SPEC-003, SPEC-035, and SPEC-057.
3. Document contradictions and their required resolution strategy.
4. List exact implementation file targets for downstream tasks.
5. Explicitly mark out-of-scope work that belongs to later packets.
6. Explicitly call out the current feasibility blockers for TASK-800: shared `Kind` ownership, dual parser-path drift (`parse_type_def.rs` vs. `parse_module.rs`), and missing source/import plumbing for interface/member identities.

## Files

- Create: `docs/plan/audits/TASK-794-type-expression-ir-audit.md`
- Inspect: `crates/ash-parser/src/parse_type_def.rs`
- Inspect: `crates/ash-parser/src/parse_module.rs`
- Inspect: `crates/ash-parser/src/surface.rs`
- Inspect: `crates/ash-parser/src/lower.rs`
- Inspect: `crates/ash-core/src/ast.rs`
- Inspect: `crates/ash-core/src/lib.rs`
- Inspect: `crates/ash-core/src/semantic_summary.rs`
- Inspect: `crates/ash-core/Cargo.toml`
- Inspect: `crates/ash-typeck/src/types.rs`
- Inspect: `crates/ash-typeck/src/type_env.rs`
- Inspect: `crates/ash-typeck/src/kind.rs`
- Inspect: `crates/ash-typeck/src/lib.rs`
- Inspect: `crates/ash-typeck/Cargo.toml`

## TDD Steps

1. Write the audit first; no Rust files change in this task.
2. Verify every contradiction claim against a concrete file/section reference.
3. Record exact downstream file targets and non-goals.
4. Re-read the audit for scope-creep before marking the task complete.

## Verification Steps

- [x] `docs/plan/audits/TASK-794-type-expression-ir-audit.md` exists
- [x] audit references live files and current specs
- [x] audit explicitly names deferred work belonging to SPEC-C/D/E/F/G
- [x] `git diff --check` passes

## Notes

This is a docs-only gate task. If the audit still shows a `Kind` ownership split, parser-path drift, or missing interface/member summary plumbing, those issues must be resolved in the plan/task docs before TASK-800 begins.
