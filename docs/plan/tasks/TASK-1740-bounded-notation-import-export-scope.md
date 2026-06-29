# TASK-1740: Implement bounded notation import/export propagation or explicit non-propagation

## Status: 📝 Planned

## Summary

Implement the TASK-1739 decision. If notation summary carriers are ready, add bounded imported/exported notation propagation. If not, formalize non-propagation with diagnostics/docs and negative leakage tests.

## Specification Reference

- PLAN-170: notation scoping track
- TASK-1739 design note: `docs/design/phase-170-notation-summary-export-semantics.md`
- SPEC-095c §7 and §10: active notation tables
- PLAN-169 TASK-1732: local notation table diagnostics

## Dependencies

- 📝 TASK-1739: Notation summary/export semantics design

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported notation propagation | PLAN-169 non-goal | Needed summary carrier design | Depends on TASK-1739 | Implement bounded propagation only if carriers are ready; otherwise add explicit non-propagation tests | Scope matrix tests pass |

## Requirements

1. Follow the TASK-1739 decision exactly; do not silently broaden notation semantics.
2. If implementing propagation, extend module summary/export carriers with notation metadata and visibility rules.
3. If preserving non-propagation, add explicit tests proving imported notation is not active and diagnostics remain stable.
4. Preserve local and inline-module no-leakage behavior from Phase 169.
5. Add conflict tests for any imported notation behavior that is implemented.
6. Ensure notation targets remain ordinary callable paths; no authority is granted by notation aliases.

## TDD Steps

1. Add scope matrix tests before code changes.
2. Implement the minimal chosen behavior.
3. Add negative leakage tests for all non-supported cases.
4. Run parser, typeck, engine, and workspace checks.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1732_local_notation_table_resolution
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Scope matrix positive cases pass.
  - [ ] Scope matrix negative leakage cases pass.
  - [ ] Imported/exported behavior matches TASK-1739 exactly.
```
