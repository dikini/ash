# TASK-1953: Migration Diagnostics

**Status:** Complete
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

- [x] Old/deprecated forms have targeted diagnostic tests.
- [x] Human and JSON diagnostic surfaces are aligned.
- [x] Migration hints avoid teaching unsupported syntax.
- [x] Current syntax does not regress.

## Evidence

- `check_parse_diagnostics` covers stale `observe ... with`, stale `act ... with`, `with role:`,
  legacy conditional/loop/decide forms, reserved Act/Proc/Workflow callable arrows, JSON
  diagnostic code/span/context/help metadata, and a pure callable arrow non-regression.
- `phase199_template_manifest` covers fail-closed template rejection for stale observe syntax,
  deprecated `Proc<`, `Act<`, and `Workflow<` tower carriers, plus stale `ambient authority` and
  `direct provider` language before template promotion.
- Existing `task_778_legacy_workflow_warning` coverage keeps old workflow entry diagnostics
  non-fatal, stable-coded, span-bearing, and absent from function-first current entry points.
- Focused verification:
  `cargo test -p ash-cli --test check_parse_diagnostics --test phase199_template_manifest -- --nocapture`
  and `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
