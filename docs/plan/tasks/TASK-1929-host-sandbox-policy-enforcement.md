# TASK-1929: Host Sandbox Policy Enforcement

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Enforce sandbox policy before host-facing providers, builtin hooks, or runtime adapters execute.

## Requirements

- Cover filesystem paths, process commands, network hosts, environment variables, clocks/time,
  LLM/MCP calls, and other host-facing resources.
- Evaluate sandbox constraints before host effects occur.
- Preserve denied-attempt evidence with redaction.
- Compose sandbox policy with provider authoring metadata, application admission profiles, row
  admission, and role/policy/resource boundaries.

## TDD Steps

1. Add failing sandbox enforcement tests for allowed and denied host operations.
2. Implement pre-execution sandbox checks at provider/hook/adapter boundaries.
3. Add denied-attempt provenance and report evidence.

## Completion Checklist

- [x] Sandbox checks run before host effects.
- [x] Denied host attempts are retained as redacted evidence.
- [x] Sandbox policy composes with row/admission/provider boundaries.
- [x] Host providers cannot bypass sandbox policy.

## Evidence

- Added `HostSandboxPolicy`, `HostSandboxDecision`, and `HostSandboxDenialRecord` runtime
  carriers.
- `RuntimeState` now retains host sandbox policies and redacted denial evidence separately from
  provider registration and application entrypoint metadata.
- Admitted host provider projections enforce the provider operation's authored sandbox policy before
  provider execution, including dependency-projection routes.
- Denied attempts return `PermissionDenied` before invoking the provider and retain redacted
  evidence keyed by policy, provider, and operation without raw argument values.
- Added TASK-1929 test:
  - `cargo test -p ash-interp --test task_1929_host_sandbox_policy`
- Verified affected gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo check -p ash-core -p ash-interp`
  - `cargo clippy -p ash-core -p ash-interp --all-targets --all-features`
