# SPEC-057: Unified Type/Module Pipeline and Semantic Summaries

**Status:** Draft
**Date:** 2026-04-30
**Promotes:** [DESIGN-034 §16.1](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#161-spec-a-unified-typemodule-pipeline-and-semantic-summaries)
**Builds on:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-009](SPEC-009-MODULES.md), [SPEC-012](SPEC-012-IMPORTS.md), [SPEC-020](SPEC-020-ADT-TYPES.md), [SPEC-030](SPEC-030-MODULE-TYPE-RESOLUTION.md)
**Related:** [SPEC-034](SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
**Plan:** [PLAN-105](../plan/PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
**Implementation Tasks:** [TASK-780](../plan/tasks/TASK-780-unified-type-module-pipeline-spec-plan-packet.md) through [TASK-791](../plan/tasks/TASK-791-spec-a-closeout-docs-examples-verification.md)

## 1. Summary

SPEC-057 is SPEC-A from DESIGN-034. It establishes the Tier 0 carrier path that later total type-computation specs require.

The required end state is:

```text
source file
  -> ash-parser ModuleFile
  -> surface ordinary type declaration items
  -> ash-core ordinary type declarations and canonical identities
  -> ash-core ModuleSemanticSummary
  -> ash-engine summary transport/import/export
  -> ash-typeck TypeEnv declaration, validation, and visibility exposure
```

Ordinary `type` declarations MUST flow through the same authoritative module-file path as other module items. They MUST NOT be discovered only by engine-side source-snippet scanning.

This spec does not add type-level computation. It creates the semantic-summary roadbed for later specs to add type-expression IR, sealed type-level domains, normalization, direct `type fn`, cross-module type-computation summaries, associated type-family computation, and propositions.

## 2. Motivation

DESIGN-034 requires total compile-time type computation to be total, terminating, normalizing, and modular. That requirement cannot be implemented safely while ordinary type metadata is fragmented across parser-only structures, engine-private export tables, typechecker maps, source snippets, and capability-specific metadata carriers.

Current implementation reality matches this risk:

- `crates/ash-parser/src/parse_type_def.rs` parses ordinary type definitions, but the normal `ModuleFile` path does not yet make ordinary `type` declarations an authoritative definition item.
- `crates/ash-engine/src/module_loader.rs` has engine-side source-snippet collection paths such as `collect_public_type_defs_from_source`, `collect_type_identity_defs_from_source`, and `extract_semicolon_snippets`.
- `crates/ash-engine/src/module_loader.rs` owns an engine-private `ModuleExports` structure that carries type exports and constructor exports.
- `crates/ash-core/src/ast.rs` already has ordinary `TypeDef`, `TypeBody`, `VariantDef`, `TypeExpr`, and `ModuleItem::Type` carriers.
- `crates/ash-typeck/src/type_env.rs` already has placeholder declaration, registration, constructor exposure, transparent aliases, interfaces, associated types, capability/resource metadata, and impl schemes.

SPEC-057 reconciles those paths by making ordinary type metadata a normal module semantic artifact owned by `ash-core`, transported by `ash-engine`, and consumed by `ash-typeck`.

## 3. Implementation Baseline

This spec assumes the existing module/import/type substrate:

1. SPEC-009 defines `ModuleFile` as the authoritative model for `.ash` module files.
2. SPEC-012 defines import and re-export behavior, including explicit `pub use` behavior.
3. SPEC-020 defines ordinary ADT/type behavior: structs, enums, aliases, generic parameters, variants, value constructors, and pattern matching.
4. SPEC-030 defines sibling type pre-declaration/order-independence behavior and the current module-file checking path.
5. SPEC-034 and SPEC-035 define current generic impl and simple associated type behavior. SPEC-057 preserves them.

SPEC-057 refines SPEC-030 by replacing the normative source-snippet ordinary type collection path with a normal ModuleFile/core-summary path. SPEC-030's useful two-pass type-registration invariant is preserved.

### 3.1 SPEC-030 amendment map

SPEC-057 amends SPEC-030 as follows:

- SPEC-030 §3.1 through §3.4 remain normative: sibling type names are pre-declared before full validation, and registration must be order-independent.
- SPEC-030 §3.5 is superseded for ordinary type metadata. The module loader/type pipeline is changed by SPEC-057.
- SPEC-030 §5.1 steps 2 through 4 are replaced with:
  1. parse the source as `ModuleFile`;
  2. collect ordinary type declarations from ModuleFile/core lowered items;
  3. build or consume `ModuleSemanticSummary`;
  4. predeclare then validate/register visible type identities through TypeEnv.
- `Engine::check_module_file` is a normal module path and MUST NOT use snippet extraction for ordinary type metadata once SPEC-057 is implemented.
- SPEC-030 §4 `pub mod` behavior, including no implicit flattening, remains unchanged.

## 4. Scope

In scope for SPEC-057:

- Existing ordinary `type` declarations, including `pub type` and `builtin type`, parsed as ordinary module-file items.
- Surface carriers that preserve visibility, name, parameters, body, builtin marker, source origin, and spans.
- Lowering from parser surface declarations to `ash-core` ordinary type declarations and summary entries.
- Canonical module-anchored identities for ordinary type declarations and ordinary ADT constructors/variants.
- A core-owned module semantic summary format for ordinary type metadata.
- Visibility and opacity rules for public/private/crate ordinary type metadata.
- Import/export and re-export of public ordinary type identities and constructor identities under existing module/import rules.
- TypeEnv consumption from semantic summaries using a two-pass declare-then-validate/expose pipeline.
- Replacement or strict quarantine of engine-side source-snippet ordinary type collection.
- Diagnostics and tests proving ordinary type metadata is no longer discovered only by snippet extraction.
- Reserved summary extension points for later type-computation specs, without implementing their semantics.
  SPEC-057 implementations MUST leave those future namespaces empty or uninterpreted except for existing current-feature metadata explicitly named in this spec.

Out of scope:

- Type functions, type-function equations, coverage, overlap, termination, or recursion checks.
- Sealed type-level domains and marker constructors.
- Promoted ADT constructors or DataKinds-style promotion.
- Type-function application IR, neutral forms, normal-form grammar, or definitional equality.
- Environment-aware normalization or normalize-and-compare equality.
- Generalized associated projections or recursive associated type-family computation.
- Higher-kinded type parameters, type-constructor holes, general source-level kind checking, or partial type-constructor application.
- Constraint/proposition solving, disequality, proof search, SMT integration, or type-function inversion.
- Changes to ADT runtime construction, pattern matching, exhaustiveness, or value semantics, except that metadata reaches existing paths through the unified pipeline.
- Changes to SPEC-034 impl search, monomorphization, coherence, or recursion limits.
- Capability/resource authority semantics, workflow/process semantics, or runtime provenance semantics.
- LSP/Salsa/incremental analysis except as future consumers of the summary boundary.
- New import syntax, package registry behavior, or cross-crate dependency semantics beyond existing module/import behavior.

## 5. Terminology

**Surface ordinary type declaration**: the parser-owned representation of an existing Ash `type`, `pub type`, or `builtin type` declaration as it appears in a module file.

**Core ordinary type declaration**: an `ash-core` semantic carrier for the same declaration, independent of parser-private syntax structures.

**Canonical type identity**: a stable identity for an ordinary type declaration anchored in the defining module and declaration, not a bare string name.

**Constructor identity**: a stable identity for an ordinary ADT constructor/variant exposed by a type representation when visibility permits.

**ModuleSemanticSummary**: a core-owned carrier that describes checked/importable semantic metadata for a module. In SPEC-057, it is an ordinary-type module semantic summary: it carries ordinary type metadata and reserved identity slots only.

**PublicWorkflowSummary**: the Phase 108 / SPEC-056 core-owned workflow contract summary carrier for imported `Workflow<A>` values and workflow-returning callables. SPEC-057 MUST preserve this carrier and its transport path; it does not replace, reinterpret, or subsume workflow contract summaries.

**Exported summary**: the public subset of a module semantic summary transported across module boundaries.

**Private summary facts**: same-module facts used for checking private implementation details. They MUST NOT become downstream reducible equality facts.

**Snippet fallback**: any source-text scan that extracts `type` snippets outside the normal ModuleFile parser path. SPEC-057 permits this only as a named, fenced compatibility path if removal cannot happen in the same task.

## 6. Required Invariants

1. There is one authoritative ordinary type declaration path for normal module checking: ModuleFile -> core declarations/summaries -> TypeEnv.
2. Source-snippet scanning MUST NOT be the normal semantic path for ordinary type metadata.
3. Canonical type identity MUST include module/declaration anchoring and MUST NOT be string-only.
4. Re-exports MUST preserve the original canonical type identity rather than minting a new identity.
5. Public signatures may reference public type identities from imported summaries.
6. Private type definitions may support private checking inside the defining module but MUST NOT leak reducible private representation facts to downstream modules.
7. Constructor identities are exposed only when the type representation is visible according to the current language rules.
8. `ash-core` owns shared semantic summary carriers and canonical identity data.
9. `ash-engine` transports, caches, imports, and exports summaries, but MUST NOT own type semantics.
10. `ash-typeck` validates, registers, and consumes summaries. It MUST NOT rediscover ordinary declarations from raw source snippets.
11. Import order MUST NOT affect canonical identity or successful sibling type registration.
12. SPEC-057 MUST preserve existing SPEC-020 ordinary ADT semantics; ordinary ADTs are not promoted type-level domains.

## 7. Parser and ModuleFile Contract

The parser MUST represent ordinary type declarations as normal module-file items. The exact Rust shape may vary, but the semantic contract is equivalent to one of:

```text
surface::Definition::Type(surface::TypeDef)
```

or:

```text
surface::ModuleItem::Type(surface::TypeDef)
```

The surface carrier MUST preserve visibility, name, type parameters, body shape, variant names/payload shapes, source span, and source origin.

The existing parser in `parse_type_def.rs` may be reused, but its output MUST feed the normal `ModuleFile` parse result for module-file checking.

The parser MUST NOT lower type declarations into typechecker artifacts. Parser ownership ends at surface syntax and spans.

A file containing only ordinary type declarations MUST parse as a module file. Unknown-item recovery MUST NOT silently skip ordinary `type` declarations that the standalone type parser would accept.

## 8. Lowering and Core Ownership

`ash-core` MUST own shared semantic carriers used across parser, engine, and typechecker boundaries.

SPEC-057 requires core-owned equivalents of:

```text
TypeDeclId
ConstructorId / VariantId
ModuleSummaryId or ModuleId anchor
ModuleSemanticSummary
TypeDeclSummary
Visibility / representation exposure metadata
SourceOrigin / source span metadata
```

The exact Rust names may differ, but these concepts MUST NOT remain engine-private or parser-private.

Minimal canonical identity contract:

- `TypeDeclId` origin is the resolved SPEC-009 module identity plus ordinary type declaration name and item kind.
- Import aliases and re-export paths are exported names pointing to the origin identity; they are not part of the origin identity.
- Source spans are diagnostic anchors only and MUST NOT affect identity.
- Duplicate ordinary type declarations in one module are errors, not span-disambiguated overloads.
- Stdlib, user, and crate-root module identities must be canonicalized before type IDs are minted.
- Constructor/variant IDs derive from the parent `TypeDeclId` plus constructor/variant name and payload kind.

Lowering MUST convert surface ordinary type declarations to core ordinary type declarations and summary entries while preserving module identity, declaration source anchor, visibility, type parameters, representation body, constructor/variant identity data, and spans suitable for diagnostics.

For SPEC-057, the core type-expression model remains the existing ordinary type expression model. SPEC-057 MUST NOT introduce `TypeFnApp`, neutral forms, generalized projection IR, or a normal-form grammar.

## 9. ModuleSemanticSummary Contract

SPEC-057 introduces or designates a core-owned module semantic summary carrier. For this packet, the minimum public summary is:

```text
ModuleSemanticSummary {
    module: ModuleIdentity,
    version: SummaryVersion,
    exported_types: Vec<TypeDeclSummary>,
    exported_constructors: Vec<ConstructorSummary>,
    re_exports: Vec<ReExportSummary>,
    imported_summary_refs: Vec<ModuleSummaryRef>,
    reserved_identity_slots: ReservedSemanticIdentitySlots,
    diagnostics_anchors: Vec<SourceAnchor>,
}
```

This is a schematic contract, not a required exact Rust layout.

`TypeDeclSummary` MUST carry enough metadata for downstream modules to register and type-check public references without reparsing source snippets:

- canonical type identity;
- exported name/path;
- visibility;
- generic parameters;
- public representation exposure status;
- public body metadata when representation is public;
- opaque identity marker when representation is private/opaque;
- source anchor for diagnostics.

Private summaries may exist for same-module checking. Exported summaries MUST NOT require downstream modules to unfold private aliases or private representations.

Reserved identity slots may include interface and associated-member identity placeholders for current metadata, but SPEC-057 MUST NOT assign type-computation semantics to those slots.

Concrete identity semantics in SPEC-057 are limited to ordinary type declarations and ordinary ADT constructor/variant identities. Future type-function identities, sealed-domain identities, generalized projection identities, computation-summary identities, and associated-family identities are reserved extension namespaces only. They MUST remain empty or uninterpreted in SPEC-057 implementations. Concrete semantics for those future identities belong to SPEC-B, SPEC-C, SPEC-E, SPEC-F, and SPEC-G.

Existing SPEC-034/SPEC-035 interface and associated-member metadata may receive opaque stable IDs only to preserve current behavior. Those IDs MUST NOT participate in generalized projection resolution, family computation, normalization, or definitional equality.

The ordinary-type `ModuleSemanticSummary` introduced by SPEC-057 MUST NOT erase or replace the Phase 108 workflow-summary path. In particular, implementations MUST preserve `ash_core::workflow_carrier::PublicWorkflowSummary`, `InlineCallable.workflow_summary`, `Workflow.imported_workflow_summaries`, and `TypeEnv` public workflow-summary bindings unless a later Workflow-specific spec explicitly migrates them. SPEC-057 may route ordinary type identities needed by workflow signatures through the new ordinary-type summary, but workflow contract/projection facts remain owned by SPEC-056 carriers.

## 10. Visibility and Opacity

Visibility and opacity are semantic-summary invariants, not ad-hoc import behavior.

Rules:

1. A public ordinary type identity may be exported and imported.
2. A private ordinary type identity may be used inside the defining module.
3. A private ordinary type MUST NOT be imported by downstream modules.
4. A public signature that exposes a private ordinary type MUST be rejected unless a separate existing opaque/builtin exception explicitly applies.
5. Public constructors/variants are exported only when representation visibility permits constructor exposure.
6. Re-exporting a type through `pub use` preserves the original canonical type identity.
7. `pub(crate)` or crate-scoped visibility, where supported, MUST be specified relative to existing module/crate graph semantics and MUST NOT be guessed per import site.

Opaque exported identities may be named downstream only for existing explicit builtin/opaque exceptions. SPEC-057 does not add new opaque type syntax, constructor privacy syntax, field privacy, or representation-hiding semantics. Ordinary `pub type` exports exactly the representation and constructors that current SPEC-020 rules already expose. Any broader public-identity/private-representation feature requires a separate spec amendment.

## 11. Import, Export, and Re-Export Behavior

The module loader MUST build type summaries from parsed ModuleFile/core items, not raw snippets, on the normal path.

Existing import syntax remains unchanged. SPEC-057 only ensures ordinary type metadata participates in those imports.

Required behavior:

- named imports bring public type identities and allowed constructor identities into scope;
- glob imports bring the same public type metadata they would bring for explicit names;
- `pub use` re-exports preserve canonical identity;
- child module summaries are available for explicit re-export without implicit flattening;
- public function/workflow signatures that mention imported public type identities resolve through summaries;
- import order does not change identity or registration success;
- missing summaries produce diagnostics rather than silent fallback to string names;
- named imports, glob imports, and `pub use` re-exports of workflow-returning callables MUST preserve existing `PublicWorkflowSummary` data while ordinary type identities move through SPEC-057 summaries.

## 12. TypeEnv Integration

`ash-typeck` MUST consume module semantic summaries using a two-pass pipeline:

1. Declare all visible ordinary type names and canonical identities.
2. Validate bodies and expose representations/constructors according to visibility.

This preserves SPEC-030 sibling type resolution while moving its inputs from source snippets to semantic summaries. TypeEnv may need canonical identity-aware keys or alias-to-identity bindings; string-only names are insufficient for re-export identity preservation, import-order independence, and duplicate detection.

The implementation MUST explicitly handle placeholders. It SHOULD avoid confusing a real empty struct declaration with a placeholder, or document and test the compatibility behavior if the current placeholder shape remains temporarily.

Imported summaries MUST register type identities before imported callables, workflow summaries, interface methods, or ordinary function signatures that mention those types are checked. This includes `PublicWorkflowSummary` users introduced by Phase 108: imported workflow summaries must see the ordinary type identities they mention before `TypeEnv::bind_public_workflow_summary`, `TypeEnv::lookup_public_workflow_summary`, or imported `do:Workflow` / `[...]: Workflow` composition checks run.

## 13. Compatibility and Migration

SPEC-057 supersedes the normal use of source-snippet ordinary type extraction from SPEC-030.

Allowed temporary state:

- A compatibility fallback may remain behind an explicitly named function, feature, or test helper.
- Any fallback MUST be documented and tested so it cannot be accidentally used for normal `ash check` / module-loading behavior.
- A diagnostic or assertion SHOULD reveal unexpected fallback use in normal module checking.

End state:

- ordinary type declarations are parsed through ModuleFile;
- engine export collection receives core/module summary data;
- TypeEnv consumes summaries;
- snippet extraction is removed or fenced as non-normative compatibility code.

## 14. Diagnostics

SPEC-057 requires diagnostics for at least:

- ordinary type declaration not represented in ModuleFile;
- duplicate canonical type identity;
- unresolved type identity in a public signature;
- private type leaked through a public summary;
- constructor or variant imported without representation visibility;
- missing summary for an imported module;
- source-snippet fallback used unexpectedly in the normal path;
- import-order-sensitive type identity conflict;
- placeholder upgrade conflict between a placeholder and a real declaration;
- re-export identity mismatch.

Diagnostics MUST include source/module context sufficient for users and future agents to find the owning declaration.

## 15. Crate Ownership

- `ash-parser`: owns surface declarations, spans, comments/source trivia as applicable, and parser diagnostics. It does not own semantic summaries.
- `ash-core`: owns canonical IDs, ordinary type declaration carriers that cross crate boundaries, and `ModuleSemanticSummary` or equivalent shared summary carriers.
- `ash-engine`: owns module loading, cache/transport, import/export plumbing, and CLI/API integration. It transports summaries but does not own type semantics.
- `ash-typeck`: owns registration, validation, visibility checking, TypeEnv consumption, and type diagnostics.
- `ash-interp`: does not depend on parser/typeck-private summary internals.
- `ash-cli` and LSP surfaces may display diagnostics and summary-derived information, but they do not own summary semantics.

## 16. Non-Interference

SPEC-057 MUST preserve SPEC-009 module paths, SPEC-012 import/re-export syntax, SPEC-020 ordinary ADT runtime semantics, SPEC-034 impl search, SPEC-035 simple associated type substitution, SPEC-054/SPEC-055 do/comprehension behavior, SPEC-056 workflow semantics, and capability/resource authority behavior.

For SPEC-056 specifically, ordinary-type summary work MUST preserve Phase 108 workflow-summary transport: `PublicWorkflowSummary`, `InlineCallable.workflow_summary`, `Workflow.imported_workflow_summaries`, `TypeEnv::bind_public_workflow_summary`, `TypeEnv::lookup_public_workflow_summary`, imported-summary origins, and TASK-777 import/export behavior. A SPEC-057 implementation may make workflow signatures depend on ordinary type identities from the new summary path, but it MUST NOT collapse workflow contract summaries into ordinary type summaries or drop workflow projection/contract metadata.

Any behavior change outside ordinary type metadata routing requires a separate task/spec amendment.

## 17. Acceptance Tests

### 17.1 Parser and ModuleFile

- A file containing `pub type Role = System | User;` parses as a ModuleFile with a type definition item.
- A file containing only ordinary type declarations parses as a module file.
- Inline module behavior for type declarations is supported and tested, or rejected with a targeted diagnostic until supported.
- Unknown-item recovery does not silently skip accepted ordinary type declarations.

### 17.2 Lowering and core summaries

- Surface ordinary type declarations lower to core type declarations and summary entries.
- Visibility, type parameters, alias/struct/enum body, variant payload shape, builtin marker, and spans are preserved.
- A core-owned summary structure carries public ordinary type metadata.

### 17.3 Type registration

- Sibling forward references register independent of declaration order.
- Self-recursive and generic ordinary type references register as before.
- Truly unbound type references produce diagnostics with module/file context.
- Duplicate non-placeholder type definitions remain errors.

### 17.4 Import/export identity

- A public type imported from another module resolves to the same canonical identity as the definition site.
- `pub use` preserves canonical identity.
- Private ordinary types cannot be imported downstream.
- Public signatures mentioning private ordinary types are rejected unless an existing explicit opaque/builtin exception applies.
- Constructors are imported/exposed only when representation visibility permits.

### 17.5 Snippet extraction containment

- Normal `ash check` and module loading obtain ordinary type declarations from ModuleFile/core summaries, not source-snippet scanning.
- Legacy snippet extraction is removed or fenced behind a named compatibility/test-only path.
- Tests fail if a normal top-level ordinary type declaration is discoverable only by snippet extraction and absent from ModuleFile.

### 17.6 Deferred feature rejection

- `type fn ...` remains rejected or unrecognized with a deferred-feature diagnostic.
- `sealed type domain ...` remains rejected or unrecognized with a deferred-feature diagnostic.
- No test relies on `Append<...>` normalization, type-level equation evaluation, or type-function inversion.
- Future type-function applications are not encoded as ordinary nominal constructors.

### 17.7 Non-regression

- Existing ADT tests continue to pass.
- Existing module/import tests continue to pass.
- Existing interface, associated type, capability/resource, workflow, do-notation, and comprehension tests continue to pass or have explicitly documented unrelated failures.
- Phase 108 TASK-777 workflow summary import/export tests continue to pass, including named import, glob import, and `pub use` paths for workflow-returning callables where those paths are supported.
- Adding ordinary-type module semantic summaries does not clear `InlineCallable.workflow_summary`, `Workflow.imported_workflow_summaries`, or TypeEnv public workflow-summary bindings.

## 18. Deferred Packet Map

| Topic | Owner packet |
|-------|--------------|
| Ordinary type declarations in ModuleFile | SPEC-057 / SPEC-A |
| Core semantic summary carrier for ordinary type metadata | SPEC-057 / SPEC-A |
| Type-expression IR, `TypeFnApp`, neutral forms, generalized projections, kind/arity substrate | SPEC-B |
| Sealed type-level domains and marker constructor sets | SPEC-C |
| Normalizer and definitional equality | SPEC-D |
| Direct structural `type fn` syntax and equation checking | SPEC-E |
| Cross-module export/import of computation-grade summaries | SPEC-F |
| Recursive/computable associated type families | SPEC-G |
| Constraint/proposition/disequality/proof layer | SPEC-H |

SPEC-A defines the carrier roadbed. SPEC-F later extends that roadbed with computation-grade public facts. SPEC-A MUST NOT smuggle SPEC-F behavior into ordinary type summaries.

## 19. Implementation Tasks

- [TASK-780](../plan/tasks/TASK-780-unified-type-module-pipeline-spec-plan-packet.md): Unified type/module pipeline spec/plan packet.
- [TASK-781](../plan/tasks/TASK-781-current-type-pipeline-audit-and-semantic-summary-gate.md): Current type pipeline audit and semantic-summary gate.
- [TASK-782](../plan/tasks/TASK-782-modulefile-ordinary-type-declaration-surface-integration.md): ModuleFile ordinary type declaration surface integration.
- [TASK-783](../plan/tasks/TASK-783-core-canonical-type-ids-and-module-semantic-summary-carriers.md): Core canonical type IDs and ModuleSemanticSummary carriers.
- [TASK-784](../plan/tasks/TASK-784-surface-to-core-type-metadata-lowering-and-source-anchors.md): Surface-to-core type metadata lowering and source anchors.
- [TASK-785](../plan/tasks/TASK-785-engine-summary-builder-and-export-collection-from-modulefile.md): Engine summary builder and export collection from ModuleFile.
- [TASK-786](../plan/tasks/TASK-786-import-pub-use-glob-visibility-and-opacity-summary-rules.md): Import, pub-use, glob, visibility, and opacity summary rules.
- [TASK-787](../plan/tasks/TASK-787-typeenv-two-pass-registration-from-semantic-summaries.md): TypeEnv two-pass registration from semantic summaries.
- [TASK-788](../plan/tasks/TASK-788-interface-and-associated-member-identity-summary-plumbing.md): Interface and associated-member identity summary plumbing.
- [TASK-789](../plan/tasks/TASK-789-legacy-type-snippet-scanner-quarantine-removal.md): Legacy type-snippet scanner quarantine/removal.
- [TASK-790](../plan/tasks/TASK-790-diagnostics-negative-tests-and-non-interference-coverage.md): Diagnostics, negative tests, and non-interference coverage.
- [TASK-791](../plan/tasks/TASK-791-spec-a-closeout-docs-examples-verification.md): SPEC-A closeout, docs, examples, and verification.

## 20. Changelog

### 2026-04-30

- Initial draft promoted from DESIGN-034 SPEC-A.
