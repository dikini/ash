# TASK-1936: Filesystem Provider Wrappers And Profiles

**Status:** Planned
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

- [ ] Filesystem wrappers parse/check through stdlib imports.
- [ ] Read-only and read-write profiles fail closed on overbroad paths.
- [ ] Sandbox denial occurs before host effects.
- [ ] Redacted evidence is emitted for success, failure, and denial.
