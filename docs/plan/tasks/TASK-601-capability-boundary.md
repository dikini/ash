# TASK-601: Capability Boundary Audit

## Status: 📝 Planned

## Description

Implement the `capability_boundary.ash` mechanism and the audit rule that flips flags when language substrates are verified.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track C
- DESIGN-SPEC-PROCESSOR.md §7

## Dependencies

- All Track B tasks (except deferred TASK-599)

## Requirements

1. Declare `expected_capabilities` record in `apps/spec_processor/capability_boundary.ash`.
2. If a capability is `false`, skip dependent validation and apply workaround.
3. If a capability is `true` but fails at runtime, emit `ToolingGap` finding.
4. The boundary module itself is auditable.

## TDD Steps

### Step 1: Write failing test

Set `regex_matching: false`, run link validation, assert no findings (skipped).

### Step 2: Implement

Create `apps/spec_processor/src/capability_boundary.ash`.

### Step 3: Verify

Flip flag to `true`, run link validation on broken spec, assert findings emitted.

## Verification Steps

- [ ] Flag `false` skips validation
- [ ] Flag `true` enables validation
- [ ] Codex verification: VERIFIED
