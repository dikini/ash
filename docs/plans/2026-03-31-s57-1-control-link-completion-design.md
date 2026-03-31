# S57-1 Control-Link Completion Payload Design

## Context

TASK-S57-1 updates SPEC-004 so spawned workflow completion is normatively defined as a runtime-internal control-link contract. The current specs define spawn, control links, and observable control behavior, but they do not yet define the payload delivered to a supervisor when a spawned workflow reaches terminal completion.

This design closes that gap without introducing new surface syntax. Completion observation remains a runtime/supervisor mechanism, not a user-visible `await` construct.

## Goals

- Define that `spawn` creates control authority alongside the spawned instance.
- Define the terminal completion payload delivered through control authority.
- Define how a supervisor observes terminal completion and projects the child result.
- Cross-align the contract with role/obligation completion in SPEC-019 and observable-boundary rules in SPEC-021.
- Introduce a distinct terminal-control error variant now, rather than overloading `RuntimeFailure`.

## Non-Goals

- No new user-facing syntax.
- No change to AST or type system in this task.
- No attempt to expose full runtime trace as part of the completion payload.
- No broad refactor of all control/instance semantics beyond what is required for the completion contract.

## Design Decisions

### 1. Runtime-Internal Completion Contract

SPEC-004 will define a new runtime-internal contract:

- `spawn` yields an `Instance` paired with `ControlAuthority`.
- The control authority acts as the communication channel from the spawned workflow back to its supervisor.
- When the child workflow terminally completes, the runtime seals a `CompletionPayload` associated with that control authority.

This observation model is explicitly supervisor/runtime-internal and must not be read as surface syntax.

### 2. Completion Payload Shape

The terminal payload is a structured summary:

```text
CompletionPayload ::= {
  result: Result<Value, Error>,
  obligations: ObligationState,
  provenance: Provenance,
  effects: EffectTrace,
}
```

The payload contains:

- `result`: the terminal child outcome projected into `Result<Value, Error>`
- `obligations`: the detailed terminal obligation state, not merely a boolean
- `provenance`: the terminal provenance summary carried by the completed child
- `effects`: an `EffectTrace` summary of terminal effect behavior, not the full runtime trace

### 3. Summary, Not Full Trace

The payload does not carry the full execution `Trace`.

Rationale:

- full trace transport would over-specify runtime internals for this task;
- the supervisor contract only needs terminal result and terminal summaries;
- observable trace/export concerns already belong to other specs and tasks.

SPEC-004 will therefore define `EffectTrace` as a terminal effect summary and keep `Trace` as the execution relation’s internal trace sequence.

### 4. New Terminal-Control Error Variant

This task introduces a new `Error` variant for terminal control outcomes rather than collapsing them into `RuntimeFailure`.

Representative shape:

```text
TerminalControl(action, target, reason)
```

The exact prose will make clear that terminally imposed control outcomes, including supervisor-driven terminal interruption, are reported through this distinct error class when reflected into `CompletionPayload.result`.

### 5. Supervisor Observation Rule

SPEC-004 will add a small runtime-internal observation rule showing:

- a supervisor waiting on control authority for terminal completion,
- receipt of `CompletionPayload`, and
- projection of `payload.result` for supervisor-side completion handling.

This rule is descriptive of the runtime contract and will not create a new surface language form.

### 6. Cross-Spec Alignment

- SPEC-019: The payload’s `obligations` field must preserve the child’s terminal obligation state, including the completion constraints defined for role and local obligations.
- SPEC-021: Completion observation through control authority remains internal. Only externally surfaced values or failures become observable under SPEC-021’s runtime-visible contract.

## Planned Spec Edits

1. Update semantic domains in SPEC-004:
   - add `EffectTrace`
   - add `CompletionPayload`
   - extend `Error` with terminal-control failure
2. Add a new control-authority/completion subsection in SPEC-004 section 3.
3. Update rejection-boundary prose to include terminal-control completion failures.
4. Update spawn semantics so the runtime meaning includes paired control authority.
5. Add supervisor-only completion observation semantics with explicit runtime-internal wording.
6. Update changelog and task/status tracking documents required by repository policy.

## Risks and Mitigations

- **Risk:** accidental introduction of user-visible `await` semantics.
  - **Mitigation:** explicitly mark all completion observation notation as runtime-internal.
- **Risk:** ambiguity between `Trace`, `Provenance`, and `EffectTrace`.
  - **Mitigation:** specify that payload uses summary fields, not the full execution trace.
- **Risk:** conflict with future typing work.
  - **Mitigation:** state that if `CompletionPayload` becomes user-visible later, SPEC-003/SPEC-022 must be updated separately.

## Validation

The task is complete when:

- SPEC-004 normatively defines control authority creation on spawn.
- SPEC-004 normatively defines terminal completion observation through control authority.
- SPEC-004 defines `CompletionPayload` and `EffectTrace`.
- SPEC-004 includes a new terminal-control error variant.
- SPEC-004 shows a supervisor projecting `payload.result`.
- Cross-references to SPEC-019 and SPEC-021 are present.
- CHANGELOG and task tracking documents reflect the completed docs work.
