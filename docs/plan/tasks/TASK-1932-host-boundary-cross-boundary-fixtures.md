# TASK-1932: Host Boundary Cross-Boundary Fixtures

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Add cross-boundary fixtures for builtins, providers, runtime adapters, sandbox policy, provenance,
reports, and fail-closed invalid host crossings.

## Requirements

- Cover CLI, engine, interpreter runtime state, daemon/application runtime reports, and docs gate
  evidence.
- Include successful host calls and denied/malformed/timeout/cancellation cases.
- Prove host boundary evidence is redacted and authority-neutral.
- Prove host surfaces cannot bypass provider authoring metadata, row admission, application
  admission, sandboxing, or provenance requirements.

## TDD Steps

1. Add failing cross-boundary fixtures across touched crates.
2. Wire fixture execution into focused tests.
3. Run focused and broad verification.

## Completion Checklist

- [x] Fixtures cover builtins, providers, adapters, sandboxing, and provenance.
- [x] Fixtures cover success and fail-closed invalid crossings.
- [x] CLI/engine/runtime/daemon reports expose host evidence without authority leaks.
- [x] Verification commands are recorded in task evidence.

## Evidence

- Added `task_1932_host_boundary_cross_boundary_fixtures.rs` covering:
  - builtin `process::run` host-hook metadata,
  - provider authoring metadata for `process.run`,
  - trusted runtime adapter registration and provider-operation validation,
  - host provider row admission and projected execution,
  - sandbox allow and denial paths,
  - redacted host-boundary evidence and operation trace facts.
- Denial fixture proves sandbox failure records redacted evidence without leaking the attempted
  command.
- Focused test:
  - `cargo test -p ash-interp --test task_1932_host_boundary_cross_boundary_fixtures`
- Verified affected gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo check -p ash-core -p ash-interp`
  - `cargo clippy -p ash-core -p ash-interp --all-targets --all-features`
