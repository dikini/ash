# TASK-810: Domain Lowering and Summary Versioning

## Status: ✅ Complete

## Description

Lower parsed sealed-domain declarations into core semantic summaries and advance summary-version handling for the new domain metadata surface.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)

## Dependencies

- [TASK-808](TASK-808-parser-surface-for-sealed-type-domains.md)
- [TASK-809](TASK-809-core-domain-kind-ids-and-summary-carriers.md)

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

Make parsed domain declarations flow through the same ModuleFile-to-core-summary path as other semantic declarations, with explicit summary-version evolution.

## Requirements

1. Lower parser surface domain declarations into core-owned domain summary carriers.
2. Preserve domain/constructor/field visibility, source anchors, canonical identities, and declaration order.
3. Advance the semantic-summary version to reflect the new domain metadata contract.
4. Reject malformed or partial summary-version mixes explicitly.
5. Preserve existing ordinary type lowering and ordinary summary generation.
6. Do not implement engine import/export behavior or TypeEnv registration in this task.

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify any summary-version helpers/consumers touched by the lowering path
- Add focused tests in parser/core crates for lowering and versioned summary output

## TDD Steps

1. Write failing lowering/versioning tests for public exposed domains, opaque-export candidates, recursive references, and version mismatch rejection.
2. Implement the minimal lowering and versioning changes.
3. Re-run focused lowering/versioning tests.
4. Confirm ordinary type-lowering outputs remain stable.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-parser --test task_810_domain_lowering
  - cargo test -p ash-core
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [x] Focused lowering/versioning tests pass
  - [x] Full ash-core suite passes
  - [x] Ordinary type-lowering outputs stable
  - [x] Clippy clean
  - [x] Formatting clean
```

## Notes

Lowering/versioning task only. Do not add engine import/export behavior or `TypeEnv` registration here.
