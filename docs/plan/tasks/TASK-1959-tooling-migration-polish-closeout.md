# TASK-1959: Tooling/Migration Polish Closeout

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Close out Phase 200 with full gates, stale-claim sweeps, docs, changelog, PLAN-INDEX reconciliation,
and review remediation.

## Requirements

- Run all Phase 200 focused tests and broad verification gates.
- Reconcile PLAN-200, task files, PLAN-INDEX, CHANGELOG, and relevant docs.
- Run stale-claim sweeps for legacy syntax, deprecated-form teaching paths, formatter/LSP legacy
  leakage, and authority-bypassing wording.
- Address code review findings before marking complete.

## TDD Steps

1. Run focused Phase 200 gates and fix failures.
2. Run broad workspace and docs gates.
3. Update status/evidence docs.
4. Complete review remediation.

## Completion Checklist

- [x] Phase 200 focused gates pass.
- [x] Workspace and docs gates pass.
- [x] PLAN-INDEX and CHANGELOG are reconciled.
- [x] Stale-syntax, deprecated-form, and stale-authority sweeps are recorded.
- [x] Review remediation is complete.

## Evidence

- Phase 200 focused gates passed:
  `cargo test -p ash-cli --test check_parse_diagnostics --test phase199_template_manifest --test phase200_formatter_current_syntax --test phase200_examples_current_syntax --test phase200_docs_current_syntax --test phase200_old_syntax_demoted --test phase200_legacy_deprecated_form_audit -- --nocapture`
  and `cargo test -p ash-lsp-core --test phase200_lsp_migration_polish -- --nocapture`.
- Broad closeout gates passed: `cargo fmt --check`, `cargo test --all`,
  `cargo clippy --all-targets --all-features`,
  `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  `bash scripts/check-docs-gate.sh`, and `git diff --check`.
- Productive-root stale-claim sweep returned no matches for legacy/deprecated form patterns across
  `docs/TUTORIAL.md`, `docs/tutorials`, `templates/apps`, `examples/10-testing-helpers`, and
  `examples/11-process-channel-helpers`.
- AUDIT-200 unresolved-language sweep returned no matches for "requires review", "pending final",
  "until migrated", or "until docs refresh".
- PLAN-200, PLAN-INDEX, task files, and CHANGELOG are reconciled for TASK-1951 through TASK-1959.
