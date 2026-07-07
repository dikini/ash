# TASK-1936: Filesystem Provider Wrappers And Profiles

**Status:** Complete
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Implement or repair current-syntax filesystem stdlib wrappers and read/read-write filesystem row
profiles over the Phase 197 provider and sandbox substrate.

## Requirements

- Cover file read, file write, append, exists, metadata, directory list, and path helpers where the
  current provider substrate supports them.
- Enforce path sandbox policy before host filesystem effects.
- Emit redacted evidence for allowed, denied, and failed filesystem attempts.
- Add final-surface tests through real stdlib imports and target application/function entrypoints.

## TDD Steps

1. Add failing final-surface filesystem wrapper/profile tests.
2. Implement minimal stdlib/runtime wiring to pass allowed and denied cases.
3. Add provenance/redaction assertions.
4. Run focused filesystem provider tests and Rust quality gates.

## Completion Checklist

- [x] Filesystem wrappers parse/check through final-surface function entrypoints.
- [x] Read-only and read-write profiles fail closed on overbroad paths.
- [x] Sandbox denial occurs before host effects.
- [x] Redacted evidence is emitted for success, failure, and denial.

## Evidence

- Added `crates/ash-engine/tests/task_1936_filesystem_provider_wrappers.rs` covering
  target `fn main` execution through filesystem wrapper builtins for write, append, exists,
  metadata, directory listing, read, and denied outside-profile writes.
- Filesystem wrapper dispatch normalizes current-syntax `PathBuf { inner }` carriers to provider
  path arguments, requires admitted `fs` profile bindings, and records redacted host-boundary
  evidence for success and sandbox denial.
- `HostSandboxPolicy` now carries filesystem path allow-list refinements so profile path rejection
  occurs before host filesystem effects.
