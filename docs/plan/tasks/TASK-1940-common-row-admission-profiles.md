# TASK-1940: Common Row/Admission Profiles

**Status:** Complete
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

- [x] Common profiles are defined and documented.
- [x] Profiles cannot grant authority by name.
- [x] Malformed and authority-widening profiles fail closed.
- [x] Runtime installation reports retain profile identity and admitted row identities.
- [x] Stale and incompatible profile metadata fixtures are covered.
- [x] Runtime reports retain evidence identity through final-surface fixtures.

## Evidence

- Added `ash_engine::standard_profiles` with read-only filesystem, read-write filesystem,
  sandboxed HTTP, deterministic test, logging-only, and application-default profiles.
- Added focused tests proving profile metadata is authority-neutral and profile installation admits
  explicit provider rows rather than granting authority by profile name.
- Added focused tests proving malformed and authority-widening profile metadata fails closed and
  projected fs/http/time/logging calls retain redacted host-boundary evidence.
- Added a stale-row fixture proving incompatible profile metadata fails closed during runtime
  admission instead of silently widening provider rows.
- Completed final-surface filesystem, HTTP, time, and logging fixtures that retain provider,
  operation, outcome, and redaction evidence identity through standard profile execution.
