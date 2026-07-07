# TASK-1958: Old Syntax Removal/Demotion

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Remove, demote, or fail-closed legacy and deprecated forms from productive paths after diagnostics,
formatter, LSP, examples, and docs have migration coverage.

## Requirements

- Use TASK-1952 audit results and TASK-1953 through TASK-1957 remediation evidence.
- Remove old-form productive fixtures that no longer need compatibility support.
- Demote retained compatibility fixtures with clear labels and gates.
- Add fail-closed productive-path checks that reject unlabeled old syntax.
- Record migration guidance for any deprecated form still accepted temporarily.

## TDD Steps

1. Add fail-closed gates for productive paths that must reject or exclude old syntax.
2. Confirm gates catch seeded old-form occurrences.
3. Remove, demote, or quarantine old-form paths.
4. Re-run all focused migration, formatter, LSP, example, and docs gates.

## Completion Checklist

- [x] Productive paths reject or exclude unlabeled old syntax.
- [x] Retained old syntax is compatibility-only, historical-reference-only, or migration-note-only.
- [x] Temporary compatibility support has explicit diagnostics and task ownership.
- [x] Focused migration gates pass.

## Evidence

- Added `phase200_old_syntax_demoted`, which fail-closes the productive roots
  `docs/TUTORIAL.md`, `docs/tutorials`, `examples/10-testing-helpers`,
  `examples/11-process-channel-helpers`, and `templates/apps` against old/deprecated forms.
- The same gate rejects unresolved audit language such as "requires review", "pending final",
  "until migrated or removed", and "until docs refresh"; compatibility-only rows are limited to
  test fixtures, LSP/CLI fixtures, std compatibility surfaces, and legacy test roots.
- Updated AUDIT-200 so older phase-era examples are historical/reference-only instead of
  unresolved productive candidates, docs/spec rows point at the TASK-1957 docs gate, and std tower
  rows are explicitly retained as TASK-1958 compatibility surfaces.
- Focused verification:
  `cargo test -p ash-cli --test phase200_old_syntax_demoted -- --nocapture` and
  `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
