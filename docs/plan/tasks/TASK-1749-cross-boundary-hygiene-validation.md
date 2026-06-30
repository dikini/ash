# TASK-1749: Add cross-boundary hygiene and negative-leakage validation tests

## Status: ✅ Complete

## Description

Validate Phase 171 as a boundary system rather than isolated parser changes. This task adds cross-crate tests proving hygiene origins, generated identifiers, notation scope, macro fail-closed behavior, and high-level module/file validation agree.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1745: Expansion identity and origin-chain carriers
- ✅ TASK-1746: Source/generated identifier hygiene fences
- ✅ TASK-1747: Notation and macro scope-table boundaries
- ✅ TASK-1748: Fail-closed macro invocation boundary

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Broad SPEC-098c lowering completion | PLAN-170 non-goals | Too broad | No | Validate only Phase 171 boundaries | Focused cross-boundary tests, not full lowering rewrite |
| Imported notation propagation | TASK-1740/TASK-1747 | Summary carriers absent | No unless implemented | Negative leakage remains required | Import/export tests must remain explicit |

## Requirements

1. Add integration coverage for high-level parser/engine/module-loader paths, not only low-level parser helpers.
2. Prove positive behavior:
   - ordinary local notation/operator-section expansion still works;
   - callable targets remain importable by ordinary names;
   - origin metadata remains available for diagnostics.
3. Prove negative leakage:
   - imported notation is not active accidentally;
   - macro invocations do not lower to Core;
   - generated identifiers do not capture source names;
   - parser-only bypasses are rejected by high-level validation.
4. Patch any status docs if a boundary was intentionally deferred.
5. Do not mark Phase 171 ready for closeout until broad verification reconciles all focused tests.

## TDD Steps

### Step 1: Add cross-boundary tests

**Expected file:** `crates/ash-engine/tests/task_1749_cross_boundary_hygiene_validation.rs` plus parser tests if needed.

Cover module import/export, public callable body validation, expanded-surface validation, and ordinary typechecking where applicable.

### Step 2: Fix implementation gaps found by tests

Patch the owned implementation tasks' files rather than adding test-specific shims.

### Step 3: Re-run focused and broad gates

Run all Phase 171 focused tests plus parser/engine/typeck checks.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-engine --test task_1749_cross_boundary_hygiene_validation -- --nocapture
  - cargo test -p ash-parser --test task_1745_expansion_origin_chain -- --nocapture
  - cargo test -p ash-parser --test task_1746_generated_identifier_hygiene -- --nocapture
  - cargo test -p ash-engine --test task_1747_notation_macro_scope_boundaries -- --nocapture
  - cargo test -p ash-parser --test task_1748_macro_invocation_boundary -- --nocapture
  - cargo test -p ash-engine --test task_1748_macro_invocation_boundary -- --nocapture
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Positive local expansion behavior is preserved.
  - [x] Negative import/export/macro/generated-name leakage tests pass.
  - [x] Broad parser/typeck/engine gates pass or blockers are mapped to TASK-1750.
```

## Completion Evidence

Added `crates/ash-engine/tests/task_1749_cross_boundary_hygiene_validation.rs` to validate Phase 171 across parser, expansion, engine/module-loader, and typechecker-facing seams:

- positive local notation/operator-section expansion preserves origin metadata and generated hygiene, and the same module is accepted through high-level engine validation;
- ordinary callable imports remain usable while imported notation remains inactive;
- macro invocations are rejected by engine/module validation and by typechecker-facing expression checking before Core acceptance.

Fresh closeout gates on the final TASK-1749 diff passed:

- `cargo test -p ash-engine --test task_1749_cross_boundary_hygiene_validation -- --nocapture`: 3 passed.
- `cargo test -p ash-parser`: passed.
- `cargo test -p ash-typeck`: passed.
- `cargo test -p ash-engine`: passed.
- `cargo check --workspace`: passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Supplies the final evidence package consumed by TASK-1750 closeout.
