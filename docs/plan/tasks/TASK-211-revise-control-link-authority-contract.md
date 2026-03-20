# TASK-211: Revise Control-Link Authority Contract

## Status: ✅ Complete

## Description

Revise the canonical design, spec, and reference documents so `ControlLink` is no longer described
as a one-shot affine supervision token.

The updated contract should treat `ControlLink` as reusable control authority over a supervised
instance, with runtime validity determined by instance lifecycle and operation semantics rather
than unconditional consumption on first use.

This is a documentation and contract-alignment task. It exists to freeze the intended semantics
before runtime hardening work lands in `TASK-205`.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
- SPEC-020: ADT Types
- SPEC-021: Runtime Observable Behavior

## Reference and Design Context

- `docs/plan/2026-03-20-runtime-boundary-steering-brief.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`
- `docs/design/WORKFLOW_ADDRESS_SYSTEM.md`
- `docs/design/AWAIT_SEMANTICS.md`

## Requirements

### Functional Requirements

1. Replace the current affine/one-shot `ControlLink` story with reusable supervision authority
2. Specify which control operations are reusable and which terminal operations invalidate future
   control
3. Keep `ControlLink`, `InstanceAddr`, and `MonitorLink` distinct in purpose and semantics
4. Update design/spec/reference docs that currently encode affine control-link usage
5. Make the revised contract an explicit gate for runtime implementation in `TASK-205`

## Expected Documentation Scope

- Update canonical specs where control-link meaning is normative
- Update design notes that still explain control authority in affine terms
- Update frozen references where observable behavior or boundary language mentions control-link
  semantics
- Update planning/task dependencies so runtime implementation follows the revised contract

## Candidate Files

- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/design/WORKFLOW_ADDRESS_SYSTEM.md`
- Modify: `docs/design/AWAIT_SEMANTICS.md`
- Modify: `docs/plan/tasks/TASK-205-implement-runtime-action-and-control-link-execution.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD / Review Steps

### Step 1: Audit current contract drift

Identify all normative or guidance documents that still say:
- `ControlLink` is affine,
- every control operation consumes the link, or
- ongoing supervision is impossible after one control action.

### Step 2: Freeze the revised contract

Document the reusable-control model explicitly, including:
- non-terminal supervision operations,
- terminal invalidation behavior,
- separation from monitor authority.

### Step 3: Verify dependency alignment

Ensure `TASK-205` is explicitly blocked on this task and that the runtime hardening phase reads in
the correct order.

## Completion Checklist

- [x] control-link semantics audited across design/spec/reference docs
- [x] reusable supervision contract documented
- [x] runtime/task dependency notes updated
- [x] `CHANGELOG.md` updated

## Completion Notes

- Revised the canonical docs so `ControlLink` is reusable supervision authority rather than a
  one-shot affine control token.
- Updated `SPEC-020`, `SPEC-021`, and `SPEC-004` to distinguish non-terminal reusable control from
  terminal invalidation.
- Updated design notes and code documentation to remove the unconditional first-use-consumption
  story.
- Added historical notes to older completed planning/task artifacts that originally described the
  affine model.

## Non-goals

- No runtime implementation changes
- No new CLI/REPL surfacing
- No monitor-authority redesign
- No runtime↔reasoner contract changes

## Dependencies

- Enables: TASK-205
