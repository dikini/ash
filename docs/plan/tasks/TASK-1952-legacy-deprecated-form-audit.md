# TASK-1952: Legacy/Deprecated Form Audit

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Audit diagnostics, formatter, LSP, examples, docs, templates, parser fixtures, and stdlib comments
for legacy and deprecated forms before polishing tooling.

## Requirements

- Inventory old-form usage across productive and compatibility paths.
- Classify each occurrence as removed, migrated, compatibility-only, historical-reference-only, or
  retained with a targeted migration diagnostic.
- Add an audit gate that fails when productive paths contain unclassified legacy/deprecated forms.
- Identify ownership for diagnostics, formatter, LSP, examples, docs, and closeout remediation.

## TDD Steps

1. Add an audit fixture or test that detects known legacy/deprecated forms in productive paths.
2. Confirm the audit fails on an intentionally unclassified old-form fixture.
3. Add the inventory document and classifications.
4. Re-run the audit and record evidence.

## Completion Checklist

- [ ] Legacy/deprecated forms are inventoried.
- [ ] Productive paths have no unclassified old-form occurrences.
- [ ] Compatibility and historical paths are explicitly labeled.
- [ ] Follow-up task ownership is recorded for every remediation row.
