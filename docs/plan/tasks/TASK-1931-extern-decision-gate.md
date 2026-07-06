# TASK-1931: Extern Decision Gate

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Decide whether an `extern` surface is still needed after explicit provider, builtin hook, trusted
adapter, sandbox, and provenance work.

## Requirements

- Compare current and target host use cases against provider/builtin/adapter coverage.
- Document whether `extern` is deferred, rejected for the MVP, or retained for a later phase.
- If retained, specify that `extern` lowers to the trusted adapter/provider substrate and cannot
  call the host directly.
- Add fail-closed diagnostics for any parsed or reserved `extern` syntax until implementation is
  explicitly planned.

## TDD Steps

1. Add tests documenting current `extern` behavior or reserved-word diagnostics.
2. Write the decision note in the task evidence.
3. Update PLAN-197 and any relevant specs/notes with the decision.

## Completion Checklist

- [x] Decision is documented.
- [x] `extern` does not bypass admission or sandboxing.
- [x] Any unimplemented `extern` path fails closed.
- [x] Follow-up ownership is recorded if `extern` remains future work.

## Decision

`extern` is rejected for the Phase 197 MVP. Current host use cases are covered by the explicit
provider authoring API, builtin host-hook metadata, trusted runtime adapters, sandbox policies, and
redacted host-boundary evidence. Adding an `extern` surface now would create a second host-call path
before the provider/adapter substrate has closeout fixtures.

`extern` remains reserved future vocabulary only. If a later phase proves it is still needed, it
must lower to a trusted runtime adapter and provider operation metadata. It must not call host code
directly, allocate authority, bypass row admission, skip sandbox policy, or omit redacted
provenance/report evidence.

## Evidence

- Added parser regression coverage proving `extern fn` remains unavailable:
  - `cargo test -p ash-parser --test task_1931_extern_reserved`
- Verified affected gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo check -p ash-parser`
  - `cargo clippy -p ash-parser --all-targets --all-features`
- Current Phase 197 host coverage is through TASK-1926 builtin host-hook metadata, TASK-1927
  provider authoring metadata, TASK-1928 trusted runtime adapters, TASK-1929 sandbox enforcement,
  and TASK-1930 host-boundary evidence.
- Follow-up ownership: any future `extern` proposal belongs in a later phase after Phase 197
  closeout fixtures, and must explicitly reuse the trusted adapter/provider substrate.
