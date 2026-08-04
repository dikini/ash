# TASK-2068 Scoped Grouped Ordinary-Function Imports Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Preserve a parsed span for each member of an inherited grouped `crate::` import, then
add one dedicated, binding-only scoped route for groups of ordinary-function imports.

**Architecture:** First extend the parser carrier to make every `UseItem` an immutable
`{ name, alias, span }` fact, where `span` covers that member alone rather than its enclosing
`use`. Then add a scope-backed resolver in `canonical_simple_import_planner` that consumes only
inherited `UsePath::Nested` routes rooted at `crate` or an existing structural child path, resolves
every member before staging a result, detects cycles over the complete staged group, and publishes
only an opaque plan. A public entry point in the private
`canonical_structural_module_binder` module delegates to that resolver and projects only its
success through `into_bound_set`; the generic resolver and binder remain unchanged.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-typeck`, `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

SPEC-103 §§5, 6, 8, and 9 require parsed source provenance, visibility before binding,
deterministic import-cycle rejection, atomic publication, no runtime authority, and eventual
file/inline parity. This planned M-GROUP slice is `partial / none / below_spec`, with Type
`partial`, Core/CPS/admission-runtime `not_applicable`, verification `not_implemented`, and
run-route impact `prerequisite` only.

The admitted grammar is deliberately only inherited grouped ordinary-function imports:

```ash
use crate::{root_a, root_b as local_b};
use crate::api::{deep_a, deep_b as local_b};
```

The `UsePath::Nested` base must begin with `crate` and may traverse only delivered structural
children. Each member names exactly one ordinary function; without `as <local-name>`, its final
function segment is the natural local binding name. Each `UseItem::span` must cover its own name
and optional alias, so identity facts and every member-specific visibility, collision, duplicate,
or cycle diagnostic retain the failing member's parser anchor rather than the enclosing `use`
span.

Every structural segment and each target function must satisfy the existing canonical
`ModuleKey` visibility predicate before any local name is staged. Public targets retain the whole
public structural-path fence. The complete group is one transaction: any malformed route, snapshot
mismatch, inaccessible path or target, unsupported member, local collision, duplicate staged local
name, or discovered cross-module `CanonicalImportCycle` returns no plan and no
`CanonicalBoundModuleSet`. Same-module aliases retain their no-edge behavior.

Out of scope: any change to `resolve_simple_parsed_imports`, `bind_simple_parsed_uses`,
`canonical_module_binder.rs`, the generic error contract, or the existing scoped simple and
explicit-alias structural routes; globs, `pub use`, non-inherited import visibility, `self`/`super`
or standard-library bases, nested groups, qualified member paths, non-function targets, other
namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and client
parity. No commit is authorized by this plan or its future implementation.

## TDD implementation tasks

### Task 1: Add the red parser group-member span contract

**Files:**

- Modify: `crates/ash-parser/src/use_tree.rs`
- Modify: `crates/ash-parser/src/parse_use.rs`
- Modify: `crates/ash-parser/src/import_resolver/tests.rs`
- Modify: `crates/ash-parser/tests/multi_crate_visibility.rs`
- Test: `crates/ash-parser/src/parse_use.rs`
- Test: `crates/ash-parser/src/use_tree.rs`

1. Add a failing parser test for `use crate::api::{first, second as local_second};`. Assert that
   the `UsePath::Nested` base remains `crate::api`, that both members preserve their existing name
   and alias values, and that their `UseItem::span`s are distinct, exact source ranges: `first` for
   the natural member and `second as local_second` for the aliased member. Record
   `TEST-MOD-REAL-004-PARSED-GROUP-MEMBER-SPAN`.
2. Run the focused parser test. Expected: it does not compile because `UseItem` has no `span`.
3. Add `span: Span` to `UseItem`; capture the parser cursor before its name and after its optional
   alias, using the established `current_span`/`Span::new` parser convention so the member span
   excludes braces, commas, surrounding whitespace, and the enclosing `Use::span`.
4. Update every in-tree `UseItem` literal in `use_tree` tests, parser import-resolver tests, and
   multi-crate visibility tests with explicit deterministic spans. Do not use `Span::default()` as
   a production parser result.
5. Re-run the focused parser test and affected parser suite. Expected: exact member-span assertions
   pass without changing `UsePath::Simple`, `UsePath::Glob`, or enclosing-use span behavior.

### Task 2: Add the red scoped grouped-import contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_scoped_grouped_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Inspect: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs` (generic-only fence; do not modify)

1. Cover root and deep inherited grouped routes containing both natural and explicit-alias members;
   require all member bindings only through the new scoped group plan. Record
   `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-POSITIVE`.
2. Compare each member binding with the scoped group plan projection; require defining `ModuleKey`,
   declaration span, origin, visibility, local spelling, and the precise member `UseItem::span` to
   remain distinct. Record `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-IDENTITY`.
3. Reject each inaccessible structural segment or function with the exact existing structural
   diagnostic anchored at the failing group member. Cover public, crate, super, `pub(in path)`,
   inherited/private, and self target regions. Record
   `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`.
4. Reject a member that collides with a local ordinary function before any group projection.
   Record `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-LOCAL-COLLISION`.
5. Reject natural/natural, natural/alias, and alias/alias duplicate local names across the group
   atomically, with the duplicate member span retained. Record
   `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-DUPLICATE-BINDING`.
6. Add a later cross-module cycle after earlier valid members; require the outer
   `ImportCycle { edges: CanonicalImportCycle }` and no plan or binding projection. Record
   `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-CYCLE-ATOMICITY`.
7. Compare equivalent file and inline graphs through only the group binding projection, including
   per-member spans. Record `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-FILE-INLINE-PARITY`.
8. Add a 16-case `proptest!` over root/deep bases, member names, alias choices, member order, and
   all six permitted visibility categories; require every successful member to equal the resolver
   plan projection. Record `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-PROPERTY`.
9. Add source/API fences: only the dedicated scoped group binder consumes the group resolver;
   `canonical_module_binder.rs` remains generic-only and mentions neither scopes, the group
   resolver, nor `CanonicalStructuralImportError`; no final-interface, Core/CPS, admission, or
   runtime path is reached. Record `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-AUTHORITY-FENCE`.
10. Run `cargo test -p ash-typeck --test task_2068_scoped_grouped_ordinary_function_imports`.
    Expected: FAIL because the scoped group resolver and dedicated binder do not exist.

### Task 3: Implement the parser-to-binder grouped route

**Files:**

- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_grouped_ordinary_function_imports.rs`

1. Add a scoped-only
   `resolve_scoped_grouped_ordinary_function_imports_with_scopes(graph, scopes)` beside the
   delivered scoped-simple resolver. It recognizes only inherited `UsePath::Nested` values whose
   base starts with `crate`, and it retains every member's `UseItem::span` in resolved facts and
   error anchors.
2. Resolve the base structural path once, then preflight every member's ordinary-function target,
   visibility, natural/explicit local name, local collision, and duplicate staged binding name
   before constructing a result. Build all cross-module edges, run canonical cycle detection over
   the complete staged group, and only then construct an opaque plan.
3. Add only the dedicated delegating API:

   ```rust
   pub fn bind_scoped_grouped_ordinary_function_imports(
       graph: &CanonicalModuleGraph,
       scopes: &CanonicalProvisionalModuleScopes,
   ) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
       resolve_scoped_grouped_ordinary_function_imports_with_scopes(graph, scopes)
           .map(|plan| plan.into_bound_set())
   }
   ```

4. Keep the structural-binder module private and re-export only this named dedicated API from
   `lib.rs`. Do not change `canonical_module_binder.rs`, `bind_simple_parsed_uses`, either generic
   resolver signature, result type, or generic planner delegation.
5. Run the focused parser and typechecker targets. Expected: all reserved group witnesses pass and
   every output/error remains parser-span and resolver equivalent.

### Task 4: Guard regressions and promote only earned evidence

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_scoped_grouped_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs`
- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Re-run the generic parsed-import/binder tests and the delivered simple/structural scoped targets.
   Confirm the generic grammar, error contracts, and existing route outputs are unchanged.
2. Confirm all in-tree manually constructed grouped `UseItem`s carry deterministic test spans and
   that member errors never substitute the enclosing `Use::span`.
3. Run `cargo fmt --check`, `cargo clippy -p ash-parser -p ash-typeck --all-targets --all-features
   -- -D warnings`, parser/typechecker suites, and the documentation validators. Request review for
   span precision, transactionality, cycle provenance, delegation-only design, and authority
   containment.
4. Replace only the deferred implementation/test nodes and edges with concrete source/test anchors
   after GREEN. Classify parser-span, positive, identity, file/inline, and property witnesses as
   positive; visibility, local-collision, duplicate-binding, and authority witnesses as negative;
   and the cycle witness as mutation evidence. Report only `partial / tested / below_spec` after
   focused tests are green; tests are not proof, final-interface evidence, or client parity.

## Handoffs and completion boundary

The first handoff produces a syntax-only `UseItem` member span; it does not bind or authorize an
import. The second consumes the parser `UsePath::Nested` and member-span facts plus delivered
canonical provisional scopes; it produces only a binding-only opaque group plan and, after the
dedicated binder's projection, a `CanonicalBoundModuleSet`. Its run-route impact is
`prerequisite`: TASK-2069 remains the consuming lowering/Engine-transport owner, while TASK-2064
owns file/inline and client parity. TASK-2068 retains complete interface/import/binder ownership,
so this route remains `partial / below_spec` until every target clause is realized.

## Traceability reservation

Reserve `IMPL-MODULE-SCOPED-GROUPED-ORDINARY-FUNCTION-IMPORTS` and exactly these deferred test
witnesses, all linked to `SEM-MODULE-REALIZATION-004`:

- `TEST-MOD-REAL-004-PARSED-GROUP-MEMBER-SPAN`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-POSITIVE`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-IDENTITY`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-LOCAL-COLLISION`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-DUPLICATE-BINDING`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-CYCLE-ATOMICITY`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-FILE-INLINE-PARITY`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-PROPERTY`
- `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-AUTHORITY-FENCE`

No source, test, changelog, task-status, or phase-status mutation is authorized during this
documentation-only activation.
