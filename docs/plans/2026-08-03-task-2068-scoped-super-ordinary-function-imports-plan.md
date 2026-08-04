# TASK-2068 Scoped `super` Ordinary-Function Imports Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Add one Type-only, binding-only route for inherited ordinary-function imports that begin with exactly one `super`.

> **Execution outcome (2026-08-03):** The bounded M-SUPER route is now delivered as
> `partial / tested / below_spec`. The focused target passes 12/12, including a 16-case property.
> This document preserves the original TDD plan; current evidence is recorded in TASK-2068, the
> semantic coverage map, and the traceability graph. This is historical evidence for TASK-2068's
> completed foundation; Phase 207 remains In progress.

**Architecture:** Keep the existing parser-owned `Use::span` as the only source anchor for this route. A new scope-backed resolver will accept a narrow `UsePath::Simple` shape, start from `ModuleKey::parent()`, resolve only structural children and one ordinary-function target, then stage all facts before returning an opaque plan. The existing private structural binder will expose one named delegating projection through `lib.rs`; generic resolver and binder routes remain untouched.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-typeck`, `proptest`; repository semantic-accounting validators.

---

## Scope and semantic boundary

Authority is SPEC-103 §6 (lexical module paths and visibility), §8 (`M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, and `M-BIND`), and §9 properties 3, 5, and 7, under MOD-REAL-004 / SEM-MODULE-REALIZATION-004. At planning time this M-SUPER slice was `partial / none / below_spec`; the delivered result is `partial / tested / below_spec`: Type and verification are `partial`, Core, CPS, and admission/runtime are `not_applicable`, and its run-route impact remains `prerequisite` only.

The admitted grammar is deliberately narrow:

```ash
use super::parent_function;
use super::sibling::function as local_function;
```

Only an inherited `UsePath::Simple` is admitted. Its importer must be below the root, its first and only path prefix must be `super`, it may traverse zero or more delivered structural child segments from `ModuleKey::parent()`, and it must end in exactly one ordinary function. An optional alias chooses the local binding spelling; otherwise the target function's final name is used. The route carries the existing full `Use::span` into edge, identity, and error facts.

The resolver must reuse the delivered provisional scope snapshot, child-origin, canonical visibility, whole-public-path, local-collision, duplicate-binding, cycle, and atomic-publication checks. Same-module imports emit no edge; this route normally creates parent- or sibling-module edges. It consumes only canonical graph units, parsed-use spans, and TASK-2068 provisional scopes, and produces only an opaque binding plan, a projected `CanonicalBoundModuleSet`, and canonical import edges. It does not create final-interface, Core, CPS, Engine, admission, runtime, or client authority.

Out of scope: root importers; repeated `super`; `self`; `crate`; unprefixed, standard-library, or external bases; groups, globs, and nested groups; `pub use` and restricted uses; module, type, capability, or other non-function targets; generic resolver/binder changes; final interfaces/export closure; Core/CPS; Engine; admission; runtime; and client parity. `self::` is intentionally not added because same-module precedence remains separately unresolved. No commit is authorized by this plan or its implementation.

## TDD implementation tasks

### Task 1: Add the red scoped-`super` resolver and binder contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_scoped_super_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_scoped_grouped_ordinary_function_imports.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Inspect: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs` (generic-only fence; do not modify)

1. Write parent and sibling positive cases with aliases and natural final names. Require the scoped plan and the dedicated binder projection to preserve defining identity, declaration origin/visibility, and the full `Use::span` edge/error anchor. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-POSITIVE`.
2. Compare resolver-plan and binder-projection identity facts for parent and sibling paths, including use span, defining `ModuleKey`, origin, declaration span, visibility, and selected local spelling. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-IDENTITY`.
3. Cover all six existing canonical visibility categories (`pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, and `pub(self)`) with permitted and rejected importer routes. Require the existing structural diagnostic to retain the complete `Use::span`. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-VISIBILITY-DIAGNOSTIC`.
4. Reject a root importer and a repeated `super` prefix before any plan or binding set is returned. Require their diagnostics to use the full parsed use span. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-ROOT-DIAGNOSTIC`.
5. Reject a local ordinary-function collision before projection. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-LOCAL-COLLISION`.
6. Reject the natural/natural, natural/alias, and alias/alias duplicate-local-name forms atomically. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-DUPLICATE-BINDING`.
7. Add a late cycle after an otherwise valid parent or sibling import and require `ImportCycle { edges: CanonicalImportCycle }` with no plan or binding projection. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-CYCLE-ATOMICITY`.
8. Compare normalized file and inline graphs, preserving the full source span assumption in their successful projections. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-FILE-INLINE-PARITY`.
9. Add a 16-case `proptest!` varying parent versus sibling target, alias selection, import order, and all six visibility categories. Require successful projection to equal the scoped resolver plan. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-PROPERTY`.
10. Add source/API fences: only the dedicated scoped-`super` binder consumes the new resolver; `canonical_module_binder.rs` remains byte-for-byte generic-only and mentions neither scopes, the new resolver, nor `CanonicalStructuralImportError`; no final-interface, Core/CPS, Engine, admission, runtime, or client import is introduced. Record `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-AUTHORITY-FENCE`.
11. Run `cargo test -p ash-typeck --test task_2068_scoped_super_ordinary_function_imports`. Expected: FAIL because neither scoped-`super` resolver nor dedicated binder API exists.

### Task 2: Implement the dedicated Type-only route

**Files:**

- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/canonical_structural_module_binder.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_super_ordinary_function_imports.rs`

1. Add only `resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)` beside the delivered scoped routes. Keep `resolve_simple_parsed_imports` and every existing scoped route unchanged.
2. Admit only inherited `UsePath::Simple` inputs from a non-root importer with exactly one leading `super`; call `ModuleKey::parent()` for the starting scope, traverse only structural children, resolve exactly one ordinary-function target, and select an explicit alias or natural final name.
3. Preserve the existing complete `Use::span` in edge and error facts. Reuse the current snapshot, visibility, whole-public-path, local-collision, duplicate, and cycle preflight; stage every result and all cross-module edges before detecting cycles, then publish only a successful opaque plan. Same-module results create no edge.
4. Add only the dedicated delegating API in `canonical_structural_module_binder.rs`:

   ```rust
   pub fn bind_scoped_super_ordinary_function_imports(
       graph: &CanonicalModuleGraph,
       scopes: &CanonicalProvisionalModuleScopes,
   ) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
       resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)
           .map(|plan| plan.into_bound_set())
   }
   ```

5. Keep the structural-binder module private and re-export only the named dedicated API from `lib.rs`. Do not change `canonical_module_binder.rs`, `bind_simple_parsed_uses`, generic resolver signatures, generic error contracts, or generic planner delegation.
6. Run the focused target. Expected: all ten reserved witnesses pass and all errors/output remain resolver-equivalent and atomic.

### Task 3: Guard regressions and promote only earned evidence

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_scoped_super_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_simple_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_grouped_ordinary_function_imports.rs`
- Test: `crates/ash-typeck/tests/task_2068_scoped_structural_binder.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs`
- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`, `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207, AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Re-run generic parsed-import/binder tests and all delivered scoped routes. Confirm that group-member spans remain group-specific while this route continues to use the full `Use::span`.
2. Confirm root, repeated-`super`, `self`, `crate`, unprefixed, standard-library/external, group/glob, `pub use`, restricted-use, and non-function target inputs are rejected rather than silently accepted.
3. Run `cargo fmt --check`, `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, affected Type tests, and the documentation validators. Request review for `ModuleKey::parent()` start selection, visibility precedence, cycle provenance, atomic projection, dedicated delegation, and authority containment.
4. Replace only the deferred implementation/test nodes and edges with concrete source/test anchors after GREEN. Classify positive, identity, file/inline, and property witnesses as positive; visibility, root/repeated-`super`, local-collision, duplicate-binding, and authority fences as negative; and the cycle witness as mutation evidence. Refresh changed source fingerprints. Report only `partial / tested / below_spec`; tests are neither proof nor final-interface or client-parity evidence.
5. Keep TASK-2068 and Phase 207 In progress, keep TASK-2069 unstarted, do not modify `CHANGELOG.md` for this bounded evidence work, and do not commit without user authorization.

## Handoffs and completion boundary

The route consumes TASK-2067 canonical graph units and parser-owned complete `Use::span` facts plus TASK-2068 provisional scopes. It produces a Type-only opaque import plan, binding projection, and canonical edges; it neither produces a checked final interface nor grants execution authority. Its run-route impact is `prerequisite`: TASK-2068 owns the remaining Type/interface/import/binder work, TASK-2069 later owns lowering and Engine transport, and TASK-2064 later owns file/inline plus client parity. This delivered bounded route remains `partial / below_spec` until the full target rule is delivered.

## Historical traceability reservation and promotion

The reserved implementation node is now `implemented`, and these ten witnesses are now `tested`, each linked to `SEM-MODULE-REALIZATION-004` with concrete source/test anchors:

- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-POSITIVE`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-IDENTITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-VISIBILITY-DIAGNOSTIC`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-ROOT-DIAGNOSTIC`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-LOCAL-COLLISION`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-DUPLICATE-BINDING`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-CYCLE-ATOMICITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-FILE-INLINE-PARITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-PROPERTY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-AUTHORITY-FENCE`

The original documentation-only activation authorized no source, test, changelog, task-status, or phase-status mutation. The later TDD implementation and review promoted only the earned bounded evidence; it did not complete TASK-2068 or Phase 207, start TASK-2069, or add a changelog entry.
