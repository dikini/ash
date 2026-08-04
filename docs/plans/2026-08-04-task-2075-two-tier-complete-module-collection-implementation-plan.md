# TASK-2075 Two-Tier Complete Module Collection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Atomically collect every target module declaration into separate checker-internal and
import-facing name-only views.

**Architecture:** `ash-typeck` consumes TASK-2074's immutable expanded graph, classifies each
declaration with canonical identity/namespace/parent keys, stages an internal snapshot and minimal
name view, revalidates every source fact, and publishes both together. Existing provisional scopes
remain a compatibility projection only.

**Tech Stack:** Rust 2024, `ash-typeck`, `ash-parser`, `ash-core::ModuleKey`, existing visibility
helpers, and `proptest`.

---

### Task 1: Activate TASK-2075

**Files:**
- Modify: `docs/plan/tasks/TASK-2075-two-tier-complete-module-collection.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`

**Step 1:** Confirm TASK-2074 is complete, then change exact status to `In progress`.

**Step 2:** Register only TASK-2075's Type-layer collection ownership and deferred witnesses.

**Step 3:** Run the semantic record self-test and manifest; expect PASS.

**Step 4:** Commit with `docs: activate TASK-2075 module collection`.

### Task 2: Establish exhaustive RED definition-domain tests

**Files:**
- Create: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Build a table covering `Notation`, `Macro`, `Capability`, `ResourceType`, `Type`,
`Newtype`, `EffectAlias`, `EffectGroup`, `DataKind`, `TypeFn`, `PropositionPredicate`, `Policy`,
`Role`, `Interface`, `Impl`, `Function`, `Handler`, `BuiltinFn`, `SealedDomain`, `Law`, and `Proof`,
plus `ModuleDecl`.

**Step 2:** Assert the expected namespace/internal-only outcome for each; assert `Capability`
rejects atomically.

**Step 3:** Run `cargo test -p ash-typeck --test task_2075_two_tier_module_collection`; expect
compile failure because collector carriers do not exist.

**Step 4:** Commit RED tests with `test(typeck): specify complete module collection domain`.

### Task 3: Align declared visibility carriers

**Checkpoint:** Complete. The focused parser target passes 5/5 for required AST carriers,
module-scope visibility propagation and full spans, inherited nested scoping, and visible nested
rejection. Policy remains construction-only. The separate collection target remains RED only at
the absent `ash_typeck::canonical_module_collection` import, so it is documented/deferred rather
than a required-success manifest command until the first collector GREEN.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: relevant parser declaration parsers for `Policy`, `Role`, `Law`, and `Proof`
- Modify: relevant parser tests
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Add RED parser/typeck assertions that every collectable policy, role, law, and proof
retains explicit/inherited declared visibility and exact declaration span.

**Step 2:** Add the minimal visibility fields and parser propagation without changing grammar.

**Step 3:** Run affected parser tests and the TASK-2075 target; expect visibility tests PASS and
remaining collector cases RED.

**Step 4:** Commit with `feat(parser): retain collection visibility carriers`.

### Task 4: Define private two-tier carriers

**Checkpoint:** Complete for the bounded carrier layer. Closed domain/namespace/disposition enums,
layout-stable typed identity/lookup/origin keys, eight private/read-only carrier shapes, one
mandatory paired map, module sidecars, derived callable bodies, private exhaustive validation, and
syntax-aware source fences are implemented. At this Task 4 checkpoint, private tests passed 2/2
and filtered domain/fence tests passed while the full target remained 3 pass/1
`CollectorNotImplemented` failure; Task 5 now supersedes that state.

**Files:**
- Create: `crates/ash-typeck/src/canonical_module_collection.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Define closed namespace/declaration-kind enums, canonical identity and lookup keys,
internal collected entries, provisional name entries, and an opaque paired collection result.

**Step 2:** Keep constructors private. Expose read-only iteration/query APIs that do not allow the
name view to reveal signatures, bodies, equations, checked types, or final exports.

**Step 3:** Add compile/source fence assertions for the forbidden fields and later-layer types.

**Step 4:** Run the focused target; expect carrier-shape tests PASS and collection cases RED.

**Step 5:** Commit with `feat(typeck): add two-tier module collection carriers`.

### Task 5: Implement namespace classification and collision rules

**Checkpoint:** Complete. The collector now stages one graph-wide paired map and passes the 22/22
focused contract for exhaustive namespaces, typed notation keys, parent-aware duplicates,
constructor/member identities, internal-only impl facts, module-qualified interface identity,
alpha-normalized full type/row overlap (including open rows), unresolved-interface rejection, and
late-sibling atomic failure. This advances only the bounded Task 5 slice; Tasks 6–8 remain open.

**Files:**
- Modify: `crates/ash-typeck/src/canonical_module_collection.rs`
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Add RED same-bucket duplicate, allowed cross-bucket spelling, ambiguous-context,
parent-scoped member/constructor, and full-interface-application impl-overlap tests.

**Step 2:** Implement exhaustive `Definition` classification with no wildcard arm, structural
module classification, canonical parents, and origin keys.

**Step 3:** Implement ordinary/newtype, sealed-domain, and promoted constructor placement exactly;
keep impls internal and macro-generated identifiers unspellable.

**Step 4:** Run the focused target and expect namespace/collision cases PASS.

**Step 5:** Commit with `feat(typeck): collect canonical module namespaces`.

### Task 6: Preserve internal facts and minimize the provisional view

**Files:**
- Modify: `crates/ash-typeck/src/canonical_module_collection.rs`
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Add RED assertions for raw declaration/callable shapes, bodies/member spans, expansion
origins/hygiene, source anchors, and ordinals in the internal snapshot.

**Step 2:** Add RED assertions that only the declared name/identity/namespace/visibility/
exportability/origin-anchor/ordinal fields appear in the provisional view.

**Step 3:** Implement graph-wide staging and deterministic source-ordinal assignment.

**Step 4:** Run the focused target and expect view-separation cases PASS.

**Step 5:** Commit with `feat(typeck): separate collected and provisional module views`.

### Task 7: Add complete revalidation and atomic publication

**Files:**
- Modify: `crates/ash-typeck/src/canonical_module_collection.rs`
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`

**Step 1:** Add mutations for name, kind, visibility, signature, body, order, and expansion-sidecar
drift plus a valid sibling followed by a failing sibling.

**Step 2:** Rebuild all declaration facts from the consumed expanded graph and compare before
constructing the paired public result.

**Step 3:** Return keyed/span-anchored errors and discard all staged views on any failure.

**Step 4:** Run the focused target and expect PASS.

**Step 5:** Commit with `feat(typeck): make module collection drift-safe and atomic`.

### Task 8: Add parity, property, compatibility, and authority evidence

**Files:**
- Modify: `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs`
- Modify: `crates/ash-typeck/src/canonical_provisional_module_scopes.rs` only if a projection helper
  is needed without changing existing semantics

**Step 1:** Compare normalized equivalent file/inline collections across both views.

**Step 2:** Add a 16+ case property varying definition variants, namespaces, spelling collisions,
visibility, parents, order, and source form.

**Step 3:** Run TASK-2068 provisional-scope/import targets and TASK-2070 self-alias target unchanged.

**Step 4:** Add an authority fence excluding binding, checked/final interface, Core/CPS, Engine,
admission, and runtime carriers.

**Step 5:** Run the focused and regression targets; expect PASS.

**Step 6:** Commit with `test(typeck): cover complete module collection invariants`.

### Task 9: Review, gates, and closeout

**Files:**
- Modify: `docs/plan/tasks/TASK-2075-two-tier-complete-module-collection.md`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `CHANGELOG.md`

**Step 1:** Run `cargo fmt --check`, strict workspace clippy, affected crate tests, and workspace tests.

**Step 2:** Request spec review, Rust/code review, and independent QA; fix every blocking issue.

**Step 3:** Promote only observed witnesses; retain target-rule `partial / tested / below_spec`.

**Step 4:** Update TASK-2072 to consume only `CanonicalProvisionalNameView` and TASK-2073 to consume
only `CanonicalCollectedModuleSnapshot` plus TASK-2072 staging.

**Step 5:** Run semantic-record, traceability, orientation, docs-gate, and diff checks.

**Step 6:** Commit with `feat(typeck): complete two-tier module collection`.
