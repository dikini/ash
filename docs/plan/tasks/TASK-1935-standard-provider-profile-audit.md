# TASK-1935: Standard Provider/Profile Audit

**Status:** Complete
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Audit stdlib provider modules, runtime provider implementations, examples, tests, profile seams, and
Phase 197 host-boundary metadata before implementing standard provider wrappers.

## Requirements

- Inspect `std/src/io/*`, `std/src/http.ash`, `std/src/time.ash`, `std/src/process.ash`,
  `std/src/runtime/*`, provider implementations, and existing provider tests.
- Classify each provider surface as usable, stubbed, stale syntax, missing metadata, or deferred.
- Map every standard provider to operation rows, sandbox policy, provenance policy, and fixtures.
- Produce an audit artifact or task evidence that assigns owners to TASK-1936 through TASK-1941.

## TDD Steps

1. Add audit assertions or fixture inventory checks for selected provider surfaces.
2. Run the checks and confirm current gaps are visible.
3. Record remediation ownership in the task evidence.

## Completion Checklist

- [x] Provider and profile seams are inventoried.
- [x] Stubbed/stale stdlib provider surfaces are classified.
- [x] Phase 197 metadata requirements are mapped to Phase 198 tasks.
- [x] Follow-up task ownership is explicit.

## Evidence

- Added [AUDIT-198](../audits/AUDIT-198-standard-provider-profile-seams.md), mapping stdlib
  provider surfaces, runtime providers, profile/evidence seams, and remaining task ownership.
- Added focused coverage in
  `crates/ash-engine/tests/task_1940_standard_provider_profiles.rs` for standard profile metadata,
  provider metadata validation, deterministic test clock installation, filesystem projection, and
  logging denial evidence.
