# TASK-1938: Clock/Time Provider And Test Clock

**Status:** Planned
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

- [ ] Time wrappers parse/check through stdlib imports.
- [ ] Deterministic test-clock profile returns stable values.
- [ ] Wall-clock access fails closed in deterministic profiles.
- [ ] Clock evidence is redacted and authority-neutral.
