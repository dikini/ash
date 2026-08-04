# TASK-2068 Scoped Structural Binder Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Add a dedicated binding-only projection for the delivered scope-backed structural
resolver without changing the generic canonical binder's grammar or authority.

**Architecture:** `bind_scoped_structural_parsed_uses(graph, scopes)` is defined only in the new
`canonical_structural_module_binder` module and re-exported only by `lib.rs`. It delegates only to
`resolve_simple_parsed_imports_with_scopes(graph, scopes)` and projects a successful opaque plan
through `into_bound_set`; it owns no independent collection, path resolution, visibility, cycle, or
publication logic. The existing `canonical_module_binder` and its
`bind_simple_parsed_uses` entry point remain behaviorally and textually unchanged: the generic
module must not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

The authority is SPEC-103 §§5, 6, 8, and 9: canonical scoped import edges, visibility before
registration, `M-IMPORT-CYCLE`, atomic `M-BIND`, and non-authorizing Type handoffs. This planned
slice is `partial / none / below_spec`, Type `partial`, Core/CPS/admission-runtime
`not_applicable`, verification `not_implemented`; run-route impact is `prerequisite` only.

The dedicated binder admits only the delivered scope-backed inherited explicit-alias grammar:

```ash
use crate::<structural-child>...::<ordinary-function> as <local-alias>;
```

The target is an ordinary function, not only a public function. Its declared visibility is admitted
only when the delivered canonical `ModuleKey` predicate permits the importing module: public,
`pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, and `pub(self)` all retain their
existing regions. For a public function, the resolver still independently validates the complete
structural public path; a declaration-level public query cannot bypass a non-public child.

The binder must propagate the resolver's `CanonicalStructuralImportError` exactly, including
scope-snapshot, structural/visibility, local collision, and outer `ImportCycle {
edges: CanonicalImportCycle }` diagnostics. It publishes `CanonicalBoundModuleSet` only after the
resolver's cycle-free atomic plan succeeds. It does not expose import edges, manufacture a success,
or grant final-interface, re-export, Core/CPS, Engine, admission, runtime, or parity authority.

Out of scope: changing `crates/ash-typeck/src/canonical_module_binder.rs`,
`bind_simple_parsed_uses`, the generic planner, their grammar/error type, or any compatibility
conversion; `pub use`, groups, globs, non-`crate` paths, non-function
targets, all final interfaces/export closure, remaining namespace/binder clauses, Core/CPS,
Engine, admission, runtime, and end-to-end parity. No commit is authorized by this plan or its
future implementation.

## TDD implementation tasks

### Task 1: Create the red dedicated-binder contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs` (generic-only fence; do not modify)

1. Add a permitted structural alias fixture and assert the binding-only result preserves defining
   `ModuleKey`, name, declaration span, origin, and declared visibility. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-POSITIVE`.
2. Compare the dedicated binder's result with the same scoped resolver plan projected through
   `into_bound_set`; require no independent route facts. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-DELEGATION`.
3. Add an inaccessible child/function fixture and assert the exact anchored structural visibility
   error reaches the binder unchanged. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-VISIBILITY-DIAGNOSTIC`.
4. Add permitted and rejected `pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, and
   `pub(self)` target cases using canonical importer keys. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-RESTRICTED-VISIBILITY`.
5. Add a late structural cycle after an earlier valid alias. Require the outer `ImportCycle` error
   and no binding-only result. Record `TEST-MOD-REAL-004-SCOPED-BINDER-CYCLE-ATOMICITY`.
6. Compare equivalent file and inline graphs through only the binding projection. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-FILE-INLINE-PARITY`.
7. Add a bounded `proptest!` over permitted module/alias names and selected visibility regions.
   Require retained definition identity and exactly resolver-equivalent binding output. Record
   `TEST-MOD-REAL-004-SCOPED-BINDER-PROPERTY`.
8. Add source/API fences: generic `bind_simple_parsed_uses` remains generic-only, the dedicated
   binder delegates only to the scoped resolver, and no final-interface, Core/CPS, admission, or
   runtime path is reached. Record `TEST-MOD-REAL-004-SCOPED-BINDER-AUTHORITY-FENCE`.
9. Run `cargo test -p ash-typeck --test task_2068_scoped_structural_binder`.
   Expected: FAIL because no dedicated scope-backed binding entry point exists.

### Task 2: Implement the binding-only projection

**Files:**

- Create: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`

1. Import `CanonicalProvisionalModuleScopes`, `CanonicalStructuralImportError`, and
   `resolve_simple_parsed_imports_with_scopes` only in
   `canonical_structural_module_binder.rs`.
2. Add:

   ```rust
   pub fn bind_scoped_structural_parsed_uses(
       graph: &CanonicalModuleGraph,
       scopes: &CanonicalProvisionalModuleScopes,
   ) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
       resolve_simple_parsed_imports_with_scopes(graph, scopes)
           .map(|plan| plan.into_bound_set())
   }
   ```

3. Declare the new module privately in `lib.rs` and re-export only its named dedicated API from
   `lib.rs`. Do not alter
   `canonical_module_binder.rs`: `bind_simple_parsed_uses`, its imports, signature, result type,
   and generic planner delegation remain unchanged, and that module must not mention scopes, the
   scoped resolver, or `CanonicalStructuralImportError`.
4. Run the focused target. Expected: all eight dedicated-binder witnesses pass and errors/output
   are resolver-equivalent.

### Task 3: Guard authority and regressions

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Test: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`

1. Re-run generic parsed-import/binder tests. Confirm the generic binder's grammar, binding shape,
   and `CanonicalModuleBindError` contract are unchanged.
2. Re-run scope17 to prove the new structural-binder module introduces no alternate
   visibility/cycle path and preserves snapshot, structural-public-path, and atomicity fencing.
3. Confirm the generic `canonical_module_binder.rs` remains unchanged and contains no scoped
   resolver, scope, or `CanonicalStructuralImportError` reference; the only public dedicated API
   export is the one from `lib.rs`.
4. Run `cargo fmt --check`,
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, and affected tests.
   Request review for delegation-only design, error identity, atomic projection, and authority
   containment.

### Task 4: Promote only earned evidence

**Files:**

- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Replace only `IMPL-MODULE-SCOPED-STRUCTURAL-BINDER` and the eight deferred binder nodes/edges
   with concrete source/test anchors.
2. Classify positive/delegation/file-inline/property witnesses as positive evidence; visibility and
   authority fences as negative evidence; and cycle rejection as mutation evidence.
3. Refresh every changed Type-layer source fingerprint. Report only `partial / tested /
   below_spec` when focused tests are green; tests are not proof, final-interface evidence, or
   client parity.
4. Run semantic-record, traceability, orientation, documentation-gate, and diff checks. Keep
   TASK-2068 and Phase 207 In progress, leave TASK-2069 unstarted, and do not update
   `CHANGELOG.md` for this planned/bounded evidence work.

## Handoffs and completion boundary

This reservation consumes only the delivered scope-backed resolver's successful opaque plan and
projects a `CanonicalBoundModuleSet`; it produces no new resolution facts. Its run-route impact is
`prerequisite`: TASK-2069 remains the consuming lowering/Engine-transport owner and TASK-2064 owns
integration/client parity. TASK-2068 retains complete interface/import/binder ownership, so this
dedicated binder remains `partial / below_spec`.
