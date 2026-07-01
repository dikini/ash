# TASK-1802: Close out Phase 176 with broad gates and independent review

## Status: ✅ Complete

## Description

Close Phase 176 by running focused and broad verification, resolving independent review findings, and reconciling all cleanup outcomes. Do not close with accepted blockers unless explicitly deferred by the user.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1795 through TASK-1801 complete or explicitly re-scoped

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Complete via TASK-1797 | Retired | Reference classification and removal tests passed |
| TASK-1580 | PLAN-158 | Needed power-tower lifting / pure-vs-Act distinction | Complete via TASK-1798 | Retired for module-callable visibility; broader tower lifting remains separate future work | Closure lookup positive and private-helper non-leakage tests passed |
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Public API/config landed; execution re-scoped | Kept with current parser/type-metadata blocker and fail-closed guard | Final-surface QuickCheck import/check fixtures pass |
| Phase 152 status drift | PLAN-152 vs PLAN-INDEX | Historical status drift | Reconciled by TASK-1801 | Retired drift | Docs gate passes |

## Requirements

### Functional Requirements

1. Run all focused tests from TASK-1797, TASK-1798, and TASK-1800.
2. Run the broad Phase 176 verification baseline.
3. Obtain independent review focused on semantic overclaim, bridge leakage, stale status, and runtime authority.
4. Fix or explicitly defer every review finding.
5. Mark Phase 176 complete only after gates and review are clean.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Run focused gates

Execute each implementation task verification command.

### Step 2: Run broad gates

Execute the full PLAN-176 verification baseline.

### Step 3: Independent review

Dispatch a reviewer with the changed files, old deferral sources, and verification output.

### Step 4: Reconcile closeout

Patch PLAN-176, PLAN-INDEX, task checklists, specs/docs, and CHANGELOG after review remediation.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core
  - cargo test -p ash-interp
  - cargo test -p ash-engine
  - cargo test -p ash-cli
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Focused tests pass
  - [x] Broad gates pass
  - [x] Independent review complete and findings addressed
  - [x] Status surfaces and changelog agree
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

Closeout should explicitly state which old deferrals were retired, superseded, or kept with current blockers.


## Verification progress

Focused and broad gates run during closeout:

```text
cargo fmt --check
cargo test -p ash-core
cargo test -p ash-interp
cargo test -p ash-engine
cargo test -p ash-cli
cargo test -p ash-typeck
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 Value::List absence assertion over crates/**/*.rs
```

Observed result before independent review: all commands passed. Independent review `deleg_73a09ffa` then found blockers around imported private-helper collision, QuickCheck recursive-combinator overclaim, and stale Phase 151 status text.

Remediation applied:

- Isolated imported private-helper runtime callables per imported module family so same-named private helpers from different providers do not collide in caller environments.
- Added `imported_private_helpers_with_same_name_stay_module_local` to `crates/ash-engine/tests/task_1798_closure_module_function_visibility.rs`.
- Narrowed QuickCheck recursive-combinator docs to the landed Phase 176 slice: public SPEC-087 names/config plus fail-closed execution guard, with real bounded recursive generation still deferred to parser/type-metadata substrate work.
- Added `recursive_combinator_execution_fails_closed_until_bounded_generation_lands` to `crates/ash-engine/tests/phase151_quickcheck_stdlib.rs`.
- Reconciled additional Phase 151/TASK-1511 status and changelog text found during review remediation.

Post-remediation focused and broad gates run:

```text
cargo fmt --check
cargo test -p ash-engine --test task_1798_closure_module_function_visibility -- --nocapture
cargo test -p ash-engine --test phase151_quickcheck_stdlib -- --nocapture
cargo run -q -p ash-cli -- check std/src/test/quickcheck/combinator.ash
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 Value::List absence assertion over crates/**/*.rs
cargo test -p ash-core
cargo test -p ash-interp
cargo test -p ash-engine
cargo test -p ash-cli
cargo test -p ash-typeck
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Observed result after remediation: all commands passed. Final review `deleg_7ba076b5` found remaining status-count drift in PLAN-176 and PLAN-INDEX; those rows were corrected to 9/9 before closeout.


## Final closeout evidence

Final review `deleg_7ba076b5` reported only stale Phase 176 count/status rows. TASK-1802 remediated those blockers by updating PLAN-176 and PLAN-INDEX from 7/9 in-progress to 9/9 complete and marking TASK-1802 complete in both task tables.
