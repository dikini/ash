# TASK-1938: Clock/Time Provider And Test Clock

**Status:** Complete
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Implement or repair current-syntax clock/time wrappers and deterministic test-clock support.

## Requirements

- Cover `now`, UTC/local formatting where supported, duration helpers, and sleep/sleep-until only
  where runtime semantics are explicit.
- Add deterministic test-clock profile inputs for repeatable tests and evidence.
- Prevent wall-clock leakage in deterministic test profiles.
- Emit trace/evidence for clock observations and sleep attempts.

## TDD Steps

1. Add failing time wrapper tests for real-clock and deterministic-clock profiles.
2. Implement deterministic clock profile plumbing.
3. Add evidence assertions for observations and denied wall-clock access.
4. Run focused time provider tests and Rust quality gates.

## Completion Checklist

- [x] Time wrappers parse/check through stdlib imports.
- [x] Deterministic test-clock profile returns stable values.
- [x] Wall-clock access fails closed in deterministic profiles.
- [x] Clock evidence is redacted and authority-neutral.

## Evidence

- Added final-surface clock/time wrapper tests for `time::now`, `time::now_iso`,
  `time::epoch_millis`, and `time::sleep` through deterministic and application-default standard
  profiles.
- Registered current time stdlib wrappers in the type environment, builtin dispatch metadata, and
  provider-backed dispatch path.
- Deterministic test-clock profiles now admit `time.sleep` only through a deny-all sandbox policy,
  so wall-clock delay attempts fail closed before host effects and emit denied host-boundary
  evidence.
- Application-default profiles allow real-clock observation and explicit zero-duration sleep
  attempts with authority-neutral evidence.
