# TASK-1958: Old Syntax Removal/Demotion

**Status:** Planned
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

- [ ] Productive paths reject or exclude unlabeled old syntax.
- [ ] Retained old syntax is compatibility-only, historical-reference-only, or migration-note-only.
- [ ] Temporary compatibility support has explicit diagnostics and task ownership.
- [ ] Focused migration gates pass.
