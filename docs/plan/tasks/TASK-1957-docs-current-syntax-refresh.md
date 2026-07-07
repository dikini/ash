# TASK-1957: Docs Current-Syntax Refresh

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Refresh tutorials, reference docs, and orientation paths so productive documentation teaches current
syntax and quarantines old syntax in migration notes only.

## Requirements

- Audit docs and snippets using the TASK-1952 classification model.
- Migrate productive tutorials and getting-started paths to current syntax.
- Add migration notes for deprecated forms that remain user-visible.
- Update orientation indexes when read paths change.
- Add docs gates that prevent unlabeled old syntax from returning to productive docs.

## TDD Steps

1. Add a docs audit/test that fails on unlabeled old syntax in productive docs.
2. Confirm current stale docs fail where applicable.
3. Migrate productive docs or move old syntax into labeled migration sections.
4. Re-run docs gates and orientation-index validation.

## Completion Checklist

- [x] Productive tutorials and getting-started docs teach current syntax.
- [x] Old syntax appears only in labeled migration/historical sections.
- [x] Orientation indexes are updated where read paths change.
- [x] Docs gates pass.

## Evidence

- Added `phase200_docs_current_syntax`, which fails on stale or unlabeled old forms in
  `docs/TUTORIAL.md`, `docs/README.md`, and `docs/tutorials`, requires the tutorial to point at
  current productive examples/templates, and requires retained reference/spec index old-form
  mentions to be explicitly labeled as historical, reference, migration, compatibility, superseded,
  or deprecated.
- Rewrote `docs/TUTORIAL.md` from a stale broad tutorial into a current productive entry path tied
  to checked Phase 199 helper examples, app templates, explicit profile/provider authority wording,
  and migration/reference boundaries.
- Labeled retained old-form mentions in `docs/reference/phase-199-app-template-manifest-schema.md`
  and `docs/spec/README.md` as migration or historical reference material.
- Classified TASK-1957 docs-gate fixture literals in AUDIT-200.
- Focused verification:
  `cargo test -p ash-cli --test phase200_docs_current_syntax -- --nocapture` and
  `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
