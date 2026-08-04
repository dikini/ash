# TASK-2068 Local-over-Simple Precedence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make a local ordinary function shadow one same-named selected cross-module M-SIMPLE import without dropping its edge needed for atomic cycle detection.

**Architecture:** Add a separate resolver for exactly one inherited, unaliased
`use crate::<public structural-child>...::<public ordinary-function>;` route. It records edges
only for cross-module targets, cycle-checks that complete edge set, then filters a binding owned by
a same-module ordinary function before returning its plan; a same-module target emits no self-edge
and does not participate in cycle detection. The dedicated binder only delegates and calls
`into_bound_set`.

**Tech Stack:** Rust 2024, `ash-typeck`, `proptest`, canonical graph/provisional scope types.

---

**Execution evidence:** Completed for the planned narrow route. The focused
`task_2068_local_over_simple_precedence` target passes 9/9. It proves only Type-layer
scope/binding behavior: local precedence, cross-module edge retention, same-module no-edge,
hidden-cycle atomicity, the stated rejection matrix, normalized file/inline scope/binding parity,
a 16-case property, the authority fence, and the legacy M-SIMPLE regression. TASK-2068 and Phase
207 remain In progress.

**Final source traceability:** planner
`sha256:7fb241da5b3bf35595e7cf3054f06dcbc9c9dc08dc9701c047d0d2c045a393d3`; dedicated binder
`sha256:500d00d4de399eaac9c6ad19b74d79a2ec694b724014fbea8cdea02470a0d470`; `lib.rs` export
boundary `sha256:68f8c3410b8bb92ee72cc85b91501a877dd357dca1456c27622f7996c150162c`; unchanged generic
binder fence `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.

### Task 1: Write RED precedence tests

**Files:**
- Create: `crates/ash-typeck/tests/task_2068_local_over_simple_precedence.rs`
- Read: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Read: `crates/ash-typeck/tests/task_2068_local_over_glob_precedence.rs`

**Step 1: Write the local-wins, noncollision, identity, and all-shadowed tests**

Build only inherited non-root public structural-child fixtures with one unaliased simple import.
Assert that the natural-name local function removes that import binding, a different imported name
still binds, identity/origin/spans/visibility and each selected cross-module edge survive
filtering, and all-shadowed cross-module candidates return an empty binding projection with
retained edges. Also assert that a selected same-module target emits no self-edge.

**Step 2: Run the focused test to verify RED**

Run: `cargo test -p ash-typeck --test task_2068_local_over_simple_precedence local_wins_and_non_colliding_simple_import_binds`
Expected: FAIL because the dedicated precedence API does not exist.

**Step 3: Add failure, parity, property, and regression tests**

Add a hidden two-module cycle that is real only when the shadowed selected cross-module edge remains; require
the outer `ImportCycle { edges: CanonicalImportCycle }` with no plan/binding publication. Add
visibility/shape failures, file/inline normalized Type-layer scope/binding parity, a 16-case
property varying depth 1–3, collision bit, natural name, and source form, an authority fence for
private M-CHECK facts and generic binder use, and a regression proving the existing M-SIMPLE API
still rejects a local collision.

**Step 4: Run the complete focused target to verify RED**

Run: `cargo test -p ash-typeck --test task_2068_local_over_simple_precedence`
Expected: FAIL until the dedicated resolver and binder exist.

### Task 2: Implement the dedicated resolver and delegating binder

**Files:**
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_local_over_simple_precedence.rs`

**Step 1: Add the exact resolver route**

Implement `resolve_scoped_simple_local_precedence_imports_with_scopes(graph, scopes)`. Admit only
one inherited, unaliased `UsePath::Simple` `crate::<public structural-child>...::<public
ordinary-function>` import; reject the root-function, alias, multiple-use, group, glob, `self`,
`super`, restricted/private path or target, re-export, and nonfunction cases through the existing
diagnostic boundary. Select the public ordinary-function target and retain its defining facts and
cross-module edge. A selected same-module target must emit no self-edge.

**Step 2: Preserve ordering and atomicity**

Run the existing deterministic cycle check over every selected cross-module edge before filtering
locally shadowed natural names. A same-module selected target emits no self-edge and skips cycle
detection. Return no result on a cross-module cycle or earlier route/visibility failure. After a
successful cycle check, filter only bindings whose natural names match a same-module ordinary
function; retain cross-module targets and edges in the opaque plan.

**Step 3: Add only a delegating binder and export**

Implement `bind_scoped_simple_local_precedence_imports(graph, scopes)` as a direct call to the new
resolver followed by `into_bound_set`, then export these dedicated APIs through `lib.rs`. Do not
change the generic binder or the existing M-SIMPLE resolver/binder, which must retain local
collision rejection.

**Step 4: Run the focused target to verify GREEN**

Run: `cargo test -p ash-typeck --test task_2068_local_over_simple_precedence`
Expected: PASS with all nine focused witnesses.

### Task 3: Promote evidence without broadening authority

**Files:**
- Modify: `docs/plan/tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`

**Step 1: Run focused code quality checks**

Run: `cargo fmt --check && cargo clippy -p ash-typeck --test task_2068_local_over_simple_precedence -- -D warnings`
Expected: PASS.

**Step 2: Replace only the planned facts after tests pass**

Update the `partial / tested / below_spec` slice with exact implementation/test evidence, source
fingerprints, test anchors, and exclusions. Keep TASK-2068 and Phase 207 In progress: this slice
does not implement final interfaces or later layers.

**Step 3: Run the documentation gates**

Run: `python3 tools/docs/validate_semantic_task_records.py --self-test && python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json && python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json && python3 tools/docs/validate_orientation_indexes.py --self-test && bash scripts/check-docs-gate.sh && git diff --check`
Expected: every command exits 0.
