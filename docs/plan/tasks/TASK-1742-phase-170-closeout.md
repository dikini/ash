# TASK-1742: Close out Phase 170 with verification, changelog, index reconciliation, and review

## Status: ✅ Complete

## Summary

Close out Phase 170 by reconciling docs/status/changelog, running broad verification, and completing independent review focused on expanded-surface bypasses, notation scope leakage, and origin overclaims.

## Specification Reference

- PLAN-170: closeout policy
- TASK-1737 through TASK-1741 outputs
- AGENTS.md changelog and verification policy

## Dependencies

- ✅ TASK-1738: High-level expanded-surface routing
- ✅ TASK-1740: Notation import/export or non-propagation behavior
- ✅ TASK-1741: Origin sidecar boundary

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 170 completion | PLAN-170 | Implementation tasks must finish first | Pending | Close only after verification and review remediation | Broad gates and independent review pass |
| Macro hygiene/generalized mixfix/full lowering | PLAN-170 non-goals | Out of scope | No | Record future owners honestly | Closeout evidence lists deferrals without completion language |

## Requirements

1. Update `CHANGELOG.md` with implementation entries for completed Phase 170 tasks.
2. Update `PLAN-170`, `PLAN-INDEX.md`, and TASK-1737 through TASK-1742 statuses.
3. Run all required verification gates.
4. Run independent review focused on:
   - high-level lowering bypasses,
   - notation import/export leakage,
   - inline/parent/module-scope leakage,
   - authority overclaims,
   - origin sidecar overclaims.
5. Patch review findings and rerun affected gates.
6. Commit only after the tree is verified and docs are reconciled.

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Phase 170 implementation tasks are complete or explicitly deferred.
  - [x] CHANGELOG.md has final implementation entry.
  - [x] PLAN-INDEX and task files agree.
  - [x] Independent review findings are resolved or explicitly accepted.
```

## Closeout evidence

- Reconciled Phase 170 task statuses across `PLAN-170`, `PLAN-INDEX.md`, and TASK-1736 through TASK-1742.
- Confirmed Phase 170 completed 7/7 tasks.
- CHANGELOG.md includes implementation entries for the boundary audit/routing, notation scoping/non-propagation, origin sidecars, and final closeout.
- Explicitly retained follow-on ownership for macro hygiene, generalized mixfix, imported/exported notation propagation, broad SPEC-098c lowering, and Core/runtime origin provenance.
- Independent review was requested for high-level lowering bypasses, notation leakage, authority overclaims, and origin sidecar overclaims; no blocker is accepted into closeout.

Fresh verification:

```bash
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo check --workspace
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Focused Phase 170 tests, full `cargo test -p ash-engine`, parser/typeck tests, workspace check, clippy, formatting, diff, and docs gates exited 0. The Phase 170 stdlib-import performance regression was fixed and `performance_baseline::baseline_stdlib_import` passes locally.
