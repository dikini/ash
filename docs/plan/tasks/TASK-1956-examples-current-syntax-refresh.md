# TASK-1956: Examples Current-Syntax Refresh

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Refresh the examples corpus so productive examples use current syntax and legacy examples are
explicitly classified, demoted, or removed.

## Requirements

- Audit `examples/`, template-generated examples, and example README references using the
  TASK-1952 classification model.
- Migrate productive examples to current syntax.
- Move retained legacy examples into clearly labeled compatibility or historical paths.
- Add or update executable gates for productive examples.

## TDD Steps

1. Add or update an example corpus gate that fails on unclassified deprecated forms.
2. Confirm current productive examples with old syntax fail the gate.
3. Migrate, demote, or remove old examples.
4. Re-run example corpus checks and record evidence.

## Completion Checklist

- [x] Productive examples use current syntax.
- [x] Retained legacy examples are compatibility-only or historical-reference-only.
- [x] Example README paths teach current syntax first.
- [x] Example corpus gates pass.

## Evidence

- Added `phase200_examples_current_syntax`, which keeps Phase 199 productive example roots free of
  deprecated form hits and requires retained old-form examples to carry reference-only,
  historical, or compatibility markers in the file or directory README.
- Updated `examples/README.md` so the current productive testing/process examples and app template
  docs are the first teaching path, while older phase-era material is explicitly migration/reference
  material.
- Example corpus compatibility fixtures are classified in AUDIT-200 as TASK-1956 rows.
- Focused verification:
  `cargo test -p ash-cli --test phase200_examples_current_syntax -- --nocapture` and
  `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
