# TASK-1952: Legacy/Deprecated Form Audit

**Status:** Complete
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

- [x] Legacy/deprecated forms are inventoried.
- [x] Productive paths have no unclassified old-form occurrences.
- [x] Compatibility and historical paths are explicitly labeled.
- [x] Follow-up task ownership is recorded for every remediation row.

## Evidence

- Added [AUDIT-200](../audits/AUDIT-200-legacy-deprecated-form-inventory.md), classifying
  legacy/deprecated form hits across diagnostic tests, LSP test roots, productive docs/reference
  paths, examples, templates, stdlib surfaces, and compatibility fixtures.
- Added `phase200_legacy_deprecated_form_audit`, a focused gate that fails if a scanned file/pattern
  pair is unclassified, uses an unsupported classification, lacks follow-up task ownership, or
  lacks a gate/exclusion reason.
- The audit records TASK-1953 ownership for migration diagnostics, TASK-1956 ownership for examples,
  TASK-1957 ownership for docs/reference/spec refresh, and TASK-1958 ownership for final
  old-syntax removal/demotion.
- Focused verification:
  `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
