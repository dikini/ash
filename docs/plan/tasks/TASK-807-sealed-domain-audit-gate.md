# TASK-807: Sealed Domain Audit Gate

## Status: 📝 Planned

## Description

Audit the live parser/core/engine/typechecker substrate before Phase 111 implementation begins. Freeze the exact contradictions, scope boundaries, and junior-safe implementation gate for SPEC-C.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)

## Dependencies

- ✅ [TASK-806](TASK-806-spec-c-spec-plan-packet.md)

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Produce the authoritative audit artifact that says what Phase 111 may change now, what must remain compatible, and what is explicitly deferred to SPEC-D/E/F/G/H.

## Requirements

1. Audit `ash-parser`, `ash-core`, `ash-engine`, and `ash-typeck` for current declaration carriers, semantic-summary transport, kind ownership, import/export behavior, and registration seams relevant to sealed domains.
2. Cross-reference live code against DESIGN-034 §16.3, SPEC-057, and SPEC-058.
3. Document contradictions and their required resolution strategy.
4. List exact implementation file targets for downstream tasks.
5. Explicitly mark out-of-scope work that belongs to normalization, direct `type fn`, associated type-family computation, and promoted data kinds.
6. Explicitly call out the feasibility blockers for TASK-812: separate domain versus ordinary-constructor registries, summary-version evolution, and declare-then-validate handling for mutually recursive domain references.

## Files

- Create: `docs/plan/audits/TASK-807-sealed-domain-audit.md`
- Inspect: `crates/ash-parser/src/surface.rs`
- Inspect: `crates/ash-parser/src/parse_module.rs`
- Inspect: `crates/ash-parser/src/lower.rs`
- Inspect: `crates/ash-core/src/kind.rs`
- Inspect: `crates/ash-core/src/semantic_summary.rs`
- Inspect: `crates/ash-core/src/lib.rs`
- Inspect: `crates/ash-engine/src/module_loader.rs`
- Inspect: `crates/ash-engine/src/lib.rs`
- Inspect: `crates/ash-typeck/src/type_env.rs`
- Inspect: `crates/ash-typeck/src/kind.rs`
- Inspect: `crates/ash-typeck/src/lib.rs`

## TDD Steps

1. Write the audit first; no Rust files change in this task.
2. Verify every contradiction claim against a concrete file/section reference.
3. Record exact downstream file targets and non-goals.
4. Re-read the audit for scope-creep before marking the task complete.

## Verification

```
strictness: clean
commands:
  - git diff --check
checklist:
  - [ ] docs/plan/audits/TASK-807-sealed-domain-audit.md exists
  - [ ] audit references live files and current specs
  - [ ] audit explicitly names deferred work belonging to SPEC-D/E/F/G/H
```

## Notes

This is a docs-only gate task. If the audit still shows ordinary constructor/domain registry conflation, unclear summary-versioning, or missing declare-then-validate handling for recursive domains, those issues must be resolved in the plan/task docs before TASK-812 begins.
