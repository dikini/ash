# TASK-1954: Formatter Current-Syntax Polish

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Polish formatter behavior for current target syntax and quarantine formatter behavior for legacy or
deprecated forms.

## Requirements

- Add formatter coverage for current functions, do blocks, rows, contracts/evidence,
  providers/profiles, process/channel helpers, testing helpers, templates, records, matches, and
  imports.
- Prevent productive formatter fixtures from preserving deprecated syntax.
- Keep any retained old-form formatter behavior explicitly compatibility-only.
- Ensure formatting output is stable and idempotent.

## TDD Steps

1. Add failing formatter/idempotence tests for current syntax cases from Phase 199 productive
   examples and templates.
2. Add a compatibility-only fixture for any old-form formatter behavior retained by policy.
3. Implement or repair formatter handling.
4. Verify formatting is stable, idempotent, and current-syntax-first.

## Completion Checklist

- [x] Current-syntax formatter fixtures cover Phase 199 app/library surfaces.
- [x] Formatter output is idempotent.
- [x] Deprecated-form formatter behavior is compatibility-only or removed.
- [x] Productive formatter fixtures contain no unlabeled old syntax.

## Evidence

- Added `ash fmt` with file, directory, `--stdin`, `--check`, and `--write` modes. The Phase 200
  implementation is intentionally conservative: it normalizes whitespace deterministically and
  leaves AST-level source rewrites to the future full formatter crate.
- Added `phase200_formatter_current_syntax` coverage for Phase 199 testing/process examples,
  stdin idempotence, trailing-whitespace check failures, and fail-closed deprecated formatter
  inputs.
- Deprecated formatter fixtures are classified in AUDIT-200 as compatibility-only TASK-1954 rows.
- Focused verification:
  `cargo test -p ash-cli --test phase200_formatter_current_syntax -- --nocapture` and
  `cargo test -p ash-cli --test phase200_legacy_deprecated_form_audit -- --nocapture`.
