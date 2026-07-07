# TASK-1939: Logging Provider Redaction And Provenance

**Status:** Planned
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

- [ ] Logging wrappers parse/check through stdlib imports.
- [ ] Structured fields and severity survive report projection.
- [ ] Secret values are redacted in evidence.
- [ ] Logging profile selection does not grant unrelated authority.
