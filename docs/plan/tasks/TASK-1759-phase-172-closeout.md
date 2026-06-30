# TASK-1759: Close out Phase 172 with verification, review, and status reconciliation

## Status: ✅ Complete

## Description

Close Phase 172 only after parser-first macro execution, origin/hygiene metadata, local scope boundaries, and cross-boundary validation have landed. This task reconciles documentation/status surfaces, runs broad gates, and performs focused closeout review for overclaiming or boundary bypasses.

## Specification Reference

- PLAN-172
- SPEC-095c
- SPEC-098c
- TASK-1752 through TASK-1758 evidence

## Dependencies

- ✅ TASK-1752: Macro execution seam audit
- ✅ TASK-1753: Macro MVP spec amendments
- ✅ TASK-1754: Parsed macro carriers
- ✅ TASK-1755: Local registry and scope validation
- ✅ TASK-1756: Expression-template expansion
- ✅ TASK-1757: Origin/hygiene metadata
- ✅ TASK-1758: Cross-boundary validation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expander | PLAN-172 non-goal | Still too broad | No | Record follow-on | Review must find no overclaiming |
| Imported macro summaries | PLAN-172 D4 | No carriers | No | Record follow-on | Negative tests pass |
| Binder macros | PLAN-172 D2 | Binder hygiene absent | No | Record follow-on | Unsupported-template tests pass |

## Requirements

1. Run all focused Phase 172 tests and broad parser/typeck/engine/workspace gates.
2. Update PLAN-172 goals, acceptance criteria, and evidence.
3. Update TASK-1751 through TASK-1759 statuses and verification checklists honestly.
4. Update PLAN-INDEX Phase 172 progress/status.
5. Update CHANGELOG.md with implementation and closeout entries.
6. Run focused closeout review for:
   - unsupported macro bypasses;
   - imported/re-exported macro leakage;
   - macro-generated identifier capture;
   - origin-chain loss;
   - docs overclaiming full macro/token-tree/typed/binder support.
7. Fix review findings or create explicit follow-on tasks before marking complete.

## Closeout Steps

### Step 1: Reconcile status surfaces

Patch PLAN-172, task files, PLAN-INDEX, SPEC-INDEX if changed, and CHANGELOG in one closeout slice.

### Step 2: Run broad gates

Use the baseline closeout commands from PLAN-172.

### Step 3: Focused closeout review

Delegate or run a focused review against the full diff and record findings in TASK-1759 evidence.

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
  - [x] Focused Phase 172 tests pass.
  - [x] Broad gates pass.
  - [x] Focused closeout review completed.
  - [x] Review blockers fixed or explicitly deferred with follow-on tasks.
  - [x] PLAN-INDEX, PLAN-172, task files, and CHANGELOG agree.
```

## Completion Evidence

Closeout reconciled Phase 172 status across PLAN-172, PLAN-INDEX, task files, specs, and CHANGELOG. The final implemented scope remains intentionally conservative: local parser-first expression macros only; no token-tree rewriting, typed macros, binder macros, imported/exported macro activation, or Core/runtime macro representation.

Fresh focused verification passed:

```bash
cargo test -p ash-parser --test task_1756_expression_macro_expansion -- --nocapture
cargo test -p ash-parser --test task_1757_macro_origin_hygiene -- --nocapture
cargo test -p ash-parser --test task_1758_macro_lowering_boundaries -- --nocapture
cargo test -p ash-engine --test task_1755_macro_registry_scope -- --nocapture
cargo test -p ash-engine --test task_1758_macro_execution_boundaries -- --nocapture
cargo test -p ash-parser --test task_1748_macro_invocation_boundary -- --nocapture
cargo test -p ash-engine --test task_1748_macro_invocation_boundary -- --nocapture
```

Fresh broad verification passed:

```bash
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo check --workspace
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Focused closeout review initially found blockers in macro-generated/free identifier capture and nested macro origin-chain preservation. Closeout remediation rejects non-parameter free variables in executable macro templates, preserves macro-to-macro origin parents for nested expansions, substitutes nested macro invocation arguments, and clarifies unexpanded-carrier diagnostics/docs. No imported/re-exported macro leakage or token-tree/binder/typed/imported-support overclaim remains in the final documented scope.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Closeout should recommend the next macro phase only after review: likely imported macro summary carriers, bracket/brace token-tree parsing, binder hygiene, or typed macro design depending on findings.
