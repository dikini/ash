# TASK-179: Formalize Receive Mailbox and Scheduling Semantics

## Status: 📝 Planned

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

### Step 3: Implement the minimal spec fix (Green)

Tighten only `receive` semantics.

### Step 4: Verify GREEN

Expected pass conditions:
- `receive` no longer depends on implementation choice for its core contract.

### Step 5: Commit

```bash
git add docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/reference/parser-to-core-lowering-contract.md docs/reference/type-to-runtime-contract.md CHANGELOG.md
git commit -m "docs: formalize receive mailbox and scheduling semantics"
```

## Completion Checklist

- [ ] mailbox selection semantics documented
- [ ] source scheduling modifier semantics documented
- [ ] timeout/fallthrough semantics documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No scheduler implementation work
- No new surface syntax beyond what the specs standardize

## Dependencies

- Depends on: TASK-177, TASK-178
- Blocks: TASK-164, TASK-167, TASK-168, TASK-170, TASK-171, TASK-184
