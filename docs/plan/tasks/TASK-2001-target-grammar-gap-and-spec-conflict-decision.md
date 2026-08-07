# TASK-2001: Target Grammar Gap and Specification-Conflict Decision

**Status:** In progress — canonical authority is selected; parser/AST declaration slices,
module-summary handoffs, and direct imported-summary registration evidence exist for aliases,
groups, handlers, and newtypes. V8 structural effect-row summaries now replace V7 text as the
only imported-summary content eligible for typed-handler normalization. The remaining work is
specified-but-unimplemented alias/group/handler/newtype/row realization.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)

**Status:** In progress

**Semantic task record:** [TASK-2001 workflow record](../semantic-task-records.json)

**Semantic coverage map:** [TASK-2001 semantic workflow record](../SEMANTIC-RULE-COVERAGE.md#task-2001-semantic-workflow-record)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.

## Semantic workflow record

The implemented grammar, row, and handler behavior is below the target-spec domain. The tests
listed in the record provide confidence only for the realized behavior.

## Description

Realize selected target grammar and semantic contracts for aliases, groups, handlers, and
newtypes, while retaining the settled rejection of historical proxy/workflow forms.

## Canonical authority is selected

This task does not need a further authority decision. The authority audit records the following
already-selected ownership:

- `GRAM-TARGET-MODULE-001` is specified by [SPEC-095b](../../spec/SPEC-095b-TARGET-GRAMMAR.md),
  including module/declaration and handler surface grammar.
- `TYPE-TARGET-ROW-001` is specified by
  [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) and
  [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), including row taxonomy, aliases,
  groups, normalization, inclusion, and discharge.
- Surface-to-Core lowering is specified by
  [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md), including impl operation identity
  and handler/provider boundaries.
- Runtime behavior is specified by
  [SPEC-099b](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); implementation must not
  substitute parser/summary metadata for its operational frame/admission contracts.

Accordingly, the remaining items below are implementation realization against these specifications,
not unresolved target-language design choices. The one genuine user-facing design gate is
[TASK-2014](TASK-2014-source-handler-runtime-boundary-decision.md): select Path A (deterministic
production rejection) or Path B (a changed checked-Core/CPS production boundary) for source
handlers. Nothing in this task selects that path.

## Settled symbolic operation-call boundary

Normal declaration-resolved symbolic calls of the form `ImplType::operation(args)` are **not** a
TASK-2001 ambiguity or blocker. Their target semantics are settled: resolution uses the declared
concrete impl operation identity and signature, the resulting effect row is non-granting, and
stringly `invoke` is not a compatibility path. TASK-2011 establishes the declaration-backed
`TestClock::sleep(0)` control, TASK-2012 establishes explicit validated provider binding, and
TASK-2017 establishes the same normal symbolic route for nominal `PosixFs::read(path)` with a
`String` argument. See [TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md),
[TASK-2012](TASK-2012-declared-operation-provider-binding.md), and
[TASK-2017](TASK-2017-posixfs-read-symbolic-concrete-operation.md).

Those implementation controls do not complete this task's remaining alias/group,
handler/newtype, or full row-realization work. In particular, they neither settle alias/group
expansion and operation/evidence identity taxonomy nor add handler execution, broad imported
resolution, generic selection, or a production Core/CPS route.

## Requirements

- Map each target grammar form to parser, AST, lowering, diagnostics, and tests.
- Classify each implementation gap as realize, explicitly exclude, or historical correction against
  the selected authority.
- Do not reintroduce Phase 201 removed proxy/workflow forms through compatibility parsing.
- Give each accepted form a stable canonical grammar rule and negative cases for rejected forms.

## TDD Steps

1. Add parser fixtures for accepted and rejected representative forms.
2. Make the current realization gap assertion fail against the selected target contract.
3. Implement parser/Core/diagnostic changes that conform to the selected authority.
4. Run parser, typechecker, docs, and removal gates.

## Completion Checklist

- [x] Canonical authority is selected for grammar, rows/types, lowering, and runtime.
- [ ] Every named form has an implemented or explicitly excluded disposition.
- [ ] Accepted forms preserve origins through lowering; rejected forms diagnose deterministically.
- [ ] Proxy history cannot become a target grammar owner.
- [ ] Canonical grammar, tests, indexes, and changelog agree.

## Evidence required

TASK-1988’s surface slice supplies the initial gap map; final evidence must include actual parser
acceptance/rejection and Core handoff behavior, not documentation wording alone.

## Current partial implementation evidence

The parser now accepts the canonical declaration shapes
`effect alias NAME = { ... };` and `effect group NAME = { ... };` at module and inline-module
scope. `EffectAliasDef` and `EffectGroupDef` preserve declaration and row spans plus the parsed
source path. Focused fixtures verify representative operation/evidence row items and confirm that
historical `proxy` definitions remain deterministic parse errors; no compatibility parser was
introduced.

### Stable stale-declaration rejection and authority disposition

The accepted source-boundary disposition is **historical/rejected** for top-level `capability`,
`proxy`, and `yield` declarations. The focused parser regressions in
`crates/ash-parser/tests/task_2001_stale_declaration_rejection.rs` require stable removed-form
diagnostics for ordinary and visible declarations. SPEC-095b now removes those forms from its active
definition list: `capability` is historical vocabulary superseded by interface operations plus
provider/admission metadata, while `proxy` and top-level `yield` are removed workflow forms.

This rejection changes no active target form and claims no handler runtime: `effect alias`,
`effect group`, `handler`, `newtype`, interfaces, impls, and ordinary functions retain their
recorded dispositions. The separate active operation spelling `proc yield` is not a declaration and
is unaffected. Parser rejection is not handler admission, lowering, dispatch, or execution.

### Parser/AST handler and newtype slice

The parser now also admits module and inline-module `handler` and `newtype` declarations through
distinct `Definition::Handler(HandlerDef)` and `Definition::Newtype(NewtypeDef)` carriers. The
handler carrier preserves its declaration-site marker, visibility, callable signature, body,
source span, and source path. The newtype carrier preserves its nominal name, generic parameters,
value constructor, representation type, visibility, source span, and source path. Focused parser
fixtures demonstrate these structural carriers and keep the historical `proxy` form rejected.

This is a parser/AST admission slice, not a claim that the full SPEC-095b handler grammar is
implemented: handler bodies currently use the existing general function-block parser, not the
canonical `on` clause grammar. No handler-marker typechecking/admission, newtype nominal identity
or inhabitation checking, Core lowering, constructor export/import summaries, runtime behavior, or
cross-layer diagnostic contract is supplied by this slice.

### TypeEnv declaration-registration slice

`TypeEnv::register_surface_module_declarations` now retains module-level callable declaration
kinds and nominal newtype metadata before a later checking or lowering boundary. The registration
query distinguishes a parsed `handler` from an ordinary `fn`, and the handler-only query rejects
the ordinary function. A parsed newtype registers a distinct nominal identity, its constructor,
and a representation-name summary without marking the wrapper as a transparent alias. Focused
tests exercise those facts after parsing the corresponding source declarations.

This remains declaration registration rather than full source-language typechecking. It is not
wired into normal module checking or `handle ... with` syntax, does not validate canonical handler
clauses, handler signatures, rows, contracts, newtype parameter well-formedness, representation
inhabitation, constructor application, or cross-module visibility, and does not lower or export
handler facts to Core/runtime summaries.

### Existing Core module-summary newtype handoff slice

`lower_module_type_metadata` now lowers a parsed top-level `NewtypeDef` through the existing
ordinary `ModuleSemanticSummary` boundary. It creates a module-anchored `TypeDeclId`, carries the
representation through the existing exposed type representation carrier, exports the declared
value constructor as a tuple-payload `ConstructorSummary`, and anchors both facts to the source
newtype span/path. The focused regression
`task_2001_module_summary_lowering.rs` establishes this handoff for `pub newtype OrderId =
OrderId(Int);`.

This is deliberately a minimal compatibility use of the existing ordinary-type summary schema. It
does not assert that the Core representation alone establishes all target newtype semantics, nor
does it supply import visibility, constructor application, inhabitation, runtime representation,
or full nominal-equality checking.

### Public nominal-newtype import and one-hop re-export

The normal provider/caller file path now preserves one public non-generic nominal newtype through
a named import. A provider declaration `pub newtype OrderId = OrderId(Int);` reaches a caller
using `use provider::{OrderId}` with parser-originated
`TypeDeclarationKind::NominalNewtype`, the provider-owned `TypeDeclId`, and the one tuple
constructor's `Int` payload contract intact. The caller can construct and pass `OrderId(7)` as
`OrderId`; a non-`Int` payload rejects, and `OrderId` and `Int` do not coerce in either direction.

Private provider newtypes do not enter either the caller type or value namespace through the named
import. An ordinary `pub type Counter = Int;` import remains transparent. Constructor metadata
alone never establishes nominality: the explicit `TypeDeclarationKind` is authoritative, so a
forged ordinary alias summary with an alias-backed tuple constructor rejects rather than gaining
newtype behavior. The focused end-to-end evidence is
[`task_2001_imported_nominal_newtype_checking.rs`](../../../crates/ash-engine/tests/task_2001_imported_nominal_newtype_checking.rs),
with summary-kind and provider-identity controls in
[`task_2001_handler_newtype_registration.rs`](../../../crates/ash-typeck/tests/task_2001_handler_newtype_registration.rs).

The same direct-import path, and one public `pub use` facade hop, now also admit the non-generic
singleton tuple constructor at the existing irrefutable-`let` boundary. In the caller,
`let OrderId(value) = OrderId(7);` binds `value` as the provider-declared `Int` representation
while the scrutinee remains nominal `OrderId`. The visible type name must resolve to the exact
provider `TypeDeclId`, and the visible imported binding must itself be public. Therefore
`inner`'s `pub newtype OrderId = OrderId(Int);`, re-exported once by
`outer` as `pub use inner::{OrderId};`, can be imported by a caller and pattern-matched without
rehoming the provider identity. A same-spelled facade remains eligible only when it resolves to
that same identity; a same-spelled or distinct local `CustomerId` constructor cannot consume an
`OrderId` scrutinee. A private provider import and a two-field
`OrderId(first, second)` pattern reject deterministically. The focused end-to-end evidence is
[`task_2001_imported_nominal_newtype_checking.rs`](../../../crates/ash-engine/tests/task_2001_imported_nominal_newtype_checking.rs).

### Completed nominal-newtype singleton pattern-universe slice

The same closed singleton constructor universe is now selected consistently for source-local,
direct public-import, and one-hop public-re-export non-generic newtypes at every supported surface
pattern boundary: irrefutable `let`, `match`, `if let`, and match exhaustiveness. The visible name
must resolve to the exact provider-owned `TypeDeclId`; the singleton constructor has exactly one
representation field. Thus `OrderId(value)` binds that declared representation in all three
eliminators, a wrong constructor or tuple arity rejects, and an empty match reports the stable
missing witness `OrderId(_0)`. An irrefutable singleton `if let` remains accepted with its ordinary
unreachable-else warning. Focused file-backed evidence is
[`task_2001_nominal_newtype_match_patterns.rs`](../../../crates/ash-engine/tests/task_2001_nominal_newtype_match_patterns.rs).

The module-summary wire contract carries this limit explicitly rather than inferring it from a
re-exported name: a provider declaration starts with `nominal_newtype_public_reexport_hops = 0`,
each public facade increments it, and a legacy/missing field deserializes to `u8::MAX` (unproved).
The pattern bridge admits only exact identity with a public-hop count at most one. This makes a
direct import and one public facade eligible while `inner → middle → outer → caller`, stale cache
data, and missing provenance fail closed; the transport metadata neither grants capability nor
authorizes runtime behavior.

This is a typechecking-only singleton-pattern slice. Generic newtypes, non-public or
identity-mismatched bindings, unproved multi-hop or other re-export topologies, proof patterns,
runtime representation erasure/execution, Core/CPS, frames, and broader cross-module nominal
behavior remain excluded.

### Completed canonical local-newtype identity propagation slice

Local nominal-newtype registration now carries a canonical declaration identity on every normal
program path. Module-aware type checking installs the actual defining module before registering
local declarations. The Engine file declaration resolver supplies that same file module identity,
and Engine inline checking supplies the exact
`ash_typeck::standalone_program_module_identity()` identity. The resulting local `TypeDeclId` is
therefore module-aware rather than a synthetic fallback on normal Engine/typechecking paths.

The sole retained fallback is intentional: callers that directly invoke `TypeEnv` declaration
registration without any module context continue to receive the documented fallback identity.
That explicit no-module API behavior is not a local registration defect and does not affect either
the normal local paths above or the provider-summary identity preserved by the named-import slice.
Focused evidence is
[`task_2001_local_newtype_identity.rs`](../../../crates/ash-typeck/tests/task_2001_local_newtype_identity.rs).

### Completed source-local nominal-newtype irrefutable-`let` pattern slice

Normal checking now recognizes a source-local, non-generic nominal newtype at an irrefutable
`let` pattern boundary. A root, zero-argument nominal type whose `TypeDeclId.module` is the
current module receives a singleton tuple-constructor universe: its declared constructor and
exactly one positional field, typed from the checked representation. Consequently,
`let OrderId(value) = OrderId(7);` binds `value` as `Int` while the scrutinee remains the nominal
`OrderId`; it does not introduce a nominal-to-representation coercion.

The bridge rejects a different nominal constructor and a wrong tuple arity before accepting the
binding. It is deliberately limited to the existing irrefutable-`let` checking route: the separate
public named-import and one-hop public re-export cases above are admitted only through a public
visible binding with the exact provider identity; generic, non-public, identity-mismatched,
multi-hop/unproved re-export, and broader cross-module newtypes retain their previous pattern
behavior. The singleton canonicalization now also feeds the `match`, `if let`,
and exhaustiveness routes above; proof-pattern routes remain unchanged. No runtime
representation/execution, Core lowering, CPS, frame, provider, or handler behavior is added.
Focused evidence is
[`task_2001_local_nominal_newtype_checking.rs`](../../../crates/ash-engine/tests/task_2001_local_nominal_newtype_checking.rs).

### Effect-row and handler module-summary handoff slice

`lower_module_type_metadata` now also emits dedicated summary metadata for parsed effect aliases,
effect groups, and handlers. Each alias/group has a module-anchored export identity, declaration
visibility, source-order row-item record, source anchor, and an explicit `NonGranting` authority
classification. Aliases are classified as `TransparentAlias`; groups are classified as
`DiagnosticGroup`, preserving their diagnostic presentation role without turning either row name
into an authority grant. A handler crosses the same summary boundary as a value-namespace export
with the distinct `Handler` marker and a declaration source anchor. The focused regression
`task_2001_module_summary_effect_exports.rs` verifies both row classifications, their origins and
module identities, and the distinct handler value marker.

This is checked export metadata only. It does not expand aliases, resolve/validate row items,
grant or discharge authority, perform alias/group cycle checks, establish import/re-export
selection or cross-module registration, validate handler clauses/signatures/contracts, admit a
handler at `handle ... with`, or establish runtime behavior. The row-item record is intentionally
source-preserving transport metadata until a full Core/typechecker row representation owns those
semantics.

### Imported effect-row and handler summary-registration slice

`TypeEnv::register_module_semantic_summaries` now registers the existing public summary carriers
transactionally. It validates effect-row export visibility, enclosing module/name identity, and the
mandatory `NonGranting` authority marker; it rejects conflicting visible effect-row names rather
than making import order observable. Registered rows are available by visible name and expand only
to their source-order `EffectRowItemSummary` metadata. Imported `ValueExportKind::Handler` entries
also retain the `Handler` callable marker, so the existing handler-only admission query accepts an
imported handler.

The focused regression `task_2001_imported_effect_handler_summaries.rs` covers transparent alias
and diagnostic-group lookup, non-granting preservation, metadata expansion, unknown-row rejection,
conflicting duplicate rejection, and imported handler admission.

This is direct summary API registration, not source import syntax or runtime authority. The former
V7 slice retained unparsed text only for compatibility; the active V8 migration below supplies the
structural normalization seam. Neither slice discharges authority, selects/re-exports imports,
lowers handler execution, or establishes canonical row-admission proof.

### Completed V8 structural effect-row summary migration

`SummaryVersion::STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8` retains V7's public
provider/binding and sanitized closure envelope, but replaces text-only row items with tagged,
source-order structural content. The witnessed carrier has the exact concrete
`(impl_type, interface, operation)` identity, evidence path, and open tail; the loader also emits
structural aliases/groups and the other currently parseable requirement forms rather than formatted
row text. A legacy/debug spelling may remain in memory for diagnostics and cache equality, but it
is not serialized or used for typed normalization. Qualified unresolved operation atoms are
transported as non-dependency symbolic requirements. V8 validation requires a coherent
non-opaque provider/binding closure and well-formed
structural payloads, rejects unknown structural fields, and rejects legacy text items. The
schema-versioned in-memory cache boundary preserves the public provider/binding/closure contract
without exposing an opaque dependency's private details.

V7 remains deserializable solely for legacy compatibility, but it rejects structural item payloads
and is ineligible at typed-handler normalization. Its exact required diagnostic is
`malformed imported-effect-row-summary: legacy V7 provider/binding row is ineligible for typed-handler normalization; require V8 structural content`.
The normalizer never reparses V7 text. For V8 it resolves a structural operation through the
declared concrete operation (including interface agreement), expands structural named rows, and
retains evidence/tail and imported-use provenance. Summary content still does not select a
provider, discharge an effect, install a provider/handler frame, admit a source program, or grant
runtime authority.

Focused evidence covers V8 operation/evidence/tail round-trip, unknown-field rejection, and V7
structural-payload rejection in
[`task_2001_v8_structural_effect_row_summary.rs`](../../../crates/ash-core/tests/task_2001_v8_structural_effect_row_summary.rs).
The ordinary file-loader witness transports a public V8 row and normalizes it without reparsing
text in
[`task_2001_v8_imported_handler_row_e2e.rs`](../../../crates/ash-engine/tests/task_2001_v8_imported_handler_row_e2e.rs).

### Source named-import transport slice

`load_ordinary_file` now preserves selected public effect-row and value exports when a source
module uses a named import such as `use provider::{Audit, audit_handler}`. Selection (including
the existing alias-bearing named-import path) filters to public metadata, transports the selected
effect-row/value carriers, and merges them into the file's imported semantic summaries. The
subsequent `TypeEnv` registration therefore retains `Audit` as non-granting, source-order
expandable metadata and retains `audit_handler` as a handler-only callable.

The end-to-end regression `task_2001_named_effect_handler_import.rs` starts with the provider
source, loads the caller through `load_ordinary_file`, and registers the resulting imported
semantic summaries in `TypeEnv`. It verifies the selected alias classification, `NonGranting`
authority, source-order expansion, and handler-only admission.

### Imported named-row validation slice

Imported alias and diagnostic-group names now participate in callable-row validation. Their
summary items use canonical surface spelling, recursively expand named/group references, reject
cycles, and pass raw operation/evidence item text through the existing row-family checks. The
end-to-end regression `task_2001_imported_row_resolution.rs` proves that a public imported alias
or group containing `requires_proof` is rejected as an unsupported `requires` row item instead
of silently becoming a row variable. The summary remains explicitly `NonGranting`; this process
creates no capability, provider, or admission state.

This is still a narrow imported named-row slice. It does not implement full operation/evidence
identity taxonomy, authority discharge, local alias registration, qualified references, handler
execution, or `pub use` / re-export transport. The evidence must not be read as a claim that
importing a row grants a capability or that a handler is executable.

### Local alias/group row-validation slice

Local effect aliases and groups are registered as `EffectRowExportSummary` entries before callable
row validation. They preserve canonical source-spelled row items, the enclosing module identity,
and explicit `NonGranting` authority, then use the same expansion/validation path as imports. The
focused typechecker regression `task_2001_local_effect_row_resolution.rs` proves that a local alias
containing `requires_proof` is rejected while a local group containing evidence remains authority
neutral (no resource or capability provenance is created).

This does not complete row semantics: operation/evidence identity taxonomy, authority discharge,
qualified/re-export imports, handler execution, and runtime behavior remain separate work.

### Local alias/group cycle rejection

Normal callable-row validation now rejects three local recursive expansion shapes before a
successful checked program can expose authority or capability provenance. It keeps the
`TypeEnvError::InvalidDefinition` boundary and reports the cycle path deterministically:

1. a direct alias self-cycle, `effect alias Audit = { Audit }`, as `Audit -> Audit`;
2. a direct group self-cycle, `effect group Audit = { group Audit }`, as `Audit -> Audit`; and
3. a mutual alias → group → alias cycle, `Audit → group Workflow → Audit`, as
   `Audit -> Workflow -> Audit`.

Each reaches the normal callable-row validation boundary and rejects as an invalid cyclic
definition whose diagnostic identifies the ordered cycle path. The focused evidence is
[`task_2001_local_effect_row_resolution.rs`](../../../crates/ash-typeck/tests/task_2001_local_effect_row_resolution.rs).
The successful local-group control in that same suite retains empty resource and capability
provenance; rejected cyclic inputs create no successful authority-bearing type-check result.

The same suite also proves stack cleanup after recursive expansion unwinds: an acyclic graph can
reuse a shared sibling row (`Shared`) from both an alias and a group, then type-check normally with
empty resource and capability provenance. That control prevents a completed sibling expansion
from being mistaken for a cycle during the remaining branch.

This is not a claim of full SPEC-097b cycle-path diagnostics, typed operation/evidence item
taxonomy, imported-cycle behavior, or alias/group versioning and invalidation. Those remain
specified realization or future contract work, and no broader import/cache behavior is inferred
from these local controls.

### Public `pub use` effect-row and handler-marker transport slice

Public `pub use` selection now transports effect-row and handler-value summary metadata through a
facade module. Re-exported summary identities are rehomed at the facade while retaining the
declaration source anchor, so diagnostics keep the defining declaration location without falsely
presenting the provider identity as the facade export identity. Effect rows retain their explicit
`NonGranting` authority and diagnostic/transparent classification; a re-exported handler retains
only its `Handler` declaration marker. The focused regression
`task_2001_named_effect_handler_import.rs` verifies a caller can import those public facade
exports, that the row remains non-granting without a capability binding, and that no runtime
handler is installed.

This is summary metadata transport only. It does not make a re-exported row an authority grant,
install/execute a handler, perform handler admission, or establish cross-module row discharge.

### Derived impl-handler source-fact slice

`derive handler name;` now materializes one checked **source-only** handler fact for the direct
impl's complete declared-method set, in declaration order. It does not select a co-located
explicit handler. The synthesized total identity-fold fact has a fresh answer type variable `A`,
an input row containing every exact impl-operation identity plus the open residual tail `r`, and
residual/output rows that retain that same open `r` after every declared operation is peeled.
Every generated continuation carries that residual row and the affine discipline.

The derived name is also registered in the existing source value-namespace marker registry as
`CallableDeclarationKind::Handler`. That marker deliberately supplies no variable type or
callable signature. It is necessary, but not sufficient, for source application: the
TASK-2013 route also requires this checked fact, so a marker with no checked declaration rejects
rather than acquiring a synthetic signature.

Each synthesized clause has `SurfaceOrigin::Desugaring` pointing to the `derive handler` source
span. The evidence is deliberately observable only through the checked declaration facts:
[`task_2001_handler_newtype_registration.rs`](../../../crates/ash-typeck/tests/task_2001_handler_newtype_registration.rs)
covers the exact two-operation union, independently quantified answer/residual facts, affine
continuations, and derive-site origin. The companion
[`task_2013_handler_application_fact.rs`](../../../crates/ash-typeck/tests/task_2013_handler_application_fact.rs)
proves the now-admitted local source use: `handle expr with name` resolves through the normal
handler marker plus checked-fact validation, instantiates the fresh answer from the operand result,
and binds the residual to the actual normalized operand row. It peels every derived impl operation
exactly once, preserves concrete residual requirements and an open-tail provenance, retains
canonical normalized row order plus operand/handler anchors, and cannot reuse an outer row fact
after lexical shadowing. The route covers the currently parseable alias/group/open-tail row forms
only through the narrow explicit zero-argument call of a row-annotated parameter; unsupported
computations remain fail closed.

This remains source/typechecking evidence only. It creates no Core handler, CPS provider frame,
admission state, lowering route, or runtime execution semantics.

This is intentionally not a completion claim. The remaining specified realization work is:

- complete operation/evidence identity taxonomy, qualified alias/group resolution,
  cross-module re-export diagnostics beyond selected public metadata, and authority-discharge behavior while retaining
  non-granting row-checking. The local direct and mutual cycle controls are complete, but
  full cycle diagnostics and imported/versioned behavior remain open;
- integrate declaration registration with full handler/newtype checking and diagnostics, then
  complete the canonical handler body grammar, handler admission, cross-module newtype behavior,
  and runtime behavior (or explicitly exclude each remaining form); and
- extend cross-layer tests and the differential corpus as those realizations land; and
- specify a future alias/group versioning and invalidation contract before implementing cache or
  incremental invalidation behavior. No current summary transport behavior implies such a policy.

Declaration-resolved `ImplType::operation(args)` itself is not in that remaining-decision list;
TASK-2011/TASK-2012/TASK-2017 are its settled target-semantics evidence.
