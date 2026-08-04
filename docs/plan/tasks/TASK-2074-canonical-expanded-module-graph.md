# TASK-2074: Canonical Expanded Module Graph

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5, and 8 (`M-SYNTAX-PREPASS`, `M-EXPAND`)
**Owned rule:** MOD-REAL-001/002 canonical syntax dependency and expanded graph handoff
**Run-route impact:** prerequisite
**Semantic task record:** [TASK-2074](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2074](../SEMANTIC-RULE-COVERAGE.md#task-2074-canonical-expanded-module-graph)
**Design:** [Canonical Expanded Module Graph](../../plans/2026-08-04-task-2074-canonical-expanded-module-graph-design.md)
**Implementation plan:** [TASK-2074 implementation plan](../../plans/2026-08-04-task-2074-canonical-expanded-module-graph-implementation-plan.md)
**Notation-import design:**
[Parenthesized Notation Import](../../plans/2026-08-04-task-2074-parenthesized-notation-import-design.md)
**Notation-import implementation plan:**
[Parenthesized Notation Import implementation plan](../../plans/2026-08-04-task-2074-parenthesized-notation-import-implementation-plan.md)

## Description

Build the parser-owned expansion boundary between the completed canonical parsed graph and complete
module collection. The complete target carrier consumes and owns `CanonicalModuleGraph`, performs a syntax-only
macro/notation import prepass, shallowly expands each keyed `ModuleBody`, preserves uses and source
order, and publishes an exact one-to-one expanded module map only when the whole graph succeeds.

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The delivered bounded parser expansion is `partial / tested / below_spec`: public `CanonicalExpandedModuleGraph` consumes the exact parsed graph and performs an AST-only prepass for invocation-backed simple canonical public macro imports, public structural provider paths, macro-namespace priority, duplicate-alias rejection, deterministic provider ordering and syntax-cycle provenance, transitive provider closure, provider-owned diagnostics, and read-only syntax-import provenance sidecars. It preserves uses, module declarations, source order, per-key sidecars, exact keys, and atomic failure; unsupported item-generation attempts reject as required. Syntax-prepass evidence is 17/17, shallow-graph evidence is 5/5, and `ash-parser` library evidence is 462/462. The approved 8/8 completion target additionally tests normalized file/inline child projections, acquired typed units after all source files are overwritten and deleted, alias/provider-template mutations, ordinary callable-import notation nonactivation, anchored graph-wide atomic nonmacro rejection, a direct-orchestration/manifest authority fence, and an exhaustive 64-case projection. Canonical public notation-summary transport and eligible notation activation, and the complete TASK-2075 handoff, remain absent. This parser-stage test evidence creates no filesystem, Engine, raw-text, general binding, checked-interface, Core/CPS, runtime, proof, final-interface, or client-parity authority. Parenthesized notation-import implementation and focused test nodes remain deferred; TASK-2074 remains partial / tested / below_spec.

**Layers:** Type `partial`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `partial`.

## Delivered bounded expansion evidence

- **Implementation:** `IMPL-MODULE-CANONICAL-EXPANDED-GRAPH`,
  `IMPL-MODULE-CANONICAL-SYNTAX-PREPASS`, and `IMPL-MODULE-SHALLOW-BODY-EXPANSION`.
- **Positive:** `TEST-MOD-REAL-001-002-LOCAL-SHALLOW-ORDER`,
  `TEST-MOD-REAL-001-002-INLINE-SIDECAR-OWNERSHIP`,
  `TEST-MOD-REAL-001-002-EXACT-KEY-ATOMIC-PUBLICATION`, and
  `TEST-MOD-REAL-001-002-GENERATED-SHALLOW-ORDER-PROPERTY`.
- **Negative:** `TEST-MOD-REAL-001-002-ANCHORED-LATE-EXPANSION-FAILURE`.
- **Mutation:** `TEST-MOD-REAL-001-002-MISSING-DEFINITION-CARDINALITY` and
  `TEST-MOD-REAL-001-002-EXTRA-DEFINITION-CARDINALITY`.
- **Syntax-prepass positive:** `TEST-MOD-REAL-001-002-LOCAL-PUBLIC-MACRO`,
  `TEST-MOD-REAL-001-002-CANONICAL-PUBLIC-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-ORDER`,
  `TEST-MOD-REAL-001-002-TRANSITIVE-PROVIDER-CLOSURE`,
  `TEST-MOD-REAL-001-002-SYNTAX-IMPORT-PROVENANCE`,
  `TEST-MOD-REAL-001-002-MACRO-NAMESPACE-PRIORITY`, and
  `TEST-MOD-REAL-001-002-PUBLIC-MACRO-ALIAS-PROPERTY`.
- **Syntax-prepass negative:** `TEST-MOD-REAL-001-002-PRIVATE-MACRO`,
  `TEST-MOD-REAL-001-002-PRIVATE-STRUCTURAL-PATH`,
  `TEST-MOD-REAL-001-002-NON-MACRO-SYNTAX-IMPORT`,
  `TEST-MOD-REAL-001-002-MISSING-MACRO-SUMMARY`,
  `TEST-MOD-REAL-001-002-DUPLICATE-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-OWNED-DIAGNOSTIC`,
  `TEST-MOD-REAL-001-002-NOTATION-NONLEAKAGE`, and
  `TEST-MOD-REAL-001-002-ITEM-GENERATION-REJECTION`.
- **Syntax-prepass mutation:** `TEST-MOD-REAL-001-002-TWO-MODULE-SYNTAX-CYCLE` and
  `TEST-MOD-REAL-001-002-THREE-MODULE-SYNTAX-CYCLE`.
- **Syntax-prepass verification:**
  `cargo test -p ash-parser --test task_2074_canonical_syntax_prepass` passed 17/17, including an
  exact 16-case key-order property.
- **Focused verification:** `cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph`
  passed 5/5, including an exact 16-case property.
- **Private invariant verification:** `cargo test -p ash-parser --lib` passed 462/462, including
  the two separate missing/extra definition-cardinality units.
- **Regression verification:** `cargo test -p ash-parser --test task_1725_expanded_surface_boundary`;
  `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`;
  `cargo test -p ash-parser --test task_1755_macro_registry_scope`;
  `cargo test -p ash-parser --test task_1756_expression_macro_expansion`;
  `cargo test -p ash-parser --test task_1757_macro_origin_hygiene`;
  `cargo test -p ash-parser --test task_1769_hygienic_binder_macros`;
  `cargo test -p ash-parser --test task_2059_file_inline_module_unit_parity`; and
  `cargo test -p ash-parser --test task_2067_canonical_module_graph` passed 54/54 in aggregate
  (6 + 6 + 7 + 6 + 6 + 3 + 8 + 12).
- **Macro summary/identity regressions:**
  `cargo test -p ash-parser --test task_1763_macro_summary_carriers` and
  `cargo test -p ash-parser --test task_1786_macro_identity` passed 6/6 in aggregate (2 + 4).
- **Quality verification:** `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `git diff --check` passed.
- **Proof/parity:** no proof. The approved normalized file/inline child projection is parser-stage
  test evidence only; it is not a final-interface, lowered/admitted/runtime, or CLI/daemon parity
  relation. TASK-2064 separately owns composed parity.
- **Fingerprints:** graph `sha256:89a1dd937b490c517e0de588d6c1aaec6c30b9bc609331d612b016dd29909d2d`;
  syntax prepass `sha256:f19e72aeaa69d3a62a8a7a4b6fc9cdb26ea1adce1f392a905c6e614928a057cd`;
  shallow seam `sha256:5fa199880a15660e5ad6ad49e84a3ecee66cb5884fd090cdc5e17c5659744977`;
  exports `sha256:4669707bf43daaf1f95776063cb8cb5bb2b1d1cfdd54d103aa130de2c064547d`;
  syntax-prepass test `sha256:76929348ca710fc260742213494cd6eaab3a5454e11dabc5d786ba38ff3426dc`;
  shallow-graph test `sha256:2e81aebd0a0bbc1bfff113270fcd48eb395a690809389f9f732057e8d400fe0e`.

**Next obligation:** Implement and test IMPL-MODULE-CANONICAL-NOTATION-IMPORT and IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION for the approved parenthesized exact structured token/hole selector, then promote TEST-MOD-REAL-001-002-NOTATION-IMPORT-PARSER, TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY, TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION, TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION, and TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION only when their source evidence exists. TASK-2075 remains planned and inactive until that complete atomic expanded graph exists.

## Deferred parenthesized notation-import contract

- **Deferred implementation:** `IMPL-MODULE-CANONICAL-NOTATION-IMPORT` and
  `IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION`.
- **Deferred tests:** `TEST-MOD-REAL-001-002-NOTATION-IMPORT-PARSER`,
  `TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY`,
  `TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION`,
  `TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION`, and
  `TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION`.
- The selector is one exact normalized parsed token/hole pattern and carries no fixity,
  associativity, or precedence. `NotationPattern.raw` and selector raw spelling are diagnostic
  only and must never be reparsed or scanned as semantic matching authority.
- Every eligible public provider variant for the selected pattern is transported deterministically
  with its full key: pattern, fixity, associativity, and precedence. Target callable identity and
  provider/use provenance are retained without binding or authorizing the callable.
- Notation imports support neither `as` nor a notation glob. Ordinary callable imports never
  activate notation. Missing, private, malformed, conflicting, and cyclic dependencies reject the
  graph atomically with the required declaration/use/cycle anchors.
- A provider exports the summary only through `pub` on its notation declaration. TASK-2074 supports
  only inherited-visibility `use module::(pattern)`; `pub use module::(pattern)` and every other
  visibly qualified notation use reject as unsupported. Notation re-export needs a separate future
  contract and owner.
- Activation installs imported summaries in the existing syntax-phase table and preserves hole
  order for downstream resolution. TASK-2074 does not own generalized mixfix use-site parsing or
  elaboration.

## Approved completion-test checkpoint

`cargo test -p ash-parser --test task_2074_expanded_graph_completion` passed **8/8**. This
approved non-notation checkpoint supplies bounded parser-stage test evidence for:

- normalized file/inline child projection parity;
- expansion from acquired typed graph units after source files are overwritten and deleted, with no
  expansion reread;
- observable alias and provider-template mutations;
- ordinary callable import nonactivation of provider notation;
- anchored graph-wide atomic rejection of a nonmacro syntax edge;
- the direct orchestration/manifest authority fence, including no loader, scanner, filesystem, or
  later-layer dependency; and
- exhaustive 64-case depth, source-form, declaration-order, alias, provider-template, and
  function-count projection.

The test does not transport or activate a canonical public notation summary. It does not prove
the target rule or establish final-interface, lowered/admitted/runtime, or client parity; therefore
TASK-2074 remains **In progress** and `partial / tested / below_spec`.

**Fingerprint:** `task_2074_expanded_graph_completion.rs`
`sha256:897c979d1ff025beea266f9ae1633adc43b7e83c61298898ed6b5185264ef347`.

## Tested bounded syntax-prepass target

- **Command:** `cargo test -p ash-parser --test task_2074_canonical_syntax_prepass`.
- **Current state:** GREEN, 17/17, including one exact 16-case key-order property.
- **Delivered boundary:** invocation-backed simple canonical public macro aliases; public
  structural provider visibility; deterministic provider-first ordering and cycle provenance;
  transitive closed-provider consumption; macro-namespace priority; duplicate-alias rejection;
  provider-owned failure context; and read-only syntax-import provenance sidecars.
- **Fail-closed boundary:** notation without a canonical public summary remains inactive, and
  unsupported item-generation attempts reject as required by SPEC-103 §5. This evidence does not
  authorize general binding, filesystem discovery,
  raw-source fallback, Engine behavior, runtime behavior, or parity.

## Requirements

1. Add a parser-owned `CanonicalExpandedModuleGraph` that consumes/owns the input
   `CanonicalModuleGraph` and exposes exactly one expanded module record per parsed `ModuleKey`.
2. Add a shallow `ModuleBody` expansion seam: expand only direct definitions owned by that key;
   retain parsed uses, nested structural declarations, and source order unchanged.
3. Gather public macro and notation summaries from AST only; notation declaration and import
   matching uses structured parsed token/hole parts, never reparsed raw spelling. Resolve only
   syntax imports through canonical keys and exact `Use` spans; reject syntax cycles; expand
   providers before consumers.
4. Parse inherited-visibility parenthesized exact notation selectors; reject visibly qualified
   notation uses, aliases, and globs; transport every
   eligible public full-key variant deterministically, and activate it in the existing syntax-phase
   table without binding or authorizing its target callable. Reject item-generating macro behavior
   outside the declared target domain.
5. Retain source path/artifact origin plus per-module expansion diagnostics, origins, and hygiene.
   Inline-child sidecars occur only in the child record.
6. Validate exact parsed/expanded key equality and fail atomically on prepass, dependency, expansion,
   or invariant errors. Never reuse `ash-engine` module loading.

## TDD steps

1. Add RED tests for the shallow `ModuleBody` API and direct-definition expansion.
2. Add RED AST-only macro/notation summary, topological-order, and syntax-cycle tests.
3. Add RED use/order retention, per-key inline sidecar, and file/inline normalization tests.
4. Add RED mutation, graph-wide atomicity, generated graph property, and no-filesystem/authority
   fence tests.
5. Run existing expansion and canonical-graph regressions before implementation.
6. Implement only after RED; promote only actual source/test evidence.

## Scope and non-goals

No generalized mixfix use-site parsing/elaboration, namespace collection, provisional view,
general import binding, body/type checking, final interface, Core/CPS lowering, Engine
transport/admission/execution, filesystem discovery, source text fallback, or client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071's contract and TASK-2067's canonical parsed graph.
- **Target produces:** one complete atomic, parser-owned `CanonicalExpandedModuleGraph`,
  non-authorizing. The delivered bounded syntax-prepass slice is not yet this complete handoff.
- **Downstream owner:** TASK-2075 alone consumes the completed graph for collection; it remains
  planned and inactive while TASK-2074 is partial.
- **Integration/proof:** TASK-2064 owns composed parity.
- [x] Positive, negative, mutation, normalized file/inline child projection, property,
  acquired-graph no-reread, and direct-orchestration/manifest authority-fence evidence exists
  (8/8 completion target).
- [x] Focused completion evidence passes; previously recorded expansion and graph regressions remain
  task-owned verification.
- [x] The graph-wide nonmacro syntax-edge witness returns one anchored failure rather than a partial
  expanded graph.
- [ ] Canonical public notation-summary transport and eligible notation activation are implemented
  and tested.
- [ ] The complete atomic expanded-graph handoff is ready for TASK-2075.
