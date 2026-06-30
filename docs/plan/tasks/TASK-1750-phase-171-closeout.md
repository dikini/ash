# TASK-1750: Close out Phase 171 with verification, changelog, index reconciliation, and review

## Status: ✅ Complete

## Description

Close Phase 171 only after hygiene, origin, notation-scope, macro-boundary, and cross-boundary validation tasks have landed and broad verification is reconciled. This task records final evidence, updates status surfaces, runs independent review, and either fixes findings or leaves explicit follow-on tasks.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1744: Hygiene, origin, and scope audit
- ✅ TASK-1745: Expansion identity and origin-chain carriers
- ✅ TASK-1746: Source/generated identifier hygiene fences
- ✅ TASK-1747: Notation and macro scope-table boundaries
- ✅ TASK-1748: Fail-closed macro invocation boundary
- ✅ TASK-1749: Cross-boundary hygiene validation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expander | SPEC-095c / PLAN-171 non-goals | Still out of scope unless implemented by future phase | No | Record as follow-on | Review must not find overclaiming |
| Generalized mixfix/binder notation | SPEC-095c | Needs full hygiene and binder model | No | Record as follow-on | Negative claims remain in docs |
| Imported notation propagation | TASK-1747 | Depends on summary carriers | Depends on actual implementation | Either document local-only or verify carriers | Positive + negative tests required if enabled |

## Requirements

1. Run all focused Phase 171 tests and broad parser/typeck/engine/workspace gates.
2. Update `PLAN-171` goals, acceptance criteria, and closeout evidence.
3. Update all TASK-1744 through TASK-1750 statuses and verification checklists honestly.
4. Update `docs/plan/PLAN-INDEX.md` Phase 171 progress/status.
5. Update `CHANGELOG.md` with implementation and closeout entries.
6. Run an independent review for overclaiming, boundary bypasses, missing negative leakage tests, and stale docs.
7. Fix review findings or create explicit follow-on tasks before marking complete.

## Closeout steps

### Step 1: Reconcile status surfaces

Patch the plan, task files, PLAN-INDEX, and changelog in one closeout slice.

### Step 2: Run broad gates

Use the baseline closeout commands from PLAN-171.

### Step 3: Independent review

Ask a review agent to check:
- source/generated identifier capture;
- origin-chain preservation;
- macro fail-closed boundary;
- notation import/export leakage;
- high-level module-loader validation;
- docs overclaiming versus implementation.

### Step 4: Remediate findings

Do not close with accepted blockers unless the user explicitly chooses to defer them and they are recorded as follow-on tasks.

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
  - [x] Focused Phase 171 tests pass.
  - [x] Broad gates pass.
  - [x] Independent review completed.
  - [x] Review blockers fixed or explicitly deferred with follow-on tasks.
  - [x] PLAN-INDEX, PLAN-171, task files, and CHANGELOG agree.
```

## Closeout Evidence So Far

Fresh verification on the Phase 171 implementation and TASK-1749/TASK-1750 documentation slice passed:

- `cargo fmt --check`: passed.
- `cargo test -p ash-parser`: passed.
- `cargo test -p ash-typeck`: passed.
- `cargo test -p ash-engine`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.
- `python3 tools/docs/validate_orientation_indexes.py --self-test`: passed.
- `bash scripts/check-docs-gate.sh`: passed.

`SPEC-095c`, `SPEC-098c`, and `SPEC-INDEX.md` were updated to record the conservative Phase 171 hygiene and fail-closed lowering invariants.

Independent review completed. The review blocker was remediated by threading parent-origin context through every recursive arm in `elaborate_operator_sections_in_expr_with_parent`, rather than only call arguments, and by adding `nested_expansion_origin_survives_non_call_recursive_shapes`. Non-blocking overclaiming findings were addressed by narrowing macro carrier wording to an unqualified-name, raw-delimited-body placeholder and by adding coverage that qualified macro-like invocations are not claimed by the Phase 171 carrier.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Closeout should recommend the next phase only after review: likely macro execution MVP, imported notation summary carriers, or generalized mixfix/binder notation depending on findings.
