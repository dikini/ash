# TASK-2070 Self Simple Ordinary-Function Alias Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bind zero or more direct same-module ordinary functions through individually eligible,
explicit different `self` aliases without creating an import edge or cycle behavior.

**Task owner:** [TASK-2070](../plan/tasks/TASK-2070-scoped-self-simple-function-aliases.md)

**Architecture:** A dedicated resolver accepts zero or more individually eligible inherited,
exactly-two-segment `UsePath::Simple` `use self::<ordinary_function> as <different_alias>;`
statements per module. It reads each importer's direct scope entry, checks visibility from the same
`ModuleKey`, stages every distinct alias with preserved provenance and full `Use::span`, reports a
repeated alias as `DuplicateBinding`, and publishes only after every module succeeds. Dedicated
`CanonicalSelfOrdinaryFunctionAliasBinding`, `CanonicalResolvedSelfOrdinaryFunctionAliases`, and
`CanonicalBoundSelfOrdinaryFunctionAliasSet` carry the result; the resolved type has no
`import_edges` field and only the dedicated binder calls its private `into_bound_alias_set`. The
resolver and binder share `CanonicalStructuralImportError`; `ImportCycle` is unreachable by
construction and source fence. The generic binder and `CanonicalBoundModuleBinding` stay untouched.

**Tech Stack:** Rust 2024, `ash-typeck`, `proptest`, canonical graph/provisional scope types.

---

**Planning state:** `partial / none / below_spec`. This plan authorizes no implementation or test
promotion. TASK-2068 is Complete for its closed foundation; TASK-2070 and Phase 207 remain In
progress.

### Task 1: Write the RED self-alias test target

**Files:**
- Create: `crates/ash-typeck/tests/task_2070_scoped_self_ordinary_function_aliases.rs`
- Read: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Read: `crates/ash-typeck/tests/task_2068_local_over_simple_precedence.rs`
- Read: `crates/ash-typeck/src/canonical_module_binder.rs`

**Step 1: Cover zero, root/nested, and multiple-distinct aliases.**

Create direct root and nested-module fixtures containing zero or more individually eligible
`use self::<ordinary_function> as <different_alias>;` statements. Assert that no eligible use
returns an empty dedicated result, multiple distinct aliases bind together, and public, crate,
super, restricted, self, and inherited visibility each bind only when
`is_visible_from(importer)` permits that same `ModuleKey`.

**Step 2: Cover provenance and no-edge behavior.**

Assert every `CanonicalSelfOrdinaryFunctionAliasBinding` exposes its local alias, target defining
identity, declaration span, origin, declared visibility, and complete `Use::span` as `use_span`;
assert that `CanonicalResolvedSelfOrdinaryFunctionAliases` has no `import_edges` field, only the
binder can call private `into_bound_alias_set`, and shared `CanonicalStructuralImportError` makes
`ImportCycle` unreachable by the dedicated source fence.

**Step 3: Add boundary and atomicity failures.**

Reject natural-name/equal aliases, public/restricted uses and re-exports, `self::child::fn`,
crate/super/unprefixed paths, groups, globs, mixed/other import forms, and nonfunctions as
`Unsupported`. Include a direct `self::<child_module>` nonfunction target. Assert that duplicate
eligible aliases reach `DuplicateBinding`, aliases colliding with local declarations reach
`LocalDeclarationCollision`, and a valid sibling paired with any failing module publishes no
dedicated result.

**Step 4: Add parity, property, and authority witnesses.**

Add file/inline normalized Type-layer scope/binding parity, exactly 16 property cases varying
name, alias count, root/nested shape, source form, and visibility, and an authority fence proving
that private M-CHECK facts, `CanonicalBoundModuleBinding`, and the generic binder cannot authorize
or carry this route.

**Step 5: Run the target to verify RED.**

Run: `cargo test -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases`
Expected: FAIL because the dedicated resolver and binder do not yet exist.

### Task 2: Implement the narrow resolver

**Files:**
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2070_scoped_self_ordinary_function_aliases.rs`

**Step 1: Define the dedicated result contract.**

Add `CanonicalSelfOrdinaryFunctionAliasBinding` with accessors for local alias, defining identity,
declaration span, origin, visibility, and `use_span`. Add
`CanonicalResolvedSelfOrdinaryFunctionAliases` with its binding collection and private
`into_bound_alias_set`, and add `CanonicalBoundSelfOrdinaryFunctionAliasSet` as the conversion
result. Use shared `CanonicalStructuralImportError`. Do not add an `import_edges` field, reuse
`CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`, modify
`CanonicalBoundModuleBinding`, or add a reachable `ImportCycle` branch.

**Step 2: Add `resolve_scoped_self_ordinary_function_imports_with_scopes`.**

For each module, accept zero or more individually eligible inherited
`self::<ordinary_function> as <different_alias>` simple uses with two segments. Select only the
importer's direct ordinary-function declarations, require different aliases, apply
`is_visible_from` with that importer key, and stage distinct aliases together. Treat a duplicate
eligible alias as `DuplicateBinding`; reject all mixed/other import forms and direct
child-module/nonfunction targets as `Unsupported`.

**Step 3: Preserve binding facts without importing behavior.**

Stage dedicated alias bindings that preserve defining identity, declaration span, origin, declared
visibility, and full use span. Create no `CanonicalSimpleImportEdge`, skip cycle detection, never
construct a reachable `ImportCycle`, and return only
`CanonicalResolvedSelfOrdinaryFunctionAliases` through shared `CanonicalStructuralImportError`.

**Step 4: Enforce graph-wide atomic failure.**

Reject every out-of-domain shape, visibility failure, duplicate alias, and alias/local collision
before publication. If any module fails, return no dedicated resolved or bound result for valid
siblings.

**Step 5: Re-run the focused target to retain RED until the binder exists.**

Run: `cargo test -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases`
Expected: still RED only for the missing dedicated binder/export path.

### Task 3: Add the delegating binder and public boundary

**Files:**
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2070_scoped_self_ordinary_function_aliases.rs`

**Step 1: Add `bind_scoped_self_ordinary_function_imports`.**

Delegate directly to `resolve_scoped_self_ordinary_function_imports_with_scopes` and then call
private `into_bound_alias_set` to return `CanonicalBoundSelfOrdinaryFunctionAliasSet`. Do not add
fallback behavior, return `CanonicalBoundModuleSet`, or change the generic binder.

**Step 2: Export only the dedicated APIs.**

Export the dedicated resolver, binder, and three dedicated result types through `lib.rs`; keep the
private `into_bound_alias_set`, generic binder, and `CanonicalBoundModuleBinding` source fences
intact.

**Step 3: Run the target to verify GREEN.**

Run: `cargo test -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases`
Expected: PASS with all eight focused witnesses.

**Step 4: Run focused quality checks.**

Run: `cargo fmt --check && cargo clippy -p ash-typeck --test task_2070_scoped_self_ordinary_function_aliases -- -D warnings && git diff --check`
Expected: PASS.

### Task 4: Promote only verified evidence

**Files:**
- Modify: `docs/plan/tasks/TASK-2070-scoped-self-simple-function-aliases.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`

**Step 1: Replace only deferred statuses after GREEN.**

Record actual source fingerprints, promote the implementation and eight witness nodes, and retain
`partial / tested / below_spec`. Do not promote TASK-2070 or Phase 207 to complete.

**Step 2: Run documentation validation.**

Run: `python3 tools/docs/validate_semantic_task_records.py --self-test && python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json && python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json && python3 tools/docs/validate_orientation_indexes.py --self-test && bash scripts/check-docs-gate.sh && git diff --check`
Expected: every command exits 0.
