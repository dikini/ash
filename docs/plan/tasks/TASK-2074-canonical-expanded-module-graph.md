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
module collection. The carrier consumes and owns `CanonicalModuleGraph`, performs a syntax-only
macro/notation import prepass, shallowly expands each keyed `ModuleBody`, preserves uses and source
order, and publishes an exact one-to-one expanded module map only when the whole graph succeeds.

## Semantic authority and axes

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

**Missing target-spec clauses:** No `CanonicalExpandedModuleGraph`, parser-native shallow `ModuleBody` expander, AST-only syntax-summary/import prepass, syntax-dependency cycle gate, per-key expansion sidecar carrier, exact key-map validation, normalized file/inline expansion projection, or graph-wide atomic failure implementation/evidence exists. The Engine module loader, filesystem/path lookup, source scanning, and Engine caches are forbidden substitutes.

**Layers:** Type `not_implemented`; Core `not_applicable`; CPS `not_applicable`;
admission-runtime `not_applicable`; verification `not_implemented`.

**Focused verification (expected RED):** `cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph`. The target currently fails because the production carrier/error API does not exist; this is TDD state, not tested semantic evidence.

**Next obligation:** Implement the parser-owned shallow-expansion seam and canonical graph through the linked TDD plan until the focused target passes. TASK-2075 remains planned and consumes only the completed atomic expanded graph.

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
- **Produces:** one atomic, parser-owned `CanonicalExpandedModuleGraph`, non-authorizing.
- **Downstream owner:** TASK-2075 alone consumes it for complete collection.
- **Integration/proof:** TASK-2064 owns composed parity.
- [ ] Positive, negative, mutation, file/inline, property, no-FS, and authority-fence evidence exists.
- [ ] Existing expansion and graph regressions pass.
- [ ] No partial expanded graph can be observed.
