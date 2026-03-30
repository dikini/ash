# TASK-S57-1: Update SPEC-004 with Control-Link Completion Payload Semantics

## Status: ⬜ Pending

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
  provenance: Provenance,        -- execution trace
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

## Open Questions

### Q1: Payload Contents
- Does completion payload include full trace or summary?
- Is obligation state just boolean (all discharged?) or detailed?
- What provenance information is included?

### Q2: Error Handling in Payload
- If workflow panics/terminates abnormally, what result?
- Is there a `Kill` or `Abort` variant in `Error`?

### Q3: User Visibility
- If CompletionPayload becomes user-visible, needs SPEC-003/SPEC-022 typing
- For now, keep as supervisor/runtime-internal

## Acceptance Criteria

- [ ] SPEC-004 defines control authority creation on spawn
- [ ] SPEC-004 defines terminal completion observation (runtime-internal)
- [ ] SPEC-004 defines `CompletionPayload` structure
- [ ] SPEC-004 shows supervisor extracting result from payload
- [ ] Explicitly notes: completion observation is supervisor/runtime-internal, not surface syntax
- [ ] Cross-references to SPEC-019 (role runtime) for supervision
- [ ] Cross-references to SPEC-021 for observable behavior

## Related

- SPEC-019: Role runtime semantics (supervision contracts)
- SPEC-021: Runtime observable behavior
- MCE-001: Entry point design (uses this contract)
- TASK-362: System supervisor (blocked on this)
- TASK-363: Runtime bootstrap (blocked on this)
- **Note:** If CompletionPayload becomes typed user value, coordinate with SPEC-003/SPEC-022

## Est. Hours: 4-6
