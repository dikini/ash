# TASK-809: Core Domain Kind, IDs, and Summary Carriers

## Status: ✅ Complete

## Description

Add the `ash-core` identity, kind, and semantic-summary carriers required for sealed domains and marker constructors.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)

## Dependencies

- [TASK-807](TASK-807-sealed-domain-audit-gate.md)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Establish the canonical domain-kind and domain-summary substrate that later lowering, transport, and TypeEnv registration will consume.

## Requirements

1. Add canonical identities for sealed domains and marker constructors.
2. Extend the shared core `Kind` carrier with a nominal domain-kind form keyed by canonical domain identity.
3. Add domain-aware semantic-summary carriers separate from ordinary `TypeDeclSummary` / `ConstructorSummary`.
4. Add field metadata carriers with stable order, kind/domain reference, structural flag, and source-anchor support.
5. Preserve ordinary type/module summary behavior and current Phase 110 canonical IR ownership.
6. Do not add normalization, coverage, or runtime execution semantics.

## Files

- Modify: `crates/ash-core/src/kind.rs`
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-core/src/lib.rs`
- Add focused tests under `crates/ash-core/tests/`
- Add minimal compatibility shims in downstream crates only if required for compilation

## TDD Steps

1. Write failing tests for domain identity equality/hash/serde behavior, domain-kind round-tripping, and summary-carrier shape.
2. Implement the minimal core carrier changes.
3. Re-run focused `ash-core` tests plus downstream compile-compat tests if needed.
4. Stop before parser lowering or engine transport logic.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-core --test task_809_domain_kind_ids_red
  - cargo test -p ash-core
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [x] Focused domain-kind/ID tests pass
  - [x] Full ash-core suite passes
  - [x] Clippy clean
  - [x] Formatting clean
```

## Notes

`ash-core` substrate task. Marker constructors must not be modeled as ordinary ADT constructors.
