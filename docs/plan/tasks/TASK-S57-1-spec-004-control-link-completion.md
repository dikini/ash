# TASK-S57-1: Update SPEC-004 with Control-Link Completion Payload Semantics

## Status: ✅ Complete

## Description

Update SPEC-004 (Operational Semantics) with normative semantics for control-link completion, including the terminal payload that flows from a completed workflow to its supervisor.

## Background

Per architectural review, the supervisor/main contract is currently underspecified:

- What information flows over the control link when a workflow completes?
- How does the supervisor observe terminal completion?
- What is the structure of the completion payload?

**Current state:** SPEC-004 has `spawn` and control links mentioned but does not normatively define completion payload.

**Target state:** SPEC-004 explicitly defines:

```
spawn creates (Instance, ControlAuthority)
ControlAuthority yields CompletionPayload upon terminal completion
CompletionPayload = (Result<Value, Error>, ObligationState, Provenance, EffectTrace)
```

**Note:** This task specifies *runtime/internal* semantics, not user-facing syntax. The `await` construct mentioned below is supervisor-internal notation, not surface language syntax (current AST has no user-visible `await`).

## Requirements

Update SPEC-004 with:

1. **Control authority creation**: `spawn` creates control authority alongside instance
2. **Completion observation**: Supervisor observes terminal completion via control authority
3. **Payload structure**: Define what the payload contains
4. **Supervisor contract**: Supervisor receives payload, extracts result

## SPEC-004 Sections to Update

### Section 3.x: Control Authority Semantics (NEW)

Add new section defining:

- Control authority as communication channel between spawner and spawned
- Terminal completion protocol
- Payload structure

### Section 3.x: Completion Payload (NEW)

Define:

```
CompletionPayload ::= {
  result: Result<Value, Error>,
  obligations: ObligationState,  -- discharged or pending
  provenance: Provenance,        -- terminal provenance summary
  effects: EffectTrace,          -- effect summary
}
```

### Update Existing Spawn Rule

Current (from SPEC-022):

```
Γ ⊢ spawn w : Handle<τ> ▷ Γ
```

Add semantics:

```
Γ ⊢ spawn w : Instance<τ> × ControlAuthority ▷ Γ
ControlAuthority observes: CompletionPayload
```

## Resolved Design Choices

### Payload Contents

- `CompletionPayload.effects` carries an `EffectTrace` summary only; the full execution `Trace`
  remains workflow-internal and is not transported over the control link.
- `CompletionPayload.obligations` carries the child's authoritative terminal `ObligationState`,
  not a boolean summary.
- `CompletionPayload.provenance` carries the child's terminal `Provenance` value.

### Error Handling in Payload

- Abnormal or interrupted terminal completion is reported through
  `CompletionPayload.result = Err(...)`.
- SPEC-004 now uses `TerminalControl(action, target, reason)` as the terminal-control-owned
  error contract instead of introducing separate `Kill` or `Abort` variants.

### User Visibility

- Completion observation remains supervisor/runtime-internal only.
- The new contract does not add surface `await` syntax and does not make
  `CompletionPayload` a typed user-visible workflow value; any future surface exposure would
  require coordinated SPEC-003/SPEC-022 updates.

## Acceptance Criteria

- [x] SPEC-004 defines control authority creation on spawn
- [x] SPEC-004 defines terminal completion observation (runtime-internal)
- [x] SPEC-004 defines `CompletionPayload` structure
- [x] SPEC-004 shows supervisor extracting result from payload
- [x] Explicitly notes: completion observation is supervisor/runtime-internal, not surface syntax
- [x] Cross-references to SPEC-019 (role runtime) for supervision
- [x] Cross-references to SPEC-021 for observable behavior

## Related

- SPEC-019: Role runtime semantics (supervision contracts)
- SPEC-021: Runtime observable behavior
- MCE-001: Entry point design (uses this contract)
- TASK-362: System supervisor (blocked on this)
- TASK-363: Runtime bootstrap (blocked on this)
- **Note:** If CompletionPayload becomes typed user value, coordinate with SPEC-003/SPEC-022

## Est. Hours: 4-6
