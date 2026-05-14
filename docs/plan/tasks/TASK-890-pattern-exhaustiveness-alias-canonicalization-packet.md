# TASK-890: Pattern/Exhaustiveness Alias Canonicalization Packet

## Status: ✅ Complete

## Description

Promote a future implementation-grade DESIGN/SPEC/PLAN packet for transparent alias/projection canonicalization in pattern checking and exhaustiveness. The live callsite audit and implementation remain planned in SPEC-068 / PLAN-117.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [SPEC-058 §6](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md#6-required-invariants)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)

## Dependencies

- ✅ TASK-801/TASK-802 equality-boundary alias canonicalization
- ✅ Design/spec/plan packet created; live audit is scheduled as the first implementation task

## Requirements

1. Create a design note that preserves the audit-first boundary for pattern/exhaustiveness canonicalization.
2. Create SPEC-068 with a mandatory audit gate before implementation.
3. Create PLAN-117 and TASK-912 through TASK-917 with explicit planned positive/negative evidence ownership.
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
  - git diff --check
checklist:
  - [x] Live callsite audit scheduled as first implementation task
  - [x] Canonicalization API decision gate recorded
  - [x] Positive/negative pattern and exhaustiveness tests planned
  - [x] Non-interference with existing ADT pattern matching specified
```

## Notes

Do not broaden Phase 110 by retroactive search-and-replace. Start with the audit and decide whether pattern/exhaustiveness needs the same canonicalization semantics as equality.

## Completion Notes

Activated as [DESIGN-039](../../design/DESIGN-039-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md), [SPEC-068](../../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md), [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md), with implementation task range TASK-912 through TASK-917. This task completed the docs/spec/plan packet only; feature implementation remains planned in the new task range.
