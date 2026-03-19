# TASK-179: Formalize Receive Mailbox and Scheduling Semantics

## Status: ✅ Complete

## Description

Tighten `receive` so the contract is precise about mailbox search, source scheduling modifier,
guard timing, consumption timing, timeouts, and control-stream behavior.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
- SPEC-013: Streams and Event Processing
- SPEC-017: Capability Integration

## Requirements

### Functional Requirements

1. Define one precise `receive` selection model
2. Make source scheduling modifier semantics explicit
3. Define guard-evaluation and message-consumption points
4. Define timeout/fallthrough behavior without local-runtime discretion

## Files

- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for ambiguity in:
- source selection,
- arm order,
- guard point,
- consumption point,
- timeout behavior.

### Step 2: Verify RED

Expected failure conditions:
- at least one of those behaviors still depends on local interpretation.

Observed before implementation:
- SPEC-013 still mixed a single-mailbox story with scheduler-driven source selection.
- source selection, guard timing, consumption timing, and timeout/fallthrough behavior were only
  partially explicit across SPEC-002, SPEC-004, SPEC-013, and SPEC-017.

### Step 3: Implement the minimal spec fix (Green)

Tighten only `receive` semantics.

### Step 4: Verify GREEN

Expected pass conditions:
- `receive` no longer depends on implementation choice for its core contract.

Verified after implementation:
- SPEC-013 now defines one source-selection model over declared stream mailboxes plus the implicit
  control mailbox.
- SPEC-013 now states the current default `priority` source scheduling modifier and the
  guard-before-consumption timing.
- SPEC-004 and the runtime reference now treat timeout expiry and receive fallthrough as normal
  control flow, not rejection.
- SPEC-017 and the lowering/type-runtime references now point to the runtime-owned scheduler
  contract rather than inventing their own receive semantics.

### Step 5: Commit

```bash
git add docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/reference/parser-to-core-lowering-contract.md docs/reference/type-to-runtime-contract.md CHANGELOG.md
git commit -m "docs: formalize receive mailbox and scheduling semantics"
```

## Completion Checklist

- [x] mailbox selection semantics documented
- [x] source scheduling modifier semantics documented
- [x] timeout/fallthrough semantics documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No scheduler implementation work
- No new surface syntax beyond what the specs standardize

## Dependencies

- Depends on: TASK-177, TASK-178
- Blocks: TASK-164, TASK-167, TASK-168, TASK-170, TASK-171, TASK-184
