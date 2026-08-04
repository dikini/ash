# AUDIT-207: Module Realization Seams

**Status:** Complete — planning baseline and seam inventory reconciled through the bounded TASK-2057 through TASK-2062 and TASK-2066 handoffs, plus the completed TASK-2067 partial/tested/below-spec parser graph handoff; TASK-2068 has a partial/tested/below-spec provisional-function collector, bounded graph-only simple-import edge/cycle planner with binder delegation, tested graph-delivered primitive-function M-CHECK leaf slice, direct primitive provider/client checker with graph-wide completeness, and tested direct-public, private-provider-helper, local-binding root-client primitive re-export interface, canonical provisional-module-scope/structural-path visibility, scoped structural import-cycle, dedicated scoped structural binder, and scoped simple ordinary-function import fragments. The root-client fragment is partial/tested/below-spec: it checks an inherited/private root function through the explicit public alias with a distinct opaque direct plan, passes 10/10 including a 16-case property, anchors only direct unqualified alias calls (or else the root body), and leaves generic `pub use` routes rejected. The canonical scope fragment is partial/tested/below-spec Type-only evidence (9/9): it rebuilds and compares declaration snapshots against current graph units so artifacts alone cannot authorize entries, rejects drift as `ScopeGraphMismatch`, and uses canonical visibility regions for ordinary-function targets; public targets retain whole-path fencing. The scoped cycle gate is partial/tested/below-spec Type-only `prerequisite` evidence (scope17, including a 16-case property): it collects only post-preflight cross-module edges, preserves structural diagnostics over `CanonicalImportCycle`, rejects atomically, and leaves the generic binder unchanged. The dedicated scope-backed structural binder is partial/tested/below-spec Type-only prerequisite evidence (8/8, including a 16-case property across six visibility categories): new `canonical_structural_module_binder.rs`, exposed only through its `lib.rs` API, delegates only to that scoped resolver and projects a binding-only set atomically; the existing generic `canonical_module_binder.rs` remains unchanged and does not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. The scoped simple ordinary-function import fragment is partial/tested/below-spec Type-only prerequisite evidence: the same dedicated binder module delegates inherited root/deep `crate::` ordinary-function imports, with optional aliases and natural final names, to `resolve_scoped_simple_ordinary_function_imports_with_scopes`; local collisions, duplicates, visibility failures, and cycles remain atomic errors, while the generic resolver/binder stay unchanged. Its focused target passes 11/11, including a 16-case property across all canonical visibility regions and root/deep, explicit-alias/natural-name positions; this is test evidence, not proof or parity. All delivered fragments are non-authorizing; they have no Core/CPS/admission/runtime or client-parity claim. TASK-2069 remains planned, and no phase-completion claim is made.

**M-GROUP evidence update:** TASK-2068 now also delivers `partial / tested / below_spec`
parser-to-binder evidence for inherited grouped `crate` ordinary-function imports. Each nested
member carries its own parser span; the dedicated scoped resolver and binder preserve that span in
identity, edge, and diagnostic facts, reject snapshots/visibility/collisions/duplicates/whole-group
cycles atomically, and do not alter the generic binder. The grouped target passes 10/10 including
a 16-case property, while the scoped-simple compatibility target now passes 11/11. This is Type
only, non-authorizing test evidence; final interfaces, remaining import forms, Core/CPS, Engine,
admission/runtime, and client parity remain open.

**M-SUPER evidence update:** TASK-2068 now also delivers `partial / tested / below_spec`
Type-only `prerequisite` evidence for inherited simple parent/sibling ordinary-function imports
that begin with exactly one `super`. The dedicated resolver starts at `ModuleKey::parent()` and
the dedicated binder only projects its successful result. Both retain the complete parser
`Use::span` in identity, edge, and error facts, preserve snapshot, visibility/whole-public-path,
collision, duplicate, cycle, and atomicity fences, and leave the generic binder unchanged.
Repeated `super` in any child or final function segment rejects before lookup; the final-`super`
callable test reinforces that root/repeated boundary. The focused target passes 12/12 including a
16-case property. Root, `self`, `crate`, unprefixed/stdlib/external, group/glob,
re-export/restricted, and non-function routes stay excluded. This is source-backed test evidence,
not proof, final-interface, Core/CPS, Engine, admission/runtime, or client-parity evidence.

**M-SUPER-GROUP delivered seam:** TASK-2068 now has `partial / tested / below_spec` Type-only
`prerequisite` evidence for inherited non-root `UsePath::Nested` grouped ordinary-function imports
with exactly one leading `super`, no outer alias, zero or more structural children after the
canonical parent, and a nonempty natural/member-alias ordinary-function group. It retains each
parser-owned member span in its identity, edge, and member-specific error facts; preflights a
final member named `super` before lookup; and reuses canonical scope snapshots, visibility and
whole-public-path, same-module-no-edge, collision, duplicate, cycle, and atomic-publication
fences. The focused target passes 13/13 including a 16-case property. The ten canonical trace
nodes are tested: POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY are positive evidence;
VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and AUTHORITY-FENCE
are negative evidence; CYCLE-ATOMICITY is mutation evidence. Root/repeated `super`, `self`,
`crate`, unprefixed/external, simple/glob/non-nested/nested groups, public/restricted/re-export
forms, nonfunctions, generic binder authority, final interfaces, later layers, and precedence
remain unapproved. This is test evidence, not proof or parity evidence.

**M-GLOB delivered evidence:** TASK-2068 records a `partial / tested / below_spec` Type-only
`prerequisite` slice for exactly one inherited
`use crate::<public structural-child>...::*` ordinary-function import in a module with exactly one
`use` and zero local ordinary functions. The dedicated resolver/binder preserves function
identity/origin/visibility, declaration and full-use spans, and one edge per selected public
function before atomically producing a plan/bound set. The 15-valid-representation shape matrix
treats leading `::` as not `UsePath::Glob`; private structural-module access is an `Inaccessible`
visibility witness. Local-function, second-glob, and cycle-shaped attempts are boundary
`Unsupported` failures with no plan/bindings: CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and
CYCLE-ATOMICITY are mutation evidence only, not local-collision, duplicate-binding, generic
ambiguity, or `ImportCycle` evidence. All ten focused witnesses are tested, including a 16-case
property. This is not proof, a final interface, generic-binder authority, or later-layer/client
authority; all remaining import forms remain deferred.

**M-GLOB local-over-glob delivered seam:** The TASK-2068 slice is partial / tested / below_spec.
Within exactly one inherited public structural-child crate glob, same-module ordinary functions
shadow same-name selected public imports only in returned public bindings while non-colliding
imports bind. Every selected cross-module target, including a shadowed one, retains its edge
through canonical cycle detection before filtering; all-shadowed input succeeds with no import
bindings but retained edges, and hidden cycles return atomic ImportCycle. The focused target
passes 8/8, including a 16-case property varying names, collision subsets, source form, and
depth 1–3. File/inline establishes normalized Type-layer scope/binding parity only. The slice
consumes canonical graph/provisional scopes only, never private M-CHECK facts; existing M-GLOB
behavior remains separate/rejecting; other imports, multiple globs, aliases/re-exports,
self/super/non-crate paths, nonfunctions, generic binder authority, final interfaces, Core/CPS,
Engine, admission/runtime, and parity remain excluded. The handoff is Type-only and
non-authorizing; TASK-2069 owns lowering and TASK-2064 owns parity.

**M-SIMPLE local-over-explicit delivered seam:** The TASK-2068 slice is partial / tested /
below_spec. It covers exactly one inherited, unaliased public structural-child `UsePath::Simple`
crate import with its natural binding name. The dedicated resolver retains edges only for selected
cross-module targets, cycle-checks those edges before filtering a same-name local ordinary
function, and only then publishes bindings. A selected same-module target emits no self-edge and
does not participate in cycle detection. Non-colliding imports bind, all shadowed cross-module
candidates retain edges with no import binding, and real hidden two-module cross-module cycles
reject atomically. The existing M-SIMPLE route stays unchanged and continues to reject local
collisions. It uses only canonical graph/provisional scopes, not private M-CHECK facts or generic
binder authority. The focused target passes 9/9; the file/inline witness is Type-only normalized
scope/binding parity. TASK-2069 owns lowering and TASK-2064 owns parity.

**M-SELF-SIMPLE-ALIAS delivered seam:** TASK-2070 completed a `partial / tested / below_spec` Type-only
prerequisite for zero or more individually eligible inherited, two-segment `UsePath::Simple`
`use self::<ordinary_function> as <different_alias>;` statements in root or nested modules. It
selects only direct same-module ordinary functions when `is_visible_from` permits the
importer, preserve identity/provenance/visibility and full `use_span` in each dedicated alias
binding, and allow distinct aliases while reporting duplicates as `DuplicateBinding`. The resolved
`CanonicalResolvedSelfOrdinaryFunctionAliases` has no `import_edges` field; only the binder calls
its private `into_bound_alias_set` to return `CanonicalBoundSelfOrdinaryFunctionAliasSet`, never
`CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`. Resolver and binder share
`CanonicalStructuralImportError`, with `ImportCycle` unreachable by construction and source fence.
They emit no canonical import edge and run no cycle detection. Groups, globs, mixed/other forms,
direct child-module/nonfunction targets, shape, visibility, local-collision, and
valid-sibling/failing-module boundaries are atomic. `CanonicalBoundModuleBinding` and the
generic binder remain unchanged. The implementation node and eight witnesses are promoted after
the focused target passed 8/8, including the exact 16-case property with alias count `1..3`;
M-CHECK private fact authority, cross-module traversal, final interfaces, Core/CPS, Engine,
admission/runtime, and parity remain excluded. TASK-2072 owns complete imports/binding, TASK-2073
owns finalization/export closure, TASK-2069 owns lowering, and TASK-2064 owns parity.

**TASK-2071 contract and task-split update:** TASK-2071 is complete as a specification handoff with
`not_implemented / none / below_spec` implementation axes. SPEC-103 now separates the AST-only
syntax prepass and `CanonicalExpandedModuleGraph` (planned TASK-2074) from two-tier collection
(planned TASK-2075). The internal `CanonicalCollectedModuleSnapshot` may retain raw
declaration/callable/body/member/order/expansion facts but no checked results. The import-facing
`CanonicalProvisionalNameView` retains only name/lookup, identity/key, namespace,
visibility/exportability, origin/anchor, and ordinal facts. TASK-2072 consumes only the latter;
TASK-2073 consumes the former plus TASK-2072 staging. No new Rust or runtime evidence exists.

**M-CHECK restricted-visibility evidence:** TASK-2068 delivers the bounded
`M-CHECK-RESTRICTED-VISIBILITY` leaf as `partial / tested / below_spec`. It accepts only
`pub(crate)`, `pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` declarations
in a file-root primitive closed ordinary-function leaf domain; `pub(in self::internal)` rejects.
Graph preflight, sibling-signature staging, and body checking remain atomic; checked restricted
functions retain identity, origin, spans, visibility, signature, and body facts only in
`private_functions`, while the public projection remains limited to `Visibility::Public`. The
focused target passes 18/18 and its implementation node plus eleven canonical witness nodes are
implemented/tested. `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY` is a
source-form boundary—file-root success versus inline rejection before projection, not
normalized-success parity. Imports, binder/re-export/final-interface, Core/CPS, admission/runtime,
and parity authority remain excluded; TASK-2069 later consumes complete Type facts and TASK-2064
owns integration parity.
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Target rule:** [SPEC-103](../../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)

## Finding

Ash has useful module fragments but not one complete module realization. The parser accepts file-backed and inline declarations. The resolver builds a file graph. The engine transports selected summaries and imports. The typechecker has a binder. These fragments do not yet share one AST-driven route or prove file/inline parity through Engine execution.

## Live seam inventory

| Layer | Existing seam | Required replacement or extension | Owner |
|---|---|---|---|
| Surface parsing | `crates/ash-parser/src/parse_module.rs` parses `ModuleFile` and `ModuleDecl` variants | Preserve one parsed declaration carrier and make every downstream graph edge originate from it | TASK-2057 |
| File graph | `crates/ash-parser/src/resolver.rs` consumes parsed `ModuleFile` declarations through public `ash_parser::discover_module_declarations`; `ModuleUnitResolver` now acquires a single file/inline parser unit with retained origins | TASK-2057 completed AST discovery; TASK-2058 supplies the carrier; TASK-2059 completed the source-acquisition handoff without changing the legacy graph | TASK-2057, TASK-2058, TASK-2059 |
| Graph identity and expansion | `crates/ash-parser/src/canonical_module_graph.rs::CanonicalModuleGraphResolver` publishes an all-or-nothing parser graph keyed by `ModuleKey`, with real acquired file/inline `ModuleUnit` values, root metadata, AST-only edges, complete lifecycle reporting, and anchored structural diagnostics. No canonical expanded graph exists. | TASK-2074 consumes/owns the graph, performs the AST-only syntax prepass, shallowly expands every keyed `ModuleBody`, retains uses/order/per-key sidecars, and publishes an exact one-to-one map without Engine/FS/text authority. | TASK-2067 (Complete parsed graph), TASK-2074 (planned expansion) |
| Imports | `crates/ash-parser/src/import_resolver.rs` and Engine loader have distinct import paths; TASK-2061 adds only an in-memory explicit/group/glob resolver over checked wrappers | TASK-2068 preserves its bounded tested planners/binders. TASK-2070 completed only the direct self alias; TASK-2072 owns complete parsed grammar, edge/cycle/precedence/duplicate semantics and staged `pub use`; TASK-2073 alone finalizes exports before TASK-2069 fences Engine readers. | TASK-2070 (Complete partial handoff), TASK-2072/TASK-2073 (planned), TASK-2069 (planned) |
| Collection and binding | `canonical_provisional_module_scopes.rs` collects only structural children and ordinary functions for bounded TASK-2068/TASK-2070 routes; the dedicated binder leaves preserve bounded evidence. No exhaustive internal snapshot or minimal name-only view exists. | Preserve the compatibility facade. TASK-2075 consumes TASK-2074 and atomically builds separate internal/name views; TASK-2072 consumes only the name view; TASK-2073 consumes the internal snapshot plus staged bindings. | TASK-2071 (Complete contract), TASK-2075/TASK-2072/TASK-2073 (planned) |
| Summaries | TASK-2060 adds the V1 Core `PublicModuleInterface` schema with public binding validation and V1--V8 summary compatibility; TASK-2066 adds a bounded wrapper; TASK-2061 stores only that wrapper. Typed summary identities, aliases/re-exports, source origins, and complete closure remain unlinked and Engine-private metadata/scanners are unchanged | TASK-2073 completes typed namespace linkage/export closure in the Type layer, then TASK-2069 transports only checked artifacts while retiring/fencing Engine scanners | TASK-2068 (Complete foundation), TASK-2073 (planned), TASK-2069 (planned) |
| Inline modules | Parser stores ordered `ModuleBody` values and TASK-2067's canonical graph retains acquired inline `ModuleUnit` values without filesystem access. It proves complete ordered payload parity and payload-only mutation at the parser graph boundary; its transient file-source reentrancy guard remains active across inline resolution, while Engine has explicit unsupported-inline guards. | TASK-2073 proves Type-layer final-interface parity, then TASK-2069 transports full artifacts without a source-form branch after acquisition. | TASK-2068 (Complete foundation), TASK-2073 (planned), TASK-2069 (planned) |
| Lowering | TASK-2062 adds `ash_typeck::module_core_cps_lowering` over TASK-2061 facts and `ash_core::module_lowering` non-executable carriers | Lower complete checked definition bodies, preserve identities/origins, and transport the full non-sealed closure without parser/source rediscovery or callable authority | TASK-2069 (planned); TASK-2063 consumes it |
| Entry/runtime | PLAN-203 requires one Engine-owned route; TASK-2063 is active but has produced no linked/admission request | TASK-2063 must first receive TASK-2069's complete non-sealed closure and seal it before admission; TASK-2064 then proves CLI/daemon terminal parity | TASK-2069, TASK-2063, TASK-2064 |

## Retired semantic authority

The following are not valid semantic authorities for the completed system:

- line-oriented discovery of `mod name;` after a `ModuleFile` has been parsed;
- snippet extraction of declarations, exports, or imports from source text;
- Engine-private export tables as the definition of language visibility;
- filesystem walking initiated by `use` rather than a resolved module identity;
- a direct evaluator fallback when a linked checked module artifact is absent.

A compatibility seam may persist during migration only if it compares against the AST result, fails closed on disagreement, and cannot publish semantic facts.

## Complete semantic-scanner inventory and retirement gate

This inventory is the planning baseline for the prohibition in SPEC-103 §5. It covers every
known production path that currently discovers a module/import/export fact from source text after
an authoritative parse is available. TASK-2057's completed denylist remains binding; planned
TASK-2069 owns Engine/synthesized-runner scanner and path-cache retirement or fencing, while
TASK-2074, TASK-2075, TASK-2072, and TASK-2073 own the remaining expansion/import/interface
authority after TASK-2071's completed contract. These tasks must update this table whenever a
call site is removed, replaced, or newly found.

| Scanner or text-derived seam | Current production caller/authority | Replacement owner | Completion criterion |
|---|---|---|---|
| `ModuleResolver::parse_module_decls` in `crates/ash-parser/src/resolver.rs` | **Removed by TASK-2057.** Repository search finds no production definition or caller. `ash_parser::discover_module_declarations` now derives file and inline structural declarations only from `ModuleFile`/`ModuleDecl`; the focused target proves lookalikes cannot create edges. | TASK-2057 complete; TASK-2058/2059 consume the handoff | Keep the resolver declaration-scan denylist closed: no line scan, `find("mod ")`, or equivalent text matcher may publish a graph edge or structural child fact. |
| `strip_module_metadata_non_definition_lines` in `crates/ash-engine/src/module_loader.rs` | Masks `use`/`mod` before selected metadata lowering. **Unchanged by TASK-2060 and TASK-2066's bounded handoffs.** | TASK-2069 (planned) | Expanded parsed module items must feed TASK-2073's complete checked-interface route; no masked text may become semantic input. |
| `crates/ash-cli/src/test_runner/synthesized.rs::strip_synthesized_metadata_non_definition_lines` | **Unapproved production semantic-source scanner, quarantined to synthesized-runner introspection.** `build_runner_introspection_snapshot` reads source, line-strips `use`/`pub use` and `mod`/`pub mod`, then parses only the filtered text for `RunnerIntrospectionSnapshot`; it has no graph, binding, interface, lowering, admission, or execution authority. TASK-2060 and TASK-2066 do not remove or fence it. | TASK-2069 (planned) | Replace or fence the preprocessing with the authoritative AST/module-unit carrier. Until then, preserve this quarantine classification; it is not removed or authorized by TASK-2057 through TASK-2068. |
| leading-import line accumulation and `import_needs_more_lines` in `crates/ash-engine/src/module_loader.rs` | Builds Engine loader import prelude from text | TASK-2072 supplies parsed binding authority; TASK-2069 retires/fences the reader | Parsed `use` items and final-interface bindings replace the prelude reader. |
| `source_scan::{extract_pub_mod_declarations, extract_semicolon_snippets}` | Finds public modules/imports/exports by prefixes and snippets. **Unchanged by TASK-2060 and TASK-2066's bounded handoffs.** | TASK-2073 supplies complete interface facts; TASK-2069 retires/fences transport use | Export and re-export facts must come only from expanded AST and a complete checked-interface route. |
| `collect_module_exports` text supplements in `crates/ash-engine/src/module_loader.rs` | Adds public capabilities, builtins, functions, child modules, imports, and re-exports after partial parsing. **Unchanged by TASK-2060 and TASK-2066.** | TASK-2069 (planned) | TASK-2073's complete checked interface must collect all supported exported namespaces; no supplement scan may be semantic authority. |
| path/string-keyed Engine module caches and raw path walking | Selects import/export identity by filesystem/path strings; the new Core `ModuleKey` is not yet consumed here | TASK-2069 (planned) | Canonical `ModuleKey` and checked interface/artifact identity are the sole semantic/cache keys before TASK-2063 seals admission. |

TASK-2065 must run a repository-wide scanner denylist/allowlist check. Any remaining raw scanner
must be explicitly listed as test-only or disagreement-only, must fail closed on disagreement, and
must have no path to graph construction, binding, interface publication, lowering, admission, or
execution. An unclassified production scanner blocks phase closeout.

TASK-2067 is complete for its partial/tested/below-spec semantic handoff and has traceability
witnesses for the canonical parser graph, actual units, complete lifecycle states, structural
diagnostics, inline-safe source reentrancy, graph-key rewrite, ordered payload parity/mutation,
root metadata, and an isolated legacy-route fence. Its error reports carry `Failed` canonical keys
and no failed resolution returns a partial graph. TASK-2068 now has tested provisional function
collection and simple parsed `crate::…` alias binding evidence, including anchored inaccessible
diagnostics, 16 generated alias identities, and a canonical-source fence. Public use, re-exports,
and non-inherited use visibility reject before publication; `pub(crate)`, `pub(super)`,
`pub(self)`, and `pub(in …)` target declarations reject as anchored `Unsupported`. These are
fail-closed boundaries, not visibility implementation. The returned set has no `Default` or public
constructor, so callers cannot fabricate a success. M-CHECK is now tested: it graph-preflights
self-contained leaf units of ordinary public/inherited primitive functions, stages sibling
signatures, atomically checks bodies through the builtin TypeEnv checker, and produces only a
fresh checked identity/private checked-function map plus non-authorizing
`CanonicalPublicFunctionInterface`. Its focused target passes 8/8, including 16 generated public
integer functions and mismatch, closed-signature, use-shape, nested-child global-preflight,
atomicity, and architecture-fence evidence. It excludes imports, nested/child modules, other definitions, generics, contracts,
unsupported visibility, user-defined types, interfaces, effects, re-exports, final full
interfaces, Core/CPS/Engine, and clients. Final interfaces, complete import/cycle binding,
lowering, transport, and Engine behavior remain open; TASK-2069 is not activated by this audit
update.

Before broader import semantics, TASK-2068 has delivered a graph-only simple-import planner
limited to inherited `UsePath::Simple` crate-root function aliases over real graph units and
provisional targets. It publishes opaque resolved import/binding facts plus canonical
importer/defining-identity/local-spelling/use-span/declaration-span/origin/visibility edges,
without an edge for same-module aliases. It rejects every discovered cycle before any result as
`ImportCycle { edges: CanonicalImportCycle }`, whose ordered wrapper exposes parser-anchored
edges, and `bind_simple_parsed_uses` delegates through it so no bind path bypasses planning. The
focused evidence passes 11/11, covering edge provenance, same-module no-edge, file/inline
two-node cycle ordering, a full-provenance `a → b → c → b` tail diagnostic that reports only
`b ↔ c`, late-backedge atomicity, and the delegation fence's `RawCoreProgram`/`CoreExpr`/
`CpsProgram` exclusions. It excludes checked interfaces, TypeEnv/body integration,
legacy/TASK-2060/2061/2066 authority, restricted visibility, `pub use`, re-exports, groups, globs,
qualified paths, Core/CPS/Engine, and clients.

TASK-2068 also delivers a bounded direct primitive provider/client check over a canonical graph and
its resolved simple-import plan. It admits only the root plus plan-selected direct provider leaves.
Before provider checking, graph-wide `module_units()` completeness rejects every unrelated
unselected non-root unit, including a nested module; a descendant of a selected provider instead
reaches the existing provider-leaf precheck and rejects as anchored
`UnsupportedProviderShape`. The check requires exact plan/graph artifacts, primitive provider
precheck, edge revalidation against checked public providers, and fresh-root imported-signature
injection before atomically returning non-authorizing checked root/provider/import facts. Its
focused evidence passes 12/12, including the 16-case property, global-topology-before-malformed-
provider ordering, and atomicity evidence. It neither establishes a final interface nor authorizes
general imports/binding, Core/CPS, Engine, admission, runtime, or client parity.

### TASK-2057 resolver declaration-scanner denylist

Within `crates/ash-parser/src/resolver.rs`, no semantic path may introduce a replacement for the
removed `parse_module_decls` scanner: line iteration over source to recognize module declarations,
`find("mod ")`, prefix matching for `mod`/`pub mod`, or equivalent string extraction is forbidden
from publishing a structural child or graph edge. The only declared authority is the parsed
`ModuleFile` through `ash_parser::discover_module_declarations`. The focused
`task_2057_ast_module_discovery` target is the current positive, negative, and mutation evidence.

### Activation accounting

TASK-2057 is complete for its `partial / tested / below_spec` structural handoff, TASK-2058 is
complete for its `partial / tested / below_spec` Core-carrier handoff, TASK-2059 is complete for
its `partial / tested / below_spec` parser module-unit handoff, TASK-2060 is complete for its
`partial / tested / below_spec` Core public-interface carrier, TASK-2066 is complete for its
`partial / tested / below_spec` TypeEnv handoff, and TASK-2061 is complete for its
`partial / tested / below_spec` wrapper-only resolver handoff. TASK-2066 stages a canonical-key
claim, function/handler declaration-signature preflight, bounded fact validation, and atomic commit
before it issues a non-forgeable wrapper over a fully equal artifact. TASK-2061 stores only those
wrappers, traverses public checked children, and resolves explicit/group/glob requests with atomic
groups, explicit precedence, deferred glob ambiguity, and preserved identity/syntax-only macro
metadata. Neither task checks bodies/full callables, links typed namespaces, aliases/re-exports, or
source origins, establishes complete closure or parsed visibility/cycles, or removes/fences an
Engine scanner. TASK-2062 is complete for its `partial / tested / below_spec` bounded Core-to-CPS
handoff: it snapshots only TASK-2061 resolved imports, preserves exact finalizer artifact and
import identity/origin facts, and delegates an already-materialized Core program through the
checked bridge. Its public carriers are non-authoritative. Completed TASK-2067 supplies a partial
parser graph handoff whose structural clauses are complete, but it does not authorize any later
layer. TASK-2068 is complete for its partial/tested bounded foundation. TASK-2071 completed the
contract; TASK-2074 owns expansion, TASK-2075 provisional/internal collection, TASK-2072 parsed imports/binding, and TASK-2073 checked bodies/final
interfaces after its bounded provisional-function/simple-alias plus graph-wide-preflighted closed
primitive M-CHECK slice. That
slice stages sibling body checks atomically and yields only a private projection plus
constructor-free non-authorizing `CanonicalPublicFunctionInterface`, never a final interface or
import/runtime authority. Planned TASK-2069 supplies the complete non-sealed Core/CPS closure plus
scanner/cache transport fence that active
TASK-2063 must then independently seal. This activation creates no request, Engine
admission, execution, or evidence. TASK-2062 does not lower parser source/full definitions, grant
typed import/callable authority, prove file/inline real-program parity, or reach Engine/CLI. No
handoff authorizes a direct-evaluator fallback.

## Required rule families

| ID | Rule | Current status | Planned owner |
|---|---|---|---|
| MOD-REAL-001 | A module declaration creates one stable child identity and structural edge from parsed AST | partial / tested / below_spec | TASK-2067 completes canonical AST-only `ModuleKey` edges and structural diagnostics; TASK-2074 must add the AST-only syntax prepass and exact keyed expanded graph |
| MOD-REAL-002 | File and inline sources produce equivalent module units after acquisition | partial / tested / below_spec | TASK-2067 proves parsed-unit payload parity. TASK-2074 owns expanded projection parity, TASK-2075 collected projection parity, TASK-2073 final-interface parity, TASK-2069 lowering transport, and TASK-2064 terminal parity |
| MOD-REAL-003 | Checked public interfaces are export-closed and preserve defining identities | partial / tested / below_spec | TASK-2060 Core carrier and TASK-2066 bounded wrapper complete; TASK-2068 provides only its tested leaf foundation. TASK-2071 defines the contract, TASK-2075 owns internal/name collection, and TASK-2073 owns full body/callable facts, typed linkage, aliases/re-exports, final interfaces, and export closure |
| MOD-REAL-004 | Imports and visibility resolve through canonical provisional name views during binding and finalized checked interfaces during publication | partial / tested / below_spec | TASK-2061 completed only a wrapper-store explicit/group/glob resolver; TASK-2068's bounded parsed leaves remain evidence. TASK-2070 owns the self-alias leaf, TASK-2075 owns the name view, TASK-2072 owns complete grammar/visibility/re-exports/cycles/precedence/binding, and TASK-2073 finalizes exports |
| MOD-REAL-005 | Resolved modules lower to linked Core/CPS artifacts without source rediscovery | partial / tested / below_spec | TASK-2062 complete bounded wrapper/resolved-binding provenance handoff; TASK-2069 owns complete body lowering, file/inline artifact parity, and Engine scanner/cache fencing |
| MOD-REAL-006 | Engine admission and CLI/daemon execution use the same linked module artifact | not_implemented / none / below_spec | TASK-2069 supplies a non-sealed complete closure; TASK-2063 seals admission and TASK-2064 owns real-program/client parity |

## Non-goals

This phase does not add dynamic imports, package discovery, import-cycle initialization, hot reload, runtime module values, or a full incremental workspace database. Structural and import cycles reject in the initial realization.
