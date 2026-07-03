# TASK-1829: Check operation rows against provider/capability admission

## Description

- Wire operation row requirements to existing provider/operation admission checks. Operation rows such as `PosixFs::read` or `hostfs.read` must require that a matching operation provider is already registered; rows alone do not register providers. Operation row identities are interface/impl-qualified per NOTE-022/025; the runtime "provider" here is an already-registered host/runtime authority, not a deprecated `capability binding`.

## Owner decision gate

D3: How should operation rows map to current capability/provider admission?

## Requirements

- In `Engine::admit_workflow` or a new row-admission helper, derive operation requirements from the workflow's `callable_row_requirements` and `core_callable_types`.
- For each operation row item, look up the provider/capability binding through existing engine/runtime APIs.
- If the provider is missing, reject admission with a precise `WorkflowFailureKind` diagnostic naming the missing capability and callable.
- If the provider is present, admit through existing paths.
- Add tests for both missing and satisfied operation authority.

## Completion criteria

- [x] Operation row items are checked during admission when `Engine::admit_workflow` is invoked with a workflow carrying explicit rows.
- [x] Missing operation provider rejects with a structured diagnostic.
- [x] Satisfied operation provider admits through existing provider/capability paths.
- [x] Tests cover local and imported row-bearing callables.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1828 admission carrier.
