---
id: plan.207.complete-module-realization
title: Complete Module Realization
kind: plan
status: in-progress
authority: planning
owner: language-semantics
last_verified: 2026-08-04
---

# PLAN-207: Complete Module Realization

## Purpose

Implement [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md): one complete, language-level module route for both `mod name;` and `mod name { ... }`.

```text
  Surface ModuleFile
  -> AST-driven graph and source acquisition
  -> AST-only syntax dependency prepass
  -> canonical expanded module graph
  -> internal collected snapshots + name-only provisional views
  -> resolved imports and visibility
  -> checked export-closed interfaces
  -> Core modules
  -> CPS modules
  -> admitted Engine artifact
  -> CLI/daemon terminal parity
```

A file-backed child and an inline child differ only before source acquisition. After that point they must have equal module semantics for equal declarations.

## Baseline and authority

- [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md) owns the target module rule.
- [AUDIT-207](audits/AUDIT-207-module-realization-seams.md) records the current split parser/resolver/summary/import/Engine seams.
- [PLAN-203](PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md) owns integration of the one Surface → Core → CPS → Engine route and client parity. This phase supplies module artifacts to that route; it does not create another evaluator.
- [SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md), [SPEC-095c](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md), [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), [SPEC-098b](../spec/SPEC-098b-TARGET-IR.md), [SPEC-098c](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md), [SPEC-099b](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md), and [SPEC-099c](../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) remain the grammar, syntax-phase, type, IR, lowering, and Engine operational owners.
- [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) and [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md) remain the implemented bounded summary substrate. This phase extends their transport compatibly; it does not recreate their type identities, closure rules, versioning, or import-order semantics.
- [PLAN-206](PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md) and its audit provide current-state limitation evidence only. They do not authorize target semantics.

## Completion contract

The phase is complete only when:

1. every structural module edge originates in a parsed `ModuleFile` declaration;
2. no normal semantic path scans raw text for a module declaration, ordinary declaration, import, export, or visibility fact after parsing;
3. file-backed and inline modules with equivalent declarations have equivalent normalized interfaces and Core/CPS artifacts;
4. imports and visibility use stable declaration identities, provisional name views during binding,
   and checked interfaces for finalized publication;
5. incomplete modules cannot publish partial public interfaces;
6. reachable entry modules lower and execute only through the Engine-owned checked Core/CPS route; and
7. one multi-module admitted program has CLI/daemon normalized-terminal parity.

## Scope

### In scope

- Stable crate-qualified module paths, graph identities, source origins, structural graph construction, and source-acquisition diagnostics.
- AST-driven replacement or strict quarantine of text scanning at all semantic module seams.
- Common file-backed/inline module-unit construction.
- Versioned export-closed interfaces, public/private views, and identity-preserving re-exports.
- Interface-based imports, qualified resolution, and all parsed visibility forms.
- Module-aware source-to-Core, Core-to-CPS, Engine linking, admission, and selected entry execution.
- Positive, negative, mutation, parity, and diagnostics evidence.

### Non-goals

- New lexical module/import spelling, package, registry, dynamic-import, hot-reload, or
  runtime-module syntax. The existing `use` and nested `mod` item forms become permitted in an
  inline module so it has the same item domain as a file module.
- Dynamic runtime module values or automatic module initialization.
- Import-cycle initialization or cross-module recursive initialization. Structural and import cycles reject in this phase.
- A full incremental workspace/LSP database.
- Macro runtime callability, new macro hygiene semantics, or imported-notation activation beyond existing syntax-phase contracts.
- Any direct evaluator, Engine bypass, or authority inferred from a module interface.

## Decision gates

No user decision remains open for the initial realization. The phase fixes these conservative rules:

| Gate | Decision | Impact |
|---|---|---|
| D1 | `mod` structural cycles reject; import cycles also reject in the initial implementation | Prevents unspecified initialization and partial interface publication |
| D2 | `ModuleFile` AST is authoritative; text scans are removed or fenced as non-authorizing migration checks | Eliminates parser/resolver/Engine disagreement |
| D3 | File-backed and inline modules share one module-unit and interface pipeline after source acquisition | Makes parity an executable invariant, not prose |
| D4 | A module is a compile/link namespace, not a runtime value; entry execution remains Engine-owned | Preserves PLAN-203's one-executor contract |

D1-D4 are target-contract decisions in SPEC-103, not implementation discretion. Any change requires a SPEC-103 amendment before code changes.

## Workstreams and task order

```text
Track A — canonical structural substrate
  TASK-2057 AST-driven discovery + TASK-2058 stable key/artifact carrier
    -> TASK-2059 common source acquisition -> TASK-2067 canonical graph/state machine + structural diagnostics

Track B — complete interface and binding semantics
  TASK-2060 Core carrier -> TASK-2066 TypeEnv finalizer -> TASK-2061 bounded wrapper resolver ─┐
  TASK-2067 canonical graph/module units ──────────────────────────────────────────────────────┴─> TASK-2068 completed partial/tested foundation
                                                                                                      -> TASK-2070 self alias + TASK-2071 namespace contract
                                                                                                      -> TASK-2074 expanded graph -> TASK-2075 two-tier collection
                                                                                                      -> TASK-2072 complete imports/binding -> TASK-2073 finalization/export closure

Track C — complete realization and admission
  TASK-2073 complete checked/export-closed handoff -> TASK-2069 full lowering + Engine scanner/cache fence
    -> TASK-2063 sealed linking/admission

Track D — evidence and closeout
  TASK-2067 + TASK-2073 + TASK-2069 + TASK-2063 -> TASK-2064 conformance and parity
    -> TASK-2065 closeout
```

TASK-2057 has completed its partial, tested, below-spec structural handoff: parser-owned
declarations now create resolver file and inline graph children, and the resolver declaration
scanner is removed. It deliberately does not establish canonical identities, module-unit parity,
interfaces, import/visibility semantics, lowering, admission, or terminal parity. Completed
TASK-2067 supplies a partial/tested/below-spec parser graph in
`crates/ash-parser/src/canonical_module_graph.rs` over canonical `ModuleKey`s, with real units,
AST-only edges, parsed root metadata, complete `Absent`/`Discovered`/`Parsed`/`Failed` reporting,
anchored missing/root+nested-duplicate/malformed-inline/cycle failures, parsed-source invalid-key
rejection, canonical-key rewrite resistance, complete ordered payload parity/mutation, and an
isolated deprecated legacy-route fence. TASK-2064 later validates the composed result.

TASK-2058 has completed its `partial / tested / below_spec` Core-carrier handoff. It publishes a
crate-qualified `ModuleKey`, `ModuleArtifactOrigin`, schema-versioned `ModuleArtifact`, and
deterministic child-key validation without changing resolver graph construction or existing
`semantic_summary::ModuleIdentity` semantics. It does not claim source-kind parity, interfaces,
imports, lowering, admission, execution, or client parity. TASK-2059 consumes both the completed
TASK-2057 handoff and TASK-2058's delivered identity/artifact carrier.

TASK-2059 has completed its bounded, parser-owned `partial / tested / below_spec` source-acquisition
handoff. `ModuleBody` and `ModuleUnit` now preserve ordered `use`, definition, and nested-module
items through one file/inline dispatcher; `ModuleUnitResolver` consumes the TASK-2058 carrier,
chooses `child.ash` before `child/mod.ash`, parses a chosen file once, and makes inline acquisition
without filesystem access. It supplies parent-anchored missing/invalid-key acquisition diagnostics,
duplicate-child rejection, and recursive isolated macro/notation expansion. It does not traverse a
structural graph or provide malformed-inline parent anchors/error atomicity; resolver graph and
legacy identity migration, checked interfaces, import binding/visibility, Core/CPS lowering,
Engine-only linked admission/execution, and CLI/daemon parity remain separately owned. No direct
evaluator fallback is authorized. TASK-2067 owns the intentionally missing canonical-graph,
state-machine, real-unit transport, and structural-diagnostic clauses.
TASK-2060 has completed a `partial / tested / below_spec` Core-carrier handoff. It publishes the
V1 `PublicModuleInterface` schema over TASK-2058 artifacts, public binding identity/visibility/
origin facts, dependency versions, strict wire validation, and existing summary V1--V8
compatibility; macro/notation facts remain syntax-only. It rejects invalid public projection facts
before a value exists but does not collect TASK-2059 `ModuleUnit` values, retain a private TypeEnv
view, link existing typed summary identities, or finalize public closure. It also does not alter
Engine scanners or transport. TASK-2066 now supplies a bounded wrapper over that Core carrier, but
it does not establish the complete interface required for imports. TASK-2071 now supplies the
completed namespace/provisional-view contract, TASK-2074 owns the canonical expanded graph,
TASK-2075 owns two-tier collection, TASK-2072 owns parsed imports and binding, and TASK-2073 owns final interface/export
closure; TASK-2069 owns complete lowering and Engine scanner/cache transport fencing; TASK-2063
consumes only TASK-2069's complete non-sealed closure and cannot authorize a
direct-evaluator fallback; TASK-2064 owns structural/import-cycle conformance and CLI/daemon
parity; TASK-2065 closes the phase.

TASK-2066 has completed its `partial / tested / below_spec` TypeEnv finalization handoff for
MOD-REAL-003. Its collection stages a clone of TypeEnv, claims the canonical `ModuleKey`,
prechecks public function/handler marker conflicts, calls `register_surface_declarations` for
declaration-signature preflight, maps failures, verifies the bounded collected facts, and commits
atomically. It requires full `ModuleArtifact` equality before issuing a non-forgeable,
immutable `FinalizedModuleInterface` wrapper. It does not check bodies or full callable facts,
link typed namespaces, collect aliases/re-exports or per-binding source origins, establish complete
export closure, or authorize imports, Engine transport, lowering, admission, execution, or client
parity. TASK-2061 has completed a separate `partial / tested / below_spec` resolver handoff: it
stores only the bounded wrappers, traverses canonical public child identities, resolves explicit/
group/glob requests atomically with explicit-over-glob precedence, preserves identity and
syntax-only macro metadata, and leaves conflicting globs ambiguous. It does not implement parsed
imports or visibility, aliases/re-exports, typed namespaces, cycles, binder integration, full
closure, lowering, Engine transport, or runtime/client authority.

TASK-2068 now has a `partial / tested / below_spec` Type-layer slice: it consumes only
TASK-2067's graph-delivered units to provisionally collect function identity/origin/span/visibility
facts and resolve simple parsed `crate::…` aliases. This is bounded `M-COLLECT` plus graph-only
`M-IMPORT-EDGE`/`M-IMPORT-CYCLE` planning: it records canonical cross-module provenance, emits no
same-module edge, rejects each discovered cycle through the ordered parser-anchored
`ImportCycle { edges: CanonicalImportCycle }` wrapper, and makes the compatibility binder delegate
through that planner. It is not a final interface, complete import edge/cycle system, or `M-BIND`
result. TASK-2068 is complete for this partial/tested/below-spec foundation. TASK-2071 now closes
the namespace/provisional-view contract with no implementation evidence; TASK-2074 owns syntax
prepass and canonical expansion, TASK-2075 owns complete two-tier collection, TASK-2072 complete
parsed import/cycle/binding semantics, and TASK-2073 complete checked bodies/export closure; only TASK-2073 may hand a lowering input to
TASK-2069.

TASK-2068 now also has a tested within-task `M-CHECK` sub-slice for graph-delivered
self-contained leaf modules containing only ordinary functions with primitive closed signatures.
Alongside inherited/public declarations, it admits only `pub(crate)`, `pub(super)`, `pub(in crate)`
or `pub(in crate::...)`, and `pub(self)` restricted leaves; a non-crate path such as
`pub(in self::internal)` rejects. It graph-preflights every unit, stages sibling signatures, checks
all bodies atomically only through the builtin TypeEnv checker, and retains fresh checked
identity/module identity/origin/spans and signature/body types. It exposes only public primitive
signatures through a non-authorizing `CanonicalPublicFunctionInterface` alongside a private
checked-function map. The focused target passes 18/18, including a 16-case property, restricted
visibility, negative, atomicity, and architecture-fence evidence. This remains neither core
`PublicModuleInterface` nor a final module interface, and excludes imports, child/nested modules,
other definitions, generics, contracts, unsupported visibility outside the bounded restricted
domain, user-defined types, interfaces, effects, re-exports, Core/CPS/Engine, and client behavior.

TASK-2068 has delivered graph-only canonical simple-import planning for bounded
`M-IMPORT-EDGE` and `M-IMPORT-CYCLE`. It consumes only real canonical graph units and provisional
function targets, admitting inherited `UsePath::Simple` crate-root function aliases. It returns
opaque resolved imports/bindings and canonical cross-module edges retaining importer, defining
identity, local spelling, use/declaration spans, origin, and visibility; same-module aliases have
no edge. Every discovered cycle rejects before any result as `ImportCycle { edges:
CanonicalImportCycle }`, whose ordered wrapper exposes parser-anchored edges, and the compatibility
simple binder delegates so planning cannot be bypassed. The focused target passes 11/11, including
edge provenance, same-module no-edge, file/inline two-node cycle ordering, a full-provenance
`a → b → c → b` tail diagnostic that reports only `b ↔ c`, late-backedge atomicity, and a fence
excluding `RawCoreProgram`, `CoreExpr`, and `CpsProgram`. Checked interfaces, TypeEnv/body
integration, legacy/2060/2061/2066 authority, restricted visibility, `pub use`, re-exports,
groups, globs, qualified paths, Core/CPS/Engine, and clients remain excluded.

TASK-2068 also delivers a bounded direct primitive provider/client check. It consumes only the
canonical graph and its resolved simple-import plan, admits the root plus plan-selected direct
provider leaves, and uses pre-provider graph-wide `module_units()` completeness to reject
unrelated unselected non-root units, including nested modules. A descendant of a selected provider
instead reaches provider-leaf precheck and rejects as anchored `UnsupportedProviderShape`. It requires exact plan/graph artifacts, primitive provider
precheck, edge revalidation against checked public providers, and fresh-root imported-signature
injection before atomically returning non-authorizing checked root/provider/import facts. It does
not establish a final interface, general import/binder authority, Core/CPS lowering, Engine
admission, or parity.

TASK-2068 also delivers a bounded direct-public primitive re-export interface fragment:
`partial / tested / below_spec`, with Type `partial`, Core/CPS/admission-runtime
`not_applicable`, and verification `partial`. It consumes the same canonical root/direct-provider
domain plus an exact direct-public plan and admits only
`pub mod api` with explicit root `pub use crate::api::greet as welcome`. The public structural
path and public primitive target must both be checked, while defining identity, origin, signature,
declaration span, and use span remain distinct; `pub mod api` never implicitly flattens `greet`
into the root. It must also reject an empty public-use plan, any public root definition outside
that exact fragment, and `pub mod api`/`pub use … as api` child-identity alias collisions with
structural/use anchors. It atomically returns only a non-authorizing fragment or no result.
Its focused target passes 13/13, including a 16-case property; its positive, named negative,
property, atomicity, and authority-fence witnesses are test evidence, not a proof or a final
interface, compatibility-carrier, Core/CPS, Engine, admission, runtime, or parity claim.

TASK-2068 also delivers a private primitive provider-helper direct re-export sub-slice:
`partial / tested / below_spec`, with Type `partial`, Core/CPS/admission-runtime
`not_applicable`, and verification `partial`. Within the same exact public root form, a selected
public primitive target may use inherited/private ordinary primitive provider helpers. Those
helpers are checked atomically as private implementation detail and never project into the
non-authorizing fragment. Its focused target passes 7/7, including a 16-case property; this is
test evidence, not proof or end-to-end parity evidence. Provider uses, nested modules, other
definitions, generic/contracts, restricted visibility, non-primitive/open signatures, other
paths, Core/CPS, Engine, admission, runtime, and parity remain excluded.

TASK-2068 also delivers a direct re-export local-binding root-client sub-slice: `partial /
tested / below_spec`, with Type `partial`, Core/CPS/admission-runtime `not_applicable`, and
verification `partial`. In that same exact form, inherited/private root
`fn internal_entry(..) -> <primitive> { welcome(..) }` calls the explicit alias through a distinct
opaque direct plan with exact graph/plan artifact snapshots, selected-provider facts, checked
private root functions, and a local binding that preserves `greet`'s defining identity and
visibility before registration. Its focused target passes 10/10, including a 16-case property.
Only a direct unqualified alias call, including an empty block tail, supplies a call diagnostic
anchor; every other root-body failure uses the enclosing body span. Generic planner/binder and
generic provider/client paths continue to reject source `pub use`; root public functions, generic
binding, final interfaces, Core/CPS, Engine, admission, runtime, and parity remain excluded.

TASK-2068 now delivers a canonical provisional-module-scope and structural-path visibility
sub-slice: `partial / tested / below_spec`, with Type `partial`, Core/CPS/admission-runtime
`not_applicable`, and verification `partial`. It builds immutable typeck-owned direct-child and
ordinary-function scopes from TASK-2067 graph units/artifacts. Before resolution, it compares
root/artifact facts and requires equality with a rebuilt current declaration snapshot, so artifacts
alone cannot authorize scope entries and same-path/topology function removal or `pub`-to-private
drift rejects `ScopeGraphMismatch` before binding. It resolves only inherited simple `crate::`
structural paths, and applies `ModuleKey` segment visibility before staging aliases. Its
declaration-level public query does not authorize a path: the resolver independently rejects a
public target behind the first non-public structural child. The focused target passes 9/9; this is
test evidence, not proof or client parity. `pub use`, groups/globs, non-`crate` paths,
non-function targets, other namespaces, final interfaces, Core/CPS, Engine, admission, runtime,
and parity remain excluded.

TASK-2068 now delivers a scoped structural import-cycle gate: `partial / tested / below_spec`, with
Type `partial`, Core/CPS/admission-runtime `not_applicable`, verification `partial`, and run-route
impact `prerequisite`. After existing scope-backed structural/visibility preflight, it collects only
cross-module resolved edges, rejects deterministic `CanonicalImportCycle` provenance through an
outer structural error before publishing a plan, omits same-module edges, preserves inaccessible
path precedence, and fails atomically. The generic planner and binder remain unchanged because
they own different grammar. The scope17 target passes its eight cycle witnesses, including a
16-case property; this is test evidence, not proof, lowering, admission, runtime, or parity.

TASK-2068 now delivers the dedicated scope-backed structural binder M-BIND slice: `partial /
tested / below_spec`, Type-only `prerequisite` evidence. The new
`crates/ash-typeck/src/canonical_structural_module_binder.rs`, exposed only through its dedicated
`lib.rs` API, delegates `bind_scoped_structural_parsed_uses(graph, scopes)` to the delivered scoped
resolver and then `into_bound_set`, preserving atomic resolver errors. The existing
`canonical_module_binder.rs` remains unchanged and generic-only; it does not mention scopes, the
scoped resolver, or `CanonicalStructuralImportError`. The focused
`task_2068_scoped_structural_binder` target passes 8/8, including a 16-case property across six
visibility categories; this is test evidence, not proof or parity. The
bounded ordinary-function domain is visibility-permitted rather than public-only: public, crate,
super, `pub(in path)`, inherited/private, and self targets are admitted only for their canonical
`ModuleKey` regions, while public targets retain the whole structural-path fence. It remains
non-authorizing and does not complete the final-interface, Core/CPS, Engine, admission, runtime,
or client-parity clauses.

TASK-2068 now delivers scoped simple ordinary-function imports M-SIMPLE as `partial / tested /
below_spec`, Type-only `prerequisite` evidence. The dedicated
`bind_scoped_simple_ordinary_function_imports(graph, scopes)` API in
`canonical_structural_module_binder.rs`, exported through `lib.rs`, delegates only to
`resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)` and then
`into_bound_set`. It admits inherited root/deep `crate::` ordinary-function imports with optional
`as`; without it, the target's final segment is the natural local name. The same canonical
public/crate/super/`pub(in path)`/inherited-private/self target regions and public
structural-path fence apply. Local collisions, duplicate local bindings, and cycles reject
atomically. The focused `task_2068_scoped_simple_ordinary_function_imports` target passes 11/11,
including a 16-case property and the retained structural-child compatibility regression; this is
test evidence, not proof or parity. The generic
resolver and binder remain unchanged, and no final-interface, Core/CPS, Engine, admission,
runtime, or parity authority is introduced.
TASK-2069 cannot begin until TASK-2073 supplies its complete checked/export-closed handoff.

TASK-2062 has completed its `partial / tested / below_spec` bounded lowering handoff. It consumes
only a TASK-2061-provided finalizer wrapper, resolved import facts, and an already-materialized
Core program; it validates Core, delegates to the checked Core-to-CPS bridge, and emits paired
non-executable, non-authoritative artifacts retaining exact module key/origin and imported defining
identity/origin. TASK-2063 must establish its own sealed dependency-linking/admission input around
TASK-2069's complete public carriers; TASK-2062 issues no admission credential and is not an
admission input.
It does not lower parser source or module bodies, establish full typed imports/callable authority,
prove file/inline real-program parity, link/admit Engine artifacts, or change CLI/daemon paths.

TASK-2069 is the planned owner of those full-body lowering clauses and of the Engine-side
scanner/path-cache retirement-or-fence transport. It receives complete TASK-2073 checked modules,
not raw source or caller-materialized Core, and supplies TASK-2063 one complete non-sealed
canonical-keyed Core/CPS closure.

TASK-2063 is now **In progress** with `not_implemented / none / below_spec` semantic accounting.
Its corrected prerequisite is TASK-2069's complete but non-sealed canonical-keyed Core/CPS closure;
it must independently validate that closure before minting an Engine-sealed linked/admission
request. No such request, admission path, focused test, or runtime evidence exists yet. Raw source,
parser or legacy graphs, loader-private exports, path/string cache facts, and a direct evaluator
remain non-authoritative; TASK-2064 remains the separate owner of real-program file/inline and
CLI/daemon parity.

## Semantic-rule ownership

| Rule | Type | Core | CPS | Admission/runtime | Integration owner |
|---|---|---|---|---|---|
| MOD-REAL-001 AST graph identity | TASK-2067 canonical parsed graph/state machine | TASK-2058 carrier + TASK-2067 graph transport | not applicable | not applicable | TASK-2064 |
| MOD-REAL-002 file/inline parity | TASK-2067 real module-unit transport | TASK-2069 full artifact parity | TASK-2069 full artifact parity | TASK-2063 | TASK-2064 |
| MOD-REAL-003 checked interfaces | TASK-2071 contract -> TASK-2074 expanded graph -> TASK-2075 internal snapshot -> TASK-2073 bodies/private-public/export closure | TASK-2073 checked interface transport | not applicable | non-authorizing | TASK-2064 |
| MOD-REAL-004 import/visibility | TASK-2070 self alias + TASK-2071 contract + TASK-2075 name view -> TASK-2072 complete imports, visibility, cycles, and binder integration | TASK-2069 consumes TASK-2073-finalized binding facts | TASK-2069 preserves facts | prerequisite | TASK-2064 |
| MOD-REAL-005 module lowering | TASK-2073 complete checked definition bodies | TASK-2069 | TASK-2069 | prerequisite for TASK-2063 | TASK-2064 |
| MOD-REAL-006 linked execution | consumes TASK-2073 checked facts | consumes TASK-2069 Core | consumes TASK-2069 CPS | TASK-2063 | TASK-2064 |

## Tasks

| Task | Title | Status | Run-route impact |
|---|---|---|---|
| [TASK-2056](tasks/TASK-2056-module-realization-spec-plan-packet.md) | Create the module realization spec, seam audit, plan, and task packet | Planned — packet authored and verified; implementation activation remains pending | none |
| [TASK-2057](tasks/TASK-2057-ast-driven-module-discovery.md) | Replace semantic module-declaration text scans with AST-driven discovery | Complete — partial/tested/below-spec parser-owned structural handoff; source-anchored missing/cycle diagnostics remain deferred | prerequisite |
| [TASK-2058](tasks/TASK-2058-canonical-module-identity-and-artifacts.md) | Establish canonical module identities and module-unit artifacts | Complete — partial/tested/below-spec Core key/artifact carrier; resolver graph and legacy identity migration remain open | prerequisite |
| [TASK-2059](tasks/TASK-2059-file-inline-module-unit-parity.md) | Build one file/inline source-acquisition and module-unit route | Complete — partial/tested/below-spec bounded parser module-unit handoff; graph/interface/import/lowering/Engine/client clauses remain deferred | prerequisite |
| [TASK-2060](tasks/TASK-2060-checked-module-interface-and-export-closure.md) | Define checked export-closed interfaces and public/private views | Complete — partial/tested/below-spec Core public-interface carrier; TypeEnv finalization, Engine scanner fencing/transport, imports, lowering, and execution remain deferred | prerequisite |
| [TASK-2066](tasks/TASK-2066-typeenv-module-unit-interface-finalization.md) | Finalize a bounded projection from a TypeEnv module unit and declaration preflight | Complete — partial/tested/below-spec staged TypeEnv wrapper with artifact equality; body/full-callable facts, typed linkage, aliases/re-exports, origin projection, and export closure remain open | prerequisite |
| [TASK-2061](tasks/TASK-2061-interface-import-resolution-and-visibility.md) | Resolve bounded checked-interface requests | Complete — partial/tested/below-spec finalizer-wrapper-only explicit/group/glob resolver; parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, binder integration, closure, lowering, Engine transport, and parity remain open | prerequisite |
| [TASK-2062](tasks/TASK-2062-module-aware-core-cps-lowering.md) | Lower resolved modules through Core and CPS with origin preservation | Complete — partial/tested/below-spec wrapper/resolved-binding Core-to-CPS artifacts preserve module/import provenance; parser source/bodies, typed imports, real-program parity, Engine, and clients remain deferred | prerequisite |
| [TASK-2067](tasks/TASK-2067-canonical-module-graph-and-structural-diagnostics.md) | Migrate to a canonical parser-owned module graph/state machine with structural diagnostics | Complete — partial/tested/below-spec canonical parser graph/unit transport, diagnostics, lifecycle reporting, payload parity/mutation, root metadata, and legacy-route fence; downstream layers remain open | prerequisite |
| [TASK-2068](tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md) | Produce the bounded Type-layer module foundation | Complete — partial/tested/below-spec provisional M-COLLECT, bounded imports/cycles/binders, selected M-CHECK leaves, and direct re-export/provider fragments only. Its preserved evidence is non-authorizing; every unresolved clause has moved to TASK-2070–2075. | prerequisite |
| [TASK-2070](tasks/TASK-2070-scoped-self-simple-function-aliases.md) | Resolve the bounded direct same-module self alias leaf | Complete — partial/tested/below-spec dedicated no-edge self aliases with eight tested M-SELF witnesses; non-authorizing handoff to TASK-2072 | prerequisite |
| [TASK-2071](tasks/TASK-2071-module-namespace-and-provisional-view-contract.md) | Define namespace, collision, syntax-prepass, and two-view collection contracts | Complete — specification handoff; not_implemented/none/below-spec | prerequisite |
| [TASK-2074](tasks/TASK-2074-canonical-expanded-module-graph.md) | Build the AST-only syntax prepass and canonical expanded graph | In progress — partial/tested/below-spec bounded public-macro syntax prepass and shallow graph; approved parenthesized exact-pattern notation imports, canonical public full-key transport, syntax-table activation, and their deferred evidence remain; generalized mixfix use-site parsing/elaboration is not owned here | prerequisite |
| [TASK-2075](tasks/TASK-2075-two-tier-complete-module-collection.md) | Collect internal snapshots and name-only provisional views | Planned — not_implemented/none/below-spec backlog owner | prerequisite |
| [TASK-2072](tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md) | Resolve all parsed imports from the name-only view and publish atomic bindings | Planned — partial/none/below-spec backlog owner | prerequisite |
| [TASK-2073](tasks/TASK-2073-checked-module-finalization-and-export-closure.md) | Check internal snapshots plus staged bindings and publish export-closed final interfaces | Planned — partial/none/below-spec backlog owner; TASK-2069’s sole complete Type input | prerequisite |
| [TASK-2069](tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md) | Lower complete checked module bodies and fence Engine scanner/cache transport | Planned — consumes TASK-2073's complete checked handoff | prerequisite |
| [TASK-2063](tasks/TASK-2063-engine-linked-module-admission.md) | Link reachable modules and admit one Engine artifact | In progress — not_implemented/none/below-spec; awaits TASK-2069's complete non-sealed closure before an Engine-sealed request can exist | active |
| [TASK-2064](tasks/TASK-2064-module-conformance-and-client-parity.md) | Prove module conformance, mutation resistance, and CLI/daemon parity | Planned | active |
| [TASK-2065](tasks/TASK-2065-module-realization-closeout.md) | Close the phase with review, traceability, documentation, and full gates | Planned | none |

## Phase evidence policy

Each implementation task starts only after it is promoted to **In progress**, linked to the `MOD-REAL-*` coverage row it owns, and given an active semantic-task record with focused commands. A completed handoff does not establish complete-feature parity. The phase reports `implemented` only after every SPEC-103 clause has implementation and evidence; until then every incomplete rule remains `partial` and `below_spec`.

TASK-2064 owns cross-layer conformance. It compares the same admitted source tree, inputs, module identities, and run-control envelope through CLI and daemon, then compares normalized terminal results. It must reject any direct-evaluator fallback.

TASK-2067 is **Complete** for its partial parser handoff and retains its semantic-task record and
traceability evidence. Its focused targets cover the complete task-owned structural and transport
clauses, not any downstream interface, lowering, admission, or client behavior. TASK-2068 is
**Complete** for its `partial / tested / below_spec` bounded parser-graph Type
slice: provisional function M-COLLECT, simple alias target lookup, graph-wide-preflighted closed
primitive M-CHECK sibling body checking, and root-plus-plan-selected-direct-provider checking with
non-authorizing import facts. M-CHECK publishes only a private checked map and a
constructor-free non-authorizing `CanonicalPublicFunctionInterface`; it neither publishes a final
interface nor establishes full import/cycle binding, Core/CPS, Engine, file/inline, or CLI/daemon
parity. Its direct-public primitive re-export interface-fragment is `partial / tested /
below_spec`: the exact `pub mod api` plus root `pub use crate::api::greet as welcome` form
retains public structural-path and public-target closure, identity/origin/signature/declaration/use
provenance, no implicit flattening, atomicity, and no authority. Its focused target passes 13/13,
including a 16-case property; the evidence is neither proof nor final-interface or parity
evidence. Its private-provider-helper companion is also `partial / tested / below_spec`: it
checks helpers atomically without exposing them and has focused 7/7 evidence including a 16-case
property. Its direct local-binding root-client companion is also `partial / tested / below_spec`:
it checks inherited/private `internal_entry` through `welcome` using a distinct opaque direct plan,
passes 10/10 including a 16-case property, and remains non-authorizing. The local-call diagnostic
uses only a direct unqualified alias call (including an empty block tail), falling back to the root
body span. TASK-2068 also delivers the scoped grouped ordinary-function import M-GROUP slice as
`partial / tested / below_spec` Type-layer prerequisite evidence: `UseItem` preserves exact nested
member spans; only inherited `crate`/structural-child grouped ordinary-function members with an
optional alias or natural name are accepted; and snapshot, visibility, local-collision,
duplicate-binding, and full-cycle checks reject atomically before a plan or binding set returns.
The dedicated grouped resolver/binder leaves generic `canonical_module_binder.rs` unchanged. The
focused grouped target passes 10/10, including a 16-case property, and the parser full suite
passes; this is not proof, a final interface, Core/CPS, Engine, admission/runtime, or client
parity. The scoped-simple compatibility target is now 11/11. TASK-2071 supplies the completed
contract; TASK-2074 owns expansion, TASK-2075 collection, TASK-2072 parsed binding, and TASK-2073
final checked/export-closed publication.
TASK-2069 remains **Planned** and must be fully recorded before its first semantic Rust change.
TASK-2064 does not absorb any of their implementation authority.

The delivered M-SUPER sub-slice is `partial / tested / below_spec` Type-only prerequisite
evidence. Its dedicated resolver and binding-only projection accept only inherited simple
parent/sibling ordinary-function imports from a non-root module with exactly one leading `super`,
starting from `ModuleKey::parent()` and retaining the whole parsed `Use::span`. Scope snapshot,
visibility and whole-public-path, local-collision, duplicate, cycle, and atomic-publication rules
remain in force; every extra or final `super` rejects before lookup. The focused target passes
12/12 including a 16-case property. This is neither proof, a final interface, generic binder
authority, Core/CPS, Engine, admission/runtime, nor client-parity evidence. `self`, root/repeated
`super`, groups/globs, re-exports, non-function namespaces, and all remaining routes remain open.

The delivered M-SUPER-GROUP sub-slice is `partial / tested / below_spec` Type-only
`prerequisite` evidence. Its dedicated resolver/binder admits only inherited non-root
`UsePath::Nested` imports with exactly one leading `super`, no outer alias, zero or more structural
children after `ModuleKey::parent()`, and a nonempty group of ordinary-function members using a
natural or member-`as` local name. It preserves parser-owned individual member spans in each
identity/edge/member-specific error, preflights a final member named `super` before lookup, and
reuses canonical scope snapshots, visibility/whole-public-path, same-module-no-edge, collision,
duplicate, cycle, and atomic-publication rules. The focused target passes 13/13 including a
16-case property. The ten canonical witnesses are tested: POSITIVE, IDENTITY,
FILE-INLINE-PARITY, and PROPERTY are positive evidence; VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC,
LOCAL-COLLISION, DUPLICATE-BINDING, and AUTHORITY-FENCE are negative evidence; and
CYCLE-ATOMICITY is mutation evidence. Root/repeated `super`, `self`, `crate`,
unprefixed/standard-library/external, simple/glob/non-nested/nested groups, outer aliases,
public/restricted/re-export uses, nonfunctions or other namespaces, generic resolver/binder
changes, final interfaces, Core/CPS, Engine, admission/runtime, parity, and precedence remain
deferred. This is test evidence, not proof or parity evidence; TASK-2068 is complete for its
delivered foundation, TASK-2074/TASK-2075/TASK-2072/TASK-2073 own the outstanding implementation
clauses after TASK-2071's completed contract,
TASK-2069 remains planned, and TASK-2064 owns integration parity.

The delivered M-GLOB sub-slice is `partial / tested / below_spec`: one inherited
`use crate::<public structural-child>...::*` ordinary-function route in an importer with exactly
one `use` and zero local ordinary functions. Its dedicated Type-only resolver/binding projection
retains identity, declaration origin/span/visibility, the complete `Use::span`, and one
cross-module edge per selected public function before atomically publishing a plan/bound set. It
does not decide local/explicit/glob precedence. A local function, second glob, and cycle-shaped
attempt are `Unsupported` boundaries that publish no plan or binding set; the three corresponding
CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY IDs are boundary mutation evidence,
not claims for local-collision, duplicate-binding, generic ambiguity, or `ImportCycle`. The
15-representation shape matrix distinguishes a leading `::` (not `UsePath::Glob`) from a private
structural module (`Inaccessible`). All ten M-GLOB witnesses are tested, including a 16-case
property across depth, function count, function/path visibility, and file/inline form. This is
not proof, a final interface, generic-binder authority, Core/CPS, Engine, admission/runtime, or
client parity. Type and verification are `partial`; Core/CPS/admission-runtime are
`not_applicable`; run-route impact is `prerequisite` for TASK-2069. Remaining forms stay deferred.

The delivered M-GLOB-LOCAL-PRECEDENCE sub-slice is partial / tested / below_spec. It admits
exactly one existing inherited public structural-child crate glob: same-module ordinary functions
shadow same-name selected public imports only in returned public bindings, non-colliding imports
bind, and every selected cross-module edge survives shadowing and is cycle-checked before
filtering. All-shadowed input succeeds with no import bindings but retained edges; actual hidden
cycles return atomic ImportCycle. The focused target passes 8/8, including a 16-case property
varying names, collision subsets, source form, and depth 1–3. File/inline evidence is normalized
Type-layer scope/binding parity only, not final/runtime parity. It uses canonical
graph/provisional scopes only, never private M-CHECK facts; existing M-GLOB behavior remains
separate/rejecting; other imports, multiple globs, aliases/re-exports, self/super/non-crate
paths, nonfunctions, the generic binder, final interfaces, Core/CPS, Engine, admission/runtime,
and parity stay excluded. This is a non-authorizing Type handoff; TASK-2069 owns lowering and
TASK-2064 owns parity.

The delivered M-SIMPLE-LOCAL-PRECEDENCE sub-slice is partial / tested / below_spec. It admits one
dedicated inherited, unaliased `UsePath::Simple`
`use crate::<public structural-child>...::<public ordinary-function>;` route: a selected
cross-module target retains its edge and cycle-checks before a same-module ordinary function
filters a same-name natural import binding. A selected same-module target emits no self-edge and
does not participate in cycle detection. Non-colliding imports bind, all shadowed cross-module
candidates retain edges with no import binding, and real hidden two-module cross-module cycles
reject atomically. The existing M-SIMPLE route remains unchanged and keeps local-collision
rejection. The focused `task_2068_local_over_simple_precedence` target passes 9/9, including
Type-only normalized file/inline scope/binding parity and the 16-case depth 1–3/name/collision-
mask/source-form property. Canonical graph/provisional scopes are the only authority; private
M-CHECK facts, generic binders, other import forms, final interfaces, and later layers remain
excluded. Planner fingerprint:
`sha256:7fb241da5b3bf35595e7cf3054f06dcbc9c9dc08dc9701c047d0d2c045a393d3`; TASK-2069 owns
lowering; TASK-2064 owns parity.

The delivered TASK-2070 M-SELF-SIMPLE-ALIAS sub-slice is `partial / tested / below_spec`. It admits zero or
more individually eligible inherited, two-segment `UsePath::Simple`
`use self::<ordinary_function> as <different_alias>;` statements per root or nested module; a
module with none produces an empty dedicated result, and groups, globs, mixed imports, or any other
form are `Unsupported`. A direct `self::<child_module>` target is likewise a nonfunction
`Unsupported`, not a traversal route. The dedicated resolver selects only direct same-`ModuleKey`
ordinary functions when `is_visible_from` permits the importer, stages distinct aliases together,
reports duplicate aliases as `DuplicateBinding`, and retains identity/provenance/visibility plus
full `use_span` in each `CanonicalSelfOrdinaryFunctionAliasBinding`. It returns
`CanonicalResolvedSelfOrdinaryFunctionAliases` with no `import_edges` field; only the dedicated
binder calls its private `into_bound_alias_set` to return
`CanonicalBoundSelfOrdinaryFunctionAliasSet`, never `CanonicalResolvedSimpleImports` or
`CanonicalBoundModuleSet`. Resolver and binder use shared `CanonicalStructuralImportError`, and
`ImportCycle` is unreachable by construction and source fence. All shape, visibility,
local-collision, and sibling-failure cases remain atomic. `CanonicalBoundModuleBinding` and the
generic binder stay unchanged. Its implementation node and eight test witnesses are
implemented/tested; the focused target passes 8/8, including the exact 16-case property with alias
count `1..3`. This is Type-only, non-authorizing prerequisite evidence: no private M-CHECK facts,
generic-binder change, cross-module traversal, final interface, or later-layer authority;
TASK-2072 owns complete imports/binding, TASK-2069 owns lowering, and TASK-2064 owns parity.

The delivered `M-CHECK-RESTRICTED-VISIBILITY` leaf is `partial / tested / below_spec`, Type
`partial`, Core/CPS/admission-runtime `not_applicable`, verification `partial`, and run-route
impact `prerequisite`. It accepts only `pub(crate)`, `pub(super)`, `pub(in crate)` or
`pub(in crate::...)`, and `pub(self)` on primitive closed ordinary-function leaves in a file-root
closed leaf with no imports, children, nonfunctions, generics, contracts, or open signatures. The
checker graph-preflights, stages sibling signatures, and body-checks atomically; restricted checked
functions remain in `private_functions` with identity/origin/spans/visibility/signature/body facts,
while the public projection remains only `Visibility::Public`. `pub(in self::internal)` is rejected.
The focused target passes 18/18. `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY`
is a tested source-form boundary—file-root success versus inline-child/module rejection before
projection—not normalized-success file/inline parity. This authorizes no imports,
binder/re-export/final-interface, Core/CPS, admission/runtime, or parity. TASK-2069 remains the
later lowering/transport consumer, and TASK-2064 owns integration parity.

## Global verification

```text
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-core
cargo test -p ash-typeck
cargo test -p ash-engine
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

The closeout task adds exact focused module conformance commands once TASK-2057 through TASK-2064 create their test targets.
