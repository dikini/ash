---
id: language.reference.lexical.modules-imports-and-visibility
title: Modules, Imports, and Visibility
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: b4dd1844
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-parser/tests/task_2059_file_inline_module_unit_parity.rs", "crates/ash-parser/tests/task_2067_canonical_module_graph.rs", "crates/ash-parser/tests/task_2067_canonical_identity_fence.rs", "crates/ash-parser/tests/task_2067_legacy_route_fence.rs", "crates/ash-core/src/module_graph.rs", "crates/ash-core/tests/task_2058_canonical_module_identity.rs", "crates/ash-core/src/module_interface.rs", "crates/ash-core/tests/task_2060_public_module_interface.rs", "crates/ash-engine/src/module_loader/**", "crates/ash-engine/tests/**"]
---

# Modules, Imports, and Visibility

[Lexical and modules index](index.md) · [Source files and literals](source-files-names-and-literals.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `b4dd1844` plus the current TASK-2071 specification-contract working tree.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `mod name;` and inline `mod name { ... }` parser surface | accepted | partial | bounded-only | not-applicable | partial | tested | below_spec |
| AST-derived resolver structural handoff | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Core `ModuleKey`/`ModuleArtifact` carrier | not-applicable | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Ordered `ModuleItem`/`ModuleBody`/`ModuleUnit` source acquisition | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Canonical parser graph and structural diagnostics | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Target syntax-prepass/expanded-graph contract | specified | not-implemented | not-applicable | not-applicable | not_implemented | none | below_spec |
| Target internal snapshot/name-only provisional-view contract | specified | not-implemented | not-applicable | not-applicable | not_implemented | none | below_spec |
| Core public module-interface carrier | not-applicable | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Direct `parse_use` statement parser and parser item grammar | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Engine ordinary-module import prelude | parser-only | partial | bounded-only | not-applicable | partial | tested | below_spec |
| Engine runtime-entry `use` prelude | parser-only | partial | bounded-only | fixture-bounded | partial | tested | below_spec |

The two Engine prelude rows are `parser-only` because they remain separate compatibility routes,
not import binding for parser `ModuleUnit` values. `module_file` parses `use` through the shared
item dispatcher but projects it away in its legacy `ModuleFile` result; the child module-unit route
retains it only as unbound syntax. Ordinary-module loading has no independent runtime behavior,
while the runtime-entry row is `fixture-bounded` only for its registered leading-import route.
Neither row makes an imported callable generally executable.

Source evidence is `crates/ash-parser/src/parse_module.rs::parse_module_decl` and
`::module_file`, `ash_parser::discover_module_declarations`, `crates/ash-core/src/module_graph.rs`, `crates/ash-parser/src/parse_use.rs::parse_use`,
`crates/ash-parser/src/parse_visibility.rs::parse_visibility`,
`ash_parser::resolver::ModuleUnitResolver::acquire_child`,
`ash_parser::canonical_module_graph::CanonicalModuleGraphResolver`, and
`crates/ash-engine/src/{module_loader.rs,entry.rs}`. Focused evidence is
`crates/ash-parser/tests/task_2057_ast_module_discovery.rs`,
`crates/ash-parser/tests/task_2059_file_inline_module_unit_parity.rs`,
`crates/ash-parser/tests/task_2067_canonical_module_graph.rs`,
`crates/ash-parser/tests/task_2067_canonical_identity_fence.rs`, and
`crates/ash-parser/tests/task_2067_legacy_route_fence.rs`,
`crates/ash-core/tests/task_2058_canonical_module_identity.rs`,
`crates/ash-engine/tests/{module_file_check_tests.rs,module_import_resolution_tests.rs}`, and the
visibility tests in `crates/ash-parser/src/parse_visibility.rs`.

This page owns AUDIT-206 LANG-001's module portion and LANG-002. It deliberately separates
parser acceptance from module loading and from executable programs.

TASK-2057 completed a partial, tested, below-spec AST discovery handoff required by SPEC-103. The
resolver parses each source to `ModuleFile`, derives public structural declarations with name,
visibility, source form, span, and source path, then creates file and inline graph children from
that carrier. It no longer scans raw resolver source text for module declarations; comments and
literals cannot create child edges, and inline declarations do not probe a file child.

This is structural prerequisite evidence only. It does not establish a complete canonical module
route, checked export-closed interfaces, imports, visibility enforcement, Core/CPS lowering, Engine
admission, or CLI/daemon parity. Completed TASK-2067 provides focused
`partial / tested / below_spec` canonical graph evidence for its parser-stage clauses. Completed
TASK-2068 supplies the bounded Type foundation; TASK-2070 supplies its completed bounded self-alias leaf,
TASK-2071 supplies the completed contract; active TASK-2074 owns expansion, while planned TASK-2075 owns collection,
TASK-2072 imports/binding, and TASK-2073 final checked
interfaces/export closure. TASK-2069 consumes TASK-2073; TASK-2063 owns admission; TASK-2064 owns
real-program and CLI/daemon parity; and TASK-2065 closes the phase.

TASK-2058 completes a separate, partial/tested/below-spec Core-carrier handoff. `ModuleKey` is a
crate-qualified, source-layout-independent, serializable identity with a deterministic cache key;
`ModuleArtifact` records a file or inline origin, schema version, structural parent, and sorted
direct child keys while rejecting malformed, duplicate, or forged wire values. This carrier now
feeds TASK-2067's parser-only canonical graph, but not the legacy `ModuleResolver`/`ModuleGraph`
or `semantic_summary::ModuleIdentity` compatibility paths. It therefore does not establish
complete graph migration, full file/inline module-unit parity, interfaces, imports, lowering,
admission, runtime behavior, or client parity.

TASK-2059 completes a separate, parser-owned `partial / tested / below_spec` source-acquisition
handoff. `ModuleItem` and `ModuleBody` retain ordered `use`, definition, and nested-module syntax;
`ModuleUnitResolver` consumes the Core carrier, selects `child.ash` before `child/mod.ash`, parses
the selected file once, and makes inline acquisition without filesystem access. It detects duplicate
children and anchors missing/invalid-key acquisition diagnostics at the parent declaration. Nested
inline macro and notation scopes expand recursively with isolated hygiene. These units do not bind
imports or visibility, create checked interfaces, lower to Core/CPS, admit or run in the Engine,
or authorize a direct-evaluator fallback. Completed TASK-2067 consumes those actual units into
`crates/ash-parser/src/canonical_module_graph.rs`: it has AST-only edges; source-anchored
missing/root+nested-duplicate/malformed-inline/cycle rejection; parsed-source invalid-key
rejection without a synthetic child; canonical-key rewrite resistance; complete
`Absent`/`Discovered`/`Parsed`/`Failed` reporting; root metadata retention; and complete ordered
file/inline payload parity plus payload-only mutation evidence. Failed states are retained on the
error report, never as publishable partial graph entries. Its explicit deprecated legacy route is
isolated by an architecture fence; that source-layout check is not semantic mutation evidence.
TASK-2068 now has a non-authorizing `partial / tested / below_spec` Type-layer slice: it collects
provisional function identity/origin/span/visibility from canonical graph units and resolves simple
parsed `crate::…` aliases. It publishes neither a final interface nor an Engine credential;
CLI/daemon parity remains TASK-2064 work.

TASK-2068 also has tested, bounded `M-CHECK` evidence. It admits only graph-delivered
self-contained leaf modules with ordinary inherited or public functions and primitive closed
signatures; graph-preflights every unit; stages sibling signatures; atomically checks all bodies
only through the builtin TypeEnv checker; retains a fresh checked identity plus module
identity/origin/spans and signature/body types; and yields a private checked-function map plus a
non-authorizing `CanonicalPublicFunctionInterface` that exports public primitive signatures. The
focused target passes 8/8, including 16 generated public integer functions, anchored mismatch and
closed-signature boundaries, late-failure atomicity, `use`-shape and nested-child global-preflight
rejection, and an architecture fence. It is not core `PublicModuleInterface` or a final module interface. Imports, child/nested
modules, other definitions, generics, contracts, unsupported visibility, user-defined types,
interfaces, effects, re-exports, Core/CPS/Engine, and client behavior remain excluded.

TASK-2068 also delivers a restricted declaration visibility M-CHECK leaf. It is `partial / tested /
below_spec`: Type is `partial`; Core/CPS/admission/runtime are `not_applicable`; verification is
`partial`; and its run-route impact is `prerequisite`. Its exact domain is `pub(crate)`,
`pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` ordinary functions with
primitive closed signatures, no imports, child modules, nonfunctions, generics, contracts, or open
signatures in a file-root closed leaf. It graph-preflights, stages sibling signatures, and checks
bodies atomically; restricted checked facts remain private with identity/origin/spans/visibility/
signature/body retention, and only `Visibility::Public` projects through
`CanonicalPublicFunctionInterface`. `pub(in self::internal)` rejects as outside the delivered
domain. The existing no-children preflight rejects an inline child/module as atomic
`UnsupportedModuleShape` before projection. The focused target passes 18/18.
`TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY` is a tested source-form
boundary witness—file-root success versus inline rejection before projection—not normalized-success
file/inline parity. This does not authorize imports, binder or re-export behavior, final interfaces,
lowering, admission/runtime, or parity.

TASK-2068 has delivered graph-only simple-import planning over real canonical graph units and
provisional function targets. It admits inherited `UsePath::Simple` crate-root function aliases and
produces opaque resolved imports/bindings plus canonical cross-module edge provenance: importer,
defining module/identity, local spelling, use span, declaration span, origin, and visibility.
Same-module aliases produce no edge; every discovered cycle rejects before a result as
`ImportCycle { edges: CanonicalImportCycle }`, whose ordered wrapper exposes parser-anchored
edges; and `bind_simple_parsed_uses` delegates through the planner. Focused test evidence passes
11/11, including edge provenance, same-module no-edge, ordered file/inline two-node cycle edges,
a full-provenance `a → b → c → b` tail diagnostic that reports only `b ↔ c`, late-backedge
atomicity, and a delegation fence that excludes `RawCoreProgram`, `CoreExpr`, and `CpsProgram`.
This remains bounded Type-layer evidence, not a complete import, final-interface, lowering,
Engine, or client-parity result. Checked interfaces, TypeEnv/body facts,
legacy/TASK-2060/2061/2066 authority, restricted visibility, re-exports, `pub use`, groups, globs,
qualified paths, Core/CPS/Engine, and client behavior remain excluded.

TASK-2068 also delivers a bounded direct primitive provider/client check over a canonical graph and
its resolved simple-import plan. It admits only the root plus plan-selected direct provider leaves.
Before provider checking, graph-wide `module_units()` completeness rejects every unrelated
unselected non-root unit, including a nested module; a descendant of a selected provider instead
reaches the existing provider-leaf precheck and rejects as anchored
`UnsupportedProviderShape`. The check requires exact plan/graph artifacts, primitive provider
precheck, edge revalidation against checked public providers, and fresh-root imported-signature
injection before atomically returning non-authorizing checked root/provider/import facts. The
focused target passes 12/12, including a 16-case property and ordering evidence that global
unselected-unit rejection occurs before a malformed selected provider is checked. This is not a
final interface, general import/binder authority, Core/CPS lowering, Engine admission, runtime, or
client-parity result.

TASK-2068 also delivers an opt-in direct-public primitive re-export interface fragment,
`partial / tested / below_spec`, separate from the delivered planner's continuing `pub use`
rejection: only a canonical-graph root plus plan-selected direct primitive providers, `pub mod api`,
and explicit root `pub use crate::api::greet as welcome` are in scope. It requires both the
public structural path and public target, retain defining identity/origin/primitive signature and
declaration/use spans, avoid implicit root-name flattening, and return a non-authorizing atomic
fragment only. It must fail closed for an empty public-use plan, any public root definition outside
the exact fragment, and a `pub mod api`/`pub use … as api` child-identity alias collision with its
structural/use anchors. A public re-export lacking `as <alias>` rejects as anchored `Unsupported`
with `an explicit re-export alias is required` before plan publication. Its focused target passes
13/13, including a 16-case property; this is
test evidence, not proof, and does not authorize other namespaces, import/path/visibility/re-export
forms, compatibility carriers, final interfaces, Core/CPS, Engine, admission, runtime, or parity.

The direct re-export helper sub-slice is delivered as `partial / tested / below_spec`: within that
same exact public root form, a selected public primitive target may use inherited/private ordinary
primitive provider helpers. They are checked atomically as implementation detail and never appear
in `CanonicalPrimitiveInterfaceFragments`; a private selected target still rejects before
publication. Its focused target passes 7/7, including a 16-case property; this is Type-layer test
evidence, not proof or end-to-end parity. Provider uses, nested modules, other definitions,
generics/contracts, restricted visibility, non-primitive/open signatures, every other path, final
interfaces, Core/CPS, Engine, admission, runtime, and parity remain excluded.

The direct re-export local-binding root-client sub-slice is delivered as `partial / tested /
below_spec`. It admits only the same exact provider/re-export form plus inherited/private root
`fn internal_entry(..) -> <primitive> { welcome(..) }`, checked through a distinct opaque direct
plan that preserves `greet`'s defining identity and checks visibility before registering the local
alias. Its opaque output contains only the fragment, checked private root functions, selected
provider facts, and local alias binding. Its focused target passes 10/10, including a 16-case
property. The root-body diagnostic takes a call anchor only from a direct unqualified alias call
(including an empty block tail); otherwise it uses the root-body span. The generic planner/binder
and generic provider/client route continue to reject source `pub use`; all root public functions,
generic binding, final interfaces, Core/CPS, Engine, admission, runtime, and parity remain
deferred. TASK-2074/TASK-2075 own expansion and complete collection after TASK-2071's contract;
TASK-2072 owns complete parsed imports/binding, and TASK-2073 complete checking/finalization;
TASK-2069 cannot begin until TASK-2073 is complete.

TASK-2068 now delivers a canonical provisional-module-scope and structural-path visibility slice:
`partial / tested / below_spec` Type-layer evidence. It derives immutable typechecker-owned direct
structural children and ordinary function declaration entries from TASK-2067 canonical graph
units/artifacts. `matches_graph` compares root/artifact facts and requires equality with a fresh
declaration-snapshot rebuild from current parser units, so artifacts alone cannot authorize entries
and same-path/topology removal or a `pub`-to-private change rejects `ScopeGraphMismatch` before
binding. It resolves only inherited simple `use crate::<structural-child>...` paths, preserves
canonical identity/origin/spans, and decides every child/function visibility region using
`ModuleKey` crate identity and segments before staging an alias. `is_visible_from` is only a
declaration-level query: the resolver separately tracks structural children and rejects a public
function below the first non-public child. The focused target passes 9/9; that is test evidence,
not proof or parity. `pub use`, groups, globs, non-`crate` paths, non-function targets, other
namespaces, final interfaces, Core/CPS, Engine, admission, runtime, and parity remain deferred.

The bounded scope-backed route's final target is an ordinary function, not a public-only target.
Its canonical visibility predicate can admit public, `pub(crate)`, `pub(super)`, `pub(in path)`,
inherited/private, and `pub(self)` when the importing module lies in the corresponding `ModuleKey`
region; public targets still require the separate whole structural-path fence. The dedicated scoped
structural binder now supplies binding-only evidence for each permitted region, without widening
the generic binder or granting final-interface or runtime authority.

The delivered dedicated scope-backed structural binder M-BIND slice is `partial / tested /
below_spec`, Type-only `prerequisite` evidence. It delegates only
`bind_scoped_structural_parsed_uses(graph, scopes)` from the new
`crates/ash-typeck/src/canonical_structural_module_binder.rs`, exposed only through its dedicated
`lib.rs` API, to the delivered scope-backed resolver then project the result through
`into_bound_set`, preserving resolver diagnostics and atomic cycle failure. The existing
`canonical_module_binder.rs` and its generic `bind_simple_parsed_uses` binder remain unchanged:
they do not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. The focused
`task_2068_scoped_structural_binder` target passes 8/8, including a 16-case property across public,
crate, super, `pub(in path)`, inherited/private, and self visibility categories; this is test
evidence, not proof. No final-interface, Core/CPS, Engine, admission, runtime, or parity authority
is introduced.

The delivered scoped simple ordinary-function imports M-SIMPLE slice is `partial / tested /
below_spec`, Type-only `prerequisite` evidence. Its dedicated
`bind_scoped_simple_ordinary_function_imports(graph, scopes)` API accepts inherited simple
`use crate::<ordinary-function>` and `use crate::<structural-child>...::<ordinary-function>`
routes with an optional `as <name>`; without `as`, it binds the final function segment naturally.
It delegates only to `resolve_scoped_simple_ordinary_function_imports_with_scopes` and
`into_bound_set`, preserving all existing canonical visibility regions and the public
whole-structural-path fence. Local function collisions, duplicate local bindings, and
`CanonicalImportCycle` reject atomically. The focused
`task_2068_scoped_simple_ordinary_function_imports` target passes 11/11, including a 16-case
property and the retained structural-child compatibility regression; this is test evidence, not
proof or parity. The generic resolver, generic binder, and
delivered explicit-alias scoped binder remain unchanged; the route has no final-interface, Core/CPS,
Engine, admission, runtime, or parity authority.

The delivered scoped grouped ordinary-function imports M-GROUP slice is also `partial / tested /
below_spec`, Type-only `prerequisite` evidence. `UseItem` retains a parser-owned span for every
nested member name plus optional alias; only inherited `use crate::<children>::{function, function
as local}` routes are accepted. The dedicated grouped resolver and binder preflight the current
scope snapshot, structural path and visibility, whole public path, local collisions, duplicate
local names, and the complete cross-module cycle set before returning any plan or binding set. A
failure is atomic and points at the applicable member; grouped structural-child members are
anchored `Unsupported`, whereas the older simple structural-child compatibility route remains
enclosing-span `Unresolved`. The focused grouped target passes 10/10, including a 16-case property;
the parser full suite passes. This is test evidence rather than a proof or parity claim, and it
does not add generic binder, final-interface, Core/CPS, Engine, admission/runtime, or client
authority.

The scoped `super` ordinary-function import M-SUPER slice is `partial / tested / below_spec`
Type-only `prerequisite` evidence. It accepts only inherited simple routes from a non-root module
with exactly one leading `super`, starts at the canonical parent module, traverses structural
children, and imports one ordinary function under an optional alias or its natural name. It keeps
the complete parser `use` span, canonical visibility and whole-public-path checks, and atomic
collision, duplicate, and cycle checks. Every extra child `super` and a final function named
`super` reject before lookup. The focused target passes 12/12 including a 16-case property. This
does not introduce `self::` (same-module precedence remains unresolved), root/repeated-`super`
imports, groups/globs, re-exports, other namespaces, final interfaces, Core/CPS, Engine,
admission/runtime, or client authority; tests are not proof or client-parity evidence.

The delivered scoped `super` grouped ordinary-function import M-SUPER-GROUP slice is
`partial / tested / below_spec` Type-only `prerequisite` evidence. It accepts only inherited,
non-root `UsePath::Nested` routes with exactly one leading `super`, no outer alias, zero or more
structural children from the canonical parent, and a nonempty ordinary-function group with
natural/member-`as` local names. It preserves each parser-owned nested-member span for its
identity, edge, and member-specific error facts; preflights a final member named `super` before
lookup; and reuses canonical scopes/snapshots/visibility/whole-public-path, same-module no-edge,
collision, duplicate, cycle, and atomic-publication rules. The focused target passes 13/13,
including a 16-case property. POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY are positive
evidence; VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and
AUTHORITY-FENCE are negative evidence; CYCLE-ATOMICITY is mutation evidence. Root/repeated
`super`, `self`, `crate`, unprefixed/standard-library/external, simple/glob/non-nested/nested
groups, public/restricted/re-export forms, nonfunctions, generic resolver/binder changes, final
interfaces, Core/CPS, Engine, admission/runtime, parity, and general precedence stay deferred.
Tests are evidence, not runtime, proof, or parity evidence.

The delivered scoped glob ordinary-function import M-GLOB slice is `partial / tested / below_spec`
Type-only `prerequisite` behavior. It admits only inherited
`use crate::<public structural-child>...::*` paths whose importer has exactly one `use` and zero
local ordinary functions. Its dedicated resolver/binder traverses public structural children,
selects visible public ordinary functions, preserves defining identity/origin/declaration
span/visibility and the whole parser `Use::span`, and stages one edge per function before atomic
publication. The 15-valid-representation shape matrix treats leading `::` as not `UsePath::Glob`;
a private structural module is instead an `Inaccessible` visibility case. The CONFLICT-ATOMICITY,
AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY tests are boundary mutations only: a local function,
second glob, or cycle-shaped attempt returns `Unsupported` with neither plan nor bindings; none
claims `LocalDeclarationCollision`, `DuplicateBinding`, generic ambiguity, `ImportCycle`, or a
precedence rule. `self::` remains deferred because same-module precedence is unresolved. Other
path/use forms, aliases, root-function globs, non-functions/namespaces, generic binder changes,
final interfaces, Core/CPS, Engine, admission/runtime, and parity remain outside this slice; tests
are evidence, not proof.

The delivered M-GLOB-LOCAL-PRECEDENCE slice is partial / tested / below_spec. It uses exactly one
inherited public structural-child crate glob and only canonical graph/provisional-scope facts: a
same-module ordinary function shadows a same-name selected public import only in returned public
bindings, a non-colliding import binds, and every selected cross-module edge remains through
actual atomic ImportCycle detection before filtering. All-shadowed input succeeds with no import
bindings but retained edges. The focused target passes 8/8, including a 16-case property varying
names, collision subsets, source form, and depth 1–3; file/inline establishes normalized
Type-layer scope/binding parity only, never final/runtime parity. It does not use private M-CHECK
facts; existing M-GLOB behavior remains separate/rejecting; other imports, multiple globs,
aliases/re-exports, self/super/non-crate paths, nonfunctions, the generic binder, final
interfaces, Core/CPS, Engine, admission/runtime, and parity remain excluded. This is a
non-authorizing Type plan; TASK-2069 owns lowering and TASK-2064 owns parity.

The delivered M-SIMPLE-LOCAL-PRECEDENCE slice is partial / tested / below_spec. It admits exactly
one inherited, unaliased `UsePath::Simple`
`use crate::<public structural-child>...::<public ordinary-function>;` route with its natural
final name, while a same-module ordinary function is permitted. Its dedicated resolver retains an
edge only for a selected cross-module target, completes deterministic cycle detection over those
edges, then filters the same-name import binding. A selected same-module target emits no self-edge
and does not participate in cycle detection. A non-colliding import binds; all shadowed
cross-module candidates retain their edges with no import binding; and a real hidden two-module
cross-module cycle rejects atomically. The existing M-SIMPLE route remains unchanged and rejects a
local collision. This slice uses only canonical graph/provisional scopes, never private M-CHECK
facts or generic binder authority. Root functions, aliases, multiple uses, groups/globs,
`self`/`super`, restricted/private targets or paths, re-exports, nonfunctions, body lexical
binding, final interfaces, Core/CPS, Engine, admission/runtime, and parity remain excluded. The
focused target passes 9/9; its file/inline witness claims only normalized Type-layer
scope/binding parity. TASK-2069 owns lowering and TASK-2064 owns parity.

The delivered TASK-2070 M-SELF-SIMPLE-ALIAS slice is `partial / tested / below_spec`. It admits zero
or more individually eligible inherited, two-segment `UsePath::Simple`
`use self::<ordinary_function> as <different_alias>;` statements in a root or nested module. A
module with none produces an empty dedicated result; groups, globs, mixed imports, and other forms
are `Unsupported`; a direct `self::<child_module>` target is a nonfunction `Unsupported`. It
resolves only direct same-`ModuleKey` ordinary functions when `is_visible_from` permits that
importer, stages distinct aliases together, reports duplicates as `DuplicateBinding`, and preserves
local alias, defining identity, declaration span, origin, declared visibility, and full `use_span`
in each `CanonicalSelfOrdinaryFunctionAliasBinding`. The dedicated
`CanonicalResolvedSelfOrdinaryFunctionAliases` has no `import_edges` field; only its binder calls
private `into_bound_alias_set` to return `CanonicalBoundSelfOrdinaryFunctionAliasSet`, never
`CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`. Resolver and binder share
`CanonicalStructuralImportError`; `ImportCycle` is unreachable by construction and source fence.
This direct self route emits no `CanonicalSimpleImportEdge` and runs no import-cycle detection; all
out-of-domain, visibility, local-collision, and valid-sibling/failing-module inputs remain atomic.
`CanonicalBoundModuleBinding` and the generic binder stay unchanged. Its implementation and eight
witnesses are implemented/tested; the focused target passes 8/8, including the exact 16-case
property with alias count `1..3`. It authorizes neither cross-module traversal nor generic-binder, M-CHECK
private-fact, final-interface, Core/CPS, Engine, admission/runtime, or parity behavior. TASK-2072
owns complete imports/binding, TASK-2073 finalization/export closure, TASK-2069 owns lowering, and
TASK-2064 owns parity.

TASK-2068 now delivers a scoped structural import-cycle gate: `partial / tested / below_spec`
Type-only `prerequisite` evidence. After the scope-backed route resolves all inherited explicit-
alias `crate::` structural paths and their visibility, it collects only cross-module canonical
edges, deterministically rejects a `CanonicalImportCycle` before a plan is returned, and emits no
edge for a same-module alias. Existing scope/path/visibility diagnostics retain precedence; thus an
inaccessible child/function that could otherwise close a cycle does not become a cycle diagnostic.
The focused scope17 target passes the eight witnesses, including a 16-case property; this is test
evidence, not proof or parity. The generic planner and binder remain different-grammar routes, and
no final-interface, Core/CPS, Engine, admission, runtime, or parity authority is introduced.

TASK-2071 completes the target namespace and provisional-view specification handoff, but no
implementation evidence. Before ordinary import binding, the target route performs an AST-only
syntax prepass, orders macro/notation providers before consumers, and creates one exact keyed
`CanonicalExpandedModuleGraph`; active TASK-2074 now supplies a partial/tested local-only graph
that owns the parsed graph, shallowly expands direct definitions, preserves uses, module
declarations, source order, and per-key sidecars, and rejects anchored local failures atomically.
It does not yet implement syntax-summary imports, provider ordering, syntax cycles, imported
notation, normalized file/inline parity, or authority fences. Planned TASK-2075 then creates
two separate collection products. Its internal snapshot may retain expanded declaration shapes,
bodies, member spans, expansion sidecars, and source order but no checked results. Its import-facing
name view contains only names/lookup keys, defining identities/module keys, namespaces,
visibility/exportability, source anchors/origins, and ordinals. It has no signature, callable/body,
type/equation, final-export, or runtime-authority fact. TASK-2072 may consume only the name view;
TASK-2073 consumes the internal snapshot plus staged bindings. TASK-2074 remains
`partial / tested / below_spec`; TASK-2075 remains `not_implemented / none / below_spec`.
Neither is current user-facing complete module behavior.

The selected notation-import spelling is parenthesized and exact:

```ash
use crate::math::(<*>);
use crate::ranges::(_ between _ and _);
```

The selector is the normalized parsed token/hole pattern; it does not carry fixity, associativity,
or precedence, and raw notation spelling is diagnostic-only rather than matching authority. It has
no `as` form and no notation glob. All eligible public full-key variants for the pattern are
transported deterministically into the consumer's existing syntax-phase notation table, preserving
hole order and target/provenance facts but neither binding nor authorizing the target callable.
Ordinary callable imports never activate notation. Invalid or cyclic notation dependencies reject
the whole graph with source anchors. A direct `pub` notation declaration exports its summary; only
plain inherited `use module::(pattern)` is supported. `pub use` and every other visibly qualified
notation use reject until a separately owned re-export contract exists. This is the approved
TASK-2074 target, not current tested behavior; generalized mixfix use-site parsing/elaboration is
outside TASK-2074.

TASK-2060 completes a `partial / tested / below_spec` Core carrier: the V1
`ash_core::module_interface::PublicModuleInterface` retains a TASK-2058 artifact, public binding
identity/visibility/origin facts, dependency versions, strict serde, and compatibility validation
through the existing semantic-summary V1--V8 contract. It rejects private/duplicate public
bindings, invalid child and inline-origin facts, forged generic typed identities, and malformed
wire data. Aliases retain defining identity, while macro/notation entries remain syntax-only and
install no runtime authority.

This is not an authoritative full module interface. TASK-2060 does not collect `ModuleUnit`
declarations, retain a private view, link existing typed summary identities, prove public closure,
bind imports, or fence any Engine scanner. TASK-2066 has now completed a bounded TypeEnv handoff:
it stages `register_surface_declarations` preflight for public function/handler declaration
signatures under one canonical `ModuleKey`, validates a limited parser/TypeEnv projection, requires
full artifact equality, and issues a non-forgeable immutable wrapper. It does not check bodies or
full callable facts, link typed namespaces, collect aliases/re-exports or per-binding source
origins, establish complete export closure, bind imports, or fence an Engine scanner. TASK-2062 is
complete for a `partial / tested / below_spec` bounded Core-to-CPS handoff: it accepts a TASK-2061
wrapper, resolved binding facts, and an already-materialized Core program, then retains exact
module-artifact and imported defining identity/origin metadata in non-executable Core/CPS
artifacts. The public carriers are non-authoritative: TASK-2063 must establish its own sealed
dependency-linking/admission input around them rather than treating either carrier as authority.
TASK-2062 does not lower parser source/full definitions, establish typed imports/callable authority,
prove file/inline real-program parity, or reach Engine/client authority. TASK-2063 is now in
progress but remains `not_implemented / none / below_spec`: it must turn only those
non-authoritative carriers into a separately Engine-sealed linked/admission request, without a
raw/source/direct-evaluator fallback. No such request or Engine evidence exists yet. TASK-2064 owns
structural/import-cycle and client-parity conformance.

TASK-2061 is complete for a `partial / tested / below_spec` bounded resolver handoff. Its
`CheckedInterfaceStore` accepts only TASK-2066 wrappers; it traverses checked public children and
supports in-memory explicit, grouped, and glob requests. Groups are atomic, explicit bindings take
precedence over globs, distinct glob identities remain ambiguous, defining identity is preserved,
and syntax macros remain non-runtime metadata. It does not use `PublicModuleInterface::new`, parser
resolver state, a legacy graph, Engine state, a filesystem, or text scans as import authority.

This is not the complete parsed `use` resolver or visibility binder. TASK-2068's tested bounded
slice resolves only simple `crate::…` aliases to provisionally collected functions and distinguishes
an anchored inaccessible private declaration from an unresolved name. It preserves defining
`ModuleKey`, name, source span, origin, and declared visibility; 16 generated aliases preserve
their defining identity. `pub use`, re-exports, and every non-inherited use visibility reject
before a binding set is published. `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in …)` target
declarations likewise produce anchored `Unsupported`, so these are fail-closed boundaries rather
than full visibility semantics. The returned set has neither `Default` nor a public constructor,
preventing callers from manufacturing a successful result outside the binder. Typed namespaces,
complete visibility/cycles, final-interface/export-closure/body facts, lowering, Engine
scanner/path-cache transport, admission, and all client parity remain owned downstream. TASK-2064
alone owns client parity. TASK-2066's wrapper is not an authoritative full interface and does not
claim body/full-callable facts, typed namespace linkage, aliases/re-exports, per-binding origins,
or export closure.

These are ownership assignments, not complete-feature claims. The support table remains
`partial / tested / below_spec`: TASK-2067 is complete for its focused parser graph handoff with
those task axes. TASK-2068 is complete for its `partial / tested / below_spec` bounded Type-layer
foundation. Its direct-public primitive re-export fragment remains delivered evidence only; its
unresolved expansion/collection, parsed-import/binding, and final-interface reservations are owned
by TASK-2074/TASK-2075, TASK-2072, and TASK-2073 respectively after TASK-2071's completed contract.
TASK-2069 must still be activated before its
behavior can be described as implemented.

## What it is and how to use it

`mod name;` declares a file-based module and `mod name { ... }` declares an inline module in the
surface parser. Both can be preceded by a visibility modifier. `pub`, `pub(crate)`, `pub(super)`,
`pub(self)`, and `pub(in path::to::module)` are the visibility forms accepted by
`parse_visibility`.

Inline modules are a parser feature, not an accepted general Engine module-file route. In
particular, `Engine::check_module_file` has a test that rejects an inline module containing an
ordinary type rather than silently omitting it. Use file-based modules for the checked module-file
route unless the specific downstream feature supplies narrower evidence.

`use` needs special care because it has more than one implementation route:

- `ash_parser::parse_use::parse_use` is a direct statement parser. It accepts optional visibility,
  simple paths, aliases, globs, and nested selections, and it **requires a terminating semicolon**.
- The Engine ordinary module loader scans a leading run of `use` or `pub use` lines. For imports
  without `@`, it adds a missing semicolon and invokes the direct parser. Imports containing `@`
  take the separate versioned-import path. This is loader convenience, not parser-unit import
  binding or visibility authority.
- The Engine runtime-entry prelude recognizes leading `use` lines and masks them before parsing
  the entry body. It accepts a semicolon-free line but only whitelists a small registered runtime
  import set. It is not a general import execution facility.

For all three routes, a resolved import contributes checked module summary information only within
its supported path. It does not prove that an imported callable can be admitted or executed.

## Examples

**Surface-parser module declaration.** This is parser evidence for `mod`; it is not a claim that
the inline child is accepted by `Engine::check_module_file`.

```ash
pub mod math;
mod local { fn value() -> Int { 1 } }
```

**Direct `parse_use` form.** The semicolon is required by
`crates/ash-parser/src/parse_use.rs::parse_use`.

```ash
pub use math::{Number as N, add};
```

**Semicolon-free Engine-prelude convenience.** The following lacks a semicolon. The ordinary
module loader/runtime-entry prelude can recognize this leading form, but calling direct `parse_use`
on the same text fails because `parse_use` requires `;`. The runtime entry path also requires the
import to be one of its registered forms.

```ash
use time::{sleep}
fn main() { 0 }
```

**Resolved-import summary example.** The test
`plain_function_with_target_body_is_importable_by_signature` in
`module_import_resolution_tests.rs` shows a public function signature becoming an imported
callable summary. It does not execute that callable.

```ash
use dispatch::{complete_with_tools}
fn main() { 0 }
```

## Syntax

`module_file` is the legacy whole-file parser projection. Its shared parser body accepts `use`,
module declarations, and definitions; `ModuleBody` retains that order for source acquisition,
while the legacy `ModuleFile` projection retains only definitions and module declarations.
`direct_use` names the standalone `parse_use` route.

```ebnf
module_file = { module_item } ;
module_item = use_declaration | module_declaration | definition ;
module_declaration = [ visibility ] "mod" identifier ( ";" | "{" { module_item } "}" ) ;
visibility = "pub" | "pub" "(" visibility_scope ")" ;
visibility_scope = "crate" | "super" | "self" | "in" visibility_path ;
visibility_path = path_segment { "::" path_segment } ;
path_segment = ( ascii_alphanumeric | "_" ) { ascii_alphanumeric | "_" } ;
identifier = identifier_start { identifier_continue } ;
identifier_start = ascii_letter | "_" ;
identifier_continue = ascii_letter | ascii_digit | "_" | "-" ;
ascii_alphanumeric = ascii_letter | ascii_digit ;
use_declaration = ordinary_use_declaration | notation_use_declaration ;
ordinary_use_declaration = [ visibility ] "use" ordinary_use_path [ "as" path_segment ] ";" ;
notation_use_declaration = "use" simple_path "::" "(" notation_selector ")" ";" ;
direct_use = use_declaration ;
ordinary_use_path = simple_path
                  | simple_path "::" "*"
                  | simple_path "::" "{" [ use_item { "," use_item } [ "," ] ] "}" ;
simple_path = path_segment { "::" path_segment } ;
use_item = path_segment [ "as" path_segment ] ;
notation_selector = notation_part { notation_part } ;
notation_part = "_" | notation_word | symbolic_operator_token ;
notation_word = ascii_letter { identifier_continue }
              | "_" identifier_continue { identifier_continue } ;
symbolic_operator_token = symbolic_operator_char { symbolic_operator_char } ;
symbolic_operator_char = "!" | "$" | "%" | "&" | "*" | "+" | "-" | "."
                       | "/" | "<" | "=" | ">" | "?" | "@" | "^" | "|" | "~" ;
```

`notation_selector` is nonempty. Lexical whitespace and comments between its atoms are
insignificant and normalize away. Bare `_` is the hole production; `notation_word` excludes bare
`_` while accepting ordinary words and underscore-prefixed words such as `_name` and `__` as
tokens. `symbolic_operator_token` is one maximal nonempty sequence of the enumerated characters,
exactly matching the notation-declaration parser's symbolic-token carrier. Thus `<*>` is one
structured atom without raw-text fallback, while `_ between _ and _` becomes the ordered parts
hole, `between`, hole, `and`, hole. Whole-import `as` and visibility belong only to the ordinary
form; a visibly qualified notation use rejects as unsupported.

`path_segment` is a direct-parser token. `import_text` is abstract source text consumed by the
Engine's prelude routes. A path segment uses the direct-path parser's ASCII
alphanumeric-or-underscore rule, which differs from an ordinary source identifier's
first-character rule.

The two Engine prelude routes are intentionally separate:

```ebnf
ordinary_module_import_prelude = { ordinary_module_import } ;
ordinary_module_import = ( "use" | "pub" "use" ) import_text [ ";" ] ;
runtime_entry_import_prelude = { runtime_entry_import } ;
runtime_entry_import = "use" import_text [ ";" ] ;
```

### Reading the rules

- `module_file` parses zero or more `module_item` values and skips whitespace and comments between
  them. Its legacy `ModuleFile` projection stores module declarations separately from definitions;
  the source-acquisition `ModuleBody` retains ordered `use` items as syntax only.
- `module_item` chooses a use declaration, a module declaration, or one definition from the form
  that owns that definition's grammar. It does not resolve an import or grant visibility.
- `module_declaration` starts with an optional visibility modifier, then `mod` and an ordinary
  identifier. It ends with `;` for a file-based module or contains zero or more definitions in
  braces for an inline module.
- `visibility` spells either plain `pub` or `pub(...)`. The surrounding `[]` in
  `module_declaration` and `ordinary_use_declaration` makes the whole modifier optional; when
  absent, the parser records inherited visibility. `notation_use_declaration` has no visibility
  alternative.
- `visibility_scope` selects the text inside `pub(...)`: `crate`, `super`, `self`, or `in` followed
  by a path.
- `visibility_path` is one or more `path_segment` values joined by `::`. It applies only to
  `pub(in ...)`; it does not describe an import path.
- `path_segment` is a route-specific name. It accepts one or more ASCII letters, digits, or `_`,
  including a leading digit. The direct-import and restricted-visibility parsers use this rule,
  not the ordinary identifier rule.
- `identifier` names a module with the ordinary source-name rule. `identifier_start` requires an
  ASCII letter or `_`; `identifier_continue` permits ASCII letters, digits, `_`, and `-` after it.
  The parser also rejects reserved words.
- `ascii_alphanumeric` is the shared character class for `path_segment`: one ASCII letter or digit.
  `ascii_letter` and `ascii_digit` name the usual ASCII character classes.
- `use_declaration` chooses `ordinary_use_declaration` or `notation_use_declaration`, and
  `direct_use` is the standalone `parse_use` route for either alternative. Both end with `;`.
- `ordinary_use_declaration` alone may have visibility or a whole-import alias.
  `ordinary_use_path` chooses a simple path, a glob below that path, or a brace list below that
  path.
- `notation_use_declaration` has inherited visibility and selects one exact nonempty parenthesized
  token/hole pattern below a simple module path. It admits neither an alias nor a glob; every
  visibly qualified notation use rejects because notation re-export is not defined here.
- `notation_selector` is a structured sequence of notation-word/symbolic-token atoms and bare `_`
  holes. `_name` and `__` are words rather than holes. Whitespace/comments normalize between atoms,
  and raw selector spelling is never matching authority.
- `simple_path` is one or more `path_segment` values joined by `::`. It is the base path shared by
  all three `ordinary_use_path` forms and by the notation-import form.
- `use_item` names one selection in a brace list and may give it a local alias. The enclosing list
  may be empty and may end with a trailing comma.
- `ordinary_module_import_prelude` is the Engine module loader's leading run of ordinary imports.
  The braces mean zero or more adjacent `ordinary_module_import` values at the start of the file;
  the scan stops at the first non-import, non-comment line.
- `ordinary_module_import` describes one ordinary loader import: `use` or `pub use`, abstract
  import text, and an optional semicolon. The loader adds a missing semicolon before it calls
  `parse_use` only when the import does not contain `@`; an import containing `@` takes the
  separate versioned-import path.
- `runtime_entry_import_prelude` is the runtime entry route's leading run of imports. The braces
  mean zero or more adjacent `runtime_entry_import` values after leading trivia; the scan stops at
  the first other source text.
- `runtime_entry_import` describes one bare `use` line with abstract import text and an optional
  semicolon. The entry path then checks whether the import names a registered runtime module.
- `definition` remains an abstract parser domain because the supported definition forms have their
  own grammars. `import_text` also remains abstract: the two Engine scans first select source text,
  then the direct parser or runtime registration check determines whether that text is valid.

The ordinary module loader accepts a leading run of ordinary imports. For imports without `@`, it
normalizes a missing semicolon before direct `parse_use`; imports containing `@` take the separate
versioned-import path. The runtime-entry prelude accepts only bare `use` imports and then applies
its registered-import whitelist. Neither route makes arbitrary `import_text` a valid program
import.

## What the loader does

No source-level sequent is supplied because the implementation exposes parser and module-summary
procedures rather than a checked formal module calculus. The relevant operational facts are:

1. The shared parser body accepts `use`, `mod`, and definition items. Legacy `module_file` stores
   only `mod` declarations and top-level definitions, while the source-unit route retains ordered
   `use` syntax without binding it.
2. `parse_module_imports` scans only a leading import prelude. For imports without `@`, it
   normalizes a missing semicolon before calling `parse_use`; imports containing `@` take the
   separate versioned-import path.
3. `mask_leading_entry_use_prelude` removes an accepted runtime-entry prelude before source-body
   parsing; `validate_runtime_entry_import_prelude` rejects unsupported registrations.

These are bounded loader/entry mechanisms, not authority, provider, or general execution rules.

## Errors and limits

- `use` is a shared parser item, but legacy `ModuleFile` projects it away and `ModuleUnit` keeps it
  as syntax only. Neither result supplies import binding, checked visibility, or runtime authority.
- **Important:** semicolon-free `use` is an Engine prelude convenience. Direct `parse_use` needs
  `;`.
- Inline modules parse, but the authoritative Engine check rejects the tested inline ordinary-type
  case. Do not infer general checked module support from parser acceptance.
- Import resolution fails closed for missing modules, cycles, unavailable locked dependencies, and
  visibility violations demonstrated by the module-loader tests.
- Visibility and imported summary availability do not grant runtime authority or prove a callable
  executes.
- Removed workflow module syntax is excluded.

## Related evidence

- [AUDIT-206 LANG-001 and LANG-002](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2052: entry and Engine admission](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
- `cargo test -p ash-engine --test module_import_resolution_tests`
- `cargo test -p ash-engine --test module_file_check_tests`
