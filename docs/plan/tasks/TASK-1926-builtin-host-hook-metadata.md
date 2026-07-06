# TASK-1926: Builtin Host Hook Metadata

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Represent builtin host hooks as explicit trusted runtime hooks with capability, row, sandbox, and
provenance metadata.

## Requirements

- Add metadata carriers for builtin host hooks.
- Link builtin hook metadata to operation identity, effect level, required provider/resource rows,
  sandbox policy, and provenance policy.
- Fail closed when a host-facing builtin is missing metadata or attempts to widen authority.
- Keep pure structural builtins separate from host hooks.

## TDD Steps

1. Add failing tests for host-facing builtins with missing or malformed metadata.
2. Implement metadata carriers and lookup.
3. Add regression coverage that pure builtins do not require host metadata.

## Completion Checklist

- [x] Host-facing builtins require explicit metadata.
- [x] Missing metadata fails closed with structured diagnostics.
- [x] Pure builtins remain ordinary internal functions.
- [x] Metadata does not grant authority by itself.

## Evidence

- Added `BuiltinHostHookMetadata`, `BuiltinHostHookMetadataError`,
  `builtin_host_hook_metadata`, `builtin_requires_host_hook_metadata`, and
  `validate_builtin_host_hook_metadata` in `ash-interp` builtin dispatch metadata.
- Added explicit host-hook metadata for implemented process builtins, including operation identity,
  effect level, required rows, sandbox policy, provenance policy, and a non-authority-grant flag.
- Wired `dispatch_builtin` to validate implemented host-facing builtins before execution.
- Added builtin dispatch tests for `process::run` metadata, missing metadata fail-closed behavior,
  pure structural builtin separation, and forward-declared provider-backed builtin compatibility.
- Verified with:
  `cargo test -p ash-interp --test builtin_dispatch host_hook_metadata -- --nocapture`
  `cargo test -p ash-interp --test builtin_dispatch`
  `cargo fmt --all -- --check`
  `cargo check -p ash-interp`
  `cargo clippy -p ash-interp --all-targets --all-features`
  `cargo test -p ash-interp`
  `python3 tools/docs/validate_orientation_indexes.py --self-test && bash scripts/check-docs-gate.sh && git diff --check`
