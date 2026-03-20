# Control-Link Retention Policy

## Status

TASK-212 handoff reference.

## Purpose

This document freezes the canonical retention policy for runtime-owned `ControlLink` lifecycle
state after terminal supervision operations.

It complements:

- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

## Canonical Policy

### 1. Ownership

`ControlLink` lifecycle state is owned by the `RuntimeState` instance that backs related workflow
executions.

- reusing the same `RuntimeState` preserves control-link lifecycle state across executions
- dropping that `RuntimeState` releases the retained control-link state it owns

No language-level workflow step currently performs per-link cleanup.

### 2. Live-Link Semantics

While a target remains live, `ControlLink` authority is reusable.

- `check_health` is non-terminal
- `pause` is non-terminal
- `resume` is non-terminal
- `kill` is terminal

This preserves the reusable-supervision contract frozen by TASK-211.

### 3. Terminal Retention

After `kill`, the target transitions to a retained terminal tombstone inside the owning
`RuntimeState`.

The tombstone policy is:

- `kill` records terminal state rather than deleting the control entry immediately
- later control attempts against that link while the same `RuntimeState` remains alive must fail as
  explicit terminal-control failures
- those failures must not silently degrade into `NotFound` due to background cleanup while the same
  runtime state is still active

In other words, terminal observability is preserved for the lifetime of the current runtime state.

### 4. Cleanup Boundary

The current cleanup boundary is whole-state teardown:

- when the owning `RuntimeState` is dropped, its retained control-link state is dropped with it
- there is no background tombstone scavenger in the current runtime contract
- there is no eager per-link cleanup path in the current language/runtime surface

This freezes a safe and deterministic baseline without widening runtime behavior beyond the current
implementation.

### 5. Diagnostics And Observable Behavior

While a retained tombstone remains in the owning `RuntimeState`:

- terminal follow-up control attempts must fail explicitly
- the failure class remains terminal-instance failure rather than unknown-link failure
- the observable contract is therefore stable across repeated executions using the same runtime
  state

If a future runtime-maintenance feature adds explicit compaction, it must not silently alter the
observable meaning of a still-owned live runtime state.

### 6. Provenance And Trace Interaction

The current runtime contract requires retention semantics to remain compatible with explicit
runtime-boundary diagnostics, but does not yet require tombstone compaction events to be traced
because no compaction surface exists today.

If a future explicit cleanup or compaction mechanism is added, that work must define:

- whether cleanup events are trace-visible
- whether provenance records terminal retention separately from terminal execution
- how compaction preserves or intentionally bounds diagnostic visibility

## Non-Goals

This policy does not:

- introduce new runtime/reasoner interaction semantics
- require child-instance execution handles or mailbox-level teardown semantics
- change the current runtime behavior established by TASK-206

## Follow-Up Boundary

This reference freezes the current long-term safe baseline:

- reusable live supervision
- retained terminal tombstones for the lifetime of the owning runtime state
- cleanup by explicit runtime-state teardown

If long-lived embedders later need bounded tombstone compaction within a still-live runtime state,
that should be added as an explicit runtime-maintenance feature, not as hidden background cleanup.
