# TASK-2068 Scoped Simple Ordinary-Function Imports Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Add a dedicated, binding-only scoped import route for inherited simple ordinary-function
imports with an optional explicit alias and a natural final-name binding when `as` is absent.

**Architecture:** A new scoped-only resolver in `canonical_simple_import_planner` owns route
recognition, canonical scope/visibility preflight, local-name selection, duplicate/collision
rejection, and atomic cycle detection. A second public entry point in the private
`canonical_structural_module_binder` module delegates only to that resolver and projects its
successful opaque plan through `into_bound_set`; `lib.rs` alone re-exports the API. The generic
resolver and `canonical_module_binder` remain unchanged and never learn this grammar.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

SPEC-103 §§5, 6, 8, and 9 require visibility before binding, canonical import-cycle rejection,
and atomic non-authorizing `M-BIND` publication. This planned slice is `partial / none /
below_spec`, Type `partial`, Core/CPS/admission-runtime `not_applicable`, verification
`not_implemented`; its run-route impact is `prerequisite` only.

The admitted inherited grammar is deliberately bounded to an ordinary function at either the
crate root or below existing structural children:

```ash
use crate::<ordinary-function>;
use crate::<ordinary-function> as <local-name>;
use crate::<structural-child>...::<ordinary-function>;
use crate::<structural-child>...::<ordinary-function> as <local-name>;
```

When `as <local-name>` is absent, the binder uses the target ordinary function's final segment as
the natural local name. The target can be public, `pub(crate)`, `pub(super)`, `pub(in path)`,
inherited/private, or `pub(self)` only if the delivered `ModuleKey` predicate permits the importer.
For public targets, every structural child remains subject to the whole public-path fence.

The scoped resolver must return its existing structural diagnostics unchanged and reject local
function collisions, duplicate staged binding names, and a discovered cross-module
`ImportCycle { edges: CanonicalImportCycle }` before returning any plan or binding set. Same-module
aliases retain the delivered no-edge behavior. This projection issues no authority for final
interfaces, re-exports, Core/CPS, Engine, admission, runtime, or client parity.

Out of scope: changing `resolve_simple_parsed_imports`, `bind_simple_parsed_uses`,
`canonical_module_binder.rs`, the generic grammar/error contract, or the delivered explicit-alias
scoped binder route; `pub use`, groups, globs, non-`crate` paths, non-function targets, remaining
namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity. No
commit is authorized by this plan or its future implementation.

## TDD implementation tasks

### Task 1: Create the red scoped-simple import contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs` (generic-only fence; do not modify)

1. Add root and deep inherited simple import fixtures without `as`; require the binding's local
   spelling to equal the final ordinary-function segment. Record
   `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-NATURAL-NAME`.
2. Compare each natural-name and explicit-alias result with the new scoped resolver plan's
   `into_bound_set` projection, preserving defining `ModuleKey`, declaration/use spans, origin,
   and visibility. Record `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-IDENTITY`.
3. Cover crate-root ordinary-function targets with and without `as`; record
   `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-ROOT-TARGET`.
4. Add accepted and rejected public, crate, super, `pub(in path)`, inherited/private, and self
   target cases, retaining structural public-path diagnostics. Record
   `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-VISIBILITY`.
5. Add a local ordinary-function name collision and require the exact resolver error with no
   binding set. Record `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-LOCAL-COLLISION`.
6. Add two imports that stage the same natural or explicit local name and require atomic duplicate
   rejection. Record `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-DUPLICATE-BINDING`.
7. Add a later cross-module cycle after an earlier valid natural-name binding; require the outer
   `ImportCycle` and no binding projection. Record
   `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-CYCLE-ATOMICITY`.
8. Compare equivalent file and inline graphs through only the binding projection. Record
   `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-FILE-INLINE-PARITY`.
9. Add a 16-case `proptest!` over generated function/alias names, root/deep positions, and all six
   permitted visibility categories; require natural-name or explicit-alias output to equal the
   scoped resolver projection. Record `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-PROPERTY`.
10. Add source/API fences: only the dedicated scoped binder consumes the new scoped resolver;
    `canonical_module_binder.rs` remains generic-only and mentions neither scopes, the scoped
    resolver, nor `CanonicalStructuralImportError`; no final-interface, Core/CPS, admission, or
    runtime path is reached. Record
    `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-AUTHORITY-FENCE`.
11. Run `cargo test -p ash-typeck --test task_2068_scoped_simple_ordinary_function_imports`.
    Expected: FAIL because the scoped-simple resolver and dedicated binding API do not exist.

### Task 2: Implement the scoped-only resolver and delegating binder API

**Files:**

- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`

1. Add a scoped-only `resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)`
   API beside the delivered scope-backed resolver. Keep `resolve_simple_parsed_imports` unchanged.
2. Recognize only inherited simple `crate::` ordinary-function paths at the root or through scoped
   structural children. Select an explicit alias when present, otherwise the target's final
   function segment; stage no name until all scope, path, visibility, collision, duplicate, and
   cycle checks pass.
3. Add `bind_scoped_simple_ordinary_function_imports(graph, scopes)` only to
   `canonical_structural_module_binder.rs`:

   ```rust
   pub fn bind_scoped_simple_ordinary_function_imports(
       graph: &CanonicalModuleGraph,
       scopes: &CanonicalProvisionalModuleScopes,
   ) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
       resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)
           .map(|plan| plan.into_bound_set())
   }
   ```

4. Keep the structural-binder module private and re-export only this named dedicated API from
   `lib.rs`. Do not change `canonical_module_binder.rs`, `bind_simple_parsed_uses`, its imports,
   signature, result type, or generic planner delegation.
5. Run the focused target. Expected: all ten scoped-simple witnesses pass and errors/output are
   exactly resolver-equivalent.

### Task 3: Guard authority and regressions

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Test: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs`

1. Re-run generic parsed-import/binder tests and confirm the generic grammar, binding shape, and
   `CanonicalModuleBindError` contract are unchanged.
2. Re-run the delivered scoped structural binder and scope17 targets. Confirm the new route does
   not bypass snapshot, whole-public-path, local-collision, duplicate, or cycle fencing.
3. Confirm `canonical_module_binder.rs` remains unchanged and contains no scoped resolver, scope,
   or `CanonicalStructuralImportError` reference; only `lib.rs` exports the dedicated scoped-simple
   binding API.
4. Run `cargo fmt --check`,
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, and affected tests.
   Request review for delegation-only design, natural-name selection, error identity, atomic
   projection, and authority containment.

### Task 4: Promote only earned evidence

**Files:**

- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Replace only `IMPL-MODULE-SCOPED-SIMPLE-ORDINARY-FUNCTION-IMPORTS` and its ten deferred
   scoped-simple test nodes/edges with concrete source/test anchors.
2. Classify natural-name, identity, root-target, file/inline, and property witnesses as positive;
   visibility, local-collision, duplicate-binding, and authority fences as negative; and the cycle
   witness as mutation evidence.
3. Refresh every changed Type-layer source fingerprint. Report only `partial / tested /
   below_spec` when focused tests are green; tests are not proof, final-interface evidence, or
   client parity.
4. Run semantic-record, traceability, orientation, documentation-gate, and diff checks. Keep
   TASK-2068 and Phase 207 In progress, leave TASK-2069 unstarted, and do not update
   `CHANGELOG.md` for this bounded evidence work.

## Handoffs and completion boundary

This reservation consumes only the future scoped-simple resolver's opaque plan and projects a
`CanonicalBoundModuleSet`; it produces no resolution or authority facts. Its run-route impact is
`prerequisite`: TASK-2069 remains the consuming lowering/Engine-transport owner and TASK-2064 owns
integration/client parity. TASK-2068 retains complete interface/import/binder ownership, so this
dedicated route remains `partial / below_spec`.
