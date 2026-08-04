# TASK-2068: Final Interfaces, Parsed Imports, and Binder Integration

**Status:** Complete — partial/tested/below-spec foundation
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5-9 and §11 (`M-EXPAND` through `M-CHECK`)
**Owned rules:** MOD-REAL-003 and MOD-REAL-004
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Activation boundary:** The bounded Type-layer work is `partial / tested / below_spec`: it
realizes provisional function `M-COLLECT` facts, bounded graph-only simple-import
`M-IMPORT-EDGE`/`M-IMPORT-CYCLE` planning, and a closed `M-CHECK` leaf pass over graph-delivered
ordinary primitive functions. It also implements the bounded canonical primitive provider/client
check sub-slice with tested, non-authorizing checked provider/client/import facts; it neither
publishes a final module
interface nor broadens into remaining import, lowering, admission, or client behavior. Its
delivered direct-public primitive re-export interface-fragment sub-slice is `partial / tested /
below_spec`; it remains a narrow non-authorizing Type-layer fact, not a final interface, general
binder, lowering, admission, runtime, or client behavior claim. Its delivered private primitive
provider-helper companion is also `partial / tested / below_spec`: helpers remain private, the
focused target passes 7/7 including a 16-case property, and the result is likewise only Type-layer
test evidence. Its delivered local-binding root-client companion is also `partial / tested /
below_spec`: it checks private root functions through the explicit alias, passes 10/10 including a
16-case property, and remains only non-authorizing Type-layer test evidence.
The canonical provisional-module-scope and structural-path visibility slice is delivered as
`partial / tested / below_spec` Type-layer evidence only; it does not complete the task or
activate any downstream layer. Its delivered scoped grouped ordinary-function import M-GROUP
companion is likewise `partial / tested / below_spec`: parser-owned member spans and the dedicated
scope-backed resolver/binder admit only the stated inherited `crate` group grammar, stage the
whole group atomically, and remain non-authorizing Type-layer facts.
**Semantic coverage map:** [TASK-2068 final interfaces, parsed imports, and binder integration](../SEMANTIC-RULE-COVERAGE.md#task-2068-final-interfaces-parsed-imports-and-binder-integration)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Bounded graph-only simple-import planning realizes M-IMPORT-EDGE and M-IMPORT-CYCLE only over graph real units/provisional function targets: inherited UsePath::Simple crate-root function aliases yield opaque resolved imports/bindings plus canonical edge `(importer, defining module/identity, local spelling, use span, decl span, origin, visibility)`; same-module aliases produce no edge; every discovered cycle rejects before any result as `ImportCycle { edges: CanonicalImportCycle }`, whose ordered wrapper exposes the parser-anchored edges; and `bind_simple_parsed_uses` delegates through the planner. M-CHECK realizes only self-contained graph-delivered leaf modules containing ordinary public/inherited functions with primitive closed signatures: it graph-preflights every unit, stages sibling signatures, atomically checks all bodies through the builtin TypeEnv body checker, retains a fresh checked identity plus ModuleKey/origin/spans/signature/body type, and yields only a private checked-function map plus a non-authorizing CanonicalPublicFunctionInterface that exports public primitive signatures. Bounded canonical primitive provider/client checking realizes only the root plus plan-selected direct provider leaves: `check_primitive_provider_client(graph, plan)` requires exact plan/graph artifact provenance; pre-provider graph-wide `module_units()` completeness rejects any unrelated unselected non-root graph unit, including a nested module; a descendant of a selected provider instead reaches provider-leaf precheck and rejects as anchored `UnsupportedProviderShape`; it prechecks primitive provider leaves, revalidates planned edges against checked public providers, injects their signatures for fresh root checking, and atomically returns checked root/provider facts plus non-authorizing import-binding facts. It does not widen the delivered leaf pass, publish final interfaces, or use legacy carriers, Core/CPS, Engine, or runtime authority. Complete M-COLLECT across every required namespace and callable/body fact; M-CHECK for all remaining forms; complete M-IMPORT-EDGE, M-IMPORT-CYCLE, and M-BIND semantics for every remaining parsed use/path/visibility/alias/re-export form, export-closed final interfaces, and atomic dependency closures; complete definition-body Core/CPS lowering and Engine scanner/cache transport fencing; Engine-linked admission; and real-program file/inline plus CLI/daemon normalized-terminal parity remain deferred.
**Current bounded M-CHECK addition:** The preceding inherited/public leaf summary is extended only by
the delivered `partial / tested / below_spec` restricted-visibility leaf: `pub(crate)`,
`pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` primitive closed ordinary
functions in the same file-root closed domain are checked privately; `pub(in self::internal)`
rejects. The public projection remains only `Visibility::Public`. Its focused target passes 18/18;
the file/inline-named witness is a source-form boundary, not normalized-success parity.
**Layers:** type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-003-PROVISIONAL-FUNCTION-COLLECTION`,
`TEST-MOD-REAL-004-PARSED-IMPORT-BINDING`, `TEST-MOD-REAL-004-ALIAS-IDENTITY-PROPERTY`,
`TEST-MOD-REAL-004-PLANNER-EDGE-PROVENANCE`, and
`TEST-MOD-REAL-004-PLANNER-SAME-MODULE-NO-EDGE`; positive fail-closed boundary
`TEST-MOD-REAL-004-PUB-USE-REJECTION`;
negative `TEST-MOD-REAL-004-VISIBILITY-DIAGNOSTIC` and
`TEST-MOD-REAL-004-RESTRICTED-VISIBILITY-REJECTION`, and
`TEST-MOD-REAL-004-CANONICAL-BINDER-FENCE`,
`TEST-MOD-REAL-004-PLANNER-UNSUPPORTED-SHAPE`,
`TEST-MOD-REAL-004-PLANNER-ORDERED-CYCLE-DIAGNOSTIC`, and
`TEST-MOD-REAL-004-PLANNER-BINDER-DELEGATION-FENCE`; atomicity control
`TEST-MOD-REAL-004-PRIVATE-ALIAS-ATOMICITY` and
`TEST-MOD-REAL-004-PLANNER-CYCLE-ATOMICITY`; full-provenance tail-cycle diagnostic
`TEST-MOD-REAL-004-PLANNER-TAIL-CYCLE-PROVENANCE`; provider/client positive
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-POSITIVE` and property
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROPERTY`; provider/client negative
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LEAF-REJECTION`,
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-CLIENT-MISMATCH-DIAGNOSTIC`,
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ARTIFACT-SNAPSHOT-MISMATCH`,
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LOCAL-IMPORT-COLLISION`,
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-IMPORT-REJECTION`,
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-DEEP-TOPOLOGY-REJECTION`, and
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-COMPLETENESS`, and
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-PREFLIGHT-ORDERING`; provider/client mutation
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ATOMICITY`; provider/client fence
`TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-AUTHORITY-FENCE`; no proof.
Delivered direct-public re-export fragment evidence is positive
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
16-case property; these tests are evidence, not a proof or a full interface/import/parity claim.
**Record-mirrored delivered-fragment target clause:** The delivered direct-public primitive re-export interface fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: `resolve_direct_primitive_interface_imports` requires a nonempty exact root `pub use crate::<direct-provider>::<primitive-function> as <alias>` plan and rejects a public re-export lacking `as <alias>` as anchored `Unsupported` with `an explicit re-export alias is required` before plan publication, while `check_direct_primitive_interface_fragments` consumes only the canonical root plus plan-selected direct primitive providers, exact artifact facts, and bounded provider/client facts; it admits only root `pub mod api` and explicit root re-exports, preserves defining identity/origin/checked primitive signature/declaration/use spans, forbids implicit flattening, rejects non-public paths, private/non-primitive targets, empty plans, root-shape/collision conditions, and mismatched artifacts before atomically returning only a non-authorizing fragment. The direct target passes 13/13, including a 16-case property; these test witnesses are evidence, not proof. Full M-COLLECT/M-CHECK/M-IMPORT-EDGE/M-IMPORT-CYCLE/M-BIND, final interfaces/export closure, lowering/admission/runtime/client parity remain deferred.
`TEST-MOD-REAL-003-FINAL-INTERFACE`,
`TEST-MOD-REAL-003-EXPORT-CLOSURE-REJECTION`,
`TEST-MOD-REAL-003-REEXPORT-IDENTITY-MUTATION`,
`TEST-MOD-REAL-004-BINDER-ATOMICITY-MUTATION`, and
`TEST-MOD-REAL-003-004-INTERFACE-PARITY` remain deferred. Delivered M-CHECK evidence is positive
`TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC`,
`TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC-PROPERTY`; negative
`TEST-MOD-REAL-003-LEAF-MCHECK-BODY-MISMATCH-DIAGNOSTIC`,
`TEST-MOD-REAL-003-LEAF-MCHECK-OPTION-CLOSED-INTERFACE-REJECTION`, and
`TEST-MOD-REAL-003-LEAF-MCHECK-UNSUPPORTED-SHAPE`, and
`TEST-MOD-REAL-003-LEAF-MCHECK-NESTED-PREFLIGHT-REJECTION`; mutation
`TEST-MOD-REAL-003-LEAF-MCHECK-SIBLING-ATOMICITY`, and
`TEST-MOD-REAL-003-LEAF-MCHECK-INTERFACE-FENCE`; no proof. The focused
`task_2068_canonical_function_interface` target passes 8/8, including 16 generated public integer
functions. The focused `task_2068_parsed_import_binder` target passes 11/11, including 16
generated aliases; it covers edge provenance, same-module no-edge, ordered file/inline two-node
cycle edges, a full-provenance `a → b → c → b` tail diagnostic that reports only `b ↔ c`,
late-backedge atomic rejection, and compatibility-binder delegation. Its architecture fence also
rejects `RawCoreProgram`, `CoreExpr`, and `CpsProgram` authority alongside the existing legacy,
interface, acquisition, checker, lowering, and runtime bypasses. It covers real
graph-unit sibling body checking with parser provenance and distinct
private/public projection; anchored body mismatch; public `Option` rejection at the closed
interface boundary; late failure atomicity; parsed-`use` shape and nested-child global-preflight
rejection; and the architecture fence against legacy final-interface, binder, source-acquisition,
Core/CPS, and runtime bypasses.
**Historical pre-closure next obligation:** Extend the canonical graph-only Type-layer slice beyond the delivered scoped grouped ordinary-function imports M-GROUP parser-span/resolver/binder slice, scoped simple ordinary-function imports M-SIMPLE slice, dedicated scope-backed structural binder, scoped structural import-cycle gate, canonical provisional module-scope/structural-path visibility, direct-public primitive re-export interface, and local-binding root-client fragments, provisional-function M-COLLECT/simple crate-root alias target resolution, closed primitive M-CHECK leaf pass, bounded simple-import M-IMPORT-EDGE/M-IMPORT-CYCLE planner, and direct primitive provider/client checker to every required namespace, remaining definition/body and export-closure check, every remaining parsed import/visibility/alias/re-export/cycle rule, and atomic M-BIND publication; TASK-2069 then owns complete lowering and Engine transport fencing, and TASK-2064 owns integration parity.
**Current successor ownership:** TASK-2068 is closed. TASK-2071 owns all remaining M-COLLECT and provisional namespace/callable collection; TASK-2072 owns all remaining parsed imports/visibility/edges/cycles/precedence/atomic binding and staged `pub use`; and TASK-2073 owns complete M-CHECK bodies, final interfaces, and export closure. TASK-2069 consumes only the complete TASK-2073 handoff.
**Bounded-slice non-goals:** Structural graph discovery or source acquisition. New syntax, dynamic imports, packages, import-cycle initialization, or runtime module values. Outside the delivered M-CHECK leaf pass, all typed namespaces beyond ordinary functions; all definition forms beyond ordinary functions; generic or contract-bearing functions; restricted visibility; non-primitive/open signatures; user-defined types, interfaces, and effects; final public/private interface publication; export closure; public aliases, re-exports, or pub use. Delivered M-CHECK excludes imports, child modules, nested modules, other definitions, generics, contracts, unsupported visibility, user-defined types, interfaces, effects, re-exports, final full interfaces, Core/CPS/Engine, and client parity. Delivered graph-only simple-import planning excludes checked interfaces, TypeEnv/body integration, legacy or TASK-2060/TASK-2061/TASK-2066 authority, restricted visibility, pub use/re-exports, groups/globs/qualified paths, every other import form, Core/CPS/Engine, and client parity. Beyond the delivered planner's inherited UsePath::Simple crate-root function aliases, parsed use forms; qualified paths, group/glob imports, non-inherited use visibilities, restricted declaration visibilities, complete visibility handling, remaining import-cycle rules, or legacy binder/graph/interface authority remain excluded. Delivered canonical primitive provider/client checking excludes any widening of the delivered primitive leaf pass; any non-root client, non-plan-selected direct provider, unrelated unselected graph unit, or non-direct/nested provider; non-primitive/open signatures; final interfaces or export closure; import forms beyond the delivered planner; legacy TASK-2060/TASK-2061/TASK-2066 carriers; and Core/CPS/Engine, admission, runtime, or client parity. Delivered direct-public primitive re-export interface fragment excludes every namespace, declaration/import/path/visibility/re-export form except root `pub mod` direct-provider identity plus exact root `pub use crate::<direct-provider>::<primitive-function> as <alias>`; it also excludes compatibility carriers, final interface/export closure, Core/CPS/Engine, admission/runtime, and parity. Direct Core/CPS lowering, Engine scanner/cache fencing, linking/admission/execution, or CLI/daemon parity. Treating an interface or binder fact as an Engine admission credential, provider/handler-frame authority, or direct-evaluator fallback.

## Historical delivered-slice record (pre-closure)

The delivery detail in this section preserves TASK-2068's source and test evidence attribution.
Any statement here that says TASK-2068 “retains”, “owns”, or that TASK-2069 “cannot begin until
TASK-2068 is complete” is a contemporaneous delivery statement, superseded by the successor
ownership above; it is not a current obligation.

## Delivered M-CHECK leaf slice

**Status:** Delivered and tested before this task closed. It preserves the task's
`partial / tested / below_spec` accounting because its selected leaf domain is far below the
complete module rule.

- **Consumes:** only canonical graph `ModuleUnit`/artifact identity and source-span facts plus the
  builtin TypeEnv body checker. It may not acquire source, inspect a legacy graph or interface,
  resolve an import, or use a Core, CPS, Engine, or runtime authority.
- **Admitted domain:** a graph-delivered, self-contained leaf module containing only ordinary
  functions with inherited or public visibility and primitive, closed signatures. Every import,
  child or nested module, other definition kind, generic, contract, unsupported visibility,
  user-defined type, interface, effect, or re-export is outside this sub-slice and must remain
  unsupported rather than silently widened.
- **Transaction:** graph-preflight every unit, stage all sibling signatures before checking any
  function body, then check every body against that staged sibling view. If any preflight,
  signature, or body fails, publish neither checked-function map nor interface. Every retained
  checked fact records a fresh checked identity, `ModuleKey`, origin, declaration/body spans,
  signature type, and checked body type.
- **Produces:** a private checked-function map and a deliberately limited,
  non-authorizing `CanonicalPublicFunctionInterface`. It exports only public primitive signatures;
  it is not core `PublicModuleInterface`, a final module interface, an import/binder credential,
  or an admission/frame/execution authority.
- **Downstream and run route:** TASK-2068 retains all remaining Type-layer interface/import/binder
  clauses. TASK-2069 still separately owns complete lowering and Engine scanner/cache transport
  fencing; TASK-2064 separately owns file/inline and CLI/daemon normalized-terminal parity. This
  sub-slice has `prerequisite` run-route impact only.
- **Task-owned evidence:** `task_2068_canonical_function_interface` passes 8/8. Positive
  `TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC` covers real graph-unit sibling checks,
  provenance, fresh identity, and private/public projection; property
  `TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC-PROPERTY` covers 16 generated public integer
  functions. Negative `TEST-MOD-REAL-003-LEAF-MCHECK-BODY-MISMATCH-DIAGNOSTIC`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-OPTION-CLOSED-INTERFACE-REJECTION`, and
  `TEST-MOD-REAL-003-LEAF-MCHECK-UNSUPPORTED-SHAPE` cover anchored mismatch, closed-signature,
  and `use`-shape rejection. Negative
  `TEST-MOD-REAL-003-LEAF-MCHECK-NESTED-PREFLIGHT-REJECTION` proves nested children reject during
  graph-wide preflight at the root declaration anchor. Mutation
  `TEST-MOD-REAL-003-LEAF-MCHECK-SIBLING-ATOMICITY` proves a late failure publishes nothing;
  `TEST-MOD-REAL-003-LEAF-MCHECK-INTERFACE-FENCE` verifies no legacy final-interface, binder,
  source, Core/CPS, or runtime bypass. These tests are evidence, not a proof or whole-feature
  parity claim.

## Delivered graph-only simple-import planner

**Status:** Delivered and tested before this task closed. It preserves the existing
`partial / tested / below_spec` accounting because its selected import/cycle domain is far below
the complete module rule.

- **Consumes:** only real units in TASK-2067's canonical graph and the provisional function targets
  already collected from those units. It admits only inherited `UsePath::Simple` crate-root aliases
  to functions; it may not acquire source, consult a checked interface or TypeEnv/body fact, or use
  a legacy graph/binder or TASK-2060/TASK-2061/TASK-2066 carrier as authority.
- **Produces:** opaque resolved-import/binding facts plus one canonical edge per cross-module
  alias: `(importer, defining module/identity, local spelling, use span, decl span, origin,
  visibility)`. A same-module alias resolves without an edge. Neither result is a final interface,
  export fact, general name-resolution authority, or Engine credential.
- **Transaction and integration:** form edges before any result is published and deterministically
  reject every discovered cycle as `CanonicalModuleBindError::ImportCycle { edges:
  CanonicalImportCycle }`; the ordered wrapper exposes the canonical parser-anchored edges. The
  compatibility `bind_simple_parsed_uses` delegates to this planner; no direct binder path may
  bypass cycle planning or publish a partial result.
- **Excluded domain:** checked-interface/TypeEnv/body integration; legacy and TASK-2060/2061/2066
  authority; restricted visibility; `pub use`/re-exports; qualified, grouped, glob, non-inherited,
  or non-crate-root use forms; child/nested modules; all other definitions; Core/CPS/Engine; and
  client behavior. No new syntax or runtime module semantics is authorized.
- **Downstream and run route:** TASK-2068 retains remaining Type-layer interface/import/binder
  clauses. TASK-2069 separately owns complete lowering and Engine scanner/cache transport fencing;
  TASK-2064 separately owns file/inline and CLI/daemon normalized-terminal parity. This has
  `prerequisite` run-route impact only.
- **Task-owned evidence:** `task_2068_parsed_import_binder` passes 11/11. Positive
  `TEST-MOD-REAL-004-PLANNER-EDGE-PROVENANCE` proves the canonical edge tuple and compatibility
  binder delegation; `TEST-MOD-REAL-004-PLANNER-SAME-MODULE-NO-EDGE` proves same-module aliases
  bind without a dependency edge. Negative
  `TEST-MOD-REAL-004-PLANNER-UNSUPPORTED-SHAPE` covers retained fail-closed import-form and
  visibility errors; `TEST-MOD-REAL-004-PLANNER-ORDERED-CYCLE-DIAGNOSTIC` proves a file/inline
  two-node cycle returns ordered use-span edges through `ImportCycle { edges:
  CanonicalImportCycle }`. `TEST-MOD-REAL-004-PLANNER-TAIL-CYCLE-PROVENANCE` proves the
  `a → b → c → b` tail reports only the ordered `b ↔ c` edges while retaining their declaration
  spans, origins, and visibility. Mutation `TEST-MOD-REAL-004-PLANNER-CYCLE-ATOMICITY` proves a late
  backedge rejects without publishing the former acyclic plan; architectural fence
  `TEST-MOD-REAL-004-PLANNER-BINDER-DELEGATION-FENCE` rejects planner/binder authority or bypass,
  including `RawCoreProgram`, `CoreExpr`, and `CpsProgram`. These tests are evidence, not a proof
  or full-import/parity claim.

## Delivered canonical primitive provider/client check sub-slice

**Status:** Delivered and tested before this task closed. It preserves the task's
`partial / tested / below_spec` accounting because it admits only root plus plan-selected direct
provider leaves, far below complete module semantics.

- **Entry point and admitted domain:** `check_primitive_provider_client(graph, plan)` consumes only
  the canonical graph and its bounded resolved-simple-import plan. It admits only the root plus
  plan-selected direct provider leaves and graph-wide `module_units()` completeness rejects every
  unrelated unselected graph unit, including a nested module, before provider checking. A descendant
  of a selected provider reaches the existing provider-leaf precheck and rejects as anchored
  `UnsupportedProviderShape`; no nested module can succeed. Every provider satisfies the existing
  primitive leaf precheck. Nested providers, non-root clients, non-primitive/open signatures, and
  any broad module shape remain outside this delivered domain.
- **Provenance and transaction:** before checking, it requires `plan` to match the exact graph
  artifacts, revalidates every consumed canonical import edge against checked public providers, and
  injects only the revalidated imported provider signatures into a fresh root `TypeEnv`. If plan/
  graph provenance,
  provider-leaf precheck, edge revalidation, signature injection, or root check fails, it must
  publish neither checked root/provider facts nor import-binding facts.
- **Produces and authority:** on success it atomically returns checked root/provider facts plus
  non-authorizing import-binding facts. These are not a final interface, export closure, general
  import/binder authority, admission credential, provider/handler-frame authority, or Core/CPS/
  Engine artifact.
- **Excluded domain:** this does not widen the delivered M-CHECK leaf pass or the delivered
  simple-import planner; it excludes all other import/visibility/path forms, every legacy or
  TASK-2060/TASK-2061/TASK-2066 carrier, Core/CPS/Engine, admission/runtime, and client parity.
- **Downstream and run route:** TASK-2068 retains every remaining Type-layer interface/import/
  binder clause. TASK-2069 separately owns complete lowering and Engine scanner/cache transport
  fencing, and TASK-2064 separately owns file/inline and CLI/daemon normalized-terminal parity.
  This sub-slice has `prerequisite` run-route impact only.
- **Task-owned evidence:** `task_2068_primitive_provider_client` passes 12/12. Positive
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-POSITIVE` retains direct inline/file provider and
  root-client artifacts, checked identities/spans/types, and import binding provenance; property
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROPERTY` covers 16 generated primitive pairs.
  Negative `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LEAF-REJECTION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-CLIENT-MISMATCH-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ARTIFACT-SNAPSHOT-MISMATCH`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-LOCAL-IMPORT-COLLISION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-IMPORT-REJECTION`,
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-PROVIDER-DEEP-TOPOLOGY-REJECTION`, and
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-COMPLETENESS` cover the primitive provider
  boundary, anchored client mismatch, plan/graph artifact mismatch, collision, provider-import,
  deep-provider, and unselected nested-module topology failures. Negative
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-TOPOLOGY-PREFLIGHT-ORDERING` proves that global
  unselected-unit rejection occurs before a malformed selected provider is checked. Mutation
  `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-ATOMICITY` proves a late root-body failure publishes
  no result; architectural fence `TEST-MOD-REAL-004-PRIMITIVE-PROVIDER-CLIENT-AUTHORITY-FENCE`
  rejects legacy binder/final-interface and Core/CPS/Engine/runtime bypasses. These tests are
  evidence, not a proof or full-interface/import/parity claim.

## Delivered direct primitive public re-export interface-fragment sub-slice

**Status:** Delivered and tested before this task closed; `implementation: partial`,
`evidence: tested`, and `parity: below_spec`. Its layers are Type `partial`;
Core/CPS/admission-runtime `not_applicable`; verification `partial`. This scoped implementation
does not complete TASK-2068 or Phase 207.

- **Canonical rule and admitted domain:** This realizes only the SPEC-103 §§3, 6--8 M-BIND/M-CHECK
  fragment for the canonical root plus plan-selected direct primitive provider leaves. The admitted
  public form is root `pub mod api` plus root `pub use crate::api::greet as welcome`; providers
  remain direct primitive leaves and graph/plan artifacts revalidate exactly. The separate
  `resolve_direct_primitive_interface_imports` planner refuses an empty public-use plan.
- **Consumes:** TASK-2067's canonical graph/module-unit/artifact facts, the exact direct-public
  simple-import plan, and bounded provider/client checked facts only as non-authorizing validation
  inputs. It may not acquire source, scan text, or read a legacy carrier.
- **Produces:** successful checking atomically returns only a constructor-free,
  non-authorizing `CanonicalPrimitiveInterfaceFragments` fact containing the public structural
  child path and explicit root alias. The alias retains defining identity, origin, checked
  primitive signature, declaration span, and use span. `pub mod api` never implicitly produces a
  root `greet` binding.
- **Public closure and transaction:** direct child structural paths and target declarations must be
  public before aliases stage. Non-public paths, private/non-primitive targets, root function or
  child-identity alias collisions, plan/artifact mismatches, unsupported public root definitions,
  and late failures return no fragment. `pub mod api` plus `pub use crate::api::greet as api`
  rejects with structural and use anchors rather than overwriting child identity. The result is not
  a final interface or general binder fact. A public re-export lacking `as <alias>` rejects as
  anchored `Unsupported` with `an explicit re-export alias is required` before plan publication.
- **Explicit-alias rejection evidence:**
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-IMPLICIT-NAME-REJECTION` verifies this
  fail-closed boundary. Full `pub use` support remains deferred under TASK-2068.
- **Task-owned evidence:** the focused
  `task_2068_direct_primitive_reexport_interface_fragments` target passes 13/13, including a
  16-case property. Positive
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-POSITIVE` covers export-closed,
  non-flattened construction. Negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-NONPUBLIC-PATH`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-PRIVATE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-NONPRIMITIVE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-IMPLICIT-NAME-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-COLLISION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ARTIFACT-SNAPSHOT-MISMATCH`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-EMPTY-ROOT-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ROOT-SHAPE-REJECTION`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-CHILD-ALIAS-COLLISION` cover the
  fail-closed boundaries. Property
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-PROPERTY` preserves identity,
  provenance, signature, and use spans; mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-ATOMICITY` proves no late invalid
  re-export publishes a fragment; fence
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-FRAGMENT-AUTHORITY-FENCE` excludes compatibility
  and runtime authority. These are test evidence, not proof or full-interface/import/parity
  evidence.
- **Source traceability:** `resolve_direct_primitive_interface_imports` is fingerprinted as
  `sha256:a01d1ba20a02acae576626fa81172c1540bdb39ee5ff1c9a31addc34b3dc211e`; the direct fragment
  checker is fingerprinted as
  `sha256:e390550b79606602171a810de4f568858f9b54de1906da9a6de2265f76f58ede` in semantic traceability.
- **Non-goals:** all other namespaces, definition/import/path/visibility/re-export forms,
  compatibility carriers, final interfaces/export closure beyond this fragment, Core/CPS, Engine,
  admission, runtime, and file/inline or CLI/daemon parity.
- **Downstream and integration:** TASK-2068 still owns remaining Type-layer interface, import, and
  binder clauses; TASK-2069 owns lowering and Engine transport fencing; TASK-2064 separately owns
  integrated file/inline and CLI/daemon normalized-terminal parity. This delivered sub-slice has
  `prerequisite` run-route impact only.

## Delivered private primitive provider-helper sub-slice

**Status:** Delivered and tested before this task closed; `implementation: partial`,
`evidence: tested`, and `parity: below_spec`. Its layers are Type `partial`;
Core/CPS/admission-runtime `not_applicable`; verification `partial`. This delivered sub-slice
does not complete TASK-2068 or Phase 207.

- **Canonical rule and domain:** Retain only root `pub mod api` plus exact root
  `pub use crate::api::greet as welcome`. The selected re-export target remains public, but its
  direct provider may contain inherited/private ordinary primitive helpers usable only within that
  provider. Helpers are checked before publication and never become public bindings.
- **Consumes:** TASK-2067 canonical graph/module-unit/artifact facts, exact planned aliases, and
  bounded provider checker facts. The generic planner and compatibility binder remain unchanged.
- **Produces:** on complete success, only the same constructor-free, non-authorizing
  `CanonicalPrimitiveInterfaceFragments` output. It retains the public structural child and
  explicit public target alias; it never exposes helper identity, signature, span, or binding.
- **Task-owned evidence:** The focused `task_2068_private_primitive_provider_helpers` target
  passes 7/7, including a 16-case property. Positive
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-POSITIVE`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-FILE-INLINE-PARITY`,
  and `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-PROPERTY` show that checked
  helpers remain private and file/inline providers normalize equally. Negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-PRIVATE-TARGET`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-NONPRIMITIVE`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-AUTHORITY-FENCE` retain
  fail-closed visibility, signature, and authority boundaries; mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-ATOMICITY` verifies that a late
  invalid helper publishes no fragment. These are test evidence, not proof or end-to-end parity
  evidence.
- **Non-goals:** provider uses or nested modules; other definitions; generic or contract-bearing
  functions; restricted visibility; non-primitive/open signatures; other paths or re-export forms;
  final interfaces/export closure; compatibility carriers; Core/CPS; Engine/admission/runtime; and
  file/inline or CLI/daemon parity.
- **Handoff:** partial/tested. TASK-2068 retains every complete-interface/import/binder clause and
  TASK-2069 cannot begin until TASK-2068 is complete; TASK-2064 separately owns integration and
  parity proof. No authority transfers to Core/CPS, admission, runtime, or clients.
- **Record-mirrored delivered-helper target clause:** The delivered private primitive provider-helper fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: it retains only exact root `pub mod <provider>` plus `pub use crate::<provider>::<public-primitive> as <alias>`, admits inherited/private ordinary primitive provider helpers only as checked implementation detail, excludes them from `CanonicalPrimitiveInterfaceFragments`, and rejects a private selected target before publication. It consumes canonical graph, exact planned aliases, and bounded provider checker facts; it atomically produces only the same non-authorizing fragment after every provider/helper check succeeds. The focused `task_2068_private_primitive_provider_helpers` target passes 7/7, including a 16-case property; these test witnesses are evidence, not proof. Provider uses, nested modules, other definitions, generics, contracts, restricted visibility, non-primitive/open signatures, all other paths, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered direct public primitive re-export local-binding root-client sub-slice

**Status:** Delivered and tested within this active task; `implementation: partial`,
`evidence: tested`, and `parity: below_spec`. Its layers are Type `partial`;
Core/CPS/admission-runtime `not_applicable`; verification `partial`. This delivered sub-slice
does not complete TASK-2068 or Phase 207.

- **Canonical rule and admitted domain:** SPEC-103 §6 requires aliases to preserve the defining
  declaration identity and visibility before registration; §§8--9 require checked local binding,
  no implicit flattening, and atomic failure. The exact admitted root contains `pub mod api` with
  inherited/private ordinary primitive helpers and public primitive `greet`, exact root
  `pub use crate::api::greet as welcome`, and inherited/private root
  `fn internal_entry(..) -> <primitive> { welcome(..) }`. Root public functions remain excluded.
- **Consumes:** TASK-2067 canonical graph/module-unit/artifact facts, exact artifact snapshots,
  a distinct opaque direct-plan kind, selected-provider facts, and the checked local alias. The
  generic planner/binder and generic provider/client route remain source-based `pub use`
  rejection controls.
- **Produces:** only an opaque non-authorizing Type handoff: the direct fragment, checked private
  root functions, selected-provider facts, and a local alias binding that retains `greet`'s
  defining identity, visibility, signature, provenance, and use span. It is not a final interface,
  generic binder, Core/CPS artifact, admission credential, runtime input, or parity result.
- **Task-owned evidence:** The focused `task_2068_direct_primitive_reexport_root_client` target
  passes 10/10, including a 16-case property. Positive
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-POSITIVE`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-FILE-INLINE-PARITY`,
  and `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PROPERTY` preserve only the
  selected public target identity. Negative
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PUBLIC-ROOT-REJECTION`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-BODY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-ARTIFACT-SNAPSHOT`,
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-PLAN-KIND-FENCE`, and
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-AUTHORITY-FENCE` retain collision,
  public-root, diagnostic, snapshot, route, and authority fences; mutation
  `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-ATOMICITY` proves no late invalid
  root client publishes staged facts. Root-body diagnostics use a direct unqualified alias-call
  span only (including an empty block tail); otherwise they use the enclosing root-body span.
  These are test evidence, not proof or end-to-end parity evidence.
- **Non-goals:** provider uses, nested modules, other definitions, generic or contract-bearing
  functions, restricted visibility, non-primitive/open signatures, all root public functions,
  all other paths/re-export forms, final interfaces/export closure, compatibility carriers,
  generic binding, Core/CPS, Engine/admission/runtime, and file/inline or CLI/daemon parity.
- **Handoff:** partial/tested. TASK-2068 retains every complete-interface/import/binder clause;
  TASK-2069 cannot begin until TASK-2068 is complete; TASK-2064 separately owns integration and
  parity proof. No authority transfers to Core/CPS, admission, runtime, or clients.
- **Source traceability:** the generic provider/client `pub use` fence is fingerprinted as
  `sha256:b55991ac7a7e7523838ee351740fde66d72ec36ff977a8307d782a4d9951de22`; the direct planner
  (including the distinct root-client plan) as
  `sha256:a01d1ba20a02acae576626fa81172c1540bdb39ee5ff1c9a31addc34b3dc211e`; and the direct
  fragment/root-client checker as
  `sha256:e390550b79606602171a810de4f568858f9b54de1906da9a6de2265f76f58ede` in semantic traceability.
- **Record-mirrored delivered-root-client target clause:** The delivered direct-public primitive re-export local-binding root-client fragment is partial/tested/below-spec with Type partial, Core/CPS/admission-runtime not_applicable, and verification partial: it admits only a root `pub mod <provider>` with inherited/private ordinary primitive helpers and one public primitive target, exact root `pub use crate::<provider>::<public-primitive> as <alias>`, and inherited/private root `fn internal_entry(..) -> <primitive> { welcome(..) }`. It consumes canonical graph and exact artifact snapshots through a distinct opaque direct plan, selected-provider facts, and a checked local alias; after every provider, alias, root-body, snapshot, and authority check succeeds, it atomically produces only a non-authorizing fragment plus checked private root functions, selected provider facts, and local alias binding while preserving the target's definition identity and visibility before registration. Root-body diagnostic anchoring recognizes only a direct unqualified `<alias>(...)` call (including an empty block tail); all other root-body failures use the enclosing root-body span. The focused `task_2068_direct_primitive_reexport_root_client` target passes 10/10, including a 16-case property; these test witnesses are evidence, not proof. The generic planner/binder and generic provider/client route continue to reject `pub use` from source; all root public functions, generic binders, remaining forms, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered canonical provisional module scopes and structural-path visibility sub-slice

**Status:** Delivered before this task closed; `implementation: partial`, `evidence: tested`, and
`parity: below_spec`. Its layers are Type `partial`; Core/CPS/admission-runtime
`not_applicable`; verification `partial`. This bounded evidence does not complete TASK-2068 or
Phase 207.

- **Canonical rule and admitted domain:** SPEC-103 §§3, 5, 6, 8, and 9 require structural paths
  to resolve from parsed canonical graph edges, aliases to preserve defining identity, visibility
  before registration, and atomic failure. The route admits only inherited simple
  `use crate::<structural-child>...::<ordinary-function> as <name>` paths over graph-unit direct
  structural children and ordinary function declarations. The bounded target domain is not
  public-only: the canonical visibility predicate admits public, crate, super, `pub(in path)`,
  inherited/private, and self regions when the importing `ModuleKey` is permitted.
- **Consumes:** TASK-2067 canonical graph units/artifacts. It builds immutable typeck-owned
  provisional scopes per `ModuleKey`, each containing direct structural children and ordinary
  function declaration entries with identity, declared visibility, origin, and source anchors.
  Before resolution, `matches_graph` compares the root and artifacts and rebuilds the declaration
  snapshot from the current parser units for equality; artifacts alone never authorize a scope
  entry. A same-path/topology snapshot that removes a function or changes it from `pub` to private
  therefore rejects `ScopeGraphMismatch` before binding. It uses `ModuleKey` crate identity,
  `parent`, and segment relations rather than a string visibility helper.
- **Produces:** on full success, one opaque non-authorizing Type handoff of provisional scopes and
  staged resolved aliases/structural import edges. Every traversed child and the final function is
  admitted before a temporary alias binding is staged; the result is not a final interface,
  general namespace binder, Core/CPS artifact, admission credential, runtime input, or parity
  result.
- **Test witnesses:**
  `TEST-MOD-REAL-004-CANONICAL-STRUCTURAL-PATH-VISIBILITY`,
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-INACCESSIBLE-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-CANONICAL-VISIBILITY-REGIONS`,
  `TEST-MOD-REAL-004-BINDER-LOCAL-DECLARATION-COLLISION`,
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-ATOMICITY`, and
  `TEST-MOD-REAL-004-STRUCTURAL-PATH-AUTHORITY-FENCE`,
  `TEST-MOD-REAL-004-CANONICAL-SCOPE-DECLARATION-SNAPSHOT-MISMATCH`, and
  `TEST-MOD-REAL-004-CANONICAL-PUBLIC-PATH-VISIBILITY-FENCE` pass in
  `task_2068_canonical_provisional_module_scopes` (9/9). The read-only
  `is_visible_from(Visibility::Public, ..)` query is declaration-level and does not itself
  authorize a structural path. The resolver independently preflights every child, retains the
  first non-public child, and rejects a public final function beneath it.
- **Non-goals:** `pub use`; groups, globs, non-`crate` paths, non-function targets, imported
  namespaces beyond ordinary functions, remaining declaration/body checking, all re-export forms,
  final interfaces/export closure, compatibility binders, Core/CPS, Engine/admission/runtime, and
  file/inline or CLI/daemon end-to-end parity.
- **Handoff:** partial/tested. TASK-2068 retains every complete-interface/import/binder clause;
  TASK-2069 cannot begin until TASK-2068 is complete; TASK-2064 separately owns integration and
  parity proof. No authority transfers to Core/CPS, admission, runtime, or clients.
- **Source traceability:** scope construction and declaration-level visibility are fingerprinted as
  `sha256:140edfa007136360b2a9266eaddeb62591405d3b74388180e95cb740331cb002`;
  scope-backed resolution as
  `sha256:28e44c326d02391317a0d979af6aef386f6668d160f3136af6b5398bfb012777`; and
  the Type-layer module export as
  `sha256:537d7fc4be63a563117f6b2ca1e81b0868e1cf8b712e4a3b2a229de5e588c667`.
- **Record-mirrored delivered-scope target clause:** The delivered canonical provisional module-scope and structural-path visibility fragment is `partial / tested / below_spec`: it builds immutable typeck-owned per-module provisional scopes of direct structural children plus ordinary function declaration entries from TASK-2067 canonical graph units/artifacts. Before resolution, `matches_graph` compares root/artifact facts and requires equality with a fresh declaration-snapshot rebuild from the current parser units, so artifacts alone never authorize a scope entry and same-path/topology removal of a function or a `pub`-to-private change rejects `ScopeGraphMismatch` before binding. It resolves only inherited simple `use crate::<structural-child>...::<ordinary-function> as <name>` through actual structural edges, preserving `ModuleKey` identity, declaration/use spans, origin, and visibility; the final ordinary function may use public, `pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, or `pub(self)` when the importing `ModuleKey` lies in the canonical visibility region. Every traversed structural child and final function must pass visibility before temporary alias staging, and a local function collision rejects. `is_visible_from` is a declaration-level visibility query, so its `pub` result alone never authorizes a path; the resolver separately preflights every structural child, retains the first non-public edge, and rejects a public function behind it. Visibility is evaluated from `ModuleKey` crate identity and segments, never a string helper: private and `pub(self)` admit only the defining module, `pub(crate)` the same crate, `pub(super)` the structural-parent subtree, and `pub(in path)` the resolved named-path subtree. The focused target passes 9/9; these tests are evidence, not proof or end-to-end parity, and route-level binding witnesses over the admitted visibility regions remain deferred to the dedicated scoped binder. `pub use`, groups, globs, non-`crate` paths, non-function targets, other namespaces, remaining definition/body checks, all re-exports, final interfaces/export closure, compatibility binders, Core/CPS, Engine, admission, runtime, and parity remain deferred; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered scoped structural import-cycle gate sub-slice

**Status:** Delivered within this active task; `implementation: partial`, `evidence: tested`, and
`parity: below_spec`. Its layers are Type `partial`; Core/CPS/admission-runtime
`not_applicable`; verification `partial`. Its run-route impact is `prerequisite` only. This bounded
evidence does not complete TASK-2068 or Phase 207.

- **Canonical rule and admitted domain:** SPEC-103 §§5, 6, 8, and 9 require canonical
  `M-IMPORT-EDGE` provenance, visibility before registration, deterministic `M-IMPORT-CYCLE`, and
  atomic `M-BIND`. The route is only the delivered scope-backed inherited explicit-alias
  `use crate::<structural-child>...::<ordinary-function> as <name>` grammar.
- **Consumes:** the delivered canonical provisional scopes and all successfully resolved staged
  cross-module structural edges. Same defining/importer module aliases produce bindings but no
  edge. The generic parsed-import planner and compatibility binder remain unchanged because they
  own a different grammar and error contract.
- **Produces:** after all existing structural diagnostics have passed, it runs deterministic
  canonical cycle detection before a `CanonicalResolvedSimpleImports` result is constructed. A
  cycle returns an outer structural `ImportCycle { edges: CanonicalImportCycle }` retaining ordered
  closing-edge provenance; scope mismatch, unsupported, unresolved, inaccessible, and collision
  diagnostics retain precedence. No cycle result, binding set, or edge output is published.
- **Test witnesses:**
  `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-CYCLE-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-STRUCTURAL-TAIL-CYCLE-PROVENANCE`,
  `TEST-MOD-REAL-004-SCOPED-SAME-MODULE-NO-EDGE`,
  `TEST-MOD-REAL-004-SCOPED-CYCLE-VISIBILITY-PRECEDENCE`,
  `TEST-MOD-REAL-004-SCOPED-CYCLE-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-CYCLE-PROPERTY`,
  `TEST-MOD-REAL-004-SCOPED-CYCLE-ATOMICITY`, and
  `TEST-MOD-REAL-004-SCOPED-CYCLE-AUTHORITY-FENCE` pass in
  `task_2068_canonical_provisional_module_scopes` (scope17, including the 16-case property).
- **Non-goals:** every route outside the delivered scope-backed inherited explicit-alias grammar;
  generic planner or binder modification; `pub use`, groups, globs, non-`crate` paths,
  non-function targets, final interfaces/export closure, compatibility binders, Core/CPS,
  Engine/admission/runtime, and file/inline or CLI/daemon end-to-end parity.
- **Handoff:** partial/tested with run-route impact `prerequisite`. TASK-2068 retains complete
  interface/import/binder ownership; TASK-2069 is the consuming lowering/Engine-transport owner;
  TASK-2064 separately owns integration and parity proof. No authority transfers to Core/CPS,
  admission, runtime, or clients.
- **Source traceability:** the scope-side outer structural error is fingerprinted as
  `sha256:140edfa007136360b2a9266eaddeb62591405d3b74388180e95cb740331cb002` and the
  scope-backed resolver/canonical cycle gate as
  `sha256:28e44c326d02391317a0d979af6aef386f6668d160f3136af6b5398bfb012777`.
- **Record-mirrored delivered-scoped-cycle target clause:** The delivered scoped structural import-cycle gate is `partial / tested / below_spec`: it consumes only the delivered canonical provisional scopes and the scope-backed inherited explicit-alias `use crate::<structural-child>...::<ordinary-function> as <name>` route's staged resolved edges. After every existing scope-snapshot, route-shape, structural-path, visibility, target, and local-collision preflight succeeds, it deterministically detects cycles over cross-module `CanonicalSimpleImportEdge` values before constructing a result; same defining/importer module aliases emit no edge. A cycle returns the outer structural `ImportCycle { edges: CanonicalImportCycle }` with ordered closing-cycle provenance, while all existing structural diagnostics retain precedence, including a visibility failure that could otherwise close a cycle. The operation is atomic and non-authorizing: no cycle-free plan, binding set, or edge result is published on error; the generic planner and compatibility binder remain unchanged because they own different grammar. The focused target passes scope17, including a 16-case property; these tests are evidence, not proof or end-to-end parity. Final interfaces/export closure, other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered dedicated scope-backed structural binder M-BIND sub-slice

**Status:** Delivered within this active task; `implementation: partial`, `evidence: tested`, and
`parity: below_spec`. Its layers are Type `partial`; Core/CPS/admission-runtime
`not_applicable`; verification `partial`. Its run-route impact is `prerequisite` only. This
bounded evidence does not complete TASK-2068 or Phase 207.

- **Canonical rule and admitted domain:** SPEC-103 §§5, 6, 8, and 9 require `M-BIND` to publish
  only resolver-admitted aliases atomically. The dedicated binder admits only the delivered
  scope-backed inherited explicit-alias `use crate::<structural-child>...::<ordinary-function> as
  <name>` route. An ordinary function target may be public, crate, super, `pub(in path)`,
  inherited/private, or self only where the canonical `ModuleKey` visibility predicate admits the
  importer; the existing public structural-path fence remains in force.
- **Consumes:** only a delivered scope-backed resolver invocation with canonical provisional
  scopes. `crates/ash-typeck/src/canonical_structural_module_binder.rs` defines
  `bind_scoped_structural_parsed_uses(graph, scopes)` and delegates directly to
  `resolve_simple_parsed_imports_with_scopes(graph, scopes)` and then `into_bound_set`; only
  `crates/ash-typeck/src/lib.rs` exports that dedicated API. It creates no independent collection,
  path, visibility, cycle, or error route. The existing
  `crates/ash-typeck/src/canonical_module_binder.rs`, its generic `bind_simple_parsed_uses`
  binder, and the generic planner remain unchanged; the generic binder must not mention scopes,
  the scoped resolver, or `CanonicalStructuralImportError`.
- **Produces:** on resolver success, an opaque non-authorizing `CanonicalBoundModuleSet`; on any
  structural, visibility, collision, snapshot, or `CanonicalImportCycle` error, no result. It
  preserves the resolver error and its provenance unchanged, including atomic cycle failure.
- **Test witnesses:**
  `TEST-MOD-REAL-004-SCOPED-BINDER-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-DELEGATION`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-RESTRICTED-VISIBILITY`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-CYCLE-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-BINDER-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-BINDER-AUTHORITY-FENCE` pass in
  `task_2068_scoped_structural_binder` (8/8, including a 16-case property across public,
  `pub(crate)`, `pub(super)`, `pub(in path)`, inherited/private, and `pub(self)` visibility
  categories). These tests are evidence, not proof or end-to-end parity.
- **Non-goals:** changes to `crates/ash-typeck/src/canonical_module_binder.rs`, generic canonical
  binder/planner changes; every route beyond the delivered
  scope-backed inherited explicit-alias grammar; `pub use`, groups, globs, non-`crate` paths,
  non-function targets, final interfaces/export closure, compatibility conversion, Core/CPS,
  Engine/admission/runtime, and file/inline or CLI/daemon end-to-end parity.
- **Handoff:** partial/tested with run-route impact `prerequisite`. TASK-2068 retains complete
  interface/import/binder ownership; TASK-2069 is the consuming lowering/Engine-transport owner;
  TASK-2064 separately owns integration and parity proof. No authority transfers to Core/CPS,
  admission, runtime, or clients.
- **Source traceability:** the dedicated scoped binder is fingerprinted as `sha256:8a98865f2119e6649ece9cd99125527ae5f8390fc03a006a8486ff0563b7ebf6`; its private-module/public-re-export boundary in `lib.rs` is `sha256:06d0b65e674ec83b3a538539fe09cf4f8fc5e42d70bd3133e7a112c57f037f8f`; and the unchanged generic `canonical_module_binder.rs` remains `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Record-mirrored delivered-scoped-binder target clause:** The delivered dedicated scope-backed structural binder M-BIND slice is `partial / tested / below_spec`: `crates/ash-typeck/src/canonical_structural_module_binder.rs` defines `bind_scoped_structural_parsed_uses(graph, scopes)`, and only `crates/ash-typeck/src/lib.rs` exports that dedicated API. It consumes only the delivered canonical provisional scopes and delegates directly to `resolve_simple_parsed_imports_with_scopes(graph, scopes)` followed by `into_bound_set`. The existing `crates/ash-typeck/src/canonical_module_binder.rs` remains unchanged and generic-only: it must not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. It admits only the delivered inherited explicit-alias `use crate::<structural-child>...::<ordinary-function> as <name>` route; its ordinary-function target may be public, crate, super, `pub(in path)`, inherited/private, or self only when the canonical `ModuleKey` visibility predicate permits the importer, with the existing whole structural-path fence for public targets. It preserves every resolver structural diagnostic and outer `CanonicalImportCycle` provenance unchanged and atomically returns no `CanonicalBoundModuleSet` on error. The focused `task_2068_scoped_structural_binder` target passes 8/8, including a 16-case property across public, crate, super, `pub(in path)`, inherited/private, and self visibility categories; these tests are evidence, not proof or end-to-end parity. The generic `bind_simple_parsed_uses` binder and generic planner remain unchanged because they own different grammar. Final interfaces/export closure, all other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered scoped simple ordinary-function imports M-SIMPLE sub-slice

**Status:** Delivered within this active task; `implementation: partial`, `evidence: tested`, and
`parity: below_spec`. Its layers are Type `partial`; Core/CPS/admission-runtime
`not_applicable`; verification `partial`. Its run-route impact is `prerequisite` only. This
bounded evidence does not complete TASK-2068 or Phase 207.

- **Canonical rule and admitted domain:** SPEC-103 §§5, 6, 8, and 9 require visibility before
  atomic non-authorizing `M-BIND` publication. This route admits only inherited simple ordinary
  function imports `use crate::<ordinary-function>` or
  `use crate::<structural-child>...::<ordinary-function>`, each optionally followed by
  `as <name>`. Without `as`, the final function segment is the natural local binding name. The
  ordinary-function target may be public, crate, super, `pub(in path)`, inherited/private, or self
  only where the canonical `ModuleKey` visibility predicate admits the importer; the existing whole
  structural public-path fence remains in force.
- **Consumes:** only a scoped-only simple ordinary-function resolver invocation with delivered
  canonical provisional scopes. The dedicated
  `bind_scoped_simple_ordinary_function_imports(graph, scopes)` entry point in
  `crates/ash-typeck/src/canonical_structural_module_binder.rs` delegates directly to
  `resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)` then
  `into_bound_set`; `crates/ash-typeck/src/lib.rs` exports that API. It creates no
  independent collection, path, visibility, local-name, duplicate, cycle, or error route. The
  existing generic `resolve_simple_parsed_imports`,
  `crates/ash-typeck/src/canonical_module_binder.rs`, and generic `bind_simple_parsed_uses` remain
  unchanged and generic-only; the generic binder must not mention scopes, the scoped resolver, or
  `CanonicalStructuralImportError`.
- **Produces:** on resolver success, an opaque non-authorizing `CanonicalBoundModuleSet`; on any
  structural, visibility, local-collision, duplicate-binding, snapshot, or `CanonicalImportCycle`
  error, no result. It preserves the resolver error and provenance unchanged, including atomic
  cycle failure.
- **Test witnesses:**
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-NATURAL-NAME`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-ROOT-TARGET`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-VISIBILITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-DUPLICATE-BINDING`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-CYCLE-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-IMPORT-AUTHORITY-FENCE` pass in
  `task_2068_scoped_simple_ordinary_function_imports` (11/11, including a 16-case property and
  the retained structural-child compatibility regression).
  These tests are evidence, not proof or end-to-end parity.
- **Non-goals:** changes to generic resolver or binder grammar; every route beyond the inherited
  simple root/deep ordinary-function grammar; `pub use`, groups, globs, non-`crate` paths,
  non-function targets, final interfaces/export closure, compatibility conversion, Core/CPS,
  Engine/admission/runtime, and file/inline or CLI/daemon end-to-end parity.
- **Handoff:** partial/tested with run-route impact `prerequisite`. TASK-2068 retains complete
  interface/import/binder ownership; TASK-2069 is the consuming lowering/Engine-transport owner;
  TASK-2064 separately owns integration and parity proof. No authority transfers to Core/CPS,
  admission, runtime, or clients.
- **Source traceability:** the scoped-simple resolver is fingerprinted as `sha256:3c0d69aa5ee0cd4668d17683c6a0691e3b913b73a29d55a679c96ce784b80332`; the dedicated binder as `sha256:bebf9c437e1f436e4ba009ebe664d4d6011fbabae772f9562ae2458889c221b6`; its `lib.rs` export boundary as `sha256:5b536601da57fb90abb298c2edb759ff24303dce680155fe2bd11070ec0bf9f9`; and the unchanged generic binder as `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Record-mirrored delivered-scoped-simple-import target clause:** The delivered scoped simple ordinary-function imports M-SIMPLE slice is `partial / tested / below_spec`: `bind_scoped_simple_ordinary_function_imports(graph, scopes)` in `crates/ash-typeck/src/canonical_structural_module_binder.rs`, exported through `crates/ash-typeck/src/lib.rs`, consumes delivered canonical provisional scopes and delegates directly to `resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)` followed by `into_bound_set`. It admits only inherited simple `use crate::<ordinary-function>` or `use crate::<structural-child>...::<ordinary-function>` routes, each optionally followed by `as <name>`; without `as`, the final ordinary-function segment is the natural local binding name. Its ordinary-function target may be public, crate, super, `pub(in path)`, inherited/private, or self only when the canonical `ModuleKey` visibility predicate permits the importer, with the existing whole structural-path fence for public targets. It preserves every resolver structural diagnostic and outer `CanonicalImportCycle` provenance unchanged and atomically returns no `CanonicalBoundModuleSet` on structural, visibility, local-collision, duplicate-binding, snapshot, or cycle error. The focused `task_2068_scoped_simple_ordinary_function_imports` target passes 11/11, including a 16-case property and the retained structural-child compatibility regression; these tests are evidence, not proof or end-to-end parity. The existing generic `resolve_simple_parsed_imports`, `crates/ash-typeck/src/canonical_module_binder.rs`, and `bind_simple_parsed_uses` remain unchanged and generic-only; the generic binder must not mention scopes, the scoped resolver, or `CanonicalStructuralImportError`. Final interfaces/export closure, all other route forms, re-exports, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered scoped grouped ordinary-function imports M-GROUP sub-slice

**Semantic accounting:** implementation `partial`, evidence `tested`, and parity `below_spec`.
The parser syntax carrier and resolver/binder are Type `partial`; Core/CPS/admission-runtime are
`not_applicable`, verification is `partial`, and the run-route impact remains `prerequisite`.
This is bounded test evidence only; it does not change the task or phase status.

- **Parser-first handoff:** `ash_parser::use_tree::UseItem` now carries `{ name, alias, span }`.
  `parse_use_item` computes each `UsePath::Nested` member span from its source offsets with
  `offset_to_span`, covering only its name and optional `as <alias>`, never braces, separators,
  whitespace, or the enclosing `Use::span`. The carrier itself installs no resolution, binding, or
  authority.
- **Delivered route:**
  `resolve_scoped_grouped_ordinary_function_imports_with_scopes(graph, scopes)` accepts only
  inherited `UsePath::Nested` routes based at `crate` or
  `crate::<structural-child>...`; every member must resolve to one ordinary function and binds its
  explicit alias or natural name. It preflights scope snapshots, structural-path and target
  visibility under the canonical `ModuleKey` predicate with the whole-public-path fence, and
  retains each member span for identity facts and diagnostics. Structural, snapshot, visibility,
  local-collision, duplicate-binding, and cross-module `CanonicalImportCycle` checks all succeed
  before a plan can project; a failure returns no plan and no binding set. The dedicated private
  `bind_scoped_grouped_ordinary_function_imports(graph, scopes)` delegates only to that resolver
  then `into_bound_set`, while `lib.rs` re-exports only the named API. `DuplicateBinding` reports
  the later member's `use_span`; existing scoped-simple imports continue to use their enclosing
  use span. A grouped structural-child member such as `use crate::{api};` is anchored
  `Unsupported`, while the older simple `use crate::api;` route remains an enclosing-span
  `Unresolved` compatibility case.
- **Task-owned witnesses:** positive `TEST-MOD-REAL-004-PARSED-GROUP-MEMBER-SPAN`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-DUPLICATE-BINDING`, and
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-GROUP-IMPORT-CYCLE-ATOMICITY` pass in the parser suite and the
  10/10 `task_2068_scoped_grouped_ordinary_function_imports` target (including a 16-case
  property). These are tests, not a proof or end-to-end parity evidence.
- **Non-goals:** globs, `pub use`, non-inherited or non-`crate` bases, `self`/`super` or standard
  library bases, nested groups, qualified member paths, non-function members, other namespaces,
  generic resolver/binder changes, final interfaces/export closure, Core/CPS, Engine, admission,
  runtime, and parity. TASK-2069 cannot begin until TASK-2068 is complete.
- **Source traceability:** parser `UseItem` is `sha256:3d1963a7fc738929a3832570d33b8d061baa0be921b4522ae93cff8d82b4e635`;
  its parser is `sha256:f36a2a0d402cdcc3e30124863c6e91f173d0e94b3309bc98968911b3da9560ea`;
  the scoped planner is `sha256:2393e7518222a85c9ffee5e3d95642030171a8d7ff3faac7df3828c683b49901`;
  the dedicated binder is `sha256:e255345997667f9ca9d93358e853afb332706a4f6d90c91c6b8d53365b62b220`;
  its `lib.rs` export boundary is `sha256:f015c142cc7ec232d7c172734c071a0a33a92cbd77c8ad3f310dd37b30aa29f8`;
  and unchanged generic `canonical_module_binder.rs` is
  `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Record-mirrored delivered-scoped-group-import target clause:** The delivered scoped grouped
  ordinary-function imports M-GROUP sub-slice is `partial / tested / below_spec`: parser-owned
  nested-member spans and the dedicated scope-backed route admit only inherited
  `UsePath::Nested` `crate`/structural-child paths to ordinary functions, with optional aliases or
  natural local names. It preserves member-specific identity and diagnostic anchors, performs all
  scope, visibility, collision, duplicate, and complete-group cycle checks before returning an
  atomic plan or `CanonicalBoundModuleSet`, and leaves the generic resolver/binder unchanged and
  generic-only. Globs, `pub use`, non-inherited bases, nested groups, non-function members, other
  namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity
  remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately
  owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Record-mirrored missing target-spec clause:** The delivered scoped grouped ordinary-function imports M-GROUP slice is `partial / tested / below_spec`: it retains parser-owned nested-member spans and accepts only inherited `UsePath::Nested` crate/structural-child ordinary-function members with optional aliases or natural local names through the dedicated scoped resolver/binder. It atomically rejects scope snapshot, structural visibility, local collision, duplicate binding, and complete-group cycle failures before publishing a plan or binding set; generic resolver/binder authority is unchanged. Globs, `pub use`, non-inherited bases, nested groups, non-function members, other namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered scoped `super` ordinary-function imports M-SUPER sub-slice

**Semantic accounting:** implementation `partial`, evidence `tested`, and parity `below_spec`.
Type is `partial`; Core/CPS/admission-runtime are `not_applicable`; verification is `partial`; and
the run-route impact is `prerequisite`. This is historical delivered evidence: TASK-2068 is now
complete for its foundation, while Phase 207 remains In progress.

- **Canonical rule and delivered route:** SPEC-103 §6 plus §8 M-IMPORT-EDGE/M-IMPORT-CYCLE/M-BIND
  and §9 properties 3, 5, and 7 authorize the dedicated Type-only
  `resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)` route. It accepts
  only inherited `UsePath::Simple` imports from a non-root module that begin with exactly one
  `super`, starts at `ModuleKey::parent()`, traverses zero or more delivered structural children,
  and ends in one ordinary function. An explicit alias or the final function name selects the local
  binding. Every identity, edge, and diagnostic retains the complete parser-owned `Use::span`.
  `bind_scoped_super_ordinary_function_imports(graph, scopes)` lives in private
  `canonical_structural_module_binder.rs`, delegates only to that resolver followed by
  `into_bound_set`, has a Rustdoc `# Errors` contract, and is re-exported only through `lib.rs`.
  The generic resolver and `canonical_module_binder.rs` remain unchanged; the generic binder hash
  is `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Transaction and boundary:** The route consumes only TASK-2067 canonical graph units,
  parser-owned use spans, and TASK-2068 provisional scopes. It reuses scope snapshots,
  child-origin, canonical visibility and whole-public-path checks, local-collision, duplicate,
  cycle, and atomic-publication checks. Same-module imports create no edge. Before target lookup,
  every child segment and the final function segment named `super` are rejected, so repeated
  `super` and `fn super` cannot slip through. Root importers, `self`, `crate`, unprefixed,
  standard-library, or external bases, groups/globs/nested groups, `pub use`/restricted uses,
  non-function targets and namespaces, final interfaces/export closure, Core/CPS, Engine,
  admission, runtime, and client parity remain excluded. `self::` remains separately unresolved
  for same-module precedence.
- **Task-owned witnesses:** positive
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-PROPERTY`; negative
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-ROOT-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-DUPLICATE-BINDING`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-AUTHORITY-FENCE`; and mutation
  `TEST-MOD-REAL-004-SCOPED-SUPER-IMPORT-CYCLE-ATOMICITY` pass in the focused
  `task_2068_scoped_super_ordinary_function_imports` target (12/12, including a 16-case
  property). The final-`super` callable test reinforces the root/repeated-`super` boundary; it is
  not a separate witness. These are tests, not a proof or end-to-end parity evidence.
- **Source traceability:** scoped planner
  `sha256:88811da1f80ff1493e66c2c95b2917fd002c606035a669c3ee91ce8c88063428`; dedicated binder
  `sha256:64133573551f7ea65f3aea3c2429be51e2530a56700909071cb5be19e2536fe4`; `lib.rs` export
  boundary `sha256:aad134f2bc334017fa4c19d9f308eb597f4fff49ebefd99859e06eb5c8217059`; and unchanged
  generic binder `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Handoff:** delivered Type-only `prerequisite`. TASK-2068 retains all remaining interface,
  import, and binder ownership; TASK-2069 later owns lowering/Engine transport; TASK-2064 later
  owns parity. No authority transfers to Core/CPS, admission, runtime, or clients.
- **Record-mirrored missing target-spec clause:** The delivered scoped `super` ordinary-function imports M-SUPER slice is `partial / tested / below_spec`: its dedicated resolver and binding-only projection admit only inherited, non-root, exactly-one-leading-`super` `UsePath::Simple` parent/sibling ordinary-function routes with an optional alias or natural local name. It retains the full `Use::span`, canonical scope/visibility/whole-public-path, collision/duplicate, cycle, and atomic-publication rules, rejects every extra or final `super` before lookup, and leaves generic resolver/binder authority unchanged. The focused target passes 12/12 including a 16-case property; this is test evidence, not proof or end-to-end parity. Root/repeated/self/crate/unprefixed/standard-library/external paths, groups/globs, public or restricted uses, non-functions, other namespaces, final interfaces/export closure, Core/CPS, Engine, admission, runtime, and parity remain deferred. Its run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.

## Delivered scoped `super` grouped ordinary-function imports M-SUPER-GROUP sub-slice

**Semantic accounting:** implementation `partial`, evidence `tested`, and parity `below_spec`.
Type and verification are `partial`; Core/CPS/admission-runtime are `not_applicable`; and the
run-route impact is `prerequisite`. This delivered Type-only sub-slice does not complete TASK-2068
or Phase 207.

- **Delivered domain and transaction:** the dedicated resolver/binder admits only inherited,
  non-root `UsePath::Nested` routes whose base has exactly one leading `super`, no outer alias,
  zero or more canonical structural children after `ModuleKey::parent()`, and a nonempty group of
  ordinary-function members using their natural name or individual `as` alias. Each selected edge,
  identity fact, and member-specific error retains that member's parser-owned `UseItem::span`,
  rather than the enclosing `Use::span`. It consumes only TASK-2067 canonical graph units,
  parser-owned member spans, and TASK-2068 provisional scopes, reusing snapshot,
  structural-path/target visibility, whole-public-path, local-collision, duplicate-binding,
  same-module-no-edge, complete-group cycle, and atomic-publication rules. A final member named
  `super` is preflighted at its own span before structural-child lookup, including when a private
  child would otherwise fail visibility first.
- **Task-owned evidence:** the focused
  `task_2068_scoped_super_grouped_ordinary_function_imports` target passes 13/13, including a
  16-case property. POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY are positive evidence;
  VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and AUTHORITY-FENCE
  are negative evidence; CYCLE-ATOMICITY is mutation evidence. The canonical visibility matrix
  includes public, crate, super, restricted, and `pub(self)` same-module zero-edge cases; the
  shape matrix includes root/repeated/final `super`, outer aliases, unsupported heads/forms/use
  visibility, and whole-use anchors only where no member AST exists. A later non-function member
  rejects atomically at its member span, and the cycle witness is a real cross-module cycle.
- **Source traceability:** the scoped-super grouped resolver is anchored at
  `crates/ash-typeck/src/canonical_simple_import_planner.rs#resolves-inherited-grouped-ordinary-function-imports-through-one-super`
  with fingerprint
  `sha256:77ff8e437ada70fc1182bb52b99a4d9e56c2fe39c669ffe87258ff71d8eb021c`; the dedicated binder
  is anchored at
  `crates/ash-typeck/src/canonical_structural_module_binder.rs#projects-scoped-grouped-super-ordinary-function-imports-into-bindings`
  with fingerprint
  `sha256:0bfb497fe11b17623bcb39485d2420f80d3b5c64a39ba4ad9642f148d4413a06`; its `lib.rs` export
  boundary is `sha256:68775641f867d47b9f4a7af344b856eb3ec132f256659fc68bdc51444e934f86`; and the
  unchanged generic `canonical_module_binder.rs` is
  `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.
- **Traceability:** implementation
  `IMPL-MODULE-SCOPED-SUPER-GROUPED-ORDINARY-FUNCTION-IMPORTS` is implemented; tests
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-POSITIVE`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-IDENTITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-ROOT-DIAGNOSTIC`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-LOCAL-COLLISION`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-DUPLICATE-BINDING`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-CYCLE-ATOMICITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-FILE-INLINE-PARITY`,
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-PROPERTY`, and
  `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-AUTHORITY-FENCE` are tested.
- **Record-mirrored missing target-spec clause:** The delivered scoped `super` grouped ordinary-function imports M-SUPER-GROUP slice is `partial / tested / below_spec`: its dedicated resolver and binding-only projection admit only inherited, non-root `UsePath::Nested` routes with exactly one leading `super`, no outer alias, zero or more structural children after the canonical parent, and a nonempty group of ordinary-function members with natural/member `as` local names. It retains every parser-owned individual member span in identity, edge, and member-specific error facts; preflights a final member named `super` before lookup; and reuses canonical scopes/snapshots/visibility/whole-public-path, same-module-no-edge, local-collision, duplicate-binding, complete-group cycle, and atomic-publication rules. The focused target passes 13/13 including a 16-case property; its ten canonical witness IDs are test evidence, not proof or parity evidence. Root/repeated `super`, `self`, `crate`, unprefixed, standard-library/external, simple/glob/non-nested/nested-group, outer aliases, public/restricted/re-export forms, nonfunctions or other namespaces, generic resolver/binder changes, final interfaces/export closure, Core/CPS, Engine, admission/runtime, client parity, and general precedence remain deferred. Type and verification are `partial`; Core/CPS/admission-runtime are `not_applicable`; run-route impact is `prerequisite` for TASK-2069; TASK-2064 separately owns integration parity; TASK-2069 cannot begin until TASK-2068 is complete.
- **Next obligation:** Extend TASK-2068 beyond the delivered M-CHECK restricted-visibility leaf
  and the delivered import/binder fragments to every required namespace, remaining definition/body
  and export-closure check, every remaining parsed import/visibility/alias/re-export/cycle rule,
  and atomic M-BIND publication. TASK-2069 owns complete lowering and Engine transport fencing;
  TASK-2064 owns integration parity.

## Delivered scoped glob ordinary-function imports (M-GLOB sub-slice)

**Semantic accounting:** implementation `partial`, evidence `tested`, and parity `below_spec`.
Type is `partial`; Core/CPS/admission-runtime are `not_applicable`; verification is `partial`; and
the run-route impact is `prerequisite`. TASK-2068 is Complete for its closed foundation; Phase 207
remains In progress.

- **Canonical rule and delivered route:** SPEC-103 §6 plus §8 M-IMPORT-EDGE/M-IMPORT-CYCLE/M-BIND
  and §9 properties 3, 5, and 7 have a dedicated Type-only resolver and binding projection. The
  route retains defining identity, declaration origin/span/visibility, the complete parser-owned
  `Use::span`, and one cross-module edge for each selected function. It stages every candidate
  before atomically returning an opaque plan and binding projection; it introduces neither a
  general glob precedence policy nor generic-binder authority.
- **Transaction and boundary:** Visibility and every supported-shape failure precede publication.
  The dedicated M-GLOB domain has no in-domain import-cycle claim: the CONFLICT-ATOMICITY,
  AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY witnesses are only boundary mutations. Their local
  function, second-glob, and cycle-shaped attempts are `Unsupported` boundary failures, not
  `LocalDeclarationCollision`, `DuplicateBinding`, generic ambiguity, or `ImportCycle`; each
  publishes neither a plan nor a bound set and resolves no precedence. The defensive planner
  collision/duplicate/cycle branches remain unclaimed.
- **Record-mirrored missing target-spec clause:** The delivered scoped glob ordinary-function imports M-GLOB slice is `partial / tested / below_spec`: it admits only inherited `use crate::<public structural-child>...::*` routes contributing ordinary public functions, with exactly one use and zero local ordinary functions, so it does not decide local/explicit/glob precedence. The dedicated Type-only resolver `resolve_scoped_glob_ordinary_function_imports_with_scopes(graph, scopes)` and binding-only projection `bind_scoped_glob_ordinary_function_imports(graph, scopes)` consume only the canonical graph, parser-owned full `Use::span`, and provisional scopes. They traverse public structural children, select only visible public ordinary functions, preserve every selected function's defining identity, declaration origin/span/visibility, and full use span, produce one cross-module edge per function, and stage all candidates before atomic plan/bound-set publication. Boundary failures return no plan or bindings: a local function is `Unsupported` at the zero-local-function boundary; a second glob is `Unsupported` at the exactly-one-use boundary; and a cycle-shaped attempted program is the same boundary `Unsupported`, never `ImportCycle`. The CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY IDs are boundary-mutation evidence only: they claim neither `LocalDeclarationCollision`, `DuplicateBinding`, generic ambiguity, `ImportCycle`, a bound set, a plan, nor precedence. The defensive planner collision/duplicate/cycle branches are unclaimed. The SHAPE-DIAGNOSTIC matrix covers 15 valid parser representations; a leading `::` is not `UsePath::Glob`, and a private structural module is an `Inaccessible` visibility case. The 16-case PROPERTY varies public-child depth, function count, function/path visibility, and inline/file-backed source form. Type is `partial`; Core/CPS/admission-runtime are `not_applicable`; verification is `partial`; run-route impact is `prerequisite`. Tests are evidence, not proof, final-interface, generic-binder, Core/CPS/Engine/admission/runtime, or client-parity evidence. `self`, root/repeated `super`, non-`crate` paths, multiple globs, local declarations, explicit/group imports, aliases, re-exports or `pub use`, non-function namespaces, and all remaining import forms remain deferred.
- **Delivered witnesses:** `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-POSITIVE` → `scoped_glob_imports_two_public_ordinary_functions`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-IDENTITY` → `scoped_glob_import_plan_and_binder_preserve_each_function_identity_and_full_use_provenance`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-VISIBILITY-DIAGNOSTIC` → `scoped_glob_imports_report_private_structural_and_function_visibility_before_any_binding`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-SHAPE-DIAGNOSTIC` → `scoped_glob_imports_reject_unsupported_shapes_atomically`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CONFLICT-ATOMICITY` → `scoped_glob_imports_reject_conflict_atomically`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AMBIGUITY-ATOMICITY` → `scoped_glob_imports_reject_ambiguous_candidate_attempt_atomically`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-CYCLE-ATOMICITY` → `scoped_glob_imports_reject_cycle_shaped_boundary_attempt_atomically`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-FILE-INLINE-PARITY` → `scoped_glob_imports_match_file_and_inline_scope_facts`; `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-PROPERTY` → `scoped_glob_imports_generated_depth_count_visibility_and_source_forms`; and `TEST-MOD-REAL-004-SCOPED-GLOB-IMPORT-AUTHORITY-FENCE` → `scoped_glob_import_route_has_only_dedicated_binding_authority_and_no_later_layer_path`, all in `crates/ash-typeck/tests/task_2068_scoped_glob_ordinary_function_imports.rs`. Positive evidence is POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY; VISIBILITY-DIAGNOSTIC, SHAPE-DIAGNOSTIC, and AUTHORITY-FENCE are negative; CONFLICT-ATOMICITY, AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY are boundary mutation evidence only.
- **Source traceability:** dedicated binder `sha256:6fd37ea25cf3aa6767b9c2175a57f3761cf947d7a23bdf4020fff653ab250aa9`; scoped planner `sha256:568bb73d47f3b96633b256a857dc606ac868ef18bd314e07968b85a9b8f795e9`; `lib.rs` export boundary `sha256:8dfaa8852bdbc697b00f5d509e9359f687284e2d502fdd918c695c8e5bc5ddd1`; and unchanged generic binder `sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`. The imported `sha2` dev-dependency is used only for portable authority-fence hashing; it is not a broader dependency claim.
- **Next obligation:** Extend the canonical graph-only Type-layer slice beyond delivered M-GLOB, M-SUPER, M-GROUP parser-span/resolver/binder, M-SIMPLE, dedicated scope-backed structural binder, scoped structural import-cycle gate, canonical provisional-module-scope/structural-path visibility, direct-public primitive re-export interface, and local-binding root-client fragments to every required namespace, remaining definition/body and export-closure check, every remaining parsed import/visibility/alias/re-export/cycle rule, and atomic M-BIND publication; TASK-2069 then owns complete lowering and Engine transport fencing, and TASK-2064 owns integration parity.

## Delivered local-over-glob precedence (M-GLOB-LOCAL-PRECEDENCE sub-slice)

**Semantic accounting:** implementation partial; evidence tested; parity below_spec.
Type is partial; Core/CPS/admission-runtime are not_applicable; verification is partial; and the
run-route impact is prerequisite. TASK-2068 is Complete for its closed foundation; Phase 207
remains In progress.

- **Delivered route:** exactly one existing inherited use crate::<public structural child>...::* route
  selects public ordinary functions. A same-module ordinary function shadows a selected same-name
  import only in returned public bindings, while a non-colliding selected import binds. Selection
  retains every selected cross-module edge, including shadowed targets, and cycle-checks before
  filtering; all-shadowed input returns no import bindings but keeps its edges, and a hidden cycle
  returns atomic ImportCycle { edges: CanonicalImportCycle }.
- **Authority and exclusions:** this uses only the canonical graph and provisional scopes, never
  private M-CHECK facts. Other imports, multiple globs, aliases/re-exports, self, super,
  non-crate paths, nonfunctions, the generic binder, final interfaces, Core/CPS,
  admission/runtime, and client parity stay excluded. The output is only a non-authorizing
  Type-layer plan/bound set; TASK-2069 owns lowering and TASK-2064 owns parity.
- **Test evidence:** the focused task_2068_local_over_glob_precedence target passes 8/8:
  IMPL-MODULE-SCOPED-GLOB-LOCAL-OVER-GLOB-PRECEDENCE;
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-WINS-NONCOLLIDING,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-IDENTITY-EDGE,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-ALL-SHADOWED-EMPTY-BINDING,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-CYCLE-ATOMICITY,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-VISIBILITY-SHAPE,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-FILE-INLINE-PARITY,
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-PROPERTY, and
  TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-AUTHORITY-FENCE are tested. Positive evidence
  is WINS-NONCOLLIDING, IDENTITY-EDGE, ALL-SHADOWED-EMPTY-BINDING, FILE-INLINE-PARITY, and
  PROPERTY; VISIBILITY-SHAPE and AUTHORITY-FENCE are negative; CYCLE-ATOMICITY is mutation
  evidence. The property has exactly 16 cases varying names, collision subsets, source form, and
  depth 1–3. File/inline is normalized Type-layer scope/binding parity only, never final/runtime
  parity.
- **Source traceability:** planner sha256:17b2ffe653d196ba295ea1e93bd57ad8c193596918f3787c3f43a1e2e6299f2a;
  dedicated binder sha256:652062ee3430667a1259f92777cf7e369b9b5e7ce167151941ade225fb0f8bf1;
  lib.rs export boundary sha256:99e7a4c81c34ced69e5fb78830176406e2b30c37b7bf8a9b5617eb78e4664aa6;
  unchanged generic binder fence sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6.
- **Record-mirrored delivered clause:** The delivered M-GLOB-LOCAL-PRECEDENCE slice is partial / tested / below_spec: it admits exactly one inherited crate::<public structural child>...::* glob domain. Same-module ordinary functions shadow same-name imported public ordinary functions only in returned public bindings; non-colliding imports remain. The resolver retains every selected cross-module edge, including shadowed targets, and cycle-checks before filtering, so all-shadowed input succeeds with no import bindings but retained edges and actual hidden cycles return atomic ImportCycle { edges: CanonicalImportCycle }. It consumes canonical graph/provisional scopes only and never private M-CHECK facts. Existing M-GLOB behavior remains separate/rejecting; other imports, multiple globs, aliases/re-exports, self/super/non-crate paths, nonfunctions, the generic binder, final interfaces, Core/CPS, Engine, admission/runtime, and parity authority remain excluded. The focused target passes 8/8, including a 16-case property varying names, collision subsets, source form, and depth 1–3; file/inline evidence establishes normalized Type-layer scope/binding parity only, never final/runtime parity. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite; TASK-2069 owns lowering and TASK-2064 owns parity.

## Delivered local-over-explicit precedence (M-SIMPLE-LOCAL-PRECEDENCE sub-slice)

**Semantic accounting:** implementation partial; evidence tested; parity below_spec. Type and
verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is
prerequisite. TASK-2068 is Complete for its closed foundation; Phase 207 remains In progress.

- **Delivered route:** exactly one inherited, unaliased `UsePath::Simple`
  `use crate::<public structural child>...::<public ordinary-function>;` route binds only its
  natural final name. A selected cross-module target retains one edge, and deterministic cycle
  detection runs over those cross-module edges before a same-module ordinary function filters its
  same-name import from returned public bindings. A selected same-module target emits no self-edge
  and does not participate in cycle detection. A non-colliding import binds; all shadowed
  cross-module candidates retain their edges with no import binding; and a real hidden two-module
  cross-module cycle returns atomic `ImportCycle { edges:
  CanonicalImportCycle }` before publication.
- **Authority and exclusions:** only canonical graph/provisional scopes may authorize this route;
  private M-CHECK facts and the generic binder may not. The delivered M-SIMPLE route remains
  unchanged and continues to reject local collisions. Root functions, aliases, multiple uses,
  groups, globs, self, super, restricted/private targets or structural paths, re-exports,
  nonfunctions, body lexical binding, final interfaces, Core/CPS, Engine, admission/runtime, and
  parity remain excluded. TASK-2069 owns lowering; TASK-2064 owns integration parity.
- **Test evidence:** the focused `task_2068_local_over_simple_precedence` target passes 9/9.
  `TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-WINS-NONCOLLIDING`, `IDENTITY-EDGE`,
  `ALL-SHADOWED-EMPTY-BINDING`, `FILE-INLINE-PARITY`, and `PROPERTY` are positive;
  `VISIBILITY-SHAPE`, `AUTHORITY-FENCE`, and `LEGACY-M-SIMPLE-REGRESSION` are negative; and
  `CYCLE-ATOMICITY` is mutation evidence. The property has 16 cases varying depth 1–3, name,
  collision mask, and source form. File/inline claims normalized Type-layer scope/binding parity
  only, never final/runtime parity. The visibility/shape matrix rejects aliases, root-function
  imports, groups, globs, self, super, multiple uses, a private structural segment, a nonfunction
  target, a private target, and `pub use`.
- **Source traceability:** planner sha256:7fb241da5b3bf35595e7cf3054f06dcbc9c9dc08dc9701c047d0d2c045a393d3;
  dedicated binder sha256:500d00d4de399eaac9c6ad19b74d79a2ec694b724014fbea8cdea02470a0d470;
  lib.rs export boundary sha256:68f8c3410b8bb92ee72cc85b91501a877dd357dca1456c27622f7996c150162c;
  unchanged generic binder fence sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6.
- **Record-mirrored delivered clause:** The delivered M-SIMPLE-LOCAL-PRECEDENCE slice is partial / tested / below_spec: it admits exactly one inherited, unaliased UsePath::Simple use crate::<public structural child>...::<public ordinary-function>; route with its natural final name, while same-module ordinary functions are permitted. It selects the public ordinary-function target. A selected cross-module target retains one edge; complete deterministic cycle detection considers only those cross-module edges before filtering a same-name local binding. A selected same-module target emits no self-edge and does not participate in cycle detection. A non-colliding import binds, all shadowed cross-module candidates return no import binding but retain their edges, and a real hidden two-module cross-module cycle rejects atomically as ImportCycle { edges: CanonicalImportCycle }. It consumes canonical graph/provisional scopes only, with no M-CHECK private-fact or generic-binder authority, and the existing M-SIMPLE route remains unchanged with local collision rejection. Root functions, aliases, multiple uses, groups, globs, self, super, restricted/private targets or structural paths, re-exports, nonfunctions, body lexical binding, final interfaces, Core/CPS, Engine, admission/runtime, and parity remain excluded. Its visibility/shape matrix rejects aliases, root-function imports, groups, globs, self, super, multiple uses, a private structural segment, a nonfunction target, a private target, and pub use. Its focused crates/ash-typeck/tests/task_2068_local_over_simple_precedence.rs target passes 9/9: local-wins/noncollision, identity/edge, all-shadowed, file/inline normalized Type-layer scope/binding parity, and the 16-case depth 1–3/name/collision-mask/source-form property are positive evidence; visibility/shape, authority fence, and legacy M-SIMPLE regression are negative; hidden-cycle atomicity is mutation evidence. Type and verification are partial; Core/CPS/admission-runtime are not_applicable; run-route impact is prerequisite; TASK-2069 owns lowering and TASK-2064 owns parity.
- **Record-mirrored non-goal:** The delivered M-SIMPLE-LOCAL-PRECEDENCE slice excludes root functions, aliases, multiple uses, groups, globs, self, super, restricted/private target or structural-path access, pub use/re-exports, nonfunctions, lexical body bindings, final interfaces, Core/CPS, Engine, admission/runtime, and parity; it neither consumes private M-CHECK facts nor changes generic-binder authority.

## Foundation closure and successor ownership

TASK-2068 is complete only for its `partial / tested / below_spec` foundation: every source/test
node recorded here remains its evidence. No unfinished clause remains task-owned: TASK-2070 owns
M-SELF-SIMPLE-ALIAS, TASK-2071 owns complete M-COLLECT namespace/callable facts, TASK-2072 owns
complete parsed import/cycle/binding semantics, and TASK-2073 owns complete M-CHECK/final
interface/export closure. TASK-2069 consumes TASK-2073's complete checked handoff; TASK-2063
awaits TASK-2069; TASK-2064 owns integration parity; and TASK-2065 owns closeout inventory.

## Delivered restricted declaration visibility M-CHECK sub-slice

**Semantic accounting:** implementation `partial`, evidence `tested`, and parity `below_spec`.
Type is `partial`; Core/CPS/admission-runtime are `not_applicable`; verification is `partial`; and
the run-route impact is `prerequisite`. TASK-2068 is Complete for its closed foundation; Phase 207
remains In progress.

- **Canonical rule and bounded domain:** SPEC-103 §6 requires visibility before registration,
  §7 separates private facts from a public projection, and §8 M-CHECK requires atomic checking.
  `check_closed_function_modules` now accepts exactly parser-preserved `pub(crate)`, `pub(super)`,
  `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` on ordinary functions with primitive
  closed signatures in a file-root closed leaf. A non-crate restricted path such as
  `pub(in self::internal)` rejects as an anchored unsupported visibility. Imports, child modules,
  nonfunctions, generics, contracts, and open signatures remain outside the slice.
- **Transaction and private/public split:** the checker graph-preflights every unit, stages sibling
  signatures, and atomically checks all bodies. Checked restricted functions remain only in
  `private_functions`, retaining fresh checked identity, defining `ModuleKey`, origin,
  declaration/body spans, declared visibility, signature type, and checked body type.
  `CanonicalPublicFunctionInterface` projects only `Visibility::Public`. The existing no-children
  preflight rejects an inline child/module as atomic `UnsupportedModuleShape` before private or
  public projection.
- **Test evidence:** the focused `task_2068_canonical_function_interface` target passes 18/18.
  Its eleven canonical witnesses are tested: `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-CRATE`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SUPER`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-IN-CRATE`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SELF`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-IDENTITY-PROVENANCE`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PROPERTY`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-SIGNATURE-BODY-DIAGNOSTICS`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-NON-CRATE-PATH-DIAGNOSTIC`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-ATOMICITY`,
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY`, and
  `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUBLIC-PROJECTION-AUTHORITY-FENCE`.
  The negative `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-NON-CRATE-PATH-DIAGNOSTIC`
  witnesses the rejected `pub(in self::internal)` boundary. The file/inline-named
  witness is a source-form boundary only: file-root success versus inline child/module atomic
  `UnsupportedModuleShape` at no-children preflight before projection, not normalized-success
  file/inline parity.
- **Source traceability:** `canonical_function_interface.rs`
  `sha256:22d2582021f2a9921f51f25786a848aed174232beb51b02b635e9ac5e595bdda`.
- **Non-goals and handoff:** imports, binders, re-exports, final interfaces, Core/CPS,
  admission/runtime, and parity remain outside this slice. It produces no authority for any later
  layer. TASK-2071 owns remaining collection, TASK-2072 remaining parsed binding, and TASK-2073
  remaining checking/finalization; TASK-2069 separately consumes only TASK-2073's complete checked
  modules for lowering/Engine transport, and TASK-2064 separately owns integration parity.
- **Record-mirrored missing target-spec clause:** The delivered M-CHECK-RESTRICTED-VISIBILITY slice
  is `partial / tested / below_spec`: it accepts exactly the four stated restricted forms for
  primitive closed ordinary-function leaves in a file-root closed leaf, with `pub(in ...)` limited
  to `crate` or `crate::...`, and rejects non-crate restricted paths. It graph-preflights, stages
  signatures, and checks bodies atomically; it retains fresh identity, defining key, origin, spans,
  visibility, signature, and body type only in `private_functions`; and public projection remains
  only `Visibility::Public`. The focused target passes 18/18, including all eleven canonical
  witnesses. The file/inline-named witness is a source-form
  boundary—file-root success versus inline-child `UnsupportedModuleShape` before projection—not
  normalized-success parity. It is not import, binder, re-export, final-interface, Core/CPS,
  admission/runtime, proof, or parity authority. Type and verification are `partial`;
  Core/CPS/admission-runtime are `not_applicable`; run-route impact is `prerequisite` for
  TASK-2069; TASK-2064 owns integration parity.
- **Record-mirrored next obligation:** TASK-2068 has no remaining implementation obligation:
  TASK-2070 owns the planned M-SELF-SIMPLE-ALIAS leaf, TASK-2071 owns complete provisional
  namespace/callable collection, TASK-2072 owns complete parsed imports and atomic binding, and
  TASK-2073 owns complete checked finalization/export closure. TASK-2069 consumes TASK-2073's
  complete checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns integration parity;
  TASK-2065 owns closeout inventory.
- **Record-mirrored exact missing clause:** The delivered M-CHECK-RESTRICTED-VISIBILITY slice is `partial / tested / below_spec`: it accepts exactly `pub(crate)`, `pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` for primitive closed ordinary-function leaves in a file-root closed leaf. It graph-preflights every unit, stages sibling signatures, and checks bodies atomically; it retains fresh identity, defining key, origin, declaration/body spans, declared visibility, signature type, and body type only in `private_functions`; `CanonicalPublicFunctionInterface` projects only `Visibility::Public`. `pub(in self::internal)` rejects as an anchored unsupported visibility. The focused target passes 18/18, with all eleven canonical witnesses tested. Its file/inline-named witness is a source-form boundary only: file-root success versus inline-child/module `UnsupportedModuleShape` before projection, never normalized-success parity. It authorizes no imports, binders, re-exports, final interfaces, Core/CPS, admission/runtime, proof, or parity. Type and verification are `partial`; Core/CPS/admission-runtime are `not_applicable`; run-route impact is `prerequisite` for TASK-2069; TASK-2064 owns integration parity.
- **Record-mirrored exact next obligation:** TASK-2068 has no remaining implementation obligation: TASK-2070 owns the planned M-SELF-SIMPLE-ALIAS leaf, TASK-2071 owns complete provisional namespace/callable collection, TASK-2072 owns complete parsed imports and atomic binding, and TASK-2073 owns complete checked finalization/export closure. TASK-2069 consumes TASK-2073's complete checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns integration parity; TASK-2065 owns closeout inventory.

## Description

Complete the parser/typechecker half of the module machine over TASK-2067's canonical graph:
expand a module scope, collect a provisional view, traverse parsed imports, resolve/bind through
checked interfaces, typecheck bodies, and publish an export-closed final interface only on success.
The delivered work is smaller: it collects parser function declarations and resolves only simple
parsed `crate::…` function aliases against that private provisional view; separately, it
graph-preflights and checks only closed primitive ordinary-function leaf units. The latter issues a
fresh Type-layer private map and deliberately limited public function projection, not a final
module interface. The existing TASK-2060, TASK-2061, and TASK-2066 carriers remain compatibility
inputs for later revalidation; none may be promoted to final-interface or binder authority without
this task's remaining checks.

## Dependencies

- 📝 TASK-2067 — canonical parsed/module-unit graph and structural failure handoff.
- ✅ TASK-2060 — non-authoritative Core public-interface carrier.
- ✅ TASK-2066 — bounded TypeEnv declaration-preflight wrapper.
- ✅ TASK-2061 — bounded wrapper-store explicit/group/glob resolver facts.

## Requirements

1. Expand each graph-delivered module unit in its module scope and collect a provisional interface
   only from parsed module items. The provisional view may expose names, defining module keys,
   declared visibility, and source anchors to resolve targets; it must not publish unchecked
   callable/type facts as a final public interface.
2. Complete parser/TypeEnv finalization for every required public/private namespace: child modules,
   functions and other callable declarations, types, constructors, interfaces, implementations,
   effect rows, macro summaries, and notation summaries. Check reachable definition bodies and
   retain stable defining identities and source origins in the private and public views required by
   SPEC-103 §7.
3. Enforce export closure before publishing `PublicInterface(m)`. Reject a public signature, row,
   type, constructor, interface, implementation, macro/notation summary, nested-module reference,
   or re-export that names a non-publicly reachable dependency. A failed module publishes neither a
   partial public interface nor partially committed binder facts.
4. Resolve only parsed `use` nodes and qualified paths through canonical structural identities and
   checked provisional/final interfaces. Implement the specified local → same-module → explicit
   import → glob → permitted parent/module-path precedence without filename walking, raw source,
   text scans, raw Core, legacy graphs, or Engine-private exports as authority.
5. Apply `private`, `pub`, `pub(crate)`, `pub(super)`, and `pub(in path)` before an imported entry
   enters the binding environment. An inaccessible diagnostic must identify the declaration,
   defining module, attempted access path, and violated boundary; it must never be reported as a
   missing name.
6. Support parsed aliases, `pub use`, and re-exports without changing the defining identity.
   `pub mod` exports only the child identity; no child declaration is flattened absent explicit
   `pub use`.
7. Build import edges from parsed imports, reject their cycles atomically before `M-BIND`, and
   integrate the resulting resolved facts with the existing binder/name-precedence machinery. An
   unresolved, ambiguous, inaccessible, duplicate, or failed dependency rejects the affected
   dependency closure without publishing bindings.
8. Establish file/inline equality at the final checked-interface and binding boundary while
   retaining only permitted provenance/display-path distinctions. TASK-2064 remains responsible
   for later Core/CPS/admission/terminal parity.

## TDD steps and reserved evidence

1. Add failing final-interface tests covering each namespace, complete definition-body facts,
   public/private views, closure validation, and no implicit flattening
   (`TEST-MOD-REAL-003-FINAL-INTERFACE`, `TEST-MOD-REAL-003-EXPORT-CLOSURE-REJECTION`).
2. Add parsed `use`, qualified-path, alias, `pub use`, and all-visibility-form tests that assert
   the specified binding precedence and anchored inaccessible diagnostics
   (`TEST-MOD-REAL-004-PARSED-IMPORT-BINDING`, `TEST-MOD-REAL-004-VISIBILITY-DIAGNOSTIC`).
3. Add mutation controls that rewrite a re-export defining identity and inject a late cycle or
   failed dependency after provisional collection; assert no final interface or binder environment
   is published (`TEST-MOD-REAL-003-REEXPORT-IDENTITY-MUTATION`,
   `TEST-MOD-REAL-004-BINDER-ATOMICITY-MUTATION`).
4. Materialize equivalent file/inline trees through TASK-2067 and compare normalized final
   interfaces and resolved defining identities (`TEST-MOD-REAL-003-004-INTERFACE-PARITY`).
5. The selected M-CHECK leaf evidence is now implemented: test real graph-unit sibling checking,
   fresh identity/provenance and private/public projection; anchored mismatch and public nonprimitive
   signature rejection; late-failure atomicity; parsed-`use` rejection; a 16-case public-int
   property; and a legacy/final-interface/runtime architecture fence
   (`TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC`,
   `TEST-MOD-REAL-003-LEAF-MCHECK-PRIMITIVE-PUBLIC-PROPERTY`,
   `TEST-MOD-REAL-003-LEAF-MCHECK-BODY-MISMATCH-DIAGNOSTIC`,
   `TEST-MOD-REAL-003-LEAF-MCHECK-OPTION-CLOSED-INTERFACE-REJECTION`,
   `TEST-MOD-REAL-003-LEAF-MCHECK-UNSUPPORTED-SHAPE`,
   `TEST-MOD-REAL-003-LEAF-MCHECK-SIBLING-ATOMICITY`, and
   `TEST-MOD-REAL-003-LEAF-MCHECK-INTERFACE-FENCE`).
6. Implement only after the focused tests are red; then run focused parser/typechecker tests,
   affected crate suites, strict clippy, and formatting before marking the task complete.

## Completion checklist

- [ ] Final private/public interfaces contain checked body and namespace facts, stable identities,
  source origins, dependency summaries, and validated export closure.
- [ ] Parsed imports, qualified paths, aliases, re-exports, and every visibility form bind only
  through canonical checked interfaces with the required diagnostics and precedence.
- [ ] Import cycles and every failed dependency reject atomically before binder/public-interface
  publication.
- [ ] Positive, negative, mutation, and file/inline interface-parity evidence is recorded in the
  activated task record and traceability graph.
- [ ] TASK-2069 receives checked body/lowering facts; no interface or binder fact authorizes Engine
  admission, a provider/handler frame, or a direct-evaluator fallback.

## Handoffs

- **Consumes:** The delivered slices consume only TASK-2067 canonical graph/state/unit facts and
  parser payloads. The M-CHECK pass graph-preflights every supplied unit and uses the builtin
  TypeEnv body checker; it does not acquire source or use a legacy graph/interface, import/binder,
  Core, CPS, Engine, or runtime authority. TASK-2060/2066/2061 remain bounded compatibility data
  for later revalidation; the slices do not use them as final-interface, import, or binder
  authority.
- **Produces:** A private provisional function collection retaining canonical defining identity,
  source anchor, origin, and declared visibility, plus atomically published bindings for simple
  inherited parsed `crate::…` function aliases. `pub use`, re-exports, and every non-inherited
  `use` visibility reject before any binding set is published. `pub(crate)`, `pub(super)`,
  `pub(self)`, and `pub(in …)` target declarations likewise reject as anchored `Unsupported`; none
  is implemented by this slice. The planner retains its canonical cross-module edge provenance,
  suppresses same-module edges, rejects discovered cycles before publishing a result as
  `ImportCycle { edges: CanonicalImportCycle }`, and the compatibility binder delegates through
  it. The returned binding set has no `Default` implementation and no public constructor, so
  callers cannot fabricate a successful set outside this binder. It produces no final
  private/public interface, complete import graph, checked body fact, or Engine credential.
  Separately, delivered M-CHECK creates a fresh checked function identity retaining
  `ModuleKey`, origin, declaration/body spans, signature type, and body type. It atomically
  publishes only a private checked-function map plus public primitive signatures through
  non-authorizing `CanonicalPublicFunctionInterface`; it is not core `PublicModuleInterface`, a
  final interface, an import/binder fact, or an Engine credential.
- **Downstream owner:** TASK-2071 completes collection, TASK-2072 completes parsed imports and
  atomic binding, and TASK-2073 completes the checked-module handoff. Only then does TASK-2069
  lower complete checked definition bodies and transport canonical artifacts to TASK-2063.
  TASK-2063 alone owns sealed linking/admission.
- **Integration/proof responsibility:** TASK-2068 retains only its delivered evidence attribution.
  TASK-2064 owns full file/inline artifact, admitted-program, and CLI/daemon normalized-terminal
  parity.
- **Run-route impact:** `prerequisite`. No client or Engine route becomes runnable here.
- **Non-goals:** The bounded-slice non-goals in Semantic accounting, including final interfaces,
  full imports/visibility/cycles, Core/CPS, Engine, and all client parity, remain binding.

## Candidate files and verification

**Candidate source/test paths on activation:** `crates/ash-parser/src/{parse_module.rs,module.rs}`,
`crates/ash-typeck/src/{type_env,name_binding.rs,canonical_simple_import_planner.rs,canonical_structural_module_binder.rs,lib.rs}`,
module-interface/import resolver integration modules, and focused parser/typechecker integration
tests including `task_2068_scoped_structural_binder`.

```text
cargo test -p ash-parser --test task_2068_final_module_interface
cargo test -p ash-typeck --test task_2068_parsed_import_binder
cargo test -p ash-typeck --test task_2068_canonical_function_interface
cargo test -p ash-typeck --test task_2068_scoped_structural_binder
cargo test -p ash-typeck --test task_2068_scoped_simple_ordinary_function_imports
cargo test -p ash-parser
cargo test -p ash-typeck
cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
