# TASK-1940: Common Row/Admission Profiles

**Status:** Planned
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Add common row/admission profile definitions and validation fixtures for standard provider use.

## Requirements

- Define profiles for read-only filesystem, read-write filesystem, sandboxed HTTP, deterministic
  test, logging-only, and application-default use.
- Profiles must select explicit expectations and fail closed when stale, malformed, incompatible, or
  authority-widening.
- Profile reports must expose selected profile identity without mutating authority state.
- Add negative tests for overbroad profile composition.

## TDD Steps

1. Add failing profile validation tests.
2. Implement minimal profile definitions and validation.
3. Add application/runtime report assertions.
4. Run focused profile tests and Rust quality gates.

## Completion Checklist

- [ ] Common profiles are defined and documented.
- [ ] Profiles cannot grant authority by name.
- [ ] Stale, malformed, and widening profiles fail closed.
- [ ] Runtime reports retain profile identity and evidence identity.
