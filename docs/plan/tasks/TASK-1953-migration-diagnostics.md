# TASK-1953: Migration Diagnostics

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Improve diagnostics for stale and deprecated syntax so users receive targeted migration guidance
instead of generic parser or type errors.

## Requirements

- Add or refine diagnostics for old workflow entry syntax, stale `observe ... with` and
  `act ... with` spellings, removed tower carrier spellings, stale capability/provider language,
  and unsupported legacy callable arrows.
- Provide stable diagnostic codes, precise spans, concise messages, and migration hints where a
  current spelling exists.
- Keep compatibility-only diagnostics tested separately from productive current-syntax paths.
- Ensure JSON and human diagnostic output remain aligned.

## TDD Steps

1. Add failing diagnostic tests for each old/deprecated form selected from the TASK-1952 audit.
2. Verify failures show missing or generic diagnostics.
3. Implement targeted diagnostic mapping.
4. Verify human and JSON diagnostics include code, span, message, and migration help.

## Completion Checklist

- [ ] Old/deprecated forms have targeted diagnostic tests.
- [ ] Human and JSON diagnostic surfaces are aligned.
- [ ] Migration hints avoid teaching unsupported syntax.
- [ ] Current syntax does not regress.
