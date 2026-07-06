# TASK-1928: Trusted Runtime Adapter Registry

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Add a trusted runtime adapter registry for host components that are admitted by the runtime but are
not Ash language primitives.

## Requirements

- Register runtime adapters with stable identity, version, trust source, admission source, sandbox
  policy, provenance policy, and report identity.
- Keep adapters separate from application entrypoints and language syntax.
- Require adapters to reference provider/builtin hook metadata before execution.
- Fail closed for unknown, stale, incompatible, or authority-widening adapters.

## TDD Steps

1. Add failing registry tests for adapter registration, lookup, stale identity, and authority
   widening.
2. Implement adapter registry carriers and runtime-state storage.
3. Wire reports/traces to adapter identity.

## Completion Checklist

- [x] Trusted adapters have stable identity and versioning.
- [x] Adapter admission is explicit and authority-neutral.
- [x] Unknown or stale adapters fail closed.
- [x] Reports/traces include adapter identity without leaking secrets.

## Evidence

- Added trusted runtime adapter carriers in `ash-core::runtime`: `TrustedRuntimeAdapterId`,
  `TrustedRuntimeAdapterTarget`, `TrustedRuntimeAdapter`, `TrustedRuntimeAdapterDiagnostic`, and
  `validate_trusted_runtime_adapter`.
- Runtime adapters now carry stable identity, version, trust source, admission source, sandbox
  policy, provenance policy, report identity, provider/builtin hook metadata target, and an explicit
  non-authority flag.
- `RuntimeState` now retains trusted runtime adapters separately from application entrypoints,
  external actor adapters, and provider registries.
- Adapter lookup fails closed for unknown and stale versions, and provider-operation validation
  rejects incompatible provider metadata before execution.
- Adapter registration emits redacted `TraceFactKind::Operation` registration facts with adapter
  name, version, and report identity but not trust-source secrets.
- Added TASK-1928 tests:
  - `cargo test -p ash-core --test task_1928_trusted_runtime_adapter_metadata`
  - `cargo test -p ash-interp --test task_1928_trusted_runtime_adapter_registry`
- Verified affected gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo check -p ash-core -p ash-interp`
  - `cargo clippy -p ash-core -p ash-interp --all-targets --all-features`
