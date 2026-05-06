# TASK-817: Normalizer / Definitional Equality Audit Gate

## Status: ✅ Complete

## Description

Audit the live canonicalization, equality, normalization-adjacent, and forcing-point seams before Phase 112 Rust implementation begins.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Audit the live canonicalization, equality, normalization-adjacent, and forcing-point seams before Phase 112 Rust implementation begins.

## Requirements

1. Inspect current ash-core type_ir and semantic_summary carriers.
2. Inspect TypeEnv canonicalization/equality functions and all current callsites.
3. Map expression-checking, return-checking, impl-overlap, projection, and final rendering seams into an explicit forcing-point matrix with exact functions/callsites, ownership, and deferred/fallback status.
4. Document exact file targets for downstream tasks.
5. Identify compatibility constraints for ordinary nominal constructor unification and SPEC-035 simple associated outputs.
6. Map the distinction between canonical abstract variables (`CanonicalTypeExpr::Var(String)`) and current inference metas (`Type::Var(TypeVar)`) before TASK-825.
7. Identify exact rendering callsites (`TypeEnv::render_type_for_diagnostics` and any direct `to_string()` diagnostics) that TASK-826/TASK-827 may touch.
8. Create docs/plan/audits/TASK-817-normalizer-defeq-audit.md.

## Files

- Create: `docs/plan/audits/TASK-817-normalizer-defeq-audit.md`
- Inspect: `crates/ash-core/src/type_ir.rs`
- Inspect: `crates/ash-core/src/semantic_summary.rs`
- Inspect: `crates/ash-typeck/src/type_env.rs`
- Inspect: `crates/ash-typeck/src/types.rs`
- Inspect: `crates/ash-typeck/src/check_expr.rs`
- Inspect: `crates/ash-typeck/src/error.rs`
- Inspect: `crates/ash-typeck/src/diagnostic.rs`

## TDD Steps

1. Write the audit/docs first; no Rust files change in this task.
2. Verify every claim against live files.
3. Re-read for scope creep before marking complete.

## Verification

```
strictness: clean
commands:
  - git diff --check
checklist:
  - [x] Audit artifact exists
  - [x] Audit cites live files/functions
  - [x] Audit includes an exact forcing-point matrix consumed by TASK-826
  - [x] Audit marks public type fn/source syntax/export work out of scope
  - [x] Audit maps canonical abstract variables versus inference metas
  - [x] Audit names rendering callsites and deferred/fallback callsites
```

## Notes

Task type: Docs/Substrate. Estimated effort: 4 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.

## Completion Notes

Completed in this slice by creating [`../audits/TASK-817-normalizer-defeq-audit.md`](../audits/TASK-817-normalizer-defeq-audit.md). The audit records the exact live `ash-core` and `ash-typeck` normalizer/equality seams, the TASK-826 forcing-point matrix with deferred/fallback status, the canonical abstract-variable versus inference-meta boundary, and the selected rendering callsites. Public `type fn` source syntax, source equations, recursive associated-family computation, and equation export/import remain out of scope.
