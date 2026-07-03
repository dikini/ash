# TASK-1830: Check resource rows against resource authority

## Description

Wire resource row requirements to existing resource authority checks. Resource rows such as `resource vault write` or `resource fs read` must require that the host/runtime has selected an appropriate resource initializer/ownership; rows alone do not select resources.

## Owner decision gate

D4: How should resource rows map to current resource ownership/initializer selection?

## Requirements

- In the row-admission helper, derive resource requirements from explicit row metadata.
- For each resource row item, check against existing engine resource initializer selections or runtime resource authority.
- If a resource initializer is selected and covers the mode, admit.
- If no resource authority is present, reject with a precise `WorkflowFailureKind` diagnostic.
- If the resource admission path is not yet implemented, fail closed with an unsupported-requirement diagnostic rather than silently admitting.
- Add tests for missing, satisfied, and unsupported resource authority.

## Completion criteria

- [x] Resource row items are checked during admission.
- [x] Missing resource authority rejects with a structured diagnostic.
- [x] Satisfied resource selection admits through existing paths.
- [x] Unsupported paths fail closed with a diagnostic, not a silent admission.
- [x] Tests cover local and imported row-bearing callables.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1828 admission carrier.
