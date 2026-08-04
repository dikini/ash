# TASK-2068 Local-over-Glob Precedence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make a same-module ordinary function shadow a same-named selected M-GLOB import while retaining every selected import edge for atomic cycle detection.

**Architecture:** Extend only the existing one-glob inherited `crate` route. The dedicated planner selects and validates every public ordinary-function target first, records every edge, detects canonical cycles, then filters bindings shadowed by a same-module ordinary function before returning its public plan. The dedicated binder only delegates that plan through `into_bound_set`. No private M-CHECK data or generic binder route may authorize imports.

**Tech Stack:** Rust 2024, `ash-typeck`, `proptest`, existing canonical graph/provisional scope types.

---

**Execution evidence:** Completed for the planned narrow route. The focused
`task_2068_local_over_glob_precedence` target passes 8/8. It proves only Type-layer
scope/binding behavior: local projection precedence, non-colliding bindings, retained
selected-edge identity, all-shadowed empty bindings, actual atomic ImportCycle, retained
visibility/shape diagnostics, normalized file/inline scope/binding parity, a 16-case property,
varying names, collision subsets, source form, and depth 1–3, and the authority fence. It does not
establish final-interface or runtime parity.

### Task 1: Write the RED integration tests

**Files:**
- Create: `crates/ash-typeck/tests/task_2068_local_over_glob_precedence.rs`
- Read: `crates/ash-typeck/tests/task_2068_scoped_glob_ordinary_function_imports.rs`

**Step 1: Write the failing local-wins and non-colliding-binding tests**

Build one valid inherited `use crate::<public-child>::*;` fixture with a same-module ordinary
function named like one imported target and another imported target with no collision. Assert that
the local name is absent from bindings while the non-colliding imported name binds.

**Step 2: Run the focused test to verify RED**

Run: `cargo test -p ash-typeck --test task_2068_local_over_glob_precedence local_wins_and_non_colliding_import_binds`
Expected: FAIL because the precedence API/behavior is absent.

**Step 3: Write failing preservation and error tests**

Add tests that verify selected target identity and every selected edge survive shadowing; all
selected imports can be shadowed leaving an empty binding set; a hidden selected edge can produce
the real outer `ImportCycle { edges: CanonicalImportCycle }` with no publication; existing
visibility/shape diagnostics remain first; file/inline forms match; the 16-case property varies
depth, selected-name collision mask, and source form; and an authority-fence test proves private
M-CHECK facts and the generic binder are unused.

**Step 4: Run the focused test target**

Run: `cargo test -p ash-typeck --test task_2068_local_over_glob_precedence`
Expected: the new assertions fail before the implementation change.

### Task 2: Implement the dedicated selection and precedence projection

**Files:**
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_local_over_glob_precedence.rs`

**Step 1: Add the smallest opaque planned result needed for the M-GLOB route**

In the dedicated planner, validate exactly one inherited public structural-child `crate` glob,
select all visible public ordinary functions from canonical scopes, preserve each identity,
origin/spans, and import edge, and run canonical cycle detection over the complete selected edge
set before any projected bindings exist.

**Step 2: Run the focused tests to confirm RED remains**

Run: `cargo test -p ash-typeck --test task_2068_local_over_glob_precedence`
Expected: still FAIL until the resolver projects local precedence.

**Step 3: Project local-over-glob precedence in the resolver and keep the binder as delegation**

In the dedicated planner/resolver, use only same-module ordinary-function names from the canonical
provisional scope to filter bindings after selection/edge/cycle validation and before returning
`CanonicalResolvedSimpleImports`. Keep every selected target and edge in the opaque plan. In the
dedicated binder, only delegate to the resolver and call `into_bound_set`; do not change the generic
binder. Export only this dedicated API through `lib.rs`.

**Step 4: Run the focused tests to verify GREEN**

Run: `cargo test -p ash-typeck --test task_2068_local_over_glob_precedence`
Expected: PASS.

### Task 3: Verify boundaries and document the implemented evidence

**Files:**
- Modify: `docs/plan/tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`

**Step 1: Run focused quality checks**

Run: `cargo fmt --check && cargo clippy -p ash-typeck --test task_2068_local_over_glob_precedence -- -D warnings`
Expected: PASS.

**Step 2: Promote only after evidence exists**

Replace the planned `partial / none / below_spec` slice record with the precise implemented/tested
facts, source fingerprints, test anchors, and remaining exclusions. Keep TASK-2068 and Phase 207
In progress until their complete target rules are delivered.

**Step 3: Run documentation gates**

Run: `python3 tools/docs/validate_semantic_task_records.py --self-test && python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json && python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json && python3 tools/docs/validate_orientation_indexes.py --self-test && bash scripts/check-docs-gate.sh && git diff --check`
Expected: every command exits 0.
