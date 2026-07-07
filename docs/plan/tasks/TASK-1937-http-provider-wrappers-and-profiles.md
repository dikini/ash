# TASK-1937: HTTP Provider Wrappers And Profiles

**Status:** Complete
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Implement or repair current-syntax HTTP stdlib wrappers and sandboxed network profiles over the
trusted provider/adapter substrate.

## Requirements

- Cover GET, POST, PUT, DELETE, HEAD where supported, with explicit method and host policy.
- Validate URL scheme, host allowlists, headers, body limits, timeout policy, and redaction policy.
- Emit redacted evidence for allowed, denied, and failed HTTP attempts.
- Preserve provider failure as host/provider failure, not policy denial or contract violation.

## TDD Steps

1. Add failing HTTP wrapper/profile tests with allowed and blocked hosts.
2. Implement minimal wrapper and sandbox wiring.
3. Add redacted evidence and diagnostic assertions.
4. Run focused HTTP provider tests and Rust quality gates.

## Completion Checklist

- [x] HTTP wrappers parse/check through stdlib imports.
- [x] Sandboxed profiles enforce allowed methods and hosts.
- [x] Provider failure taxonomy remains distinct.
- [x] Redacted evidence is emitted for success, failure, and denial.

## Evidence

- Added final-surface HTTP wrapper tests for `http::get`, `http::post`, `http::put`, and
  `http::delete` through sandboxed standard profiles, including success, denied host, provider
  failure, and redacted evidence assertions.
- Registered current HTTP stdlib wrappers in the type environment, builtin dispatch metadata, and
  async provider-backed dispatch path.
- Updated `EngineBuilder::with_http_capabilities` from a placeholder configuration error to
  built-in HTTP provider registration.
- Preserved the current stdlib decision that `HEAD` remains deferred until it has a non-colliding
  runtime bridge separate from list `head`.
