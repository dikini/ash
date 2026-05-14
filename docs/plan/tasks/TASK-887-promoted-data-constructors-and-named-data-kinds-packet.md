# TASK-887: Promoted Data Constructors and Named Data Kinds Packet

## Status: ✅ Complete

## Description

Promote a future implementation-grade SPEC/PLAN packet for promoted ADT/runtime constructors and named data kinds, if Ash chooses to support DataKinds-style promotion beyond sealed-domain marker constructors.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ SPEC-057 through SPEC-064 implemented MVPs
- ✅ Design/spec/plan packet created; feature implementation remains in the new planned task range

## Requirements

1. Decide whether Ash should promote existing ADT/runtime constructors, add separate type-level constructor declarations, or keep sealed-domain marker constructors as the only type-level data constructors.
2. Define identity, visibility, module-summary, kinding, pattern, normalization, and diagnostics rules.
3. Preserve the current invariant that sealed-domain marker constructors are not ordinary runtime `ConstructorId`s.
4. Include migration/non-interference tests for existing ADT construction, pattern matching, and exhaustiveness.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: no-blocking
commands:
  - git diff --check
checklist:
  - [x] SPEC packet written
  - [x] PLAN packet written
  - [x] Task range created with audit-bound file/test targeting
  - [x] Non-interference with sealed-domain markers and runtime ADTs specified
```

## Notes

Do not implement this under any existing SPEC-A through SPEC-H task. This is a new language-feature packet.

## Completion Notes

Activated as [DESIGN-036](../../design/DESIGN-036-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md), [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md), [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md), with implementation task range TASK-892 through TASK-897. This task completed the docs/spec/plan packet only; feature implementation remains planned in the new task range.
