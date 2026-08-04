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

## Description

Build the parser-owned expansion boundary between the completed canonical parsed graph and complete
module collection. The complete target carrier consumes and owns `CanonicalModuleGraph`, performs a syntax-only
macro/notation import prepass, shallowly expands each keyed `ModuleBody`, preserves uses and source
order, and publishes an exact one-to-one expanded module map only when the whole graph succeeds.

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The delivered initial local-only slice is `partial / tested / below_spec`: public `CanonicalExpandedModuleGraph` consumes and owns the exact parsed graph, shallowly expands direct definitions only, retains uses, module declarations, source order, and per-key diagnostics/origins/hygiene, publishes exactly one `BTreeMap` record per parsed key, and returns anchored `Expansion` or `BodyInvariant` failures without a partial public value. Focused evidence is 5/5, including one exact 16-case property and anchored late-failure atomicity; `ash-parser` library evidence is 462/462, including separate missing/extra definition-cardinality units. The required AST-only public macro/notation summary import prepass, canonical syntax-import edges and spans, provider-before-consumer topological ordering, syntax-dependency cycle rejection, imported-notation activation, normalized file/inline expansion parity, no-FS/authority fences, broader graph mutations, and complete handoff to TASK-2075 remain absent. The slice supplies no namespace/import binding, checked interface, Core/CPS, Engine, runtime, proof, or client-parity authority.

**Layers:** Type `partial`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `partial`.

## Delivered local-only slice evidence

- **Implementation:** `IMPL-MODULE-CANONICAL-EXPANDED-GRAPH` and
  `IMPL-MODULE-SHALLOW-BODY-EXPANSION`.
- **Positive:** `TEST-MOD-REAL-001-002-LOCAL-SHALLOW-ORDER`,
  `TEST-MOD-REAL-001-002-INLINE-SIDECAR-OWNERSHIP`,
  `TEST-MOD-REAL-001-002-EXACT-KEY-ATOMIC-PUBLICATION`, and
  `TEST-MOD-REAL-001-002-GENERATED-SHALLOW-ORDER-PROPERTY`.
- **Negative:** `TEST-MOD-REAL-001-002-ANCHORED-LATE-EXPANSION-FAILURE`.
- **Mutation:** `TEST-MOD-REAL-001-002-MISSING-DEFINITION-CARDINALITY` and
  `TEST-MOD-REAL-001-002-EXTRA-DEFINITION-CARDINALITY`.
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
- **Quality verification:** `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `git diff --check` passed.
- **Proof/parity:** none. The property is test evidence; no normalized file/inline parity witness or
  proof exists.
- **Fingerprints:** graph `sha256:50f1f30221b5b7fba3da7ec7d2d458051730b796590b6d4b939224f33733fd52`;
  shallow seam `sha256:ac7011a3b78b1164468c5d4f3ad4f77be78491da8f1d0c26497c527f924f34e8`;
  exports `sha256:e03e757ee0813684f9ea3f0f7471f880167a0881a39d3bf72cffa729eb3e7dc9`;
  focused test `sha256:2e81aebd0a0bbc1bfff113270fcd48eb395a690809389f9f732057e8d400fe0e`.

**Next obligation:** Implement and test the AST-only syntax-summary import prepass, stable provider ordering and cycle failures, imported notation, normalized file/inline parity, and no-FS/authority fences before closing TASK-2074. TASK-2075 remains planned and inactive until that complete atomic expanded graph exists.

## Requirements

1. Add a parser-owned `CanonicalExpandedModuleGraph` that consumes/owns the input
   `CanonicalModuleGraph` and exposes exactly one expanded module record per parsed `ModuleKey`.
2. Add a shallow `ModuleBody` expansion seam: expand only direct definitions owned by that key;
   retain parsed uses, nested structural declarations, and source order unchanged.
3. Gather public macro and notation summaries from AST only; resolve only syntax imports through
   canonical keys and exact `Use` spans; reject syntax cycles; expand providers before consumers.
4. Keep imported notation inactive without a canonical notation summary. Reject item-generating
   macro behavior outside the declared target domain.
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

No namespace collection, provisional view, general import binding, body/type checking, final
interface, Core/CPS lowering, Engine transport/admission/execution, filesystem discovery, source
text fallback, or client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2071's contract and TASK-2067's canonical parsed graph.
- **Target produces:** one complete atomic, parser-owned `CanonicalExpandedModuleGraph`,
  non-authorizing. The delivered local-only slice is not yet this complete handoff.
- **Downstream owner:** TASK-2075 alone consumes the completed graph for collection; it remains
  planned and inactive while TASK-2074 is partial.
- **Integration/proof:** TASK-2064 owns composed parity.
- [ ] Positive, negative, mutation, file/inline, property, no-FS, and authority-fence evidence exists.
- [ ] Existing expansion and graph regressions pass.
- [ ] No partial expanded graph can be observed.
