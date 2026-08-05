# TASK-2073: Checked Module Finalization and Export Closure

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§6-8 (`M-CHECK`, final export closure)
**Owned rule:** MOD-REAL-003 complete checked bodies/private-public/export-closed interface
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2073](../SEMANTIC-RULE-COVERAGE.md#task-2073-checked-module-finalization-and-export-closure)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Imported type-path closure:** Public imported type-bearing dependencies and callable signatures
now require a publicly reachable defining module path; root and fully public paths remain accepted,
while private, crate-only, and restricted enclosing paths reject atomically.
**Imported namespace-path closure:** Public imported namespace dependencies now require the same
public defining module path; this role-row slice remains minimum metadata and non-authorizing.
**Missing target-spec clauses:** The delivered Type-only slice checks ordinary and bodyless builtin callable signatures, canonical handler body facts, public ordinary types, nominal newtypes, resource schemas, public interface method metadata, sealed-domain facts and parent-scoped marker constructors, export-closed effect-row alias/group metadata with private and missing unqualified plus qualified row-path dependency rejection, promoted data-kind/proposition-predicate metadata with private and missing source-ADT dependency rejection, public role metadata, bounded type-function metadata with private type/function dependency rejection plus private equation-pattern-constructor and proposition-tail dependency rejection, public callable proposition-tail type/predicate/row dependency rejection, export-closed notation metadata with private, qualified-target, and missing target dependency rejection, parser-owned public macro summaries with syntax-only metadata and typed-signature dependency rejection plus imported private template-callable dependency rejection, checked public module-law evidence metadata with private parameter-type dependency rejection plus imported private evidence-callable dependency rejection, parent-scoped interface-law/implementation-proof fact matching with explicit checked nested kind/visibility summaries, and parser-carried public policy schema metadata with missing and private field-type dependency rejection plus checked default/invariant expressions and imported private value-callable dependency rejection, minimal named policy binding transport, and body-free public implementation summaries with private implementation dependency rejection. Rich policy-instance, persistence, inheritance, authority, and runtime semantics, remaining namespace forms, complete visibility/export closure, forged/cyclic dependency coverage, downstream Core/CPS/admission-runtime, and client parity remain incomplete. It retains private/public callable and namespace facts and origins, rejects unsupported public namespace facts before publication, validates staged `pub use` identity/origin, rejects missing or private public type-bearing dependencies for declarations and callable signatures, private signature/type dependencies, and private imported row, promoted-kind, notation, macro-template, evidence-expression, and policy-expression callable dependencies, revalidates collection drift, and tests normalized file/inline interface projection.
**Additional delivered clause:** Public interface-law propositions apply callable export closure to local and imported dependencies while preserving parent-scoped interface methods. Qualified implementation calls in public evidence, policy, and macro expressions use the implementation-registry visibility boundary without turning implementation members into standalone exports.
Qualified implementation-call closure is now included in the delivered Type-layer evidence; the remaining gaps listed above continue to exclude downstream lowering, admission, runtime, and client parity.
Public implementation `where T: Interface` bounds now use the interface namespace visibility boundary, rejecting local-private, imported-private, and missing bounds before publication while retaining public bounds as non-authorizing summary metadata.
Qualified public effect-row group paths and qualified notation callable targets now use the staged module-key and declaration visibility boundary, rejecting private enclosing modules or targets before publication while accepting public targets. This remains a Type-layer, non-authorizing closure check.
The finalizer must not recover signatures or bodies from TASK-2075's name-only view; it consumes the checker-internal snapshot directly. This bounded result is non-authorizing, and downstream Core/CPS, admission/runtime, and client parity remain separately owned or deferred.
The next bounded closure slice covers remaining declaration facts and remaining forged/incomplete/cyclic
dependency rejection while preserving the explicit role and policy-binding non-goals.
Public effect-row `Impl::operation` items validate resolved local implementation-registry visibility and parent-scoped operation identity; unknown/resource operation rows remain checker-owned non-authorizing metadata.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Next obligation:** Extend the bounded finalizer to remaining declaration facts while keeping named policy bindings deliberately transient and minimal (local alias, defining identity, policy namespace, provenance, and public schema only), and complete remaining forged/incomplete/cyclic dependency and visibility/export-closure rejection, including remaining imported namespace dependency visibility, while preserving downstream ownership boundaries. Keep
the Core/CPS, admission/runtime, and client-parity handoffs separate.

## Activation checkpoint

TASK-2073 is an active semantic owner. Its executable activation contract and focused finalization
target live in `crates/ash-typeck/tests/task_2073_checked_module_finalization.rs`; the target passes
102/102. The delivered slice is `partial / tested / below_spec`: it consumes
`CanonicalCollectedModuleSnapshot` plus TASK-2072's `CanonicalParsedImportResult`, publishes no
interfaces until all staged checks succeed, and leaves unsupported callable/namespace forms and
downstream layers explicit.

Imported public type dependencies validate the complete defining module path, not only the target
declaration's visibility. Root and fully public paths remain accepted; private, crate-only, and
restricted enclosing module paths reject atomically for public types and callable signatures.

The delivered policy slice parses and projects public policy schemas. It preserves the policy name,
generic parameters, field schema, defaults, and invariant carrier as checked namespace metadata,
checks concrete field defaults and Bool-typed invariants, and rejects those failures atomically
without treating a policy schema as an admitted policy binding or runtime authority; field-type
dependencies satisfy the same public closure as other exported metadata. A named policy binding
is only the staged local alias plus defining identity, policy namespace, provenance, and public
schema projection; private `use` bindings do not become persistent final-interface state, and no
policy instance, inheritance, lowering, admission, or runtime semantics are implied.

## Delivered bounded finalization checkpoint

`canonical_checked_module_finalizer` now publishes an opaque atomic set of checked module
interfaces. It stages ordinary and bodyless builtin callable signatures plus canonical handler
body facts from the internal snapshot, applies explicit missing/private/imported dependency
closure to public callable signatures, checks public ordinary types, nominal newtypes, resource
schemas, interface method metadata, sealed-domain facts and parent-scoped marker constructors,
projects public effect-row alias/group metadata while rejecting private and missing unqualified named group dependencies plus qualified row-path visibility failures,
and projects promoted data-kind/proposition-predicate metadata while rejecting missing or private source/predicate
type dependencies, rejects missing or private type dependencies for public type-bearing metadata and
accepts imported public type identities through an opaque checked carrier, public role metadata, bounded type-function metadata, export-closed notation
metadata with qualified callable-target closure, parser-owned syntax-only macro summaries, public policy schema metadata, and body-free
public implementation summaries while
rejecting private type/function, notation-target, macro typed-signature/template-callable, policy
field-type/default/invariant, imported evidence/policy-expression callable, public law parameter-type,
and public implementation type/interface/bound dependencies;
Public implementation `where T: Interface` bounds use the interface namespace visibility boundary, rejecting local-private, imported-private, and missing bounds before publication while retaining public bounds as non-authorizing summary metadata.
Staged public re-exports also require every enclosing defining module declaration to be publicly
reachable; declaration visibility alone cannot publish a path under a private, crate-only, or
restricted module.
public implementation `where` bounds use the same interface-namespace visibility checks for local,
imported, and missing interfaces;
public effect-row aliases and groups now follow staged local, imported, and qualified row-carrier
dependencies transitively. A private transitive leaf or a public row cycle rejects before any
private/public interface is published; bare unresolved whole-row variables remain checker-owned
and are not reinterpreted as namespace declarations;
public type-function equation constructor patterns and proposition-tail type/predicate dependencies
use the same export-closure checks;
public callable proposition-tail type, named-predicate, and effect-row dependencies use the same
export-closure checks across ordinary functions, handlers, and builtins;
public callable proposition-tail rows also apply the staged visibility check to bare named row
items and unqualified single-segment operation items, while unresolved row variables remain
checker-owned;
public effect-row role and policy items now apply the same staged local, imported, and qualified
visibility check while retaining roles as minimum metadata and policies as transient schema-only
metadata;
public effect-row `Impl::operation` items validate a resolved local implementation's public
registry visibility and parent-scoped operation member, while unknown/resource operation rows
remain non-authorizing metadata owned by their existing checker;
public interface-law propositions apply the same callable export-closure checks to local and
imported dependencies while retaining the interface's own methods as parent-scoped checked
members;
parent-scoped interface-law and implementation-proof facts are retained after checker registration
validates their pairing, with checked nested kind/visibility summaries that never flatten into the
module evidence namespace;
it rejects unsupported public namespace facts before publication,
and retains
declaration identity, visibility, spans, origin, signature, and body type in the private view,
projects checked namespace facts into private/public views, keeps same-spelled exports in separate
namespace buckets, and turns staged `pub use` facts
into identity-preserving exports only when their defining declaration is public and its origin and
lookup name still agree. Collection drift is revalidated against the acquired expanded graph before
any interface is returned. Public signatures that mention a private local declaration reject before
body publication. Public row, promoted-kind, and notation dependencies also reject when their
imported binding is crate-private or otherwise not publicly reachable; public macro templates, laws,
and proofs apply the same rule to resolved imported value-callable dependencies, as do public policy
defaults and invariants.
Imported binding declaration visibility is also revalidated against the acquired checked target;
same-identity visibility drift rejects atomically with no interface publication. The defining
declaration span and source ordinal carried by an import are revalidated against the checked target
and identity origin key before imported type collection; import-path visibility remains its own
parser-owned fact and is not compared to declaration visibility.
Duplicated staged `pub use` carriers are revalidated against the authoritative selected binding
before export projection: the carrier must remain a re-export and exactly match its importing
module/local-name binding, or finalization rejects atomically.
Imported defining identities are also checked against the canonical structural module scopes;
bindings that cross a private enclosing module path reject before imported type collection.
Positive finalizer witnesses cover parent-owned inherited, `pub(self)`, `pub(super)`, `pub(crate)`,
and restricted module boundaries, so valid parent imports are not rejected by the revalidation pass.
Staged public re-exports additionally require every enclosing defining module declaration to be
publicly reachable; a public declaration under a private module path is rejected atomically before
export projection, while an equivalent public path remains export-closed. Diagnostics retain the
use/declaration spans, attempted canonical access path, first offending segment, and a readable
visibility boundary. Only fully public `pub use` enters the external export projection; staged
`pub(crate)`, `pub(super)`, and restricted re-exports remain non-public metadata, including when a
later `pub use` republishes the narrow alias.
The dedicated unit witnesses
`canonical_checked_module_finalizer::tests::forged_imported_binding_private_defining_module_rejects_atomically`
and
`canonical_checked_module_finalizer::tests::forged_public_use_binding_reexport_flag_rejects_atomically`,
`canonical_checked_module_finalizer::tests::public_use_nested_private_path_diagnostic_preserves_access_context`,
`canonical_checked_module_finalizer::tests::public_use_projection_excludes_narrow_reexports`,
`canonical_checked_module_finalizer::tests::public_use_nested_private_module_path_rejects`,
`canonical_checked_module_finalizer::tests::public_use_nested_pub_crate_module_path_rejects`,
`canonical_checked_module_finalizer::tests::public_use_nested_pub_super_module_path_rejects`, and
`canonical_checked_module_finalizer::tests::public_use_nested_restricted_to_allowed_module_path_rejects`
and
`canonical_checked_module_finalizer::tests::public_use_projection_does_not_promote_narrow_reexport`,
`canonical_checked_module_finalizer::tests::imported_impl_operation_private_defining_module_path_rejects_atomically`, and
`canonical_checked_module_finalizer::tests::imported_impl_operation_public_defining_module_path_preserves_closure`
exercise defining-module visibility, diagnostic context, public projection, and carrier drift
independently of the 102/102 integration target.

Focused evidence inventory in the 102/102 target includes positive and negative witnesses:
`TEST-MOD-REAL-003-TASK-2073-CHECKED-PRIVATE-PUBLIC` and
`TEST-MOD-REAL-003-TASK-2073-FINAL-PUB-USE`,
`TEST-MOD-REAL-003-TASK-2073-BUILTIN-PUBLIC-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-HANDLER-CHECKED-BODY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-SIGNATURE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-NEWTYPE-SIGNATURE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PUBLIC-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROW-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PUBLIC-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PUBLIC-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PRIVATE-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PUBLIC-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PUBLIC-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PRIVATE-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PUBLIC-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PRIVATE-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-DATA-KIND-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-NOTATION-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-PUBLIC-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-IMPORTED-PRIVATE-MODULE-PATH`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-IMPORTED-TYPE-PRIVATE-MODULE-PATH`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-DOMAIN-RESOURCE-NAMESPACE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-SEALED-DOMAIN-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-ROW-PUBLIC-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-DATA-KIND-PREDICATE-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-ROLE-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-TAIL-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-NOTATION-PUBLIC-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-SUMMARY-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-PROJECTION`, and
`TEST-MOD-REAL-003-TASK-2073-IMPL-PROOF-LAW-PAIR`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-PROJECTION`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPLEMENTATION-SUMMARY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-NESTED-EVIDENCE-VISIBILITY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PROOF-VISIBILITY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-PUBLIC-CALLABLE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PUBLIC-QUALIFIED-IMPL-CALL`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PUBLIC-WHERE-BOUND`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PUBLIC-QUALIFIED-IMPL-OPERATION`, and
`TEST-MOD-REAL-003-TASK-2073-FORGED-IMPORTED-IMPL-OPERATION-PUBLIC-MODULE-PATH`; negative
`TEST-MOD-REAL-003-TASK-2073-EXPORT-CLOSURE-REJECTION` and
`TEST-MOD-REAL-003-TASK-2073-BUILTIN-PRIVATE-SIGNATURE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-FUNCTION-MISSING-SIGNATURE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-BUILTIN-MISSING-SIGNATURE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-HANDLER-MISSING-SIGNATURE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-TYPE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-ROW`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-PRIVATE-UNQUALIFIED-ROW`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-CALLABLE-PROPOSITION-IMPORTED-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-PRIVATE-CALLABLE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-LAW-IMPORTED-PRIVATE-CALLABLE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-UNSUPPORTED-PUBLIC-NAMESPACE`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-MISSING-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-NEWTYPE-MISSING-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-RESOURCE-MISSING-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-MISSING-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-INTERFACE-MISSING-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-SEALED-DOMAIN-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-ROLE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-ROLE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-POLICY-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPORTED-POLICY-ROW-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-ROW-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-TRANSITIVE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-CYCLIC-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-MISSING-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-DATA-KIND-MISSING-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-PREDICATE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PATTERN-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-TYPE-FUNCTION-PROPOSITION-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-NOTATION-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-QUALIFIED-NOTATION-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-TYPED-SIGNATURE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-MACRO-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EVIDENCE-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-FIELD-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-MISSING-FIELD-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-IMPORTED-CALLABLE-PRIVATE-DEPENDENCY`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPLEMENTATION-PRIVATE-DEPENDENCY`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-MODULE-LAW-PRIVATE-QUALIFIED-IMPL-CALL`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-PRIVATE-QUALIFIED-IMPL-OPERATION`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-EFFECT-ROW-MISSING-QUALIFIED-IMPL-OPERATION`,
`TEST-MOD-REAL-003-TASK-2073-FORGED-IMPORTED-IMPL-OPERATION-PRIVATE-MODULE-PATH`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-PRIVATE-WHERE-BOUND`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-IMPORTED-PRIVATE-WHERE-BOUND`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-IMPL-MISSING-WHERE-BOUND`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-MODULE-PATH-CLOSURE`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-DEFAULT-TYPE-MISMATCH`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-POLICY-INVARIANT-NOT-BOOL`, and
`TEST-MOD-REAL-003-TASK-2073-AUTHORITY-FENCE`; mutation
`TEST-MOD-REAL-003-TASK-2073-STALE-ATOMICITY` and
`TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-VISIBILITY-DRIFT`, and
`TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-MODULE-VISIBILITY-DRIFT`, and
`TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-SHAPE-MISMATCH`, and
`TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-LOCAL-NAME-DRIFT`, and
`TEST-MOD-REAL-003-TASK-2073-IMPORTED-BINDING-DECLARATION-METADATA-DRIFT`,
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-PRIVATE-MODULE-PATH`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-BINDING-CARRIER-DRIFT`, and
`TEST-MOD-REAL-003-TASK-2073-PUBLIC-USE-BINDING-METADATA-DRIFT`; generated/property
`TEST-MOD-REAL-003-TASK-2073-GENERATED-CLOSURE-PROPERTY`; and normalized file/inline
`TEST-MOD-REAL-003-TASK-2073-FILE-INLINE-FINAL-PARITY`. These are tests, not a proof, Core/CPS,
Engine, admission/runtime, or CLI/daemon parity claim.

## Description

Own the final Type-layer interface boundary omitted from TASK-2068: complete body checking,
private/public fact retention, export closure, and final public-use projection. This task alone may
turn staged facts into a versioned final interface; it still cannot lower, admit, or execute.

## Requirements

1. Check every supported body and callable/namespace fact against complete provisional and binding
   inputs while preserving canonical provenance and diagnostic anchoring.
2. Keep private checked facts distinct from public projections and validate complete export closure,
   including final `pub use`, only after all inputs succeed.
3. Reject stale, forged, incomplete, failed, cyclic, or export-inconsistent dependencies before any
   final interface publishes.
4. Establish normalized Type-layer file/inline final-interface parity.

## TDD Steps

1. Add red complete-body/private-public collection tests.
2. Add red final `pub use`/export-closure and stale/forged/incomplete rejection tests.
3. Add red public callable signature dependency closure, including missing, private, and imported types,
   plus imported private row, promoted-kind, and notation dependencies.
4. Add red public callable proposition-tail type, predicate, and row dependency closure.
5. Add red macro-summary projection and private typed-signature/imported template-callable
   dependency rejection.
6. Add red public type-function equation-pattern and proposition-tail dependency rejection.
7. Add red public policy schema projection, missing/private field-type dependency rejection, imported
   private policy-expression callable dependency rejection, default-type mismatch rejection, and
   invariant-type rejection.
8. Add red parent-scoped interface-law/implementation-proof fact matching and evidence separation,
   checked nested kind/visibility summaries, body-free public implementation summaries, and private
   implementation dependency rejection, including interface-law proposition dependency rejection.
9. Add red public implementation where-bound visibility closure for public, private, imported-private,
   and missing interface bounds.
10. Add red atomic-finalization, normalized file/inline final-interface parity, generated closure,
   and authority-fence tests.
11. Implement after RED, run focused Type tests/quality gates, then promote actual evidence only.

## Scope and non-goals

This task excludes parser acquisition/graph construction, import grammar/binding ownership,
Core/CPS lowering, Engine transport/link/admission/execution, direct evaluation, and CLI/daemon
terminal parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2075 internal collected snapshots and TASK-2072 atomic resolved bindings/staged
  `pub use` facts. TASK-2071 supplies only the contract.
- **Produces:** complete versioned final checked module/interface/export closure, non-authorizing.
- **Downstream owner:** TASK-2069 exclusively consumes this handoff for source-to-Core/CPS and
  Engine transport fencing; TASK-2063 awaits TASK-2069; TASK-2064 consumes TASK-2073/2069/2063.
- **Integration/proof:** TASK-2064 proves end-to-end file/inline/client terminal parity.
- [x] Bounded ordinary/builtin/handler callable and type/domain/resource/interface/sealed-domain/effect-row/data-kind/proposition/role/type-function/notation private/public,
  export-closure rejection, atomic stale-input, file/inline, generated/property, and
  authority-fence evidence is recorded, including missing type-dependency rejection, qualified
  row/notation dependency visibility, and imported public type identity transport.
- [x] Public type-function equation constructor patterns and proposition-tail type/predicate
  dependencies participate in atomic export-closure rejection.
- [x] Public callable proposition-tail type, predicate, and row dependencies participate in atomic
  export-closure rejection.
- [x] Public parser-owned macro summaries preserve syntax-only metadata and reject private typed-signature dependencies.
- [x] Public module-law evidence preserves the evidence namespace and rejects private parameter-type dependencies; parent-scoped implementation proofs retain matched proof facts without becoming standalone exports.
- [x] Public policy schema summaries preserve fields/defaults/invariants, check concrete defaults and Bool invariants, and reject missing or private field-type dependencies.
- [x] Public implementation summaries preserve body-free metadata, reject private dependencies, and
  keep implementation members parent-scoped and non-standalone.
- [x] Parent-scoped interface laws and implementation proofs preserve checked kind/visibility
  metadata without flattening into standalone evidence exports.
- [x] Public interface-law propositions apply callable export closure while allowing parent-scoped
  interface methods.
- [x] Public implementation where-bounds use interface-namespace visibility closure for public,
  local-private, imported-private, and missing interfaces.
- [x] Imported binding local names, declaration shape, origin, visibility, declaration span, and
  defining source ordinal are revalidated before imported type collection and interface publication.
- [x] Imported defining module paths revalidate canonical parent-owned structural visibility,
  including private, `pub(self)`, `pub(super)`, `pub(crate)`, and restricted module boundaries.
- [x] Staged public re-exports require publicly reachable defining module paths and reject private
  enclosing modules atomically before export projection.
- [x] Public re-export diagnostics retain use/declaration spans, attempted path, offending segment,
  and violated visibility; narrow re-exports stay outside the external public projection.
- [x] Imported public type dependencies and callable signatures require publicly reachable defining
  module paths, including root/public acceptance and private/crate/restricted rejection.
- [x] Imported namespace dependencies require publicly reachable defining module paths; role rows
  remain minimum, non-authorizing metadata.
- [x] Forged imported implementation-operation carriers revalidate publicly reachable defining
  module paths before a public effect row can preserve the parent-scoped operation metadata.
- [x] Minimal named policy binding transport preserves the local alias, defining identity, policy namespace,
  provenance, and public schema without persisting a policy instance or granting authority.
- [ ] Remaining declaration facts satisfy complete export closure.
- [ ] Complete body/private/public/export-closure evidence for every remaining target namespace and callable
  form is recorded.
- [ ] No final interface is treated as an admission credential or execution fallback.
