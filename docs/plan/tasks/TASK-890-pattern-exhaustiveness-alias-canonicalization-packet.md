# TASK-890: Pattern/Exhaustiveness Alias Canonicalization Packet

## Status: ⏸️ Deferred

## Description

Audit and, if justified, plan the rollout of transparent alias/projection canonicalization into pattern checking and exhaustiveness after Phase 110 intentionally limited canonicalization adoption to equality boundaries.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [SPEC-058 §6](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md#6-required-invariants)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)

## Dependencies

- ✅ TASK-801/TASK-802 equality-boundary alias canonicalization
- ⏸️ Requires a fresh audit before implementation

## Requirements

1. Audit `crates/ash-typeck/src/check_pattern.rs`, `crates/ash-typeck/src/exhaustiveness.rs`, constructor resolution, and ADT pattern tests.
2. Decide whether these paths should consume `TypeEnv::canonicalize_type_for_equality` or a narrower pattern-specific canonicalization API.
3. Add positive alias-equivalent pattern/exhaustiveness cases and negative leakage cases for unrelated same-visible-name aliases.
4. Preserve current ADT/runtime pattern semantics and avoid solving under neutral computation heads.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 18
toolsets: [terminal, file]
```

## Verification

```
strictness: no-blocking
commands:
  - false # Deferred task: replace with concrete audit/spec commands when activated
checklist:
  - [ ] Live callsite audit complete
  - [ ] Canonicalization API decision recorded
  - [ ] Positive/negative pattern and exhaustiveness tests planned
  - [ ] Non-interference with existing ADT pattern matching specified
```

## Notes

Do not broaden Phase 110 by retroactive search-and-replace. Start with the audit and decide whether pattern/exhaustiveness needs the same canonicalization semantics as equality.
