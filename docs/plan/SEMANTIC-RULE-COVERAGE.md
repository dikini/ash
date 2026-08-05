---
id: docs.plan.semantic-rule-coverage
title: Semantic Rule Coverage Map
kind: implementation-coverage-map
status: active
authority: planning-and-review
last_verified: 2026-07-27
---

# Semantic Rule Coverage Map

This is the human-review surface for target semantic-rule coverage. Canonical specs own the full
feature domain; `docs/spec/SEMANTIC-TRACEABILITY.json` owns machine-validated links. This map
reports each rule's implementation, evidence, parity, and missing target-spec clauses.

For semantic work, link a task to one or more rows below and update the row before writing a
fixture. A source example is evidence only. Every row reports these independent axes:

- **Implementation:** `implemented`, `partial`, or `not_implemented`.
- **Evidence:** `proved`, `tested`, or `none`.
- **Parity:** `matches_spec` or `below_spec`.

`implemented` requires realization of the rule's complete target-spec domain. A completed task or
layer handoff does not change an incomplete rule from `partial`/`below_spec`. New behavior outside
the target rule requires a specification update before implementation.

## How to read this map

This is a composition map of target-rule realization, not a whole-language progress scorecard.
Layer values are `implemented`, `partial`, `not_implemented`, or `not_applicable`.
`not_applicable` means the layer is outside the rule's realization path; `non-authorizing` means
the layer transports requirements or metadata without installing runtime/admission authority.

Read every omitted or non-authorizing layer as a named handoff, not a demand for cross-layer work.
For example, TASK-2013 produces checked typed-handler facts; TASK-2014 consumes those facts to
construct and authorize an admission artifact and frame instructions; TASK-2008 projects the
resulting terminal envelope. Handoffs, tests, and proofs provide evidence for their stated scope;
they do not by themselves establish target-spec parity.

## Executable-realization composition

[PLAN-203](PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md) consumes these handoffs to realize one
Surface → Core → CPS → Engine execution path. It is an integration owner, not an additional layer
that changes any family above. New or materially revised PLAN-203 tasks declare whether their
run-route impact is `none`, `prerequisite`, or `active`; `active` routes require a named
CLI/daemon integration case over the same source contract and normalized terminal result.

The traceability graph carries optional Verus assurance work. A deferred proof obligation is a
visible future item, not a missing runtime layer or a release blocker.

## Rule families
### Surface forms and source-to-Core

- **Canonical owner:** `SPEC-095b`, `SPEC-098c`
- **Layer status:** Type partial; Core partial; CPS not_applicable; admission/runtime
  not_applicable; verification partial.
- **Missing target-spec clauses:** expression, call, closure, pattern, and import lowering.

### Complete modules, imports, and visibility

- **Canonical owner:** `SPEC-103`, `MOD-REAL-001` through `MOD-REAL-006`
- **Implementation:** partial
- **Evidence:** tested
- **Parity:** below_spec
- **Layers:** Type partial; Core partial; CPS not_implemented; admission/runtime not_implemented;
  verification partial.
- **Missing target-spec clauses:** export-closed final checked interfaces with complete typed
  namespaces/body facts;
  parsed imports, qualified paths, all visibility forms, aliases/re-exports, atomic import-cycle
  rejection, and binder integration; complete definition-body Core/CPS lowering plus Engine
  scanner/path-cache transport fencing; linked Engine admission; and CLI/daemon terminal parity.
- **Handoffs:** TASK-2057 has delivered tested AST-derived structural declarations; TASK-2058 has
  delivered a tested Core `ModuleKey`/`ModuleArtifact` carrier, but it is not yet consumed by the
  resolver graph or legacy `ModuleIdentity`; TASK-2059 has completed a tested parser module-unit
  handoff with source-acquisition diagnostics, while graph/interface/import/lowering/admission/client
  clauses remain separately owned; TASK-2060 produces the bounded Core carrier; TASK-2066 produces
  a bounded TypeEnv wrapper; TASK-2061 produces a bounded checked-store import environment; and
  TASK-2062 produces bounded Core/CPS artifacts. Completed TASK-2067 supplies a tested partial
  canonical `ModuleKey` graph in `canonical_module_graph.rs` with real units, AST-only edges,
  parsed root metadata, complete `Absent`/`Discovered`/`Parsed`/`Failed` reporting, anchored
  missing/root+nested-duplicate/malformed-inline/cycle diagnostics, canonical-key rewrite
  resistance, complete ordered file/inline payload parity and payload mutation, and an isolated
  deprecated legacy-route fence. It remains below-spec because the target's downstream layers are
  incomplete;
  TASK-2068 is Complete for its tested partial Type-layer foundation: provisional function
  collection, explicit parsed alias binding, graph-wide-preflighted closed primitive M-CHECK
  sibling body checking, and a direct root/provider primitive check with non-authorizing import
  facts. Its
  constructor-free, non-authorizing `CanonicalPublicFunctionInterface` exposes only
  the public primitive projection alongside private checked facts; it is not a final interface,
  import/binder credential, or runtime authority. TASK-2070 completed the bounded M-SELF alias;
  TASK-2071 completed the namespace/provisional-view contract with no implementation evidence;
  completed TASK-2074 supplies expansion, completed TASK-2075 owns the paired two-tier collection handoff, completed TASK-2072
  owns complete parsed imports/binding, and active TASK-2073 owns complete M-CHECK/final interface/export closure. TASK-2069 consumes only
  TASK-2073's complete checked handoff for lowering and Engine scanner/path-cache transport
  fencing. TASK-2063 is active but waits for TASK-2069's complete non-sealed closure; no request
  or admission evidence exists yet. TASK-2064 owns conformance and active-route parity; TASK-2065
  owns closeout.

### Phase 207 ownership repair

TASK-2067 is a complete semantic handoff with `partial / tested / below_spec` axes. Its parser
graph/unit and structural diagnostic clauses are complete, but the full target remains partial and
below-spec until its distinct downstream owners finish their layers. TASK-2068 is Complete for its
`partial / tested / below_spec` bounded Type-layer
`M-COLLECT`, bounded graph-only simple-import `M-IMPORT-EDGE`/`M-IMPORT-CYCLE`, and closed
primitive M-CHECK leaf slice, and a direct primitive provider/client checker. It also has a delivered
`partial / tested / below_spec` direct-public primitive re-export interface-fragment sub-slice
with bounded direct evidence only. The delivered planner retains parser-anchored cross-module edge
provenance, suppresses same-module edges, rejects discovered cycles through
`ImportCycle { edges: CanonicalImportCycle }`, and remains non-authorizing. M-CHECK
graph-preflights every unit, stages/checks sibling function bodies atomically, and returns only
private checked facts plus a constructor-free non-authorizing public primitive projection; it
authorizes no final interface, general import/binder, Core/CPS, Engine, or runtime behavior. The
provider/client checker admits only the root plus plan-selected direct provider leaves;
pre-provider `module_units()` completeness rejects unrelated unselected non-root graph units,
while a selected-provider descendant reaches the anchored provider-leaf shape rejection. It
  returns only non-authorizing checked root/provider/import facts. TASK-2070 is the completed bounded
  M-SELF owner; TASK-2071 is the completed contract owner; TASK-2074 is complete for canonical
  expansion, TASK-2075 is complete for its paired two-tier collection handoff, TASK-2072 is complete for its parsed-import/binding handoff, and
  TASK-2073 for complete M-CHECK/final interface/export closure. TASK-2069
consumes only TASK-2073 and must receive its own active record/coverage/traceability evidence
before its first semantic Rust change.

| Required target clauses | Owner / status | Consumes | Produces | Downstream / integration proof |
|---|---|---|---|---|
| MOD-REAL-001/002 canonical graph state, real module units, expansion, and anchored structural failures | [TASK-2067](tasks/TASK-2067-canonical-module-graph-and-structural-diagnostics.md) — Complete, partial/tested/below-spec; [TASK-2074](tasks/TASK-2074-canonical-expanded-module-graph.md) — Complete non-authorizing parser handoff, broader rule partial/tested/below_spec | TASK-2057 declarations, TASK-2058 identities, TASK-2059 units, TASK-2071 contract | complete parser-stage canonical structural graph plus AST-only syntax prepass/expanded graph | TASK-2075 may consume the completed expansion; TASK-2064 proves composed parity |
| MOD-REAL-003/004 final interfaces, namespaces, parsed imports/visibility, re-exports, cycles, and binder atomicity | TASK-2068 — Complete, partial/tested/below-spec foundation; [TASK-2070](tasks/TASK-2070-scoped-self-simple-function-aliases.md) — Complete partial/tested M-SELF handoff; [TASK-2071](tasks/TASK-2071-module-namespace-and-provisional-view-contract.md) — Complete specification handoff; TASK-2075 — Complete paired collection handoff; TASK-2072 — Complete parsed-import/binding handoff; TASK-2073 — In progress activation for checking/closure | completed TASK-2067 graph, TASK-2074 expanded graph, and bounded TASK-2060/2066/2061 carriers to revalidate | TASK-2075 produces the internal snapshot and name-only view; TASK-2072 consumes only the name view for atomic binding/staged `pub use`; TASK-2073 consumes the internal snapshot plus staging for checked/export-closed interfaces. | TASK-2069 consumes only TASK-2073; TASK-2064 proves composed parity |
| MOD-REAL-005 complete body lowering and Engine scanner/path-cache fence | [TASK-2069](tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md) | TASK-2073 complete checked modules plus TASK-2067 provenance | complete non-sealed Core/CPS closure and canonical transport | TASK-2063 seals/admission; TASK-2064 proves terminal parity |

## TASK-2057: AST-Driven Module Discovery

- **Task:** [TASK-2057](tasks/TASK-2057-ast-driven-module-discovery.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-001`, and traceability rule
  `SEM-MODULE-REALIZATION-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core not_applicable; cps not_applicable;
  admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Source-anchored ModuleNotFound and CircularDependency diagnostics; canonical module identities; source-kind-independent module units and parity; checked interfaces; interface-driven imports and visibility; module-aware Core/CPS lowering; linked Engine admission; and CLI/daemon terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-001-AST-DISCOVERY`; negative
  `TEST-MOD-REAL-001-LOOKALIKE-REJECTION`; mutation
  `TEST-MOD-REAL-001-SCAN-NONAUTHORITY`; parity is not_applicable because this prerequisite
  handoff has no paired execution relation.
- **Non-goals:** Canonical identity, common file/inline module units, source-anchored missing/cycle diagnostics, inline checking, import binding, visibility enforcement, summaries, lowering, admission, runtime execution, and client parity.
- **Next obligation:** TASK-2059 consumes the TASK-2058 carrier for source acquisition, module units, and structural source diagnostics; TASK-2060 interfaces; TASK-2061 imports/visibility; TASK-2062 lowering; TASK-2063 admission; TASK-2064 diagnostic conformance and client parity; TASK-2065 closeout.
- **Handoff:** complete. TASK-2057 supplies public AST-derived structural declarations with
  parser-owned source origins and file/inline structural graph edges. This task evidence does not
  upgrade the aggregate Phase-207 family, which remains not_implemented / none / below_spec until
  its separately owned clauses are realized.

## TASK-2058: Canonical Module Identity and Artifacts

- **Task:** [TASK-2058](tasks/TASK-2058-canonical-module-identity-and-artifacts.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-001`, and traceability rule
  `SEM-MODULE-REALIZATION-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core partial; cps not_applicable;
  admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Resolver graph construction does not yet consume ModuleKey/ModuleArtifact; legacy semantic_summary::ModuleIdentity remains unchanged; source-kind-independent module units and file/inline parity; identity-preserving aliases/re-exports, checked export-closed interfaces, interface-driven imports and visibility; module-aware Core/CPS lowering; linked Engine admission; source-anchored ModuleNotFound and CircularDependency diagnostics; and CLI/daemon terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-001-CANONICAL-KEY` and
  `TEST-MOD-REAL-001-KEY-GRAMMAR-PARITY`; negative `TEST-MOD-REAL-001-DUPLICATE-ORIGIN`; mutation
  `TEST-MOD-REAL-001-CACHE-KEY-FORGERY`; parity is not_applicable because this non-authorizing
  Core carrier has no paired execution relation.
- **Non-goals:** Migrating resolver graph construction or existing semantic_summary::ModuleIdentity; source-kind parity or source-anchored structural diagnostics; checked interfaces, import binding, visibility enforcement, Core/CPS lowering, Engine admission, persistent disk cache, runtime module values, or client parity.
- **Next obligation:** TASK-2059 must consume ModuleKey/ModuleArtifact for source-kind-independent module units and structural diagnostics; TASK-2060/2061 consume stable identities for interfaces/imports; TASK-2062/2063 consume checked artifacts for lowering/admission; TASK-2064 owns conformance/parity; TASK-2065 closes the phase.
- **Handoff:** complete. TASK-2058 publishes the tested Core `ModuleKey`,
  `ModuleArtifactOrigin`, schema-versioned `ModuleArtifact`, and deterministic child-key contract.
  No existing resolver graph construction or legacy `ModuleIdentity` consumer has migrated; this
  non-authorizing carrier does not establish source parity, interfaces, imports, lowering,
  admission, execution, or terminal parity.

## TASK-2059: File/Inline Module-Unit Parity

- **Task:** [TASK-2059](tasks/TASK-2059-file-inline-module-unit-parity.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-002`, and traceability rule
  `SEM-MODULE-REALIZATION-002`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core not_applicable; cps not_applicable;
  admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Structural-cycle/CircularDependency rejection and malformed-inline parent-anchor/error-atomicity remain outside this one-unit source-acquisition route; resolver ModuleGraph and legacy semantic_summary::ModuleIdentity migration/persistence; checked export-closed interfaces; interface-driven import binding and visibility; module-aware Core/CPS lowering; linked Engine admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-002-FILE-INLINE-UNIT` and
  `TEST-MOD-REAL-002-INLINE-NESTED-MOD-PARSE`; negative
  `TEST-MOD-REAL-002-SOURCE-DIAGNOSTICS` and `TEST-MOD-REAL-002-DUPLICATE-CHILD`; mutation
  `TEST-MOD-REAL-002-SOURCE-KIND-ERASURE`; source-unit parity is covered by
  `TEST-MOD-REAL-002-FILE-INLINE-PARITY`, not an Engine/client parity claim.
- **Non-goals:** Structural-cycle graph traversal and malformed-inline parse diagnostics beyond this parser unit handoff; resolver ModuleGraph and legacy semantic_summary::ModuleIdentity migration/persistence; checked interface/export closure, import binding, visibility enforcement, Core/CPS lowering, Engine admission/execution or a direct-evaluator fallback, dynamic loading, import-cycle initialization, runtime module values, or client parity.
- **Next obligation:** TASK-2060 must consume the completed TASK-2059 units for checked export-closed interfaces; TASK-2061 owns interface-driven import binding and visibility; TASK-2062 module-aware Core/CPS lowering; TASK-2063 Engine-only linked admission with no direct-evaluator fallback; TASK-2064 owns source-diagnostic conformance, structural-cycle coverage, and CLI/daemon parity; TASK-2065 closes the phase.
- **Handoff:** complete. TASK-2059 consumes the completed TASK-2057 AST declarations and TASK-2058
  Core carrier into a parser-owned ordered `ModuleBody`/`ModuleUnit` acquisition route. It proves
  no checked interface, import binding, Core/CPS artifact, Engine admission, or client route.

## TASK-2060: Checked Module Interface and Export Closure

- **Task:** [TASK-2060](tasks/TASK-2060-checked-module-interface-and-export-closure.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-003`, and traceability rule
  `SEM-MODULE-REALIZATION-003`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable;
  verification partial.
**Missing target-spec clauses:** Complete TypeEnv-private/interface collection beyond TASK-2066's staged public function/handler declaration-signature preflight; body/full-callable facts, typed summary-binding linkage, aliases/re-exports, per-binding source origins, and closure finalization; Engine export-scanner retirement or disagreement-only fencing and interface transport; interface-driven import binding and visibility; module-aware Core/CPS lowering; Engine-only linked admission/execution with no direct-evaluator fallback; structural/import-cycle conformance; and CLI/daemon normalized-terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-003-EXPORT-CLOSURE`; negative
  `TEST-MOD-REAL-003-PRIVATE-LEAK`; mutation `TEST-MOD-REAL-003-INTERFACE-SCHEMA`; no proof.
  `TEST-MOD-REAL-003-ENGINE-EXPORT-SCAN` remains deferred. **Parity:** not applicable because the
  Core carrier has no paired source, Engine, or client execution relation.
- **Non-goals:** AST-derived ModuleUnit collection, private typechecking state, typed-summary identity linkage, import/visibility binding, Core/CPS lowering, Engine scanner fencing/transport, Engine admission/execution or a direct-evaluator fallback, dynamic imports, package resolution, runtime module values, structural/import-cycle conformance, or client parity.
- **Next obligation:** TASK-2066 now supplies a bounded TypeEnv wrapper after staged declaration-signature preflight and full artifact equality, and TASK-2061 consumes only that wrapper in a bounded checked store. Neither supplies complete private facts, typed summaries, closure, parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, or binder integration. TASK-2062 then owns lowering, TASK-2063 consumes only linked artifacts, TASK-2064 owns conformance/parity, and TASK-2065 closes Phase 207.
- **Handoff:** complete. TASK-2060 supplies a tested, non-authorizing Core V1
  `PublicModuleInterface` schema over canonical artifacts, public binding identity/visibility/origin
  facts, dependency versions, strict serde, and V1--V8 summary compatibility. It rejects private
  or duplicate bindings, forged/missing children, invalid inline parents, generic typed identities,
  malformed cache data, and unknown schema fields. It publishes no parser-derived final interface,
  private TypeEnv view, binding fact, Core/CPS artifact, Engine transport/admission authority, or
  runtime/client route.

## TASK-2061: Interface Import Resolution and Visibility

- **Task:** [TASK-2061](tasks/TASK-2061-interface-import-resolution-and-visibility.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-004`, and traceability rule
  `SEM-MODULE-REALIZATION-004`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core not_applicable; cps not_applicable; admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Parsed use integration and all parsed visibility forms with inaccessible diagnostics; parsed aliases, pub use, and re-exports; full typed namespaces and binder integration; import-cycle detection; complete final-interface/export-closure validation; module-aware Core/CPS lowering; Engine scanner fencing/transport and linked admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
- **Evidence detail:** positive/property `TEST-MOD-REAL-004-EXPLICIT-CHILD-IDENTITY`; negative
  `TEST-MOD-REAL-004-MISSING-CHECKED-CHILD`; mutation
  `TEST-MOD-REAL-004-GROUP-ATOMICITY`; no proof. The focused
  `task_2061_interface_import_resolution` target passes 11/11. Parity is not_applicable because
  this bounded checked-interface resolver has no paired source, Engine, or client execution relation.
- **Non-goals:** Treating raw PublicModuleInterface, parser resolver state, legacy ModuleGraph, Engine state, filesystem paths, or text scans as import authority; parsed use/visibility/re-export/alias support, full typed namespaces or binder integration, import cycles, complete final-interface/export closure, Core/CPS lowering, Engine transport/admission/execution or scanner fencing, dynamic imports, runtime module values, or client parity.
- **Next obligation:** The parser and binder integration owners must consume only this resolver's FinalizedModuleInterface-backed checked store. They must add parsed visibility/inaccessible diagnostics, aliases/re-exports, typed namespaces, cycle rejection, complete closure, lowering, Engine transport, and parity without reintroducing raw Core, parser, Engine, filesystem, legacy graph, or text-scan authority.
- **Handoff:** complete. TASK-2061 stores only finalizer-issued wrappers, traverses canonical public
  child identities, resolves bounded explicit/group/glob requests, stages groups atomically,
  gives explicit bindings precedence over globs, keeps distinct glob identities ambiguous, and
  preserves defining identity/syntax-only macro metadata. It is not a parsed import resolver,
  full interface authority, binder integration, Engine authority, or runtime authority.

## TASK-2066: TypeEnv Module-Unit Interface Finalization

- **Task:** [TASK-2066](tasks/TASK-2066-typeenv-module-unit-interface-finalization.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-003`, and traceability rule
  `SEM-MODULE-REALIZATION-003`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core not_applicable; cps not_applicable; admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Checking callable bodies or producing complete callable facts; typed namespace linkage for types, constructors, interfaces, effect rows, and implementations; aliases/re-exports and per-binding source-origin projection; complete export closure and diagnostics; interface-driven imports, visibility, and cycles; module-aware Core/CPS lowering; Engine scanner fencing/transport and linked admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-COLLECTION`; negative
  `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-KEY-CONTEXT`; mutation
  `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-DECLARATION-PREFLIGHT`; no proof. The focused
  `task_2066_module_interface_finalization` target passes 11/11. Parity is not_applicable because
  this bounded TypeEnv handoff has no paired source, Engine, or client execution relation.
- **Non-goals:** Treating the wrapper or raw ash_core::PublicModuleInterface as an authoritative full interface or import authority; checking bodies or full callable facts; typed namespace linkage, aliases/re-exports, per-binding source-origin projection, complete export closure, imports/visibility/cycles, Core/CPS lowering, Engine transport/admission/execution or scanner fencing, dynamic imports, runtime module values, import-cycle initialization, or client parity.
- **Next obligation:** TASK-2061 now consumes this bounded FinalizedModuleInterface wrapper through a wrapper-only checked store, but its resolver is not parsed import/visibility or full interface authority. Parser/binder integration must add typed linkage, re-export/alias handling, closure, cycles, lowering, Engine transport, and parity without admitting raw Core, parser, Engine, filesystem, legacy graph, or text-scan authority.
- **Handoff:** complete. TASK-2066 stages `TypeEnv::register_surface_declarations` for public
  function/handler declaration signatures under one canonical module key, validates the bounded
  parser/TypeEnv projection, requires full artifact equality, and issues a non-forgeable wrapper.
  It is not a full authoritative interface, import binder, Engine authority, or runtime authority.

## TASK-2062: Module-Aware Core/CPS Lowering

- **Task:** [TASK-2062](tasks/TASK-2062-module-aware-core-cps-lowering.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-005`, and traceability rule
  `SEM-MODULE-REALIZATION-005`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core partial; cps partial; admission-runtime not_applicable;
  verification partial.
**Missing target-spec clauses:** Parser/source lowering from a complete ModuleUnit and all reachable definition bodies; full typed imports, callable/type authority, parsed visibility/aliases/re-exports, and import-cycle handling; full type namespace and export-closure validation; file/inline real-program artifact parity; checked dependency-closure linking and Engine-only admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
- **Evidence detail:** positive `TEST-MOD-REAL-005-CORE-CPS-MODULE`; negative
  `TEST-MOD-REAL-005-UNRESOLVED-IMPORT-LOWER`; mutation
  `TEST-MOD-REAL-005-IDENTITY-FORGERY`; no proof. The focused
  `task_2062_module_core_cps_lowering` target passes 3/3. Parity is not_applicable because the
  bounded envelope has no real-program source, Engine, or client execution relation.
- **Non-goals:** Parser/source rediscovery, raw PublicModuleInterface or legacy ModuleGraph authority, parser/source lowering or full definition bodies, typed imports or callable authority, parsed import/visibility/alias/re-export/cycle semantics, complete namespaces or export closure, file/inline real-program parity, Engine linking/admission/execution, direct-evaluator fallback, filesystem or text scans, runtime module values, or CLI/daemon behavior.
- **Next obligation:** The parser/binder integration owner must supply complete checked definition bodies and typed import facts to a later lowering slice. TASK-2063 must first seal its own dependency-linking/admission input around these non-authoritative public Core/CPS carriers; it cannot treat either carrier as authority. TASK-2064 alone owns real-program file/inline and CLI/daemon parity.
- **Handoff:** complete. `lower_finalized_module_to_core_cps` resolves expected TASK-2061 imports
  before Core validation, then delegates only to the checked Core-to-CPS bridge. Its paired
  non-executable artifacts retain exact finalizer module key/origin and deterministic imported
  defining-identity/origin snapshots; they do not rediscover source, issue a sealed admission
  input, link/admit artifacts, or execute CPS.

## TASK-2063: Engine Linked-Module Admission

- **Task:** [TASK-2063](tasks/TASK-2063-engine-linked-module-admission.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-006`, and traceability rule
  `SEM-MODULE-REALIZATION-006`.
**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
- **Layers:** type not_applicable; core partial; cps partial; admission-runtime not_implemented;
  verification not_implemented.
**Missing target-spec clauses:** An Engine-sealed linked/admission request over the complete reachable checked Core/CPS dependency closure; canonical dependency identity/version/origin validation; rejection of missing, incomplete, stale, forged, or failed entries before execution; Engine-only consumption with no raw/source/direct-evaluator or alternate module path; and one admitted real program for TASK-2064 file/inline and CLI/daemon normalized-terminal parity.
- **Evidence detail:** none. Parity is not_applicable because no Engine-sealed linked/admission
  request or admitted real program exists yet.
- **Non-goals:** Treating TASK-2062 public Core/CPS carriers as sealed authority; raw/source or legacy ModuleGraph/module-loader import authority; parser/source rediscovery, text scans, or filesystem walking; direct-evaluator or alternate execution paths; provider/handler frame authority; dynamic imports, package/cache persistence, runtime module values, or CLI/daemon parity.
- **Next obligation:** Implement one Engine-sealed linked/admission request that consumes only a complete checked Core/CPS dependency closure, rejects missing, incomplete, stale, forged, or failed entries before execution, and supplies that admitted request to TASK-2064; no unavailable link/admission stage may select raw-source, loader, or direct-evaluator authority.
- **Handoff:** active but not implemented. TASK-2063 consumes only TASK-2062's public,
  non-authoritative Core/CPS carriers and must mint its own Engine-sealed linked/admission request.
  It may not select raw source, parser/legacy graph, loader-private exports, or a direct evaluator
  as module or execution authority.

## TASK-2067: Canonical Module Graph and Structural Diagnostics

- **Task:** [TASK-2067](tasks/TASK-2067-canonical-module-graph-and-structural-diagnostics.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-001`, `MOD-REAL-002`, and traceability rules
  `SEM-MODULE-REALIZATION-001` and `SEM-MODULE-REALIZATION-002`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable;
  verification partial.
**Missing target-spec clauses:** Complete checked interfaces, parsed imports, visibility, re-exports, import-cycle rejection, and binder integration remain owned by TASK-2068; complete definition-body lowering and the Engine scanner/cache transport fence remain owned by TASK-2069; Engine-linked admission remains owned by TASK-2063; real-program file/inline and CLI/daemon terminal parity remain owned by TASK-2064; and TASK-2065 owns phase closeout.
- **Historical task-record mirror:** The preceding exact clause is retained for the closed
  TASK-2067 record/task mirror. The current ownership is TASK-2068 Complete foundation;
  TASK-2070 completed bounded M-SELF; TASK-2071 completed the specification contract; TASK-2074
  is active for expansion, while TASK-2075 and TASK-2072 have completed their paired-collection and parsed-binding handoffs;
  TASK-2073 plans complete M-CHECK/final interface/export closure; and
  TASK-2069 consumes only TASK-2073.
- **Evidence detail:** positive `TEST-MOD-REAL-001-CANONICAL-GRAPH` and
  `TEST-MOD-REAL-002-REAL-UNIT-TRANSPORT`; negative
  `TEST-MOD-REAL-001-STRUCTURAL-DIAGNOSTICS`; mutation
  `TEST-MOD-REAL-001-INLINE-SOURCE-REENTRANCY-GUARD`,
  `TEST-MOD-REAL-001-GRAPH-KEY-REWRITE`, and
  `TEST-MOD-REAL-002-GRAPH-UNIT-PAYLOAD-MUTATION`; source-unit parity
  `TEST-MOD-REAL-002-GRAPH-UNIT-PARITY`; and architectural fence
  `TEST-MOD-REAL-001-LEGACY-ROUTE-FENCE`. The focused parser targets passed:
  `task_2067_canonical_module_graph.rs` (12),
  `task_2067_canonical_identity_fence.rs` (3), and
  `task_2067_legacy_route_fence.rs` (2), followed by the full parser suite, formatting, and
  strict clippy. The source-layout legacy fence is an architecture check, not a semantic mutation
  proof.
- **Non-goals:** Import-edge resolution, typechecking, complete public-interface closure, aliases/re-exports, typed namespace linkage, Core/CPS lowering, Engine scanner transport, admission/execution, import-cycle initialization, runtime module values, or CLI/daemon parity.
- **Historical task-record mirror:** TASK-2068 consumes the completed, non-authorizing graph/unit handoff for complete checked interfaces and parsed import/binder semantics; TASK-2069, TASK-2063, TASK-2064, and TASK-2065 retain their separately owned downstream boundaries.
- **Current successor handoff:** TASK-2068 remains only the completed foundation consumer.
  TASK-2074 consumes graph/unit facts for expansion, TASK-2075 consumes that graph for two-tier
  collection, TASK-2072 consumes only its name view for binding, and TASK-2073 consumes the internal
  snapshot plus staging for finalization; TASK-2069 then consumes only TASK-2073's complete checked
  handoff. TASK-2064 retains composed parity.
- **Handoff:** complete. TASK-2067 consumes TASK-2057 parsed declarations/spans, TASK-2058
  canonical identity/artifact facts, and TASK-2059 acquired ordered module units. It produces a
  parser-only, non-authorizing graph handoff for TASK-2068/TASK-2070's completed slices,
  TASK-2071's completed contract, completed TASK-2074 parser handoff, completed TASK-2075 paired collection handoff, and completed TASK-2072 parsed-binding handoff, with active TASK-2073;
  TASK-2064 separately owns composed file/inline and
  client parity.

## TASK-2068: Final Interfaces, Parsed Imports, and Binder Integration

- **Task:** [TASK-2068](tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-003`, `MOD-REAL-004`, and traceability rules
  `SEM-MODULE-REALIZATION-003` and `SEM-MODULE-REALIZATION-004`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
- **Layers:** type partial; core/cps/admission-runtime not_applicable; verification partial.
**Missing target-spec clauses:** Complete M-COLLECT across every required namespace and callable/body fact; M-CHECK for all remaining forms; complete M-IMPORT-EDGE, M-IMPORT-CYCLE, and M-BIND semantics for every remaining parsed use/path/visibility/alias/re-export form, export-closed final interfaces, and atomic dependency closures; complete definition-body Core/CPS lowering and Engine scanner/cache transport fencing; Engine-linked admission; and real-program file/inline plus CLI/daemon normalized-terminal parity remain deferred.
**Current successor ownership:** TASK-2068 is complete for its delivered foundation and TASK-2071
is complete for its specification contract. TASK-2074 owns expansion, TASK-2075 owns complete
two-tier M-COLLECT, TASK-2072 owns all remaining parsed imports,
visibility, edges, cycles, precedence, atomic binding, and staged `pub use`, and TASK-2073 owns
complete M-CHECK bodies, final interfaces, and export closure. TASK-2069 consumes only the
complete TASK-2073 handoff.
- **Historical delivery detail:** The delivered-slice descriptions and pre-closure obligations below
  preserve TASK-2068's evidence attribution only; they do not assign any current unfinished clause
  to TASK-2068.
- **Evidence detail:** positive `TEST-MOD-REAL-003-PROVISIONAL-FUNCTION-COLLECTION`,
  `TEST-MOD-REAL-004-PARSED-IMPORT-BINDING`, and
  `TEST-MOD-REAL-004-ALIAS-IDENTITY-PROPERTY`,
  `TEST-MOD-REAL-004-PLANNER-EDGE-PROVENANCE`, and
  `TEST-MOD-REAL-004-PLANNER-SAME-MODULE-NO-EDGE`; negative
  `TEST-MOD-REAL-004-VISIBILITY-DIAGNOSTIC` and
  `TEST-MOD-REAL-004-RESTRICTED-VISIBILITY-REJECTION`, and
  `TEST-MOD-REAL-004-CANONICAL-BINDER-FENCE`,
  `TEST-MOD-REAL-004-PLANNER-UNSUPPORTED-SHAPE`,
  `TEST-MOD-REAL-004-PLANNER-ORDERED-CYCLE-DIAGNOSTIC`, and
  `TEST-MOD-REAL-004-PLANNER-BINDER-DELEGATION-FENCE`; full-provenance tail-cycle diagnostic
  `TEST-MOD-REAL-004-PLANNER-TAIL-CYCLE-PROVENANCE`; atomicity control
  `TEST-MOD-REAL-004-PRIVATE-ALIAS-ATOMICITY` and
  `TEST-MOD-REAL-004-PLANNER-CYCLE-ATOMICITY`; no proof. The focused
  `TEST-MOD-REAL-004-PUB-USE-REJECTION` is positive fail-closed boundary evidence: public use,
  re-exports, and all non-inherited use visibilities reject before publication and are not
  implemented. Restricted target declaration visibilities (`pub(crate)`, `pub(super)`,
  `pub(self)`, and `pub(in …)`) likewise reject as anchored `Unsupported`; they are boundaries,
  not visibility implementation. The late-private-import control rejects the entire module without
  publishing an earlier alias. The delivered planner test evidence retains the full edge
  provenance and binder delegation, confirms no same-module edge, verifies the ordered
  parser-anchored `CanonicalImportCycle` edges for a file/inline two-node cycle, verifies that an
  `a → b → c → b` tail reports only the full-provenance `b ↔ c` cycle, and rejects a late back-edge
  atomically. Its delegation fence also excludes `RawCoreProgram`, `CoreExpr`, and `CpsProgram`.
  The focused `task_2068_parsed_import_binder` target passes 11/11, including 16 generated aliases.
  Parity is
  not_applicable because this bounded Type-layer collector/binder has no final interface, lowered
  artifact, admitted program, or paired source-to-terminal relation. The remaining
  `REQ-TASK-2073-FINAL-INTERFACE-BACKLOG-DEFERRED`,
  `REQ-TASK-2073-EXPORT-CLOSURE-REJECTION-BACKLOG-DEFERRED`,
  `REQ-TASK-2073-REEXPORT-IDENTITY-BACKLOG-DEFERRED`,
  `REQ-TASK-2072-BINDER-ATOMICITY-BACKLOG-DEFERRED`, and
  `REQ-TASK-2073-TYPE-LAYER-INTERFACE-PARITY-BACKLOG-DEFERRED` backlog dispositions remain
  deferred. Delivered provider/client
  evidence is positive `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-POSITIVE` and property
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROPERTY`; negative
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LEAF-REJECTION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-CLIENT-MISMATCH-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ARTIFACT-SNAPSHOT-MISMATCH`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LOCAL-IMPORT-COLLISION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-IMPORT-REJECTION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-DEEP-TOPOLOGY-REJECTION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-COMPLETENESS`, and
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-PREFLIGHT-ORDERING`; mutation
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ATOMICITY`; and architectural fence
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-AUTHORITY-FENCE`. The focused
  `task_2068_primitive_provider_client` target passes 12/12, including a 16-case property. It
  proves exact root/provider artifact and import provenance, primitive provider admission,
  anchored client mismatch, same-key artifact mismatch, collision and provider topology rejection,
  graph-wide unselected-unit topology completeness before malformed selected-provider checking,
  selected-provider descendant provider-leaf rejection, late-root-body atomicity, and the
  no-legacy/no-final-interface/no-Core/CPS/Engine fence. It is test evidence, not a proof or full
  provider/client/interface/import parity claim. Delivered M-CHECK
  evidence is positive `TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC` and
  `TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC-PROPERTY`; negative
  `TEST-MOD-REAL-003-LEAF-MCHECK-BODY-MISMATCH-DIAGNOSTIC`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-OPTION-CLOSED-INTERFACE-REJECTION`, and
  `TEST-MOD-REAL-003-LEAF-MCHECK-UNSUPPORTED-SHAPE`, and
  `TEST-MOD-REAL-003-LEAF-MCHECK-NESTED-PREFLIGHT-REJECTION`; mutation
  `TEST-MOD-REAL-003-LEAF-MCHECK-SIBLING-ATOMICITY`; and architectural fence
  `TEST-MOD-REAL-003-LEAF-MCHECK-INTERFACE-FENCE`. The focused
  `task_2068_canonical_function_interface` target passes 8/8, including 16 generated public
  integer functions. It proves selected graph-unit sibling body checking, fresh
  identity/provenance/private-public projection, anchored mismatch, public `Option` closed-boundary
  rejection, late-failure atomicity, parsed-`use` shape rejection, nested-child global-preflight
  rejection, and the no-legacy/no-runtime architecture fence. It is test evidence, not a proof or
  full-module parity claim.
- **Delivered direct-public fragment:** implementation `partial`; evidence `tested`; parity
  `below_spec`; layers Type `partial`, Core/CPS/admission-runtime `not_applicable`, verification
  `partial`. Its task-owned evidence is positive
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-POSITIVE`; negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-NONPUBLIC-PATH`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-PRIVATE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-NONPRIMITIVE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-IMPLICIT-NAME-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-COLLISION`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ARTIFACT-SNAPSHOT-MISMATCH`;
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-EMPTY-ROOT-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ROOT-SHAPE-REJECTION`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-CHILD-ALIAS-COLLISION`;
  property `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-PROPERTY`; mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ATOMICITY`; and fence
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-AUTHORITY-FENCE`. The focused
  `task_2068_direct_primitive_reexport_interface_fragments` target passes 13/13, including a
  16-case property. These tests are evidence, not proof or full-interface/import/parity evidence.
- **Record-mirrored delivered-fragment target clause:** The delivered direct-public primitive re-export interface fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: `resolve_direct_primitive_interface_imports` requires a nonempty exact root `pub use crate::<direct-provider>::<primitive-function> as <alias>` plan and rejects a public re-export lacking `as <alias>` as anchored `Unsupported` with `an explicit re-export alias is required` before plan publication, while `check_direct_primitive_interface_fragments` consumes only the canonical root plus plan-selected direct primitive providers, exact artifact facts, and bounded provider/client facts; it admits only root `pub mod api` and explicit root re-exports, preserves defining identity/origin/checked primitive signature/declaration/use spans, forbids implicit flattening, rejects non-public paths, private/non-primitive targets, empty plans, root-shape/collision conditions, and mismatched artifacts before atomically returning only a non-authorizing fragment. The direct target passes 13/13, including a 16-case property; these test witnesses are evidence, not proof. Full M-COLLECT/M-CHECK/M-IMPORT-EDGE/M-IMPORT-CYCLE/M-BIND, final interfaces/export closure, lowering/admission/runtime/client parity remain deferred.
- **Explicit-alias rejection evidence:** `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-IMPLICIT-NAME-REJECTION` is tested: a public re-export lacking `as <alias>` rejects as anchored `Unsupported` with `an explicit re-export alias is required` before plan publication. Full `pub use` support remains deferred under TASK-2068.
- **Delivered private-provider-helper fragment:** implementation `partial`; evidence `tested`;
  parity `below_spec`; layers Type `partial`, Core/CPS/admission-runtime `not_applicable`,
  verification `partial`. Its focused `task_2068_private_primitive_provider_helpers` target
  passes 7/7, including a 16-case property. Positive
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-POSITIVE`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-PROPERTY`; negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-PRIVATE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-NONPRIMITIVE`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-ATOMICITY` are test evidence, not
  proof or end-to-end parity evidence. The delivered private primitive provider-helper fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: it retains only exact root `pub mod <provider>` plus `pub use crate::<provider>::<public-primitive> as <alias>`, admits inherited/private ordinary primitive provider helpers only as checked implementation detail, excludes them from `CanonicalPrimitiveInterfaceFragments`, and rejects a private selected target before publication. It consumes canonical graph, exact planned aliases, and bounded provider checker facts; it atomically produces only the same non-authorizing fragment after every provider/helper check succeeds. The focused `task_2068_private_primitive_provider_helpers` target passes 7/7, including a 16-case property; these test witnesses are evidence, not proof. Provider uses, nested modules, other definitions, generics, contracts, restricted visibility, non-primitive/open signatures, all other paths, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered direct-public local-binding root-client fragment:** implementation `partial`;
  evidence `tested`; parity `below_spec`; layers Type `partial`, Core/CPS/admission-runtime
  `not_applicable`, verification `partial`. Its focused
  `task_2068_direct_primitive_reexport_root_client` target passes 10/10, including a 16-case
  property. Positive `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-POSITIVE`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PROPERTY`; negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PUBLIC-ROOT-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-BODY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-ARTIFACT-SNAPSHOT`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PLAN-KIND-FENCE`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-ATOMICITY` are test evidence, not
  proof or end-to-end parity evidence. The delivered direct-public primitive re-export local-binding root-client fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: it admits only a root `pub mod <provider>` with inherited/private ordinary primitive helpers and one public primitive target, exact root `pub use crate::<provider>::<public-primitive> as <alias>`, and inherited/private root `fn internal_entry(..) -> <primitive> { welcome(..) }`. It consumes canonical graph and exact artifact snapshots through a distinct opaque direct plan, selected-provider facts, and a checked local alias; after every provider, alias, root-body, snapshot, and authority check succeeds, it atomically produces only a non-authorizing fragment plus checked private root functions, selected provider facts, and local alias binding while preserving the target's definition identity and visibility before registration. Root-body diagnostic anchoring recognizes only a direct unqualified `<alias>(...)` call (including an empty block tail); all other root-body failures use the enclosing root-body span. The focused `task_2068_direct_primitive_reexport_root_client` target passes 10/10, including a 16-case property; these test witnesses are evidence, not proof. The generic planner/binder and generic provider/client route continue to reject `pub use` from source; all root public functions, generic binders, remaining forms, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered canonical provisional module-scope and structural-path visibility fragment:**
  implementation `partial`; evidence `tested`; parity `below_spec`; layers Type `partial`,
  Core/CPS/admission-runtime `not_applicable`, verification `partial`. Its one implementation and
  nine structural-path visibility, inaccessible diagnostic, visibility-region, local-collision,
  file/inline, atomicity, authority-fence, declaration-snapshot-mismatch, and
  public-path-visibility-fence witnesses pass in the focused target (9/9). Positive
  `TEST-MOD-REAL-004-CANONICAL-STRUCTURAL-PATH-VISIBILITY`,
  `TEST-MOD-REAL-004-CANONICAL-VISIBILITY-REGIONS`, and
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-FILE-INLINE-PARITY`; negative
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-INACCESSIBLE-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-BINDER-LOCAL-DECLARATION-COLLISION`,
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-AUTHORITY-FENCE`,
  `TEST-MOD-REAL-004-CANONICAL-SCOPE-DECLARATION-SNAPSHOT-MISMATCH`, and
  `TEST-MOD-REAL-004-CANONICAL-PUBLIC-PATH-VISIBILITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-ATOMICITY` are test evidence. The delivered canonical provisional module-scope and structural-path visibility fragment is `partial / tested / below_spec`: it builds immutable typeck-owned per-module provisional scopes of direct structural children plus ordinary function declaration entries from TASK-2067 canonical graph units/artifacts. Before resolution, `matches_graph` compares root/artifact facts and requires equality with a fresh declaration-snapshot rebuild from the current parser units, so artifacts alone never authorize a scope entry and same-path/topology removal of a function or a `pub`-to-private change rejects `ScopeGraphMismatch` before binding. It resolves only inherited simple `use crate::<structural-child>...::<ordinary-function> as <name>` through actual structural edges, preserving `ModuleKey` identity, declaration/use spans, origin, and visibility; the final ordinary function may use public, `pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, or `pub(self)` when the importing `ModuleKey` lies in the canonical visibility region. Every traversed structural child and final function must pass visibility before temporary alias staging, and a local function collision rejects. `is_visible_from` is a declaration-level visibility query, so its `pub` result alone never authorizes a path; the resolver separately preflights every structural child, retains the first non-public edge, and rejects a public function behind it. Visibility is evaluated from `ModuleKey` crate identity and segments, never a string helper: private and `pub(self)` admit only the defining module, `pub(crate)` the same crate, `pub(super)` the structural-parent subtree, and `pub(in path)` the resolved named-path subtree. The focused target passes 9/9; these tests are evidence, not proof or end-to-end parity, and route-level binding witnesses over the admitted visibility regions remain deferred to the dedicated scoped binder. `pub use`, groups, globs, non-`crate` paths, non-function targets, other namespaces, remaining definition/body checks, all re-exports, final interfaces/export closure, compatibility binders, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered scoped structural import-cycle gate:** implementation `partial`; evidence `tested`;
  parity `below_spec`; layers Type `partial`, Core/CPS/admission-runtime `not_applicable`,
  verification `partial`; run-route impact `prerequisite`. Its one scoped-cycle implementation and
  eight cycle witnesses pass in the focused scope17 target, including a 16-case property. Positive
  `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-CYCLE-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-TAIL-CYCLE-PROVENANCE`,
  `TEST-MOD-REAL-004-SCOPED-SAME-MODULE-NO-EDGE`,
  `TEST-MOD-REAL-004-SCOPED-CYCLE-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-CYCLE-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-CYCLE-VISIBILITY-PRECEDENCE` and
  `TEST-MOD-REAL-004-SCOPED-CYCLE-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-CYCLE-ATOMICITY` are test evidence. The delivered scoped structural import-cycle gate is `partial / tested / below_spec`: it consumes only the delivered canonical provisional scopes and the scope-backed inherited explicit-alias `use crate::<structural-child>...::<ordinary-function> as <name>` route's staged resolved edges. After every existing scope-snapshot, route-shape, structural-path, visibility, target, and local-collision preflight succeeds, it deterministically detects cycles over cross-module `CanonicalSimpleImportEdge` values before constructing a result; same defining/importer module aliases emit no edge. A cycle returns the outer structural `ImportCycle { edges: CanonicalImportCycle }` with ordered closing-cycle provenance, while all existing structural diagnostics retain precedence, including a visibility failure that could otherwise close a cycle. The operation is atomic and non-authorizing: no cycle-free plan, binding set, or edge result is published on error; the generic planner and compatibility binder remain unchanged because they own different grammar. The focused target passes scope17, including a 16-case property; these tests are evidence, not proof or end-to-end parity. Final interfaces/export closure, other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered dedicated scope-backed structural binder M-BIND slice:** implementation `partial`;
  evidence `tested`; parity `below_spec`; layers Type `partial`, Core/CPS/admission-runtime
  `not_applicable`, verification `partial`; run-route impact `prerequisite`. Its one dedicated
  binder implementation and eight focused witnesses pass in `task_2068_scoped_structural_binder`
  (8/8, including a 16-case property across six visibility categories). Positive
  `TEST-MOD-REAL-004-SCOPED-BINDER-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-DELEGATION`, and
  `TEST-MOD-REAL-004-SCOPED-BINDER-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-BINDER-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-BINDER-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-RESTRICTED-VISIBILITY`, and
  `TEST-MOD-REAL-004-SCOPED-BINDER-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-BINDER-CYCLE-ATOMICITY` are test evidence, not proof or end-to-end
  parity evidence. The delivered dedicated scope-backed structural binder M-BIND slice is `partial / tested / below_spec`: `crates/ash-typeck/src/canonical_structural_module_binder.rs` defines `bind_scoped_structural_parsed_uses(graph, scopes)`, and only `crates/ash-typeck/src/lib.rs` exports that dedicated API. It consumes only the delivered canonical provisional scopes and delegates directly to `resolve_simple_parsed_imports_with_scopes(graph, scopes)` followed by `into_bound_set`. The existing `crates/ash-typeck/src/canonical_module_binder.rs` remains unchanged and generic-only: it must not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. It admits only the delivered inherited explicit-alias `use crate::<structural-child>...::<ordinary-function> as <name>` route; its ordinary-function target may be public, crate, super, `pub(in path)`, inherited/private, or self only when the canonical `ModuleKey` visibility predicate permits the importer, with the existing whole structural-path fence for public targets. It preserves every resolver structural diagnostic and outer `CanonicalImportCycle` provenance unchanged and atomically returns no `CanonicalBoundModuleSet` on error. The focused `task_2068_scoped_structural_binder` target passes 8/8, including a 16-case property across public, crate, super, `pub(in path)`, inherited/private, and self visibility categories; these tests are evidence, not proof or end-to-end parity. The generic `bind_simple_parsed_uses` binder and generic planner remain unchanged because they own different grammar. Final interfaces/export closure, all other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered scoped simple ordinary-function imports M-SIMPLE slice:** implementation `partial`;
  evidence `tested`; parity `below_spec`; layers Type `partial`, Core/CPS/admission-runtime
  `not_applicable`, verification `partial`; run-route impact `prerequisite`. Its one scoped-simple
  implementation and eleven focused witnesses pass in
  `task_2068_scoped_simple_ordinary_function_imports` (11/11, including a 16-case property and
  the retained structural-child compatibility regression).
  Positive `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-NATURAL-NAME`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-ROOT-TARGET`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-VISIBILITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-DUPLICATE-BINDING`, and
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-CYCLE-ATOMICITY` are test evidence, not proof or
  end-to-end parity evidence. The delivered scoped simple ordinary-function imports M-SIMPLE slice is `partial / tested / below_spec`: `bind_scoped_simple_ordinary_function_imports(graph, scopes)` in `crates/ash-typeck/src/canonical_structural_module_binder.rs`, exported through `crates/ash-typeck/src/lib.rs`, consumes delivered canonical provisional scopes and delegates directly to `resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)` followed by `into_bound_set`. It admits only inherited simple `use crate::<ordinary-function>` or `use crate::<structural-child>...::<ordinary-function>` routes, each optionally followed by `as <name>`; without `as`, the final ordinary-function segment is the natural local binding name. Its ordinary-function target may be public, crate, super, `pub(in path)`, inherited/private, or self only when the canonical `ModuleKey` visibility predicate permits the importer, with the existing whole structural-path fence for public targets. It preserves every resolver structural diagnostic and outer `CanonicalImportCycle` provenance unchanged and atomically returns no `CanonicalBoundModuleSet` on structural, visibility, local-collision, duplicate-binding, snapshot, or cycle error. The focused `task_2068_scoped_simple_ordinary_function_imports` target passes 11/11, including a 16-case property and the retained structural-child compatibility regression; these tests are evidence, not proof or end-to-end parity. The existing generic `resolve_simple_parsed_imports`, `crates/ash-typeck/src/canonical_module_binder.rs`, and `bind_simple_parsed_uses` remain unchanged and generic-only; the generic binder must not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. Final interfaces/export closure, all other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered scoped grouped ordinary-function imports M-GROUP sub-slice:** implementation
  `partial`; evidence `tested`; parity `below_spec`; parser syntax carrier `partial`, Type
  `partial`, Core/CPS/admission-runtime `not_applicable`, verification `partial`; run-route
  impact `prerequisite`. Parser-owned `UseItem { name, alias, span }` spans each nested member's
  name plus optional alias through source offsets, never the enclosing use span. The dedicated
  resolver/binder admits only inherited `UsePath::Nested` `crate`/structural-child ordinary
  function members, with optional aliases or natural local names; it stages scope snapshots,
  visibility and public-path fencing, collisions, duplicates, and the full cross-module cycle set
  before atomically returning a plan or binding set. Parser-span positive
  `TEST-MOD-REAL-004-PARSED-GROUP-MEMBER-SPAN`; Type positive
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-DUPLICATE-BINDING`, and
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-CYCLE-ATOMICITY` are test evidence, not proof or
  end-to-end parity evidence. The focused grouped target passes 10/10, including a 16-case
  property; the parser full suite also passes. `DuplicateBinding` uses the later grouped member's
  span, and grouped structural-child members reject anchored `Unsupported`, while the pre-existing
  simple structural-child compatibility route remains enclosing-span `Unresolved`. The generic
  resolver/binder remains unchanged and generic-only. Globs, `pub use`, non-inherited bases,
  nested groups, non-function members, other namespaces, final interfaces/export closure,
  Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is
  `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot
  begin until TASK-2068 is complete.
- **M-GROUP record-mirrored missing target-spec clause:** The delivered scoped grouped ordinary-function imports M-GROUP slice is `partial / tested / below_spec`: it retains parser-owned nested-member spans and accepts only inherited `UsePath::Nested` crate/structural-child ordinary-function members with optional aliases or natural local names through the dedicated scoped resolver/binder. It atomically rejects scope snapshot, structural visibility, local collision, duplicate binding, and complete-group cycle failures before publishing a plan or binding set; generic resolver/binder authority is unchanged. Globs, `pub use`, non-inherited bases, nested groups, non-function members, other namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered scoped `super` ordinary-function imports M-SUPER sub-slice:** implementation
  `partial`; evidence `tested`; parity `below_spec`; Type `partial`, Core/CPS/admission-runtime
  `not_applicable`, verification `partial`; run-route impact `prerequisite`. The dedicated
  `resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)` and
  `bind_scoped_super_ordinary_function_imports(graph, scopes)` route admits only inherited,
  non-root `UsePath::Simple` paths with exactly one leading `super`, starts from
  `ModuleKey::parent()`, traverses zero or more delivered structural children, and ends at one
  ordinary function with an optional alias or natural final local name. It preserves complete
  `Use::span` identity, edge, and diagnostic anchors and stages scope snapshots, child origin,
  canonical visibility/whole-public-path checks, local collisions, duplicate bindings, and cycles
  before atomic publication; same-module occurrences create no edge. Every child segment and the
  final function segment named `super` reject before lookup, preventing repeated `super` and
  `fn super` bypasses. Positive
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-ROOT-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-DUPLICATE-BINDING`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-CYCLE-ATOMICITY` pass in the focused target (12/12,
  including a 16-case property). The final-`super` callable test reinforces the root/repeated
  boundary rather than adding a witness. These tests are not proof or end-to-end parity evidence.
  Root/repeated/self/crate/unprefixed/stdlib/external bases, groups/globs/nested groups, `pub use`
  or restricted uses, non-function targets, all other namespaces, generic resolver/binder changes,
  final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain
  excluded. `self::` stays deferred because same-module precedence is separately unresolved. The
  route consumes only canonical graph units, parser spans, and provisional scopes, produces only a
  Type-layer plan/bound set/edges, leaves the generic binder unchanged at
  `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`, and keeps
  TASK-2069 as the lowering/transport owner and TASK-2064 as the parity owner.
- **M-SUPER record-mirrored missing target-spec clause:** The delivered scoped `super` ordinary-function imports M-SUPER slice is `partial / tested / below_spec`: its dedicated resolver and binding-only projection admit only inherited, non-root, exactly-one-leading-`super` `UsePath::Simple` parent/sibling ordinary-function routes with an optional alias or natural local name. It retains the full `Use::span`, canonical scope/visibility/whole-public-path, collision/duplicate, cycle, and atomic-publication rules, rejects every extra or final `super` before lookup, and leaves generic resolver/binder authority unchanged. The focused target passes 12/12 including a 16-case property; this is test evidence, not proof or end-to-end parity. Root/repeated/self/crate/unprefixed/standard-library/external paths, groups/globs, public or restricted uses, non-functions, other namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Delivered scoped `super` grouped ordinary-function imports M-SUPER-GROUP sub-slice:**
  implementation `partial`; evidence `tested`; parity `below_spec`; Type and verification
  `partial`, Core/CPS/admission-runtime `not_applicable`; run-route impact `prerequisite`.
  The dedicated
  `resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(graph, scopes)` and
  `bind_scoped_super_grouped_ordinary_function_imports(graph, scopes)` route consumes only
  canonical graph units, parser-owned individual `UseItem::span` facts, and provisional scopes.
  It leaves generic resolver/binder authority unchanged, with the generic binder fingerprinted as
  `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
  Implementation node `IMPL-MODULE-SCOPED-SUPER-GROUPED-ORDINARY-FUNCTION-IMPORTS` is
  implemented and its ten canonical witnesses are tested:
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-ROOT-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-DUPLICATE-BINDING`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-CYCLE-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-AUTHORITY-FENCE`. POSITIVE, IDENTITY,
  FILE-INLINE-PARITY, and PROPERTY are positive evidence; VISIBILITY-DIAGNOSTIC,
  ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and AUTHORITY-FENCE are negative evidence;
  CYCLE-ATOMICITY is mutation evidence. The focused target passes 13/13 including a 16-case
  property. These tests are evidence, not proof or end-to-end parity evidence. Source
  fingerprints: scoped resolver
  `sha256:77ff8e437ada70fc1182bb52b99a4d9e56c2fe39c669ffe87258ff71d8eb021c`; dedicated binder
  `sha256:0bfb497fe11b17623bcb39485d2420f80d3b5c64a39ba4ad9642f148d4413a06`; `lib.rs` export
  boundary `sha256:68775641f867d47b9f4a7af344b856eb3ec132f256659fc68bdc51444e934f86`.
- **M-SUPER-GROUP record-mirrored missing target-spec clause:** The delivered scoped `super` grouped ordinary-function imports M-SUPER-GROUP slice is `partial / tested / below_spec`: its dedicated resolver and binding-only projection admit only inherited, non-root `UsePath::Nested` routes with exactly one leading `super`, no outer alias, zero or more structural children after the canonical parent, and a nonempty group of ordinary-function members with natural/member `as` local names. It retains every parser-owned individual member span in identity, edge, and member-specific error facts; preflights a final member named `super` before lookup; and reuses canonical scopes/snapshots/visibility/whole-public-path, same-module-no-edge, local-collision, duplicate-binding, complete-group cycle, and atomic-publication rules. The focused target passes 13/13 including a 16-case property; its ten canonical witness IDs are test evidence, not proof or parity evidence. Root/repeated `super`, `self`, `crate`, unprefixed, standard-library/external, simple/glob/non-nested/nested-group, outer aliases, public/restricted/re-export forms, nonfunctions or other namespaces, generic resolver/binder changes, final interfaces/export closure, Core/CPS, Engine, admission/runtime, client parity, and general precedence remain deferred. Type and verification are `partial`; Core/CPS/admission-runtime are `not_applicable`; run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Current successor obligation:** TASK-2074 completes expansion and TASK-2075 completes
  namespace/callable collection; TASK-2072
  completes parsed import/visibility/edge/cycle/precedence/atomic-binding and staged-`pub use`
  semantics; TASK-2073 completes body checking, final interfaces, and export closure. TASK-2069
  then consumes only TASK-2073's complete checked handoff; TASK-2064 owns integration parity.
- **Delivered M-GLOB scoped glob ordinary-function imports:** implementation `partial`; evidence
  `tested`; parity `below_spec`; Type `partial`, Core/CPS/admission-runtime `not_applicable`,
  verification `partial`; run-route impact `prerequisite`. The dedicated
  `resolve_scoped_glob_ordinary_function_imports_with_scopes(graph, scopes)` and
  `bind_scoped_glob_ordinary_function_imports(graph, scopes)` route remains Type-only and
  binding-only. It has no proof, final-interface, generic-binder, Core/CPS/Engine,
  admission/runtime, or client-parity claim. Implementation node
  `IMPL-MODULE-SCOPED-GLOB-ORDINARY-FUNCTION-IMPORTS` is implemented and its ten canonical
  witnesses are tested: `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-SHAPE-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CONFLICT-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AMBIGUITY-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CYCLE-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AUTHORITY-FENCE`. POSITIVE, IDENTITY,
  FILE-INLINE-PARITY, and PROPERTY are positive evidence; VISIBILITY-DIAGNOSTIC,
  SHAPE-DIAGNOSTIC, and AUTHORITY-FENCE are negative; CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY,
  and CYCLE-ATOMICITY are boundary mutation evidence only. Source fingerprints: dedicated binder
  `sha256:6fd37ea25cf3aa6767b9c2175a57f3761cf947d7a23bdf4020fff653ab250aa9`; scoped planner
  `sha256:568bb73d47f3b96633b256a857dc606ac868ef18bd314e07968b85a9b8f795e9`; `lib.rs` export
  boundary `sha256:8dfaa8852bdbc697b00f5d509e9359f687284e2d502fdd918c695c8e5bc5ddd1`; unchanged generic
  binder `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Record-mirrored missing target-spec clause:** The delivered scoped glob ordinary-function imports M-GLOB slice is `partial / tested / below_spec`: it admits only inherited `use crate::<public structural-child>...::*` routes contributing ordinary public functions, with exactly one use and zero local ordinary functions, so it does not decide local/explicit/glob precedence. The dedicated Type-only resolver `resolve_scoped_glob_ordinary_function_imports_with_scopes(graph, scopes)` and binding-only projection `bind_scoped_glob_ordinary_function_imports(graph, scopes)` consume only the canonical graph, parser-owned full `Use::span`, and provisional scopes. They traverse public structural children, select only visible public ordinary functions, preserve every selected function's defining identity, declaration origin/span/visibility, and full use span, produce one cross-module edge per function, and stage all candidates before atomic plan/bound-set publication. Boundary failures return no plan or bindings: a local function is `Unsupported` at the zero-local-function boundary; a second glob is `Unsupported` at the exactly-one-use boundary; and a cycle-shaped attempted program is the same boundary `Unsupported`, never `ImportCycle`. The CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY IDs are boundary-mutation evidence only: they claim neither `LocalDeclarationCollision`, `DuplicateBinding`, generic ambiguity, `ImportCycle`, a bound set, a plan, nor precedence. The defensive planner collision/duplicate/cycle branches are unclaimed. The SHAPE-DIAGNOSTIC matrix covers 15 valid parser representations; a leading `::` is not `UsePath::Glob`, and a private structural module is an `Inaccessible` visibility case. The 16-case PROPERTY varies public-child depth, function count, function/path visibility, and inline/file-backed source form. Type is `partial`; Core/CPS/admission-runtime are `not_applicable`; verification is `partial`; run-route impact is `prerequisite`. Tests are evidence, not proof, final-interface, generic-binder, Core/CPS/Engine/admission/runtime, or client-parity evidence. `self`, root/repeated `super`, non-`crate` paths, multiple globs, local declarations, explicit/group imports, aliases, re-exports or `pub use`, non-function namespaces, and all remaining import forms remain deferred.
- **Current successor obligation:** The delivered M-GLOB/M-SUPER/M-GROUP/M-SIMPLE and related
  fragments remain TASK-2068 historical evidence. TASK-2074/TASK-2075 own expansion/collection; TASK-2072
  owns every remaining parsed import/binding rule; TASK-2073 owns all remaining body/interface/
  export-closure rules; and TASK-2069 consumes only TASK-2073's completed handoff.
- **Non-goals:** Structural graph discovery or source acquisition.
- New syntax, dynamic imports, packages, import-cycle initialization, or runtime module values.
- Outside the delivered M-CHECK leaves, all typed namespaces beyond ordinary functions; all definition forms beyond ordinary functions; generic or contract-bearing functions; restricted declaration visibility beyond the delivered `pub(crate)`, `pub(super)`, `pub(in crate)`/`pub(in crate::...)`, and `pub(self)` closed primitive domain; non-primitive/open signatures; user-defined types, interfaces, and effects; final public/private interface publication; export closure; public aliases, re-exports, or pub use.
- Delivered M-CHECK excludes imports, child modules, nested modules, other definitions, generics, contracts, restricted visibility outside the delivered closed primitive domain, user-defined types, interfaces, effects, re-exports, final full interfaces, Core/CPS/Engine, and client parity.
- Delivered graph-only simple-import planning excludes checked interfaces, TypeEnv/body integration, legacy or TASK-2060/TASK-2061/TASK-2066 authority, restricted visibility, pub use/re-exports, groups/globs/qualified paths, every other import form, Core/CPS/Engine, and client parity.
- Beyond the delivered planner's inherited UsePath::Simple crate-root function aliases, parsed use forms; qualified paths, group/glob imports, non-inherited use visibilities, restricted declaration visibilities, complete visibility handling, remaining import-cycle rules, or legacy binder/graph/interface authority remain excluded.
- Delivered canonical primitive provider/client checking excludes any widening of the delivered primitive leaf pass; any non-root client, non-plan-selected direct provider, unrelated unselected graph unit, or non-direct/nested provider; non-primitive/open signatures; final interfaces or export closure; import forms beyond the delivered planner; legacy TASK-2060/TASK-2061/TASK-2066 carriers; and Core/CPS/Engine, admission, runtime, or client parity.
- Delivered direct-public primitive re-export fragment excludes all but root `pub mod` direct
  provider identity plus exact root `pub use crate::<direct-provider>::<primitive-function> as
  <alias>`; every other namespace, declaration/import/path/visibility/re-export form,
  compatibility carrier, final interface/export closure, Core/CPS/Engine, admission/runtime, and
  parity.
- Delivered direct-public primitive re-export interface fragment excludes every namespace, declaration/import/path/visibility/re-export form except root `pub mod` direct-provider identity plus exact root `pub use crate::<direct-provider>::<primitive-function> as <alias>`; it also excludes compatibility carriers, final interface/export closure, Core/CPS/Engine, admission/runtime, and parity.
- Direct Core/CPS lowering, Engine scanner/cache fencing, linking/admission/execution, or CLI/daemon parity.
- Treating an interface or binder fact as an Engine admission credential, provider/handler-frame authority, or direct-evaluator fallback.
- **Current successor obligation:** TASK-2074 completes canonical expansion and TASK-2075 completes
  every required namespace/callable collection; TASK-2072 completes every remaining parsed import/visibility/alias/re-export/cycle
  rule and atomic M-BIND publication; TASK-2073 completes definition/body checking and final
  interface/export closure. TASK-2069 then consumes only TASK-2073's complete checked handoff,
  and TASK-2064 owns integration parity.
- **Record-mirrored next obligation:** TASK-2068 has no remaining implementation obligation:
  TASK-2070 supplies the completed bounded M-SELF-SIMPLE-ALIAS leaf, TASK-2071 supplies the completed
  contract, TASK-2074/TASK-2075 own expansion and two-tier collection, TASK-2072 owns complete parsed imports and atomic binding, and
  TASK-2073 owns complete checked finalization/export closure. TASK-2069 consumes TASK-2073's
  complete checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns integration parity;
  TASK-2065 owns closeout inventory.
- **Delivered M-GLOB-LOCAL-PRECEDENCE:** implementation partial; evidence tested; parity
  below_spec; Type partial; Core/CPS/admission-runtime not_applicable; verification partial;
  run-route impact prerequisite. It extends only the existing exactly-one inherited
  crate::<public structural child>...::* ordinary-function glob route. Same-module ordinary
  functions shadow same-name selected public imports only in returned public bindings, while
  non-colliding imports bind. Every selected cross-module edge, including a shadowed target,
  survives through canonical cycle detection before filtering; all-shadowed input succeeds with
  no import bindings but retained edges, while actual hidden cycles return atomic ImportCycle {
  edges: CanonicalImportCycle }. Existing M-GLOB behavior remains separate/rejecting. This uses
  canonical graph/provisional scopes only, never private M-CHECK facts, and produces only
  non-authorizing Type facts. Other imports, multiple globs, aliases/re-exports,
  self/super/non-crate paths, nonfunctions, the generic binder, final interfaces, Core/CPS,
  Engine, admission/runtime, and parity remain excluded; TASK-2069 owns lowering and TASK-2064
  owns parity.
- **Delivered M-SIMPLE-LOCAL-PRECEDENCE:** implementation partial; evidence tested; parity
  below_spec; Type and verification partial; Core/CPS/admission-runtime not_applicable; run-route
  impact prerequisite. It adds one dedicated resolver and delegating binder for exactly one
  inherited, unaliased `UsePath::Simple`
  `use crate::<public structural child>...::<public ordinary-function>;` route with its natural
  final name. A selected cross-module target retains one edge and cycle-checks before filtering a
  same-name local ordinary-function binding; a selected same-module target emits no self-edge and
  does not participate in cycle detection. Non-colliding imports bind, all shadowed cross-module
  candidates retain edges with no import binding, and a real hidden two-module cross-module cycle
  rejects atomically. The existing M-SIMPLE route remains unchanged and rejects local collisions.
  This consumes canonical graph/provisional scopes only; M-CHECK private facts and generic binder
  authority remain excluded. The focused target passes 9/9; its 16-case source-form claim is
  normalized Type-layer scope/binding parity, never final/runtime parity. Its visibility/shape
  matrix rejects aliases, root-function imports, groups, globs, self, super, multiple uses, a
  private structural segment, a nonfunction target, a private target, and `pub use`.
- **Record-mirrored delivered clause:** The delivered M-SIMPLE-LOCAL-PRECEDENCE slice is partial / tested / below_spec: it admits exactly one inherited, unaliased UsePath::Simple use crate::<public structural child>...::<public ordinary-function>; route with its natural final name, while same-module ordinary functions are permitted. It selects the public ordinary-function target. A selected cross-module target retains one edge; complete deterministic cycle detection considers only those cross-module edges before filtering a same-name local binding. A selected same-module target emits no self-edge and does not participate in cycle detection. A non-colliding import binds, all shadowed cross-module candidates return no import binding but retain their edges, and a real hidden two-module cross-module cycle rejects atomically as ImportCycle { edges: CanonicalImportCycle }. It consumes canonical graph/provisional scopes only, with no M-CHECK private-fact or generic-binder authority, and the existing M-SIMPLE route remains unchanged with local collision rejection. Root functions, aliases, multiple uses, groups, globs, self, super, restricted/private targets or structural paths, re-exports, nonfunctions, body lexical binding, final interfaces, Core/CPS, Engine, admission/runtime, and parity remain excluded. Its visibility/shape matrix rejects aliases, root-function imports, groups, globs, self, super, multiple uses, a private structural segment, a nonfunction target, a private target, and pub use. Its focused crates/ash-typeck/tests/task_2068_local_over_simple_precedence.rs target passes 9/9: local-wins/noncollision, identity/edge, all-shadowed, file/inline normalized Type-layer scope/binding parity, and the 16-case depth 1–3/name/collision-mask/source-form property are positive evidence; visibility/shape, authority fence, and legacy M-SIMPLE regression are negative; hidden-cycle atomicity is mutation evidence. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite; TASK-2069 owns lowering and TASK-2064 owns parity.
- **Record-mirrored non-goal:** The delivered M-SIMPLE-LOCAL-PRECEDENCE slice excludes root functions, aliases, multiple uses, groups, globs, self, super, restricted/private target or structural-path access, pub use/re-exports, nonfunctions, lexical body bindings, final interfaces, Core/CPS, Engine, admission/runtime, and parity; it neither consumes private M-CHECK facts nor changes generic-binder authority.
- **Test evidence:** TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-WINS-NONCOLLIDING,
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-IDENTITY-EDGE,
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-ALL-SHADOWED-EMPTY-BINDING,
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-FILE-INLINE-PARITY, and
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-PROPERTY are positive evidence;
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-VISIBILITY-SHAPE,
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-AUTHORITY-FENCE, and
  TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-LEGACY-M-SIMPLE-REGRESSION are
  negative evidence; TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-CYCLE-ATOMICITY
  is mutation evidence.
- **Test evidence:** IMPL-MODULE-SCOPED-GLOB-LOCAL-OVER-GLOB-PRECEDENCE plus
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-WINS-NONCOLLIDING,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-IDENTITY-EDGE,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-ALL-SHADOWED-EMPTY-BINDING,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-CYCLE-ATOMICITY,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-VISIBILITY-SHAPE,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-FILE-INLINE-PARITY,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-PROPERTY, and
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-AUTHORITY-FENCE are implemented/tested. The
  focused target passes 8/8. Positive evidence is WINS-NONCOLLIDING, IDENTITY-EDGE,
  ALL-SHADOWED-EMPTY-BINDING, FILE-INLINE-PARITY, and PROPERTY; VISIBILITY-SHAPE and
  AUTHORITY-FENCE are negative; CYCLE-ATOMICITY is mutation evidence. The property has exactly 16
  cases varying names, collision subsets, source form, and depth 1–3. File/inline proves
  normalized Type-layer scope/binding parity only, never final/runtime parity.
- **Source traceability:** planner sha256:17b2ffe653d196ba295ea1e93bd57ad8c193596918f3787c3f43a1e2e6299f2a;
  dedicated binder sha256:652062ee3430667a1259f92777cf7e369b9b5e7ce167151941ade225fb0f8bf1;
  lib.rs export boundary sha256:99e7a4c81c34ced69e5fb78830176406e2b30c37b7bf8a9b5617eb78e4664aa6;
  unchanged generic binder fence sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6.
- **Record-mirrored delivered clause:** The delivered M-GLOB-LOCAL-PRECEDENCE slice is partial / tested / below_spec: it admits exactly one inherited crate::<public structural child>...::* glob domain. Same-module ordinary functions shadow same-name imported public ordinary functions only in returned public bindings; non-colliding imports remain. The resolver retains every selected cross-module edge, including shadowed targets, and cycle-checks before filtering, so all-shadowed input succeeds with no import bindings but retained edges and actual hidden cycles return atomic ImportCycle { edges: CanonicalImportCycle }. It consumes canonical graph/provisional scopes only and never private M-CHECK facts. Existing M-GLOB behavior remains separate/rejecting; other imports, multiple globs, aliases/re-exports, self/super/non-crate paths, nonfunctions, the generic binder, final interfaces, Core/CPS, Engine, admission/runtime, and parity authority remain excluded. The focused target passes 8/8, including a 16-case property varying names, collision subsets, source form, and depth 1–3; file/inline evidence establishes normalized Type-layer scope/binding parity only, never final/runtime parity. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite; TASK-2069 owns lowering and TASK-2064 owns parity.
- **Delivered M-CHECK restricted declaration visibility slice:** implementation `partial`; evidence
  `tested`; parity `below_spec`; Type `partial`; Core/CPS/admission-runtime `not_applicable`;
  verification `partial`; run-route impact `prerequisite`. It accepts only `pub(crate)`,
  `pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` primitive closed
  ordinary-function leaves in a file-root closed leaf. It graph-preflights, stages sibling
  signatures, and checks bodies atomically; it retains identity/origin/spans/visibility/signature/
  body facts only in `private_functions`, while only `Visibility::Public` projects publicly.
  `pub(in self::internal)` rejects as an anchored unsupported visibility. The focused target passes
  18/18. Its eleven canonical witness nodes are tested; the file/inline-named witness is a source-form
  boundary—file-root success versus inline-child `UnsupportedModuleShape` before projection—not
  normalized-success parity. Source fingerprint:
  `sha256:22d2582021f2a9921f51f25786a848aed174232beb51b02b635e9ac5e595bdda`.
- **Record-mirrored missing target-spec clause:** The delivered M-CHECK-RESTRICTED-VISIBILITY slice
  is `partial / tested / below_spec`: it accepts exactly the four stated restricted forms for
  primitive closed ordinary-function leaves in a file-root closed leaf, limits `pub(in ...)` to
  `crate` or `crate::...`, and rejects non-crate restricted paths. It graph-preflights, stages
  signatures, and checks bodies atomically; it retains fresh identity, defining key, origin, spans,
  visibility, signature, and body type only in `private_functions`; public projection remains only
  `Visibility::Public`. The focused target passes 18/18, including all eleven canonical witnesses.
  The file/inline-named witness is a source-form boundary—file-root
  success versus inline-child `UnsupportedModuleShape` before projection—not normalized-success
  parity. It is not import, binder, re-export, final-interface, Core/CPS, admission/runtime,
  proof, or parity authority. Type and verification are `partial`; Core/CPS/admission-runtime are
  `not_applicable`; run-route impact is `prerequisite` for TASK-2069; TASK-2064 owns integration
  parity.
- **Record-mirrored evidence identifiers:** `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-CRATE`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SUPER`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-IN-CRATE`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SELF`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-IDENTITY-PROVENANCE`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PROPERTY`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-SIGNATURE-BODY-DIAGNOSTICS`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-NON-CRATE-PATH-DIAGNOSTIC`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY`, `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUBLIC-PROJECTION-AUTHORITY-FENCE`, and `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-ATOMICITY` are recorded respectively as positive, negative, or mutation evidence in the task record.
- **Record-mirrored non-goal:** Outside the delivered M-CHECK leaves, all typed namespaces beyond ordinary functions; all definition forms beyond ordinary functions; generic or contract-bearing functions; restricted declaration visibility beyond the delivered pub(crate), pub(super), pub(in crate)/pub(in crate::...), and pub(self) closed primitive domain; non-primitive/open signatures; user-defined types, interfaces, and effects; final public/private interface publication; export closure; public aliases, re-exports, or pub use.
- **Record-mirrored exact missing clause:** The delivered M-CHECK-RESTRICTED-VISIBILITY slice is `partial / tested / below_spec`: it accepts exactly `pub(crate)`, `pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` for primitive closed ordinary-function leaves in a file-root closed leaf. It graph-preflights every unit, stages sibling signatures, and checks bodies atomically; it retains fresh identity, defining key, origin, declaration/body spans, declared visibility, signature type, and body type only in `private_functions`; `CanonicalPublicFunctionInterface` projects only `Visibility::Public`. `pub(in self::internal)` rejects as an anchored unsupported visibility. The focused target passes 18/18, with all eleven canonical witnesses tested. Its file/inline-named witness is a source-form boundary only: file-root success versus inline-child/module `UnsupportedModuleShape` before projection, never normalized-success parity. It authorizes no imports, binders, re-exports, final interfaces, Core/CPS, admission/runtime, proof, or parity. Type and verification are `partial`; Core/CPS/admission-runtime are `not_applicable`; run-route impact is `prerequisite` for TASK-2069; TASK-2064 owns integration parity.
- **Record-mirrored exact next obligation:** TASK-2068 has no remaining implementation obligation: TASK-2070 supplies the completed bounded M-SELF-SIMPLE-ALIAS handoff; TASK-2071 supplies the completed contract; TASK-2074 and TASK-2075 own expansion and two-tier collection; TASK-2072 owns complete parsed imports and atomic binding; and TASK-2073 owns complete checked finalization/export closure. TASK-2069 consumes TASK-2073's complete checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns integration parity; TASK-2065 owns closeout inventory.
- **Handoff:** partial/tested. TASK-2068 consumes only TASK-2067's canonical graph/unit
  payloads. `M-COLLECT` publishes provisional function defining identity, source anchor, origin,
  and declared visibility for this binding pass. Its bounded planner resolves only simple inherited
  `crate::…` function aliases, retains the stated canonical cross-module edge provenance, emits no
  same-module edge, and rejects every discovered cycle before a result as `ImportCycle { edges:
  CanonicalImportCycle }`; the compatibility binder delegates through it. It atomically returns no
  set on a private target. Public
  use, re-exports, non-inherited use visibility, and restricted target declaration visibility all
  reject before publication. The opaque returned set has neither `Default` nor a public constructor,
  so no caller may fabricate success outside the binder. These are not final interfaces, complete
  graph import edges, Engine credentials, Core/CPS artifacts, or a client/Engine parity claim.
  Delivered M-CHECK separately graph-preflights every canonical unit, stages ordinary-function
  sibling signatures in a fresh builtin TypeEnv, checks all bodies atomically, and retains fresh
  checked identity plus canonical key/origin/spans/signature/body type. It exports only public
  primitive signatures through non-authorizing `CanonicalPublicFunctionInterface`; it is neither
  core `PublicModuleInterface` nor a final interface, import/binder credential, or Engine
  authority. Delivered provider/client checking consumes only that canonical graph and planner
  facts, accepts only the root plus plan-selected direct provider leaves, and pre-provider
  `module_units()` completeness rejects unrelated unselected non-root units. A descendant of a
  selected provider instead reaches provider-leaf precheck and rejects as anchored
  `UnsupportedProviderShape`; no nested module can succeed. The checker revalidates checked public
  provider edges before fresh-root signature injection and atomically emits non-authorizing checked
  root/provider/import facts.
  The delivered direct-public fragment consumes that bounded domain plus an exact direct-public
  plan and atomically returns only an explicit public structural-child and alias projection
  retaining defining identity/origin/signature/declaration/use spans. It validates `pub mod api`
  plus explicit root `pub use crate::api::greet as welcome`, public path/target closure, no
  implicit flattening, exact artifact revalidation, empty-plan/root-shape/collision rejection, and
  atomic failure. It remains `partial / tested / below_spec` and grants no final-interface,
  binder, Core/CPS, Engine, admission, runtime, or parity authority.

### Calls and continuations

- **Canonical owner:** `SEM-CPS-CALL-001`, `SEM-CPS-JUMP-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** calls with parameters, closures, recursion, and imports.

### Core control and terminals

- **Canonical owner:** `SEM-CPS-LETVAL-001`, `SEM-CPS-IF-001`, `SEM-CPS-RETURN-001`,
  `SEM-CPS-TRAP-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** source control lowering beyond the currently realized forms.

**TASK-2031B evidence handoff:** verification-only reconciliation of two lexical-scope CLI negative
assertions to the existing checked Core-to-CPS bridge-domain rejection. It consumes
TASK-2003/TASK-2004/TASK-2014 admission facts and changes no Type, Core, CPS, admission/runtime,
or terminal layer.

**Handoff:** complete. **Evidence:** tested by the focused lexical-scope target 6/6 with the
canonical shared run-admission rejection; the recorded workspace Rust gate passed. This handoff
does not change production Rust or semantic-layer realization.

### Operations and lookup

- **Canonical owner:** `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** arbitrary operations, arguments, imports, and chains.

### Handlers and deep affine resume

- **Canonical owner:** `SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DEEP-AFFINE-HANDLE-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** multi-clause, open-row, imported, and multi-shot behavior.

### Rows and imported summaries

- **Canonical owner:** `SPEC-097b`, `TYPE-TARGET-ROW-001`
- **Layer status:** Type partial; Core partial metadata; CPS not_applicable; admission/runtime
  non-authorizing; verification partial.
- **Missing target-spec clauses:** row polymorphism, expansion, and discharge.

### Production admission and frames

- **Canonical owner:** `TASK-2004`, `TASK-2014`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** production artifacts and route coverage required by the target
  rules.

### Terminal envelopes and async control

- **Canonical owner:** `TASK-2008`, `TASK-2014`
- **Layer status:** Type partial; Core not_applicable; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** terminal outcomes and routes not yet realized.

**TASK-2031C prerequisite handoff:** Linux test-host verification that a programmatic
SIGINT reaches an isolated Tokio listener before TASK-2008's exact admitted `time::sleep` route is
evaluated. Type/Core/CPS/admission, the existing CLI forwarding, and Engine control precedence are
consumed existing layers. TASK-2008 consumes the terminal outcome, while TASK-2032 owns
client-parity integration.

**Handoff:** complete. **Evidence:** tested; the test-only probe explicitly classifies the managed
sandbox as unavailable; capable-host controls retain exit 130 and the exact V1 cancellation
envelope on stdout and `--output`. No production CLI/Engine or semantic-layer realization changed.

**TASK-2031F evidence handoff:** correction of three existing stdlib callable negative
assertions to TASK-2003's current PureAnf bridge-domain rejection. Type/Core/CPS/admission/runtime
layers are consumed existing behavior; run-route impact is none and TASK-2032 retains parity
evidence.

**Handoff:** complete. **Evidence:** tested by three controls retaining parse/check success and the
exact shared current PureAnf bridge-domain diagnostic; `module_resolution` passed 17/17. No
semantic-layer or production behavior changed.

### Engine-only client contracts

**TASK-2035 semantic workflow record:**
[TASK-2035](tasks/TASK-2035-canonical-client-test-contracts.md) defines
`CONF-SYNTH-SOURCE-WRAPPER-001`, `OBS-REPL-ENGINE-CLIENT-001`, and
`CONF-ENGINE-ONLY-CLIENT-001` for the one Engine executor route.

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

**Missing target-spec clauses:** Realize every selected wrapper, REPL route, and daemon route through Engine; then realize the remaining target SPEC-077 and SPEC-011 domains before claiming parity.

**Layers:** type partial; core partial; cps partial; admission-runtime not_implemented;
verification not_implemented.

**Run-route impact:** prerequisite.

**Consumes:** `AUDIT-204-TEST-EXEC-002`, `AUDIT-204-REPL-001`, `AUDIT-204-REPL-002`, and the
seven named `AUDIT-204-DEFERRED-*` cases; target grammar/type/Core/CPS rules; and the existing
Engine admitted-request seam.

**Produces:** exact source-wrapper and fail-closed results in SPEC-077, the SPEC-011 REPL
Engine-client rule, and the SPEC-026 single-executor comparison rule.

**Downstream owner:** TASK-2038 implements test wrappers; TASK-2039 implements REPL; TASK-2042
implements daemon transport and `ash run` parity; TASK-2041 owns four-client parity.

**Evidence detail:** none. The source and deferred examples in TASK-2035 are contract text, not
test or proof evidence. **Parity evidence:** not applicable; no client route is realized by this
documentation task.

**Non-goals:** Source lowering, Engine APIs, test-runner execution, REPL execution, daemon transport, a general source synthesizer, and Lean implementation.

**Next obligation:** TASK-2038, TASK-2039, and TASK-2042 must implement their named routes with focused tests; TASK-2041 must establish the same-source-contract four-client terminal comparison.

## TASK-2070: Scoped Self Simple Function Aliases

- **Task:** [TASK-2070](tasks/TASK-2070-scoped-self-simple-function-aliases.md)
- **Canonical owner:** `SPEC-103`, `MOD-REAL-004`, and `SEM-MODULE-REALIZATION-004`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Layers:** type partial; core not_applicable; cps not_applicable; admission-runtime not_applicable; verification partial.

**Run-route impact:** prerequisite.

**Missing target-spec clauses:** The delivered M-SELF-SIMPLE-ALIAS slice is `partial / tested / below_spec`. Its dedicated resolver and binder admit zero or more individually eligible inherited two-segment `use self::<ordinary_function> as <different_alias>;` statements in root or nested modules. They resolve only direct same-`ModuleKey` ordinary functions, apply exact declared visibility through `is_visible_from`, stage distinct aliases together, reject duplicate/local collisions atomically, and retain identity, declaration span, origin, visibility, and full `use_span`. `CanonicalResolvedSelfOrdinaryFunctionAliases` has no import-edge field; only the dedicated binder calls private `into_bound_alias_set` to produce `CanonicalBoundSelfOrdinaryFunctionAliasSet`. No `CanonicalSimpleImportEdge` or cycle check is created, and `ImportCycle` is unreachable by construction and the tested source fence. Groups, globs, mixed/other forms, direct child modules, and all out-of-domain shapes reject without publishing a result. The generic binder and private M-CHECK facts remain outside its authority. Focused evidence is 8/8, including the exact 16-case property with alias count `1..3`; predecessor evidence is 32/32 and `ash-typeck` library evidence is 477/477. Complete import grammar/precedence/cycles, final interfaces/export closure, Core/CPS, Engine, admission/runtime, and client parity remain deferred to TASK-2072 through TASK-2064.
**Record-mirrored exact missing clause:** The delivered M-SELF-SIMPLE-ALIAS slice is `partial / tested / below_spec`: its dedicated resolver and binder admit zero or more individually eligible inherited two-segment `use self::<ordinary_function> as <different_alias>;` statements in root or nested modules. They resolve only direct same-`ModuleKey` ordinary functions, apply exact declared visibility through `is_visible_from`, stage distinct aliases together, reject duplicate/local collisions atomically, and retain identity, declaration span, origin, visibility, and full `use_span`. `CanonicalResolvedSelfOrdinaryFunctionAliases` has no import-edge field; only the dedicated binder calls private `into_bound_alias_set` to produce `CanonicalBoundSelfOrdinaryFunctionAliasSet`. No `CanonicalSimpleImportEdge` or cycle check is created, and `ImportCycle` is unreachable by construction and the tested source fence. Groups, globs, mixed/other forms, direct child modules, and all out-of-domain shapes reject without publishing a result. The generic binder and private M-CHECK facts remain outside its authority. Focused evidence is 8/8, including the exact 16-case property with alias count `1..3`; predecessor evidence is 32/32 and `ash-typeck` library evidence is 477/477. Complete import grammar/precedence/cycles, final interfaces/export closure, Core/CPS, Engine, admission/runtime, and client parity remain deferred to TASK-2072 through TASK-2064.
- **Record-mirrored non-goal:** The delivered M-SELF-SIMPLE-ALIAS slice excludes natural-name or equal self aliases, same-module child traversal, direct child-module/nonfunction targets, cross-module import traversal/edges/cycles, public/restricted use/re-exports, crate/super/unprefixed paths, groups/globs/mixed or other import forms, successful duplicate or local-colliding bindings, final interfaces, generic-binder changes, M-CHECK private-fact authority, Core/CPS, Engine, admission/runtime, and parity. Zero or more individually eligible aliases remain in scope; direct child modules are `Unsupported` and duplicate eligible aliases are rejected as `DuplicateBinding`.
- **Tested traceability:** `IMPL-MODULE-SCOPED-SELF-SIMPLE-ORDINARY-FUNCTION-ALIASES` plus
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-ROOT-NESTED-VISIBILITY`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-IDENTITY-PROVENANCE`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-NO-EDGE-NO-FALSE-CYCLE`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-SHAPE-VISIBILITY-REJECTION`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-ATOMIC-VALID-SIBLING-FAILURE`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-SELF-SIMPLE-ALIAS-AUTHORITY-FENCE` are implemented/tested evidence.
- **Fingerprints:** planner `sha256:0e7131c8fa00458a6de421c6ef54e041d715df11b2ffca3af4fc8a01777e4025`; structural binder `sha256:a80da73c9c86b66237bcca59bb33b1494aa5f1bb1cce5e32d41fa29763518b76`; public exports `sha256:307975ee0f5da786a47068c0aef6ef00cc1e7d7f7674ca1ba774ec55711a303a`; focused test `sha256:1e9aa6317f2fdc44257bc04398d7a2c155d0261e1bc85c7715e94370f8a499f0`.
- **Record-mirrored next obligation:** TASK-2072 consumes this delivered non-authorizing Type handoff while completing parsed imports and atomic binding. TASK-2069 consumes only TASK-2073's complete checked handoff, and TASK-2064 owns integration parity.

## TASK-2071: Module Namespace and Provisional View Contract

- **Task:** [TASK-2071](tasks/TASK-2071-module-namespace-and-provisional-view-contract.md)
- **Canonical owner:** `SPEC-103`; `MOD-REAL-001`, `MOD-REAL-002`, `MOD-REAL-003`, and
  `MOD-REAL-004`; `SEM-MODULE-REALIZATION-001`, `SEM-MODULE-REALIZATION-002`,
  `SEM-MODULE-REALIZATION-003`, and `SEM-MODULE-REALIZATION-004`.
**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Layers:** type not_implemented; core not_applicable; cps not_applicable;
admission-runtime not_applicable; verification not_implemented.

**Run-route impact:** prerequisite.

**Missing target-spec clauses:** TASK-2071 completes the normative syntax-prepass, expansion, namespace, and provisional-view contract only. TASK-2074 must implement the AST-only syntax prepass and one-to-one `CanonicalExpandedModuleGraph`; TASK-2075 must implement `CanonicalCollectedModuleSnapshot` and the name-only `CanonicalProvisionalNameView`, including declaration visibility-carrier prerequisites, complete revalidation, file/inline normalized projection, and atomic failure. TASK-2072 must consume only the provisional name view for parsed import binding; TASK-2073 must consume the internal snapshot plus TASK-2072 staging for checked finalization. No production implementation or test, proof, lowering, admission, runtime, or parity evidence is supplied by this contract task.

- **Evidence detail:** none. The amended spec, task files, and plans are contract documents, not
  implementation, test, proof, or parity evidence.
- **Non-goals:** Rust carriers or behavior, binding, body checking, final interfaces, Core/CPS, Engine transport/admission/execution, and client parity.
- **Next obligation:** TASK-2074 is complete for its non-authorizing parser-stage handoff while the broader target remains partial/tested/below-spec. TASK-2075 is complete for its partial/tested/below-spec paired collection handoff; TASK-2072 consumes the name-only view, and TASK-2073 consumes the internal snapshot plus TASK-2072 staging.

## TASK-2074: Canonical Expanded Module Graph

- **Task:** [TASK-2074](tasks/TASK-2074-canonical-expanded-module-graph.md)
- **Canonical owner:** `SPEC-103`; `MOD-REAL-001` and `MOD-REAL-002`;
  `SEM-MODULE-REALIZATION-001` and `SEM-MODULE-REALIZATION-002`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Layers:** type partial; core not_applicable; cps not_applicable;
admission-runtime not_applicable; verification partial.

**Run-route impact:** prerequisite.

**Missing target-spec clauses:** TASK-2074's owned parser-stage handoff is complete: the public `CanonicalExpandedModuleGraph` consumes the acquired parsed graph, performs the AST-only macro/notation prepass with deterministic provider-before-consumer ordering and combined cycle rejection, transports and activates exact public notation summaries only in their consumers, preserves typed keys/order/provenance/sidecars, and publishes atomically. The completion audit combines 17/17 syntax-prepass, 5/5 shallow-graph, 8/8 expanded-graph completion, 21/21 notation-import, 14/14 notation-selector parser, and 37/37 legacy Engine-fence tests. Malformed selectors reject in the parser at their first invalid byte; they never become graph requests, so the unreachable graph `MalformedPattern` kind is removed. Legacy Engine compatibility paths reject live notation imports before lookup, binding, activation, export publication, cache mutation, or cycle-state mutation, including restricted visibility, versioned paths, multiline selectors, comment punctuation, and string/comment lookalikes; this is a non-authorizing fence, not Engine support. The broader MOD-REAL-001/002 target remains `partial / tested / below_spec`: TASK-2075 collection, TASK-2072 binding, TASK-2073 checked finalization, TASK-2069 lowering/transport, TASK-2063 admission, and TASK-2064 client parity remain independently owned and absent.

- **Tested traceability:** `IMPL-MODULE-CANONICAL-EXPANDED-GRAPH`,
  `IMPL-MODULE-CANONICAL-SYNTAX-PREPASS`, `IMPL-MODULE-SHALLOW-BODY-EXPANSION`,
  `IMPL-MODULE-NOTATION-IMPORT-PARSER`, `IMPL-MODULE-STRUCTURED-NOTATION-PATTERN-KEY`,
  `IMPL-MODULE-CANONICAL-NOTATION-SUMMARY-CARRIER`,
  `IMPL-MODULE-CANONICAL-NOTATION-IMPORT`,
  `IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION`,
  `TEST-MOD-REAL-001-002-NOTATION-IMPORT-PARSER`,
  `TEST-MOD-REAL-001-002-MALFORMED-NOTATION-SELECTOR-ANCHORS`,
  `TEST-MOD-REAL-001-002-NOTATION-IMPORT-RESOLVER-FENCE`,
  `TEST-MOD-REAL-001-002-TYPED-NOTATION-KEY`,
  `TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY`,
  `TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION`,
  `TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION`,
  `IMPL-ENGINE-LEGACY-NOTATION-IMPORT-FENCE`,
  `TEST-MOD-REAL-001-002-LEGACY-ENGINE-NOTATION-FENCE`,
  `TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION`,
  `TEST-MOD-REAL-001-002-LOCAL-SHALLOW-ORDER`,
  `TEST-MOD-REAL-001-002-INLINE-SIDECAR-OWNERSHIP`,
  `TEST-MOD-REAL-001-002-EXACT-KEY-ATOMIC-PUBLICATION`,
  `TEST-MOD-REAL-001-002-GENERATED-SHALLOW-ORDER-PROPERTY`,
  `TEST-MOD-REAL-001-002-ANCHORED-LATE-EXPANSION-FAILURE`,
  `TEST-MOD-REAL-001-002-MISSING-DEFINITION-CARDINALITY`,
  `TEST-MOD-REAL-001-002-EXTRA-DEFINITION-CARDINALITY`,
  `TEST-MOD-REAL-001-002-LOCAL-PUBLIC-MACRO`,
  `TEST-MOD-REAL-001-002-CANONICAL-PUBLIC-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-ORDER`,
  `TEST-MOD-REAL-001-002-TRANSITIVE-PROVIDER-CLOSURE`,
  `TEST-MOD-REAL-001-002-SYNTAX-IMPORT-PROVENANCE`,
  `TEST-MOD-REAL-001-002-MACRO-NAMESPACE-PRIORITY`,
  `TEST-MOD-REAL-001-002-PUBLIC-MACRO-ALIAS-PROPERTY`,
  `TEST-MOD-REAL-001-002-PRIVATE-MACRO`,
  `TEST-MOD-REAL-001-002-PRIVATE-STRUCTURAL-PATH`,
  `TEST-MOD-REAL-001-002-NON-MACRO-SYNTAX-IMPORT`,
  `TEST-MOD-REAL-001-002-MISSING-MACRO-SUMMARY`,
  `TEST-MOD-REAL-001-002-DUPLICATE-MACRO-ALIAS`,
  `TEST-MOD-REAL-001-002-PROVIDER-OWNED-DIAGNOSTIC`,
  `TEST-MOD-REAL-001-002-NOTATION-NONLEAKAGE`,
  `TEST-MOD-REAL-001-002-ITEM-GENERATION-REJECTION`,
  `TEST-MOD-REAL-001-002-TWO-MODULE-SYNTAX-CYCLE`, and
  `TEST-MOD-REAL-001-002-THREE-MODULE-SYNTAX-CYCLE` cover only the delivered bounded expansion
  slice.
- **Approved completion checkpoint:**
  `TEST-MOD-REAL-001-002-EXPANDED-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-001-002-ACQUIRED-GRAPH-NO-REREAD`,
  `TEST-MOD-REAL-001-002-ALIAS-PROVIDER-TEMPLATE-MUTATION`,
  `TEST-MOD-REAL-001-002-CALLABLE-IMPORT-NOTATION-NONACTIVATION`,
  `TEST-MOD-REAL-001-002-ATOMIC-NONMACRO-SYNTAX-EDGE`,
  `TEST-MOD-REAL-001-002-DIRECT-ORCHESTRATION-MANIFEST-FENCE`, and
  `TEST-MOD-REAL-001-002-EXPANDED-PROJECTION-64-CASE-PROPERTY` passed in the 8/8 completion
  target. They are bounded parser-stage evidence, not canonical public notation-summary transport
  or activation and not final-interface, lowered/admitted/runtime, or client parity.
- **Valid notation-summary carrier:** `IMPL-MODULE-CANONICAL-NOTATION-SUMMARY-CARRIER` and
  `TEST-MOD-REAL-001-002-CANONICAL-NOTATION-SUMMARY` are implemented/tested by the 3/3 focused
  target. They exact-match typed token/hole selectors and retain every public full-fixity variant,
  callable target, visibility, and provider/use provenance in read-only expanded records without
  raw-text matching, ordinary binding, authority, or activation.
- **Notation-import activation:** invalid dependency handling in
  `IMPL-MODULE-CANONICAL-NOTATION-IMPORT` and consumer-local activation in
  `IMPL-MODULE-IMPORTED-NOTATION-ACTIVATION` are implemented/tested by
  `TEST-MOD-REAL-001-002-NOTATION-DEPENDENCY-REJECTION` and
  `TEST-MOD-REAL-001-002-IMPORTED-NOTATION-ACTIVATION`. The complete
  `TEST-MOD-REAL-001-002-EXPANDED-GRAPH-COMPLETION` is tested after the independent whole-task
  audit.
  The selected contract parses `use crate::math::(<*>);` and
  `use crate::ranges::(_ between _ and _);` as exact structured normalized token/hole selectors.
  Raw pattern spelling is diagnostic-only; matching may not reparse or scan it. The selector does
  not encode fixity, associativity, or precedence, and transports every eligible public full-key
  variant deterministically. Provider export is direct `pub` notation declaration only; plain
  inherited `use module::(pattern)` imports it, while visibly qualified notation uses, including
  `pub use`, reject because notation re-export has no TASK-2074 contract. It has no `as` form or
  notation glob, never binds or authorizes the retained callable target, and fails atomically for
  missing/private/conflicting/cyclic dependencies with the applicable use/declaration/cycle
  anchors. Malformed selectors fail earlier at exact parser-owned anchors and have no graph-level
  failure kind. Activation targets the existing
  syntax-phase table and preserves hole order; generalized mixfix use-site parsing/elaboration is
  not a TASK-2074 claim.
- **Compatibility-fence audit:** the 14/14 selector parser target owns exact malformed-selector
  anchors and the 37/37 Engine module-loader target proves explicit fail-closed rejection before
  lookup/binding/activation/publication or cache/cycle-state mutation. Restricted visibility,
  versioned, multiline, comments, and string lookalikes are covered. This is a non-authorizing
  legacy fence, not Engine notation support.
- **Observed totals:** syntax-prepass target 17/17 and shallow-graph target 5/5, each with an exact
  16-case property; `ash-parser` library 463/463; eight named predecessor regression targets 56/56
  in aggregate (6 + 8 + 7 + 6 + 6 + 3 + 8 + 12); macro summary/identity regressions 6/6 (2 + 4).
  Exact commands and source fingerprints are recorded in the task evidence section.
- **Proof/parity:** no proof. The normalized file/inline child projection is parser-stage test
  evidence only, not a final-interface, lowered/admitted/runtime, or client parity relation.
- **Non-goals:** Namespace collection, provisional views, general import binding, body/type checking, final interfaces, Core/CPS lowering, Engine transport/admission/execution, filesystem discovery, source-text fallback, and client parity.
- **Next obligation:** TASK-2075 may now consume the completed non-authorizing expanded graph and must independently implement the internal snapshot and name-only provisional view. The broader module-realization rule remains partial and below spec until the downstream owners complete their layers.

## TASK-2075: Two-Tier Complete Module Collection

- **Task:** [TASK-2075](tasks/TASK-2075-two-tier-complete-module-collection.md)
- **Canonical owner:** `SPEC-103`; `MOD-REAL-003` and `MOD-REAL-004`;
  `SEM-MODULE-REALIZATION-003` and `SEM-MODULE-REALIZATION-004`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Layers:** type partial; core not_applicable; cps not_applicable;
admission-runtime not_applicable; verification partial.

**Run-route impact:** prerequisite.

**Missing target-spec clauses:** Visibility/carrier prerequisites plus Tasks 5–8 graph-wide atomic paired collection, internal-fact/minimal-view retention, keyed/span-anchored drift revalidation, normalized file/inline Type-layer projection, generated/property coverage, bounded TASK-2068/TASK-2070 compatibility, and the complete later-layer authority fence are implemented and tested. Internal entries retain expanded raw definitions, callable bodies, nested member spans through direct source anchors, deterministic ordinals, and module-owned expansion/hygiene sidecars; the provisional view remains the exact name/identity/namespace/visibility/exportability/origin-anchor/ordinal subset. Impl coherence remains bounded to interfaces found in the current module or lexical canonical-module ancestors; imported interface binding fails closed until TASK-2072 supplies the defining identity. TASK-2072 and TASK-2073 have non-authorizing carrier inputs but still own binding and finalization.

- **Delivered Task 8 traceability:** `TEST-MOD-REAL-003-004-COLLECTION-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-003-004-COLLECTION-GENERATED-PROPERTY`,
  `TEST-MOD-REAL-003-004-COLLECTION-COMPATIBILITY`, and
  `TEST-MOD-REAL-003-004-COLLECTION-AUTHORITY-FENCE` are tested witnesses. They establish
  normalized Type-layer collection equivalence, generated/property coverage, bounded compatibility,
  and a complete later-layer authority fence; they do not claim final-interface, imported-binding,
  Core/CPS, admission/runtime, proof, or client parity authority.
- **Delivered traceability:** `IMPL-MODULE-COLLECTION-VISIBILITY-CARRIERS`,
  `IMPL-MODULE-COLLECTION-VISIBILITY-PARSER`, and
  `IMPL-MODULE-CANONICAL-COLLECTION-CARRIER-BOUNDARY`, and
  `IMPL-MODULE-CANONICAL-TWO-TIER-COLLECTION`, and
  `IMPL-MODULE-COLLECTION-INTERNAL-SOURCE-ANCHOR` and
  `IMPL-MODULE-COLLECTION-REVALIDATION` are implemented. Positive evidence is
  `TEST-MOD-REAL-003-004-VISIBILITY-CARRIER-CONSTRUCTION`,
  `TEST-MOD-REAL-004-INHERITED-DECLARATION-SPANS`,
  `TEST-MOD-REAL-004-EXPLICIT-DECLARATION-SPANS`,
  `TEST-MOD-REAL-004-NESTED-INHERITED-SCOPING`,
  `TEST-MOD-REAL-003-004-COLLECTION-DOMAIN`,
  `TEST-MOD-REAL-003-004-COLLECTION-PRIVATE-SOURCE-FENCE`, and
  `TEST-MOD-REAL-003-004-COLLECTION-ADVERSARIAL-SOURCE-FENCE`,
  `TEST-MOD-REAL-003-004-COLLECTION-CARRIER-SHAPE`,
  `TEST-MOD-REAL-003-004-COLLECTION-NAMESPACE-COLLISION-MEMBERS`,
  `TEST-MOD-REAL-003-004-COLLECTION-IMPL-COHERENCE`, and
  `TEST-MOD-REAL-003-004-COLLECTION-ATOMICITY`,
  `TEST-MOD-REAL-003-004-COLLECTION-INLINE-SIDECARS-RAW-FACTS`,
  `TEST-MOD-REAL-003-004-COLLECTION-NESTED-MEMBER-RAW-FACTS`, and
  `TEST-MOD-REAL-003-004-COLLECTION-STRICT-PROVISIONAL-VIEW`. Negative evidence is
  `TEST-MOD-REAL-004-NESTED-VISIBLE-REJECTION` and
  `TEST-MOD-REAL-003-004-PRIVATE-CAPABILITY-ATOMICITY` and
  `TEST-MOD-REAL-003-004-COLLECTION-UNRESOLVED-INTERFACE`. Input-mutation evidence is
  `TEST-MOD-REAL-004-VISIBILITY-FORM-MUTATION`, and the eight keyed/span-anchored
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-*` mutations, which cover name, kind, visibility,
  signature, body, source order, expansion sidecars, and changed-sibling atomicity. Parity is
  `not_applicable`; no parity or proof witness exists.
- **Delivered Task 5 checkpoint:**
  `crates/ash-typeck/tests/task_2075_two_tier_module_collection.rs` defines the approved 22-row
  declaration domain, exact private/read-only carrier boundary, complete Task 5 namespace and
  parent/member classification, typed notation identity, full impl-head coherence, and graph-wide
  atomic paired publication. The Task 5 target passed 22/22;
  the private validator tests remain 2/2.
- **Delivered Task 6 checkpoint:** internal entries now retain raw declaration/callable facts,
  bodies, nested member spans, source anchors/ordinals, and module-local expansion/hygiene
  sidecars, while the exact provisional view remains name-only and non-authorizing. The current
  full focused target passes 24/24 and is required-success verification. This is test evidence,
  not proof, normalized collected file/inline parity, generated/property, compatibility, drift,
  imported-interface binding, or later-layer authority evidence.
- **Delivered Task 7 checkpoint:** `CanonicalModuleCollection::revalidate_against` rebuilds the
  candidate collection and rejects keyed/span-anchored `SourceDrift` before replacement
  publication. The eight mutation/atomicity cases
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-NAME`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-KIND`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-VISIBILITY`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-SIGNATURE`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-BODY`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-ORDER`,
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-SIDECAR`, and
  `TEST-MOD-REAL-003-004-COLLECTION-SOURCEDRIFT-SIBLING-ATOMICITY` pass as part of the
  required-success 36/36 focused target; the exact private carrier fence remains green. This is
  test evidence, not proof, imported-interface/final-interface authority, Core/CPS, admission,
  runtime, or client parity evidence.
- **Delivered Task 8 checkpoint:** the focused target passes 36/36. Its normalized projection
  compares equivalent file-backed and inline collections using canonical Type-layer identity,
  lookup, namespace, visibility, and exportability facts while excluding source-layout spans,
  paths, raw payloads, ordinals, and expansion sidecars. The 32-case generated witness varies
  source form, visibility, namespace/collision shape, parent/member placement, source order, and
  supported definition forms. Compatibility and authority-fence witnesses keep TASK-2068/TASK-2070
  routes bounded and exclude downstream binding, checked/final-interface, Core/CPS, Engine,
  admission, and runtime carriers/routes.
- **Non-goals:** No syntax expansion ownership, parsed general import binding, body/type checking, final public or private interface, export closure, Core/CPS, Engine transport/admission/execution, or client parity.
**Handoff status:** Complete for the task-owned non-authorizing paired collection handoff at
  `partial / tested / below_spec`. Task 9 review, focused quality gates, and handoff documentation
  are complete. TASK-2072 consumes only the name-only view; TASK-2073 consumes the internal snapshot
  plus staged bindings. No final-interface, Core/CPS, admission/runtime, or client-parity authority
  is added here. The workspace-wide clippy/test gate remains blocked by the pre-existing TASK-2063
  missing linked-module admission APIs.
**Next obligation:** The TASK-2075 task-owned paired collection handoff is complete at `partial / tested / below_spec`; TASK-2072 consumes only the name-only view, TASK-2073 consumes the internal snapshot plus staged bindings, and no final-interface, Core/CPS, admission/runtime, or client-parity authority is added here.

## TASK-2072: Parsed Import Resolution and Atomic Binding

- **Task:** [TASK-2072](tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md)
- **Status:** In progress / partial / tested / below_spec. It consumes only TASK-2075's name-only view and owns all parsed import grammar,
  structural traversal, visibility, precedence, ambiguity/duplicate/cycle rejection, and atomic
  M-IMPORT-EDGE/M-IMPORT-CYCLE/M-BIND publication, including staged `pub use` facts. TASK-2073
  alone finalizes those staged facts into export closure; this task creates no final interface or
  later-layer authority.
- **Canonical rules:** `SEM-MODULE-REALIZATION-004`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** The implemented Type-layer slice resolves the admitted parsed
use/path/visibility/alias/re-export forms against TASK-2075's name-only
`CanonicalProvisionalNameView`, stages canonical bindings/edges and non-authorizing `pub use`
facts, applies deterministic local/explicit/glob precedence and ambiguity/duplicate rules,
preserves defining identity, namespace, provenance, spans, visibility, and source order, and
rejects graph mismatches, inaccessible paths, unsupported shapes, and complete ordinary/public
re-export cycles before publication. Notation dependency edges and syntax-prepass cycle authority
remain the TASK-2074 parser-stage handoff; this resolver transports notation facts without
duplicating that graph authority. Checked bodies, final export closure, complete interface
finalization, Core/CPS, Engine admission/runtime, and client parity remain downstream. The target
rule remains `partial / tested / below_spec`.
Notation dependency edges and syntax-prepass cycle authority remain the TASK-2074 parser-stage handoff; this resolver transports notation facts without duplicating that graph authority. Checked bodies, final export closure, complete interface finalization, Core/CPS, Engine admission/runtime, and client parity remain downstream.
- **Delivered evidence:** The focused `crates/ash-typeck/tests/task_2072_parsed_import_resolution.rs`
  target passes 21/21 and its focused clippy target passes with `-D warnings`. Positive witnesses
  cover admitted grammar/path families, identity/provenance, notation summaries, visibility,
  public-use staging, transitive re-exports, and normalized file/inline projection. Negative
  witnesses cover inaccessible paths, namespace ambiguity, duplicate bindings, and ordinary/public
  re-export cycles. Mutation/property witnesses cover graph-key mismatch and generated visibility
  and grammar/anchor preservation. These tests are not proof or final-interface/runtime/client
  parity evidence.
- **Evidence IDs:** `TEST-MOD-REAL-004-TASK-2072-GRAMMAR-PROVENANCE`,
  `TEST-MOD-REAL-004-TASK-2072-MODULE-ALIAS`, `TEST-MOD-REAL-004-TASK-2072-NOTATION`,
  `TEST-MOD-REAL-004-TASK-2072-VISIBILITY`, `TEST-MOD-REAL-004-TASK-2072-LEXICAL-PATHS`,
  `TEST-MOD-REAL-004-TASK-2072-TRANSITIVE-REEXPORT`, `TEST-MOD-REAL-004-TASK-2072-PUBLIC-USE`,
  `TEST-MOD-REAL-004-TASK-2072-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-TASK-2072-LOCAL-PRECEDENCE`,
  `TEST-MOD-REAL-004-TASK-2072-GENERATED-VISIBILITY`,
  `TEST-MOD-REAL-004-TASK-2072-GENERATED-GRAMMAR`,
  `TEST-MOD-REAL-004-TASK-2072-AUTHORITY-FENCE`,
  `TEST-MOD-REAL-004-TASK-2072-PARENT-SCOPED-REJECTION`,
  `TEST-MOD-REAL-004-TASK-2072-NAMESPACE-AMBIGUITY`,
  `TEST-MOD-REAL-004-TASK-2072-INACCESSIBLE-LEXICAL-SHADOW`,
  `TEST-MOD-REAL-004-TASK-2072-DUPLICATE-ATOMICITY`,
  `TEST-MOD-REAL-004-TASK-2072-ORDINARY-CYCLE`,
  `TEST-MOD-REAL-004-TASK-2072-PUBLIC-REEXPORT-CYCLE`,
  `TEST-MOD-REAL-004-TASK-2072-EMPTY-GROUP`, and
  `TEST-MOD-REAL-004-TASK-2072-GRAPH-MISMATCH`.
- **Layer statuses:** type partial; core not_applicable; cps not_applicable; admission-runtime not_applicable; verification partial.
- **Additional bounded type-function closure evidence:** The focused target now passes 62/62. Public
  equation constructor patterns and proposition tails retain checked metadata; private pattern
  constructors and private named predicates reject atomically. Evidence is
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PATTERN-PRIVATE-DEPENDENCY`,
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-PRIVATE-DEPENDENCY`, and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-TAIL-PROJECTION`.
- **Additional bounded callable proposition-tail closure evidence:** The focused target now passes
  68/68. Public callable tails retain public predicate dependencies; private local/imported tail
  predicates, types, and effect-row groups reject atomically. Evidence is
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PUBLIC-DEPENDENCY`,
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-DEPENDENCY`,
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-TYPE`, and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-ROW`, and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-UNQUALIFIED-ROW`, and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-IMPORTED-PRIVATE-DEPENDENCY`.
- **Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
- **Non-goals:** This task excludes provisional collection ownership, checked bodies, final interface/export closure, Core/CPS, Engine transport/admission/runtime, execution, and client parity.
- **Handoff status:** Complete for the task-owned non-authorizing parsed-import/binding handoff at
  `partial / tested / below_spec`. Task 9 review, focused quality gates, and handoff documentation
  are complete. TASK-2073 consumes the staged bindings for finalization; TASK-2074 retains
  notation-edge/cycle authority; TASK-2064 owns composed parity. No final-interface, Core/CPS,
  admission/runtime, or client-parity authority is added here. The workspace-wide clippy/test gate
  remains blocked by the pre-existing TASK-2063 missing linked-module admission APIs.
**Next obligation:** The TASK-2072 task-owned parsed-import/binding handoff is complete at `partial / tested / below_spec`; TASK-2073 consumes its staged bindings for finalization, while TASK-2074 retains notation-edge/cycle authority and TASK-2064 owns parity. No final-interface, Core/CPS, admission/runtime, or client-parity authority is added here.

## TASK-2073: Checked Module Finalization and Export Closure

- **Task:** [TASK-2073](tasks/TASK-2073-checked-module-finalization-and-export-closure.md)
- **Status:** In progress / partial / tested / below_spec. It consumes TASK-2075's internal snapshot plus TASK-2072 staging and owns complete M-CHECK bodies,
  private/public facts, final `pub use` projection, export closure, atomic finalization, and
  Type-layer file/inline final-interface parity. TASK-2069 exclusively consumes its complete
  checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns executed/client parity.
- **Canonical rules:** `SEM-MODULE-REALIZATION-003`
- **Current focused target:** `crates/ash-typeck/tests/task_2073_checked_module_finalization.rs` passes
  102/102; the Type-layer slice remains `partial / tested / below_spec`.
- **Imported type-path closure:** Public imported type-bearing dependencies and callable signatures
  now require publicly reachable defining module paths. Root and fully public paths remain accepted;
  private, crate-only, and restricted enclosing paths reject atomically. Positive evidence is
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-PUBLIC-MODULE-PATH`; negative evidence is
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-PRIVATE-MODULE-PATH` and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-TYPE-PRIVATE-MODULE-PATH`.
- **Imported namespace-path closure:** Public imported namespace dependencies now require publicly
  reachable defining module paths. Positive role-row evidence is
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PUBLIC-MODULE-PATH`; the private-path negative
  is `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PRIVATE-MODULE-PATH`. Roles remain minimum,
  non-authorizing metadata. Policy-row and notation controls add positive
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PUBLIC-MODULE-PATH` and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PUBLIC-MODULE-PATH` plus negative
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PRIVATE-MODULE-PATH` and
  `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PRIVATE-MODULE-PATH`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** The delivered Type-only slice checks ordinary and bodyless builtin callable signatures, canonical handler body facts, public ordinary types, nominal newtypes, resource schemas, public interface method metadata, sealed-domain facts and parent-scoped marker constructors, export-closed effect-row alias/group metadata with private and missing unqualified plus qualified row-path dependency rejection, promoted data-kind/proposition-predicate metadata with private and missing source-ADT dependency rejection, public role metadata, bounded type-function metadata with private type/function dependency rejection plus private equation-pattern-constructor and proposition-tail dependency rejection, public callable proposition-tail type/predicate/row dependency rejection, export-closed notation metadata with private, qualified-target, and missing target dependency rejection, parser-owned public macro summaries with syntax-only metadata and typed-signature dependency rejection plus imported private template-callable dependency rejection, checked public module-law evidence metadata with private parameter-type dependency rejection plus imported private evidence-callable dependency rejection, parent-scoped interface-law/implementation-proof fact matching with explicit checked nested kind/visibility summaries, and parser-carried public policy schema metadata with missing and private field-type dependency rejection plus checked default/invariant expressions and imported private value-callable dependency rejection, minimal named policy binding transport, and body-free public implementation summaries with private implementation dependency rejection. Rich policy-instance, persistence, inheritance, authority, and runtime semantics, remaining namespace forms, complete visibility/export closure, forged/cyclic dependency coverage, downstream Core/CPS/admission-runtime, and client parity remain incomplete. It retains private/public callable and namespace facts and origins, rejects unsupported public namespace facts before publication, validates staged `pub use` identity/origin, rejects missing or private public type-bearing dependencies for declarations and callable signatures, private signature/type dependencies, and private imported row, promoted-kind, notation, macro-template, evidence-expression, and policy-expression callable dependencies, revalidates collection drift, and tests normalized file/inline interface projection.
- **Additional delivered clause:** Public interface-law propositions apply callable export closure to local and imported dependencies while preserving parent-scoped interface methods. Qualified implementation calls in public evidence, policy, and macro expressions use the implementation-registry visibility boundary without turning implementation members into standalone exports.
- **Imported declaration metadata clause:** Staged imports carry defining declaration spans and source ordinals; finalization revalidates both against the checked declaration and canonical identity origin before imported type collection or interface publication. Import-path visibility remains parser-owned metadata and is not conflated with declaration visibility.
- **Qualified implementation-call closure:** This Type-layer slice is included in the delivered evidence; downstream lowering, admission, runtime, and client parity remain outside TASK-2073.
- **Implementation where-bound closure:** Public implementation `where T: Interface` bounds use the interface namespace visibility boundary, rejecting local-private, imported-private, and missing bounds before publication while retaining public bounds as non-authorizing summary metadata.
- **Qualified namespace dependency closure:** Public qualified row-group paths and qualified notation callable targets now use staged canonical module identities and enclosing declaration visibility, rejecting private targets before publication while preserving public targets. This remains Type-layer, non-authorizing evidence.
- **Transitive row dependency closure:** Public effect-row aliases/groups now follow staged local, imported, and qualified row-carrier dependencies transitively, rejecting private transitive leaves and cycles before publication. Bare unresolved whole-row variables remain checker-owned rather than becoming namespace authority.
- **Callable whole-row closure:** Public callable proposition tails now apply staged visibility checks to bare named row items and unqualified single-segment operation items while leaving unresolved row variables checker-owned. Negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-UNQUALIFIED-ROW` covers the local private case.
- **Role/policy-row closure:** Public effect-row role and policy items now apply staged local, imported, and qualified visibility checks while retaining roles as minimum metadata and policies as transient schema-only metadata.
- **Qualified implementation-operation row closure:** Public effect-row `Impl::operation` items validate a resolved local implementation-registry declaration and its parent-scoped callable member. Private implementations reject atomically; public implementations preserve the row as non-authorizing metadata. Unknown/resource operation rows such as `PosixFs::read` remain owned by their existing checker and are not reclassified as implementations.
- **Forged imported implementation-operation closure:** A shape-consistent imported implementation carrier also revalidates the defining module path before a public effect row can preserve the parent-scoped operation. Fully public paths remain accepted; private enclosing paths reject atomically. This is defensive Type-layer validation and does not publish implementations into the provisional name view.
- Public effect-row `Impl::operation` items validate resolved local implementation-registry visibility and parent-scoped operation identity; unknown/resource operation rows remain checker-owned non-authorizing metadata.
- **Binding metadata revalidation:** Imported declaration visibility is checked against the acquired checked target; same-identity visibility drift rejects atomically before interface publication with `BindingVisibilityMismatch`. Defining module paths are revalidated through the canonical provisional module scopes, so a forged binding cannot cross a private enclosing module boundary. A forged imported lookup namespace or declaration kind is rejected at the same boundary with `BindingShapeMismatch`, a forged local alias is rejected against the authoritative import-map key with `BindingLocalNameMismatch`, and forged declaration span/source-order metadata is rejected with `BindingDeclarationMetadataMismatch` before imported type collection or interface publication.
- **Public re-export path closure:** Staged `pub use` projection now walks the checker-owned structural module declarations for the defining identity and rejects any private, crate-only, or restricted enclosing module path before export projection; a fully public path remains accepted. This is distinct from ordinary importer access and enforces SPEC-103 §§6–7 public reachability.
- **Public re-export path evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-MODULE-PATH-CLOSURE` preserves a staged re-export through a fully public defining path; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-PRIVATE-MODULE-PATH` rejects a public re-export through a private enclosing module before projection.
- **Public re-export boundary evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NARROW-REEXPORT-EXCLUSION` and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NARROW-REEXPORT-PROMOTION` prove direct and transitive `pub(crate)`, `pub(super)`, and restricted staged re-exports remain outside the external projection; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NESTED-PATH-DIAGNOSTIC`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NESTED-PRIVATE-PATH`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NESTED-PUB-CRATE-PATH`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NESTED-PUB-SUPER-PATH`, and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-NESTED-RESTRICTED-PATH` cover diagnostic context and non-public defining paths.
- **Updated focused target:** The qualified namespace, implementation-call, qualified implementation-operation, where-bound, transitive-row, callable-whole-row, role-row, policy-row, binding-metadata, forged-binding-shape, forged-local-name, forged declaration-metadata, forged imported implementation-operation, row-carrier shape precedence, and staged public-use carrier RED/GREEN slices leave the focused TASK-2073 integration target at 102/102, plus twenty-four dedicated finalizer unit witnesses.
- The finalizer source contains twenty-four `#[test]` unit witnesses plus one shared fixture helper; the helper is not counted as a witness.
- **Qualified implementation-call evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PUBLIC-QUALIFIED-IMPL-CALL`; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PRIVATE-QUALIFIED-IMPL-CALL`. The operation remains parent-scoped and the visibility check consults only the non-authorizing implementation registry.
- **Where-bound evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PUBLIC-WHERE-BOUND`; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PRIVATE-WHERE-BOUND`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-IMPORTED-PRIVATE-WHERE-BOUND`, and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-MISSING-WHERE-BOUND`. These tests establish Type-layer visibility closure only; they do not add interface authority or runtime semantics.
- **Qualified namespace evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-ROW-PUBLIC-DEPENDENCY` and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-NOTATION-PUBLIC-DEPENDENCY`; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-ROW-PRIVATE-DEPENDENCY` and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-NOTATION-PRIVATE-DEPENDENCY`. These tests establish staged Type-layer visibility closure only.
- **Transitive row evidence:** Negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-TRANSITIVE-PRIVATE-DEPENDENCY` and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-CYCLIC-DEPENDENCY` reject private transitive leaves and public row cycles atomically; the imported-row witness exercises the bare whole-row spelling as well as explicit groups. These tests establish Type-layer closure only.
- **Role/policy-row evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PUBLIC-DEPENDENCY` and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PUBLIC-DEPENDENCY`; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-ROLE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-POLICY-DEPENDENCY`, and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PRIVATE-DEPENDENCY` reject private local/imported role and policy paths atomically. Roles remain minimum metadata and policies remain transient schema-only metadata; no authority, persistence, inheritance, or runtime semantics are added.
- **Qualified implementation-operation evidence:** Positive `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PUBLIC-QUALIFIED-IMPL-OPERATION` and `TEST-MOD-REAL-003-TASK-2073-FORGED-IMPORTED-IMPL-OPERATION-PUBLIC-MODULE-PATH`; negative `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-QUALIFIED-IMPL-OPERATION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-MISSING-QUALIFIED-IMPL-OPERATION`, and `TEST-MOD-REAL-003-TASK-2073-FORGED-IMPORTED-IMPL-OPERATION-PRIVATE-MODULE-PATH`. The implementation registry is consulted only for a resolved implementation qualifier; parent-scoped methods are not exported as standalone callables, and unknown/resource rows retain their existing non-authorizing owner.
- **Binding metadata evidence:** Mutation `TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-VISIBILITY-DRIFT` proves same-identity imported visibility drift is rejected before publication. Mutation `TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-MODULE-VISIBILITY-DRIFT` proves a defining identity cannot cross a private enclosing module path, while finalizer visibility-boundary witnesses preserve parent-owned private, `pub(self)`, `pub(super)`, `pub(crate)`, and restricted structural visibility semantics. Mutation `TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-SHAPE-MISMATCH` proves a forged imported namespace/kind carrier is rejected before publication, and `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-IMPORTED-BINDING-SHAPE-PRECEDENCE` proves shape rejection precedes public-row module-path diagnostics. Mutation `TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-LOCAL-NAME-DRIFT` proves a forged local alias is rejected against the authoritative import-map key before publication. Mutation `TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-DECLARATION-METADATA-DRIFT` proves forged declaration span and defining source ordinal carriers are rejected before imported type collection or interface publication. Mutation `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-BINDING-CARRIER-DRIFT` proves a duplicated staged public-use carrier must retain re-export status and exact equality with the authoritative binding before export projection; `TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-BINDING-METADATA-DRIFT` independently reaches the exact-equality check with re-export status preserved. This is Type-layer mutation evidence only.
- **Additional evidence inventory:** `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-TRANSITIVE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-CYCLIC-DEPENDENCY`.
- **Activation and implementation evidence:** `crates/ash-typeck/tests/task_2073_checked_module_finalization.rs` passes 102/102, and the existing binding/visibility/carrier witnesses plus `canonical_checked_module_finalizer::tests::public_use_nested_private_path_diagnostic_preserves_access_context`, `canonical_checked_module_finalizer::tests::public_use_projection_excludes_narrow_reexports`, `canonical_checked_module_finalizer::tests::public_use_nested_private_module_path_rejects`, `canonical_checked_module_finalizer::tests::public_use_nested_pub_crate_module_path_rejects`, `canonical_checked_module_finalizer::tests::public_use_nested_pub_super_module_path_rejects`, `canonical_checked_module_finalizer::tests::public_use_nested_restricted_to_allowed_module_path_rejects`, `canonical_checked_module_finalizer::tests::imported_impl_operation_private_defining_module_path_rejects_atomically`, `canonical_checked_module_finalizer::tests::imported_impl_operation_public_defining_module_path_preserves_closure`, and `canonical_checked_module_finalizer::tests::forged_imported_effect_row_binding_shape_rejects_before_module_path_diagnostic` pass as dedicated unit witnesses. Existing positive and negative evidence remains as previously recorded; these are tests, not proof or downstream execution/client parity.
- **Evidence inventory:** `TEST-MOD-REAL-003-TASK-2073-CHECKED-PRIVATE-PUBLIC`, `TEST-MOD-REAL-003-TASK-2073-FINAL-PUB-USE`, `TEST-MOD-REAL-003-TASK-2073-GENERATED-CLOSURE-PROPERTY`, `TEST-MOD-REAL-003-TASK-2073-FILE-INLINE-FINAL-PARITY`, `TEST-MOD-REAL-003-TASK-2073-BUILTIN-PUBLIC-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-HANDLER-CHECKED-BODY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-SIGNATURE`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-NEWTYPE-SIGNATURE`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PUBLIC-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-DOMAIN-RESOURCE-NAMESPACE`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-SEALED-DOMAIN-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-DATA-KIND-PREDICATE-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-ROLE-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-TAIL-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-SUMMARY-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-IMPL-PROOF-LAW-PAIR`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-PROJECTION`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPLEMENTATION-SUMMARY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-NESTED-EVIDENCE-VISIBILITY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PROOF-VISIBILITY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-PUBLIC-CALLABLE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PUBLIC-QUALIFIED-IMPL-CALL`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PUBLIC-WHERE-BOUND`, `TEST-MOD-REAL-003-TASK-2073-EXPORT-CLOSURE-REJECTION`, `TEST-MOD-REAL-003-TASK-2073-BUILTIN-PRIVATE-SIGNATURE`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-FUNCTION-MISSING-SIGNATURE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-BUILTIN-MISSING-SIGNATURE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-HANDLER-MISSING-SIGNATURE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-TYPE`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-ROW`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-IMPORTED-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-PRIVATE-CALLABLE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-IMPORTED-PRIVATE-CALLABLE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROW-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-DATA-KIND-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-NEWTYPE-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-RESOURCE-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-SEALED-DOMAIN-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-DATA-KIND-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-PREDICATE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PATTERN-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-MISSING-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-TYPED-SIGNATURE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-FIELD-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-MISSING-FIELD-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-DEFAULT-TYPE-MISMATCH`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-INVARIANT-NOT-BOOL`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPLEMENTATION-PRIVATE-DEPENDENCY`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PRIVATE-QUALIFIED-IMPL-CALL`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PRIVATE-WHERE-BOUND`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-IMPORTED-PRIVATE-WHERE-BOUND`, `TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-MISSING-WHERE-BOUND`, `TEST-MOD-REAL-003-TASK-2073-AUTHORITY-FENCE`, `TEST-MOD-REAL-003-TASK-2073-STALE-ATOMICITY`.
- **Layer statuses:** type partial; core not_applicable; cps not_applicable; admission-runtime not_applicable; verification partial.
- **Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
- **Non-goals:** No parser acquisition/graph construction, import grammar/binding ownership, Core/CPS lowering, Engine transport/link/admission/execution, direct evaluation, or CLI/daemon terminal parity.
- **Next obligation:** Extend the bounded finalizer to remaining declaration facts while keeping named policy bindings deliberately transient and minimal (local alias, defining identity, policy namespace, provenance, and public schema only), and complete remaining forged/incomplete/cyclic dependency and visibility/export-closure rejection, including imported namespace dependency visibility, while preserving downstream ownership boundaries.

## TASK-2037 Engine-owned CPS executor boundary

**Task:** [TASK-2037](tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md)
**Canonical rules:** `SEM-TARGET-CORE-CPS-001`, `SEM-EFFECT-ADMISSION-001`,
`OBS-TARGET-PROJECTION-001`, `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-CPS-TRAP-001`,
`SEM-EFFECT-TIMEOUT-001`, and `SEM-EFFECT-CANCEL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Selected client routes, full target Core/CPS domains, deletion of direct-AST and differential material, and TASK-2041's four-client terminal comparison remain incomplete.

**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.

**Run-route impact:** prerequisite.

**Consumes:** TASK-2035's Engine-only client contract; `AUDIT-204-CPS-001` through
`AUDIT-204-CPS-008`; checked Core/CPS artifacts; and Engine admission provenance.

**Produces:** the Engine-private checked-CPS executor boundary, migrated private CPS regression
coverage, and private Engine test placement for retained AUDIT-204 differential material. That
placement removes public invocation only; TASK-2040 retains the frozen-audit deletion ownership.
It does not activate a client route or rename the residual support crate.

**Downstream owner:** TASK-2038, TASK-2039, TASK-2040, and TASK-2042 consume this boundary;
TASK-2041 owns integration proof and API-absence closeout.

**Evidence detail:**
- **Positive:** `TEST-TASK-2037-ENGINE-OWNED-CPS-POSITIVE`
- **Trap:** `TEST-TASK-2037-ENGINE-OWNED-CPS-TRAP`
- **Timeout:** `TEST-TASK-2037-ENGINE-OWNED-CPS-TIMEOUT`
- **Cancellation:** `TEST-TASK-2037-ENGINE-OWNED-CPS-CANCELLATION`
- **Negative:** `TEST-TASK-2037-ENGINE-OWNED-CPS-NEGATIVE`
- **Mutation:** `TEST-TASK-2037-ENGINE-OWNED-CPS-MUTATION`
- **Parity:** not applicable; no client route or reference-executor comparison is performed by this
  prerequisite boundary task.

**Non-goals:** Test-runner, REPL, daemon, or ash run client-route implementation. Deletion of direct-AST evaluation, the Rust differential stack, or Lean material. Renaming ash-interp while TASK-2040-owned AST material remains. Transferring TASK-2040 deletion ownership when retained audit-listed differential tests move into Engine-private test modules.

**Next obligation:** TASK-2038, TASK-2039, TASK-2042, and TASK-2040 must consume the Engine-private executor boundary; TASK-2041 must prove API absence and four-client normalized-terminal parity.

## TASK-2038 `ash test` canonical Engine execution

**Task:** [TASK-2038](tasks/TASK-2038-ash-test-canonical-engine-execution.md)
**Canonical rules:** `CONF-SYNTH-SOURCE-WRAPPER-001` and `CONF-ENGINE-ONLY-CLIENT-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only the two exact TASK-2035 source identities are selected. The remaining SPEC-077 synthesized-test domain, unselected client routes, residual direct-evaluator deletion, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification
partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-SYNTH-WRAPPER-001`, `TASK-2035-SHARED-ROUTE-001`,
`AUDIT-204-TEST-EXEC-002`, `AUDIT-204-DEFERRED-001` through `AUDIT-204-DEFERRED-007`, and the
TASK-2037 Engine-private executor boundary.

**Produces:** selected `ash test` Engine submissions, source identity/repro linkage, exact
deferred-case result records, and focused terminal observations for the two selected source
identities.

**Downstream owner:** TASK-2040 removes residual direct test-evaluator material; TASK-2041 owns
the same-source-contract four-client terminal comparison.

**Evidence detail:**
- **Positive:** `TEST-TASK-2038-SYNTH-WRAPPER-POSITIVE`,
  `TEST-TASK-2038-CATALOGUE-PROPERTY`
- **Negative:** `TEST-TASK-2038-DEFERRED-CATALOGUE`
- **Mutation:** `TEST-TASK-2038-MUTATION-NO-FALLBACK` rejects an altered parse-success source
  shape at Engine admission and records the compatibility result as explicit deferred output.
- **Parity:** `TEST-TASK-2038-SHARED-ROUTE-PARITY` compares the selected shared source's
  normalized terminal result through the test client and Engine. It is not TASK-2041's four-client
  parity evidence.

**Non-goals:**
- A general source synthesizer, forms absent from the TASK-2035 catalogue, REPL, daemon, or ash run client implementation.
- Target grammar expansion or a direct-evaluator compatibility mode.
- TASK-2040-owned removal of residual direct test-evaluator and differential material.
- TASK-2041's four-client same-source-contract terminal comparison.

**Next obligation:** Retain the selected Engine route while TASK-2040 removes residual direct test-evaluator material and TASK-2041 supplies the four-client terminal comparison.

## TASK-2039 REPL canonical Engine execution

**Task:** [TASK-2039](tasks/TASK-2039-repl-canonical-engine-execution.md)
**Canonical rules:** `OBS-REPL-ENGINE-CLIENT-001` and `CONF-ENGINE-ONLY-CLIENT-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only the two exact TASK-2035 REPL source identities are selected. Stored-session shapes beyond the selected controls, remaining SPEC-011 submission forms, residual direct-evaluator deletion, daemon and ash run transport, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification
partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-REPL-ROUTE-001`, `TASK-2035-REPL-ROUTE-002`,
`TASK-2035-SHARED-ROUTE-001`, `AUDIT-204-REPL-001`, `AUDIT-204-REPL-002`, and the TASK-2037
Engine-private executor boundary.

**Produces:** source-derived admitted REPL requests, normalized terminal rendering for the selected
controls, and focused REPL-to-Engine terminal observations.

**Downstream owner:** TASK-2040 deletes residual REPL direct-evaluator calls; TASK-2041 owns the
same-source-contract four-client terminal comparison.

**Evidence detail:**
- **Positive:** `TEST-TASK-2039-REPL-ENGINE-POSITIVE`,
  `TEST-TASK-2039-REPL-MULTILINE`
- **Negative:** `TEST-TASK-2039-REPL-ADMISSION-REJECTION`,
  `TEST-TASK-2039-REPL-INSPECTION-NO-EVALUATION`
- **Mutation:** `TEST-TASK-2039-REPL-DECLARED-CORPUS-PROPERTY` ranges only over the declared
  source IDs and preserves their admitted Engine terminal observations.
- **Parity:** `TEST-TASK-2039-REPL-SHARED-ROUTE-PARITY` compares the shared source's normalized
  terminal result through REPL and Engine. It is not TASK-2041's four-client parity evidence.

**Non-goals:** A new REPL language, persistent evaluation beyond the specified session state, target grammar expansion, daemon or ash run transport, or a direct-evaluator compatibility mode.

**Non-goals:** TASK-2041's four-client same-source-contract terminal comparison.

**Next obligation:** Retain the selected Engine route while TASK-2040 removes residual REPL direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.

## TASK-2042 daemon descriptor and terminal-envelope parity

**Task:** [TASK-2042](tasks/TASK-2042-daemon-admitted-request-terminal-envelope-parity.md)
**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-EFFECT-ADMISSION-001`,
`SEM-EFFECT-TIMEOUT-001`, `SEM-EFFECT-CANCEL-001`, and `SEM-EFFECT-TERMINAL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only `TASK-2035-SHARED-ROUTE-001` is selected. The remaining daemon protocol domain, residual direct-evaluator deletion, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-SHARED-ROUTE-001`, `AUDIT-204-CLIENT-006`, TASK-2032's Engine request
seam, TASK-2036's no-fallback guard, and TASK-2037's Engine-private executor boundary.

**Produces:** descriptor validation, local-Engine admission and request minting, terminal-envelope
transport, and `ash run`/daemon same-source-contract parity evidence.

**Downstream owner:** TASK-2040 deletes residual daemon direct-evaluator calls. TASK-2041 owns the
four-client descriptor/envelope comparison and API-absence closeout.

**Evidence detail:**
- **Positive:** `TEST-TASK-2042-DAEMON-DESCRIPTOR-SUCCESS`
- **Negative:** `TEST-TASK-2042-DAEMON-DESCRIPTOR-ADMISSION-REJECTION`,
  `TEST-TASK-2042-DAEMON-DESCRIPTOR-PRE-EXECUTION-CLASSIFICATION`,
  `TEST-TASK-2042-DAEMON-DESCRIPTOR-RUN-CONTROLS`
- **Mutation:** `TEST-TASK-2042-DAEMON-DESCRIPTOR-MUTATION` ranges only over named descriptor
  mutations, including nonzero deadlines and an invalid deadline/cancellation combination, and
  never generates source forms.
- **Parity:** `TEST-TASK-2042-DAEMON-DESCRIPTOR-PARITY` compares the selected source contract:
  direct `ash run` retains the manifest source bytes in its local Engine and daemon validates the
  complete wire descriptor before its local Engine mints a request. It is not TASK-2041's
  four-client parity evidence.

**Non-goals:** A shared Engine service, cross-process request handles, source synthesis, admission reconstruction, a new daemon language, formatting, or Lean execution.

**Next obligation:** Retain the selected daemon descriptor route while TASK-2040 removes residual daemon direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.

## TASK-2040 Engine-only removal

**Task:** [TASK-2040](tasks/TASK-2040-remove-direct-ast-and-differential.md)
**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-TARGET-CORE-CPS-001`,
`OBS-TARGET-PROJECTION-001`, and `CONF-IMPLEMENTATION-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The target Core/CPS domains and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.

**Run-route impact:** active.

**Consumes:** the frozen `AUDIT-204` dispositions, TASK-2035 selected source contracts,
TASK-2036's re-entry guard, and the Engine boundary delivered by TASK-2037 through TASK-2042.

**Produces:** retired Rust legacy execution, an evaluator-free `ash-runtime` support crate, and
preserved Lean material with its separate-project handoff.

**Downstream owner:** TASK-2041 validates the zero-use state, closes documentation and
traceability, and owns the four-client normalized-terminal comparison.

**Evidence detail:**
- **Positive:** `TEST-TASK-2040-ENGINE-TERMINAL-POSITIVE`
- **Negative:** `TEST-TASK-2040-MANIFEST-REMOVAL`,
  `TEST-TASK-2040-EXTERNAL-API-ABSENCE`, and `TEST-TASK-2040-REPLACEMENT-LEAN-CONTROLS`
- **Mutation:** `TEST-TASK-2040-DECLARED-CONTRACT-ENGINE-PROPERTY`
- **Parity:** not applicable; TASK-2041 owns the four-client comparison.

**Non-goals:** Lean implementation or deletion, a direct-evaluator compatibility route, source synthesis, a new execution domain, or TASK-2041's four-client parity proof.

**Next obligation:** TASK-2041 validates the zero-use state, documentation and traceability, and four-client parity.

## TASK-2041 Engine-only closeout

**Task:** [TASK-2041](tasks/TASK-2041-engine-only-closeout-docs-traceability-and-gate.md)
**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001` and `CONF-IMPLEMENTATION-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** The target Core/CPS domains remain partial; TASK-2041 compares only the declared shared source contract across four independent local Engine clients.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.

**Run-route impact:** active.

**Consumes:** all Phase-204/205 handoffs, the frozen audit, and focused client evidence.

**Produces:** zero-use enforcement, current documentation and historical routing, and the
four-client normalized-terminal evidence record.

**Downstream owner:** Later target-rule realization tasks own every remaining partial/below-spec clause.

**Evidence detail:**
- **Positive:** `TEST-TASK-2041-FOUR-CLIENT-PARITY`
- **Negative:** `TEST-TASK-2041-ZERO-USE-GATE` and `TEST-TASK-2041-LEAN-BOUNDARY`
- **Mutation:** `TEST-TASK-2041-DECLARED-CORPUS-PROPERTY`
- **Parity:** `TEST-TASK-2041-FOUR-CLIENT-PARITY` covers the one declared shared contract only.

**Non-goals:** A shared Engine service, daemon execution for ash run or REPL, source synthesis, deferred-case implementation, Lean execution, or a runtime refinement proof.

**Next obligation:** Later target-rule realization tasks own every remaining partial/below-spec clause.

### Retired differential material

- **Historical owners:** `TASK-2005`, `TASK-439`
- **Layer status:** not applicable to the current Engine route.
- **Current status:** Rust differential implementation and tests were removed by TASK-2040. Their
  retained records are historical and provide no execution or conformance evidence.

### Contracts, predicates, and proofs

- **Canonical owner:** `SPEC-098b`, `SPEC-100`
- **Layer status:** Type partial; Core partial sidecars; CPS not_applicable; admission/runtime
  not_applicable; verification partial.
- **Missing target-spec clauses:** predicate discharge, proof, and runtime contract semantics.

## Required task record

Each linked task records: canonical rule/spec section; implementation, evidence, and parity status;
missing target-spec clauses; layer status; positive, negative, mutation, and parity evidence where
applicable; non-goals; and the next gap.
Each new or materially revised linked semantic task/record must also contain a **Handoffs** block
with its **Consumes**, **Produces**, intentionally unowned layer and its **downstream owner**, and
**integration/proof responsibility**. Reviewers reject a claim that a passing fixture completes a
target rule without target-spec parity and stated evidence.

## TASK-2001 semantic workflow record

**Task:** [TASK-2001](tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `GRAM-TARGET-MODULE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.
**Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-V8-IMPORTED-HANDLER-ROW-E2E`
- **Negative:** `TEST-PARSER-STALE-DECLARATION-REJECTION`
- **Mutation:** `TEST-CORE-V8-STRUCTURAL-EFFECT-ROW-UNKNOWN-FIELD-REJECTION`
- **Parity:** not applicable; this parser/type-summary slice has no paired execution relation.
**Non-goals:** General grammar, row, and handler realization.
**Next obligation:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.

## TASK-2002 semantic workflow record

**Task:** [TASK-2002](tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `LOWER-SURFACE-CORE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Carry every required target sidecar or an explicit unsupported outcome through lowering.
**Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-AMBIENT-DO-SOURCE-ENTRY-BOUNDARY`
- **Negative:** `TEST-ENGINE-NAMED-DO-TARGET-REJECTION`
- **Mutation:** `TEST-ENGINE-INVALID-HELPER-CONTRACT-SIDECAR-GUARD`
- **Parity:** not applicable; the retained sidecars are metadata, not a paired execution relation.
**Non-goals:** General sidecar completeness and runtime contract semantics.
**Next obligation:** Carry every required target sidecar or an explicit unsupported outcome through lowering.

## TASK-2003 semantic workflow record

**Task:** [TASK-2003](tasks/TASK-2003-return-authority-and-cps-kernel-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Extend checked Core/CPS realization only through separately admitted source forms.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-SEALED-LOCAL-CALL-CORE-CPS-PRODUCTION`
- **Negative:** `TEST-CORE-CPS-RETURN-AUTHORITY`
- **Mutation:** `TEST-ENGINE-SEALED-LOCAL-CALL-PROVENANCE-GUARD`
- **Parity:** not applicable; this lowering route does not claim a full reference-runtime parity relation.
**Non-goals:** General source control, call, and continuation lowering.
**Next obligation:** Extend checked Core/CPS realization only through separately admitted source forms.

## TASK-2004 semantic workflow record

**Task:** [TASK-2004](tasks/TASK-2004-core-cps-production-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Admit further source forms only after validated typed lowering and checked Core/CPS evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-RUN-HANDLER-FREE-CHECKED-CPS-ADMISSION`
- **Negative:** `TEST-ENGINE-UNARY-NEGATION-PRODUCTION-REJECTION`, `TEST-ENGINE-RUN-FILE-UNARY-NEGATION-PRODUCTION-REJECTION`
- **Mutation:** `TEST-CORE-CPS-ADMISSION-GUARD`
- **Parity:** not applicable; the production boundary does not itself claim a parity relation.
**Non-goals:** A legacy direct-evaluator fallback or general source admission.
**Next obligation:** Admit further source forms only after validated typed lowering and checked Core/CPS evidence.

## TASK-2005 retired historical material

**Task:** [TASK-2005](tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md)
**Status:** Retired historical material; not an active workflow record.
**Canonical rules:** Historical references only: `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The target conformance domain remains unrealized; a future owner
must establish it without reviving the retired direct-runtime route.
**Layers:** type not_implemented; core not_implemented; cps not_implemented; admission-runtime not_applicable; verification not_implemented.
**Evidence detail:** None. TASK-2040 removed the Rust direct-runtime differential implementation
and tests; this retained material provides no current execution or conformance evidence.
**Non-goals:** Reactivating the retired direct-runtime to checked-CPS comparison route.
**Next obligation:** A future target-rule owner must establish conformance through the Engine-only route.

## TASK-2008 semantic workflow record

**Task:** [TASK-2008](tasks/TASK-2008-json-variant-observable-projection.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Add canonical envelope cases only with an admitted checked route and focused observable evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-CLI-CANONICAL-TERMINAL-PROJECTION`
- **Negative:** `TEST-CLI-UNADMITTED-TRAP-SLEEP-TERMINAL-ENVELOPE`
- **Mutation:** `TEST-CLI-POSTEXECUTION-INVALID-EXIT-PROJECTION`
- **Parity:** not applicable; terminal projection is not a direct-runtime parity claim.
**Non-goals:** A complete terminal matrix for every future execution route.
**Next obligation:** Add canonical envelope cases only with an admitted checked route and focused observable evidence.

## TASK-2013 semantic workflow record

**Task:** [TASK-2013](tasks/TASK-2013-source-handler-and-handle-lowering.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Connect validated typed handler lowering to separately authorized production admission.
**Layers:** type partial; core partial; cps partial; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-TYPECK-CHECKED-HANDLER-SIDECAR`
- **Negative:** `TEST-TYPECK-V7-IMPORTED-HANDLER-ROW-INELIGIBLE`
- **Mutation:** `TEST-TYPECK-HANDLER-CORE-INSPECTION`
- **Parity:** not applicable; the typed handler slice does not claim direct-runtime parity.
**Non-goals:** General handler execution, inference, and residual-row realization.
**Next obligation:** Connect validated typed handler lowering to separately authorized production admission.

## TASK-2014 semantic workflow record

**Task:** [TASK-2014](tasks/TASK-2014-source-handler-runtime-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Admit only further validated handler forms with sealed bindings and terminal-envelope evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-CLOSED-EMPTY-HANDLER-PRODUCTION-RUN`
- **Negative:** `TEST-ENGINE-HANDLER-SOURCE-RUNTIME-CLOSED`
- **Mutation:** `TEST-ENGINE-FORGED-TRAP-SLEEP-CORE-CLASSIFICATION`
- **Parity:** not applicable; selected production admission is not a full parity relation.
**Non-goals:** General handler/provider execution or row-derived frame installation.
**Next obligation:** Admit only further validated handler forms with sealed bindings and terminal-envelope evidence.

## TASK-2031 λAsh-Effect correspondence record

**Task:** [TASK-2031](tasks/TASK-2031-lambda-ash-effect-correspondence.md)
**Status:** Complete
**Canonical rules:** `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`,
`OBS-TARGET-PROJECTION-001`, `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`,
`SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DISCHARGE-001`, `SEM-EFFECT-MISSDISCHARGE-001`,
`SEM-EFFECT-RESUME-001`, `SEM-EFFECT-HANDLERTRAP-001`, `SEM-EFFECT-PROVIDER-001`,
`SEM-EFFECT-ADMISSION-001`, `SEM-EFFECT-TIMEOUT-001`, `SEM-EFFECT-CANCEL-001`, and
`SEM-EFFECT-TERMINAL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** TASK-2032 must consume this correspondence through the one shared admitted Engine path and prove client parity without a fallback evaluator.
**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime
not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-CONTRACT`
- **Negative:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-INCOMPLETE-REJECTION`
- **Mutation:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-MISMAPPING-REJECTION`
- **Parity:** not applicable; this prerequisite-only mathematical correspondence has no active
  Engine, CLI, or daemon route; TASK-2032 owns integration parity.
**Non-goals:** Parser acceptance, Core lowering, admission/frame installation, Engine execution, and CLI/daemon parity.
**Next obligation:** TASK-2032 must consume this correspondence through the one shared admitted Engine path and prove client parity without a fallback evaluator.

## TASK-2032 shared Engine execution seam record

**Task:** [TASK-2032](tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md)
**Canonical rules:** `SEM-TARGET-CORE-CPS-001`, `OBS-TARGET-PROJECTION-001`,
`SEM-EFFECT-ADMISSION-001`, `SEM-EFFECT-HANDLERTRAP-001`, `SEM-EFFECT-TIMEOUT-001`,
`SEM-EFFECT-CANCEL-001`, and `SEM-EFFECT-TERMINAL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** A separately owned daemon transport/profile/binding task must carry an admitted request and V1 terminal envelope before a selected noncanonical provider or handler route can be daemon-active.
**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.
**Run-route impact:** active. This task consumes selected checked artifacts and terminal
projection into one Engine execution seam; it does not claim target executor behavior.
**Consumes:** TASK-2004/TASK-2014 checked admissions and authorized frames, TASK-2008 terminal
projection, and TASK-2031 correspondence.
**Produces:** opaque Engine admitted-program request/result integration, in-process client adapters
over the same request, explicit daemon-service activation/rejection evidence, and the
`RUNNABLE-ASH-MATRIX.md` ledger.
**Downstream owner:** Feature-realization tasks own each matrix source/lowering/provider gap;
TASK-2032 retains integration/parity evidence for the selected artifact slices.
**Evidence detail:**
- **Positive:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE` (including the exact
  `deep_affine_clock` checked-CPS `Int(107)` slice)
- **Negative:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-NEGATIVE`
- **Mutation:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-MUTATION`
- **Parity:** `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY` and
  `TEST-TASK-2032-CLIENT-ADAPTER-DEADLINE-REUSE-PARITY` (same in-process request only)
- **Daemon service boundary:** `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`
**Non-goals:** Parser acceptance, Core/CPS lowering, provider implementation, handler semantics, frame authorization, terminal taxonomy, and daemon transport redesign.
**Next obligation:** A separately owned daemon transport/profile/binding task must carry an admitted request and V1 terminal envelope before a selected noncanonical provider or handler route can be daemon-active.

## TASK-439 retired historical material

**Task:** [TASK-439](tasks/TASK-439-differential-conformance-harness-rust-first.md)
**Status:** Retired historical material; not an active workflow record.
**Canonical rules:** Historical references only: `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The target conformance domain remains unrealized; a future owner
must establish it without reviving the retired direct-runtime route.
**Layers:** type not_applicable; core not_implemented; cps not_implemented; admission-runtime not_applicable; verification not_implemented.
**Evidence detail:** None. TASK-2040 removed the Rust differential harness and tests; this
retained material provides no current execution or conformance evidence.
**Non-goals:** Reactivating the retired differential harness or claiming it as a reference implementation.
**Next obligation:** A future target-rule owner must establish conformance through the Engine-only route.
