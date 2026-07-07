# TASK-1939: Logging Provider Redaction And Provenance

**Status:** Complete
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)

## Description

Add current-syntax logging provider wrappers with structured redaction and provenance evidence.

## Requirements

- Cover debug/info/warn/error events or the smallest current-syntax equivalent.
- Support structured fields, severity, target/module identity, and redaction marks.
- Ensure log events are evidence/report artifacts and do not grant provider authority.
- Add tests for secret redaction, denied logging attempts, and report projection.

## TDD Steps

1. Add failing logging wrapper tests for structured and redacted events.
2. Implement minimal logging provider metadata and wrapper surface.
3. Add report/provenance assertions.
4. Run focused logging tests and Rust quality gates.

## Completion Checklist

- [x] Logging wrappers parse/check through stdlib imports.
- [x] Structured fields and severity survive report projection.
- [x] Secret values are redacted in evidence.
- [x] Logging profile selection does not grant unrelated authority.

## Evidence

- Added final-surface logging wrapper tests for `logging::debug`, `logging::info`,
  `logging::warn`, and `logging::error` through application-default and logging-only standard
  profiles.
- Registered current logging stdlib wrappers in the type environment, builtin dispatch metadata,
  and provider-backed dispatch path.
- Updated logging stdlib declarations to expose structured log event records containing severity,
  redaction marker, and field count.
- Verified allowed log writes emit authority-neutral redacted host-boundary evidence, denied
  logging-only profile writes fail closed with redacted evidence, and logging-only profile
  selection does not admit unrelated provider authority.
