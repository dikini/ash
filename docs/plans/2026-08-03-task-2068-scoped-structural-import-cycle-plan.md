# TASK-2068 Scoped Structural Import-Cycle Gate Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Make the existing scope-backed inherited explicit-alias `crate::` function route reject
deterministically detected cross-module structural import cycles before publishing its opaque plan.

**Architecture:** Keep `CanonicalProvisionalModuleScopes` and the generic binder unchanged. The
scope-backed resolver first completes existing structural-path, declaration-snapshot, visibility,
target, and local-collision preflight, staging only cross-module resolved edges. It then reuses the
canonical deterministic edge-cycle algorithm to reject `CanonicalImportCycle` provenance through a
new outer structural error; only a cycle-free staged set becomes the existing non-authorizing Type
result.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

The authority is SPEC-103 §§5, 6, 8, and 9: canonical import-edge identity/provenance, visibility
before registration, `M-IMPORT-CYCLE`, `M-BIND`, diagnostic preservation, and failure atomicity.
This reservation is `partial / none / below_spec`, Type `partial`, Core/CPS/admission-runtime
`not_applicable`, verification `not_implemented`; its run-route impact is `prerequisite` only.
It does not authorize lowering, admission, runtime execution, or parity.

The admitted source route remains exactly the delivered scope-backed form:

```ash
use crate::<structural-child>...::<ordinary-function> as <local-alias>;
```

Only inherited explicit-alias simple `crate::` paths whose structural children and final ordinary
function have already passed the delivered scope/visibility rules participate. The gate runs after
all selected candidates have been resolved into deterministic `CanonicalSimpleImportEdge` values
and before `CanonicalResolvedSimpleImports` is constructed. An edge where importer and defining
module are identical is not emitted and cannot form a cycle.

Structural diagnostics keep precedence. A missing/mismatched scope, unsupported route form,
unresolved segment, inaccessible child/function, or local declaration collision returns its existing
`CanonicalStructuralImportError` before cycle detection. In particular, a visibility-denied edge
that could otherwise complete a cycle returns its anchored accessibility error, never an
`ImportCycle` error. The new outer cycle variant carries `CanonicalImportCycle`, retaining the
ordered canonical edges including the closing edge and every importer/definer identity, local
alias, use/declaration span, origin, and visibility fact.

All work is staged: a detected cycle returns no `CanonicalResolvedSimpleImports`, bindings, or
cross-module edge result. The generic `resolve_simple_parsed_imports` and
`bind_simple_parsed_uses` routes remain unchanged because they implement a different grammar and
already retain their separate cycle contract. This is not final-interface, re-export, generic
binder, Core/CPS, Engine, admission, runtime, file/inline client, or CLI/daemon parity authority.
No commit is authorized by this plan or its future implementation.

## TDD implementation tasks

### Task 1: Create the red scoped-cycle contract

**Files:**

- Modify: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Inspect: `crates/ash-typeck/src/canonical_provisional_module_scopes.rs`

1. Add a two-module structural cycle fixture. Assert the outer structural error is
   `ImportCycle { edges: CanonicalImportCycle }`, the ordered two edges retain importing/defining
   `ModuleKey`, alias, use/declaration spans, origins, and the closing edge. Record
   `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-CYCLE-DIAGNOSTIC`.
2. Add an acyclic leading/tail dependency plus one reachable internal cycle. Assert that the
   returned `CanonicalImportCycle` contains only the ordered closing-cycle edges, not unrelated
   edges. Record `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-TAIL-CYCLE-PROVENANCE`.
3. Add a same-module alias alongside a cross-module acyclic edge. Assert same-module resolution
   produces a binding but no `CanonicalSimpleImportEdge` and cannot create a self-cycle. Record
   `TEST-MOD-REAL-004-SCOPED-SAME-MODULE-NO-EDGE`.
4. Make an otherwise cyclic path cross a private/restricted structural child. Require the existing
   anchored `Inaccessible` error before cycle collection/result publication. Record
   `TEST-MOD-REAL-004-SCOPED-CYCLE-VISIBILITY-PRECEDENCE`.
5. Compare inline and file-backed graphs with the same cycle. Normalize the cycle edge sequence
   and require identical identity/local-alias/visibility projections; this is Type representation
   parity only. Record `TEST-MOD-REAL-004-SCOPED-CYCLE-FILE-INLINE-PARITY`.
6. Add a bounded generated/property case over canonical module names and alias spellings. For each
   generated reachable cycle, require deterministic ordered closing provenance and no result.
   Record `TEST-MOD-REAL-004-SCOPED-CYCLE-PROPERTY`.
7. Stage a valid early cross-module alias before a late cycle. Require the whole returned plan to
   be absent, proving no early binding or edge is published. Record
   `TEST-MOD-REAL-004-SCOPED-CYCLE-ATOMICITY`.
8. Add source/API fences proving the new cycle gate is callable only from the scope-backed route;
   the generic planner and compatibility binder remain unchanged, and no final-interface, Core,
   CPS, admission, or runtime authority is introduced. Record
   `TEST-MOD-REAL-004-SCOPED-CYCLE-AUTHORITY-FENCE`.
9. Run
   `cargo test -p ash-typeck --test task_2068_canonical_provisional_module_scopes`.
   Expected: the target fails because scope-backed resolution returns staged results without a
   post-collection cycle gate and has no structural outer `ImportCycle` variant.

### Task 2: Add the structural cycle error and deterministic gate

**Files:**

- Modify: `crates/ash-typeck/src/canonical_provisional_module_scopes.rs`
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Test: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`

1. Add an `ImportCycle { edges: CanonicalImportCycle }` variant to
   `CanonicalStructuralImportError`. Preserve the existing structural variants and their anchored
   fields unchanged; do not erase them into a generic cycle string or reuse the generic planner's
   error type.
2. Expose the existing deterministic `find_cycle` helper only at the narrow crate-visible boundary
   needed by `resolve_simple_parsed_imports_with_scopes`; do not create a public cycle API or a
   second algorithm.
3. Continue to accumulate `CanonicalSimpleImportEdge` only when importer and defining module
   differ. After every candidate/visibility/collision preflight has succeeded, run `find_cycle` on
   the deterministic staged edge order. Map a result to the new structural `ImportCycle` error
   before constructing `CanonicalResolvedSimpleImports`.
4. Keep all provisional scopes, candidate resolution, structural visibility, declaration snapshot
   equality, error anchoring, and local collision ordering intact. A visibility failure must leave
   cycle detection unreached; a cycle must leave no returned binding or edge set.
5. Run the focused target. Expected: all eight new witnesses and the delivered nine scope tests
   pass.

### Task 3: Protect grammar, authority, and regression boundaries

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect/Test: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs`

1. Re-run generic parsed-import planner/binder cycle tests. Confirm their existing grammar and
   `CanonicalModuleBindError::ImportCycle` behavior remain independent; do not route the binder
   through provisional scopes.
2. Re-run the delivered provisional-scope target's visibility, declaration-snapshot, public-path,
   and authority fences to prove the cycle gate neither weakens preflight nor widens scope
   authority.
3. Run `cargo fmt --check`,
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, and the affected
   `ash-typeck` tests. Request review for canonical ordering, provenance preservation, diagnostic
   precedence, atomic publication, and separation from generic-binder/runtime authority.

### Task 4: Promote only earned evidence

**Files:**

- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Replace only `IMPL-MODULE-SCOPED-STRUCTURAL-IMPORT-CYCLE-GATE` and the eight deferred scoped
   cycle nodes/edges with concrete source and test anchors.
2. Classify cycle diagnostic, tail provenance, same-module-no-edge, file/inline, and property
   witnesses as positive evidence; visibility precedence and authority fence as negative evidence;
   and late-cycle absence of a result as mutation evidence.
3. Refresh every changed Type-layer source fingerprint. Report this gate as
   `partial / tested / below_spec` only if the focused witnesses are green; tests are not proof,
   final-interface evidence, or client parity.
4. Run semantic-record, traceability, orientation, documentation-gate, and diff checks. Keep
   TASK-2068 and Phase 207 In progress, leave TASK-2069 unstarted, and do not update
   `CHANGELOG.md` for this planned or bounded-evidence work.

## Handoffs and completion boundary

This reservation consumes the delivered canonical provisional scopes and the scope-backed route's
staged structural import edges. It would produce only a cycle-free opaque Type plan or an outer
structural error retaining `CanonicalImportCycle` provenance; the output remains non-authorizing.
Its run-route impact is `prerequisite`: TASK-2069 is the consuming lowering/Engine-transport owner,
while TASK-2064 separately owns integration/client parity. TASK-2068 still owns all remaining
SPEC-103 interface/import/visibility/binder clauses, so this gate remains `partial / below_spec`.
