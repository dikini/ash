# SPEC-062: Module-Summary Export/Import for Type Computation

**Status:** Implemented MVP
**Date:** 2026-05-07
**Promotes:** [DESIGN-034 §16.6](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)
**Builds on:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
**Related:** [SPEC-009](SPEC-009-MODULES.md), [SPEC-012](SPEC-012-IMPORTS.md), [SPEC-030](SPEC-030-MODULE-TYPE-RESOLUTION.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md)
**Plan:** [PLAN-110](../plan/PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
**Implementation Tasks:** [TASK-843](../plan/tasks/TASK-843-spec-f-spec-plan-packet.md) through [TASK-856](../plan/tasks/TASK-856-phase114-review-remediation.md)

## 1. Summary

> **SPEC-103 compatibility boundary:** Sections 5–12 remain the bounded implemented substrate for
> type-computation summary export/import. They may not authorize raw source scans, bare/path
> semantic identities, Engine-private export ownership, or a second import route in the complete
> module path; see [SPEC-103](SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md).

SPEC-062 is DESIGN-034 SPEC-F. It makes checked type computation coherent across module boundaries by exporting and importing public sealed-domain and public type-function semantic summaries through a shared `ash-core` module-summary contract.

The required end state is:

```text
source module with public sealed domains and public type functions
  -> parser preserves declarations/spans only
  -> typechecker validates public export closure and opacity
  -> ash-core ModuleSemanticSummary V3 carries public computation summaries
  -> ash-engine transports/reconciles summaries without owning semantics
  -> TypeEnv batch-registers imported public computation heads/equations
  -> normalizer reduces public type-function applications deterministically
```

SPEC-062 deliberately keeps definitional equality as normalize-and-compare. It does not add type-function inversion, proof search, associated recursive families, proposition solving, higher-kinded holes, or generalized type lambdas.

## 2. Motivation

SPEC-061 intentionally kept `type fn` declarations module-local and rejected public/cross-module computation-head leakage. That protected downstream modules from seeing unversioned private equations, but it also prevents useful public type-level libraries such as exported `Append`, `Map`, or `Normalize` over public sealed domains.

DESIGN-034 §16.6 requires the public boundary to become an explicit module-summary contract rather than another engine-private metadata path. Public reductions must be reproducible from imported summaries; private helper equations must remain opaque; import order must not change normal forms.

## 3. Live Baseline

The live post-SPEC-061 substrate is:

- `ash-core::semantic_summary::ModuleSemanticSummary` carries ordinary public types, constructors, interface/member identities, and `exported_sealed_domains`, but has no exported type-function summary field.
- `ash-core::type_ir` already owns checked `TypeFunctionDef`, `TypeFunctionEquation`, `TypeFunctionPattern`, `TypeFunctionResultExpr`, and `TypeComputationHeadId` carriers with serde support.
- `ash-parser::surface::TypeFnDef` preserves raw `type fn` syntax/spans, but `pub type fn` is currently rejected as a SPEC-F handoff boundary.
- `ash-typeck::TypeEnv` stores `local_type_function_heads` and `local_type_functions`; the normalizer looks up only local source-backed definitions.
- `ash-engine::module_loader::ModuleExports` remains engine-private transport. It carries a core semantic summary, but summary selection/dedup logic is ordinary-type-biased and does not include type-function dimensions.
- Imported summaries are registered sequentially. Existing sealed-domain validation may depend on import order for cross-summary references.

## 4. Scope

In scope:

- public `pub type fn` declarations over public sealed-domain substrates;
- a core-owned public computation-summary schema and version bump;
- public transparent equation export for export-closed `pub type fn` definitions;
- canonical public computation-head identity import/export using `TypeComputationHeadId`;
- dependency closure for public sealed domains, marker constructors, ordinary type identities, public helper type functions, and public projection identities referenced by exported equations;
- batch/two-pass imported-summary registration for import-order independence;
- normalizer lookup over local and imported public type-function summaries;
- private equation opacity and diagnostics for unavailable private reductions;
- reconciliation of fragmented export carriers so type-computation summary semantics live in `ash-core`, not engine-private `ModuleExports` or parser/capability metadata;
- summary versioning, unsupported-version rejection, and cache/dedup invalidation inputs;
- acceptance tests for downstream public reduction, private opacity, import-order independence, and stable opaque neutral results.

Out of scope:

- associated recursive type-family computation (SPEC-G);
- proposition/equality constraint solving (SPEC-H);
- type-function inversion, injectivity, disequality solving, or proof search;
- exporting private equations, private marker constructors, or private ordinary type representations;
- public header-only type-function declarations as a user-facing feature;
- opaque normalized fact export as a separate fact language;
- persistent on-disk summary caches beyond specifying invalidation keys;
- changes to runtime ADT construction, pattern matching, exhaustiveness, Act/Proc/Workflow semantics, or capability authority.

## 5. Normative Export Policy

SPEC-062 chooses **direct checked public equation export** for the MVP.

A `pub type fn` is transparent across modules if and only if its public summary is export-closed:

1. its parameter and return type identities are public-summary-visible;
2. all sealed-domain scrutinees are public domains;
3. every pattern constructor in an exported equation is a public marker constructor of a public sealed domain;
4. every result expression mentions only public ordinary type identities, public sealed-domain marker constructors, public projection identities already legal to import, and public type-function heads whose summaries are also exportable;
5. recursive calls to the same public head remain governed by SPEC-061 structural recursion validation;
6. no equation result depends on a private helper type function or private sealed-domain/constructor identity.

If any of these conditions fails, exporting the type function is rejected. The implementation must not silently downgrade such a definition to a public opaque fact, because that would make source-level behavior depend on a hidden export policy.

Opaque/stable downstream results still exist as normalizer outcomes: when an imported public head cannot reduce because an argument is abstract, neutral, fuel-limited, or intentionally blocked by the public summary boundary, the result is a stable `NormalTypeExpr::NeutralComputationApp` keyed by the canonical imported `TypeComputationHeadId` with a precise blocker reason.

Future specs may add explicit header-only/opaque public type functions or opaque normalized fact export. SPEC-062 reserves that design space but does not expose it.

## 6. Summary Schema

### 6.1 Versioning

`ash-core::semantic_summary::SummaryVersion` must add a new V3 value for SPEC-062 summaries. Backward compatibility rules:

- V1 ordinary-type summaries remain accepted for ordinary type metadata only.
- V2 sealed-domain summaries remain accepted and interpreted as carrying no type-function summaries.
- V3 summaries may carry public type-computation data.
- A V1 or V2 summary that contains a non-empty `exported_type_functions` field is malformed and must be rejected before any partial registration. Implementations must not silently ignore those facts and must not consume them under an older version tag.
- A V3 summary may have an empty `exported_type_functions` field when a module has no public computation exports.
- Unknown future versions are rejected with a diagnostic naming the unsupported summary version and module identity before any partial registration.

### 6.2 Public type-function summary

`ModuleSemanticSummary` must gain a serde-defaulted public type-function field, conceptually:

```rust
pub exported_type_functions: Vec<TypeFunctionSummary>
```

`TypeFunctionSummary` is core-owned and must preserve enough checked metadata for importers either to revalidate the SPEC-061 invariants listed in §8 or to reject the summary before use. It must preserve:

- exported name and canonical `TypeComputationHeadId`;
- visibility (`Public` only for exported summaries);
- parameter names, canonical parameter type expressions, kinds, and sealed-domain constraints;
- canonical return type, return kind, and result-domain constraint;
- public transparency mode (`TransparentEquations` for SPEC-062 MVP);
- source anchors for diagnostics;
- checked source-order public equations when transparent;
- dependency summaries or references sufficient to validate import closure.

Implementations may reuse `TypeFunctionDef` internally, but the public summary type must make export mode and public-closure guarantees explicit. It must not require importers to trust engine-private filtering.

### 6.3 Dependency closure

A selected public type-function import must transport the public closure required to validate and reduce it:

- the type-function summary itself;
- public sealed-domain summaries referenced by params, result constraints, patterns, and RHS marker constructors;
- public ordinary type summaries referenced by canonical nominal applications;
- public type-function summaries referenced by RHS computation-head applications;
- public interface/member identities needed by canonical projections;
- dependency summary refs and version/digest metadata sufficient for in-memory dedup now and future persistent-cache invalidation.

Private dependencies are not transported. A public definition that requires private dependencies is not export-closed and must fail at export validation.

## 7. Import and Re-Export Semantics

1. Named imports, glob imports, and `pub use` re-exports preserve canonical origin IDs. Aliases affect visible names only.
2. Importing a public type function must make its canonical head available to type resolution and normalizer lookup under the selected visible name.
3. Dependency-closure helper heads required to reduce the selected public function may be registered as **normalizer-available** canonical heads without becoming **source-visible** names. They become source-visible only when explicitly selected, glob-imported, or re-exported.
4. Importing a public type function must not expose unrelated sibling public type functions, and must not expose dependency helper heads as source-visible names unless the import selection itself makes them visible.
5. Re-exported type functions retain their original `TypeComputationHeadId`; re-exporting does not mint a new computation identity.
6. Re-export summaries must not duplicate facts in a way that changes first-match equation order. Equation order is the defining module's checked source order.
7. Repeated imports of the same public computation summary are idempotent.

## 8. TypeEnv Consumption

Imported computation summaries are registered through a new batch API, conceptually `TypeEnv::register_module_semantic_summaries(...)`, not by repeatedly calling the existing one-summary path. The batch/two-pass process covers every imported summary identity class, not only type-function heads:

1. reject unsupported summary versions and collect duplicate/conflicting canonical identities;
2. declare all ordinary type identities, sealed-domain identities, interface/member identities, and type-computation heads from all summaries before validating any cross-summary references;
3. validate and register sealed-domain summaries after all referenced public domains are declared;
4. validate and register public type-function signatures/headers;
5. revalidate imported public computation summaries before registration. At minimum, validate signature kind/domain validity, equation arity/head consistency, pattern linearity/domain correctness, coverage/overlap/order validity, SPEC-061 structural-recursion/decreases obligations, result kind/domain conformance, and public dependency closure. A future trusted-summary model may replace revalidation only if it specifies digest/provenance guarantees strong enough to prove these invariants;
6. register transparent public equations with the normalizer table only after the revalidation above succeeds;
7. expose selected aliases/visible names without changing canonical IDs, while keeping dependency-closure-only heads normalizer-available but not source-visible.

This process must be import-order independent. A downstream module must observe the same normal forms for a set of imported public summaries regardless of textual import order. Implementations must add regressions where two summaries reference each other through public sealed-domain fields and public type-function equations; those tests must fail if summaries are registered one-at-a-time in source import order.

## 9. Normalization and Equality

The normalizer must consult a unified computation-head lookup path covering:

- current-module checked local definitions;
- imported public transparent type-function summaries;
- compiler-internal fixture tables retained for tests.

Reduction is permitted only when the matched equation is public/available in the current environment. In ordinary MVP source code, private or non-export-closed definitions are rejected before export. If an imported/stale/future opaque public application reaches a head whose equations are unavailable at the current boundary, the normalizer returns a neutral computation app or emits the SPEC-062 unavailable-reduction diagnostic at a boundary that requires reduction.

Definitional equality remains normalize-and-compare over normal forms. SPEC-062 does not add inversion or solving under type-function heads.

## 10. Engine and Export-Carrier Reconciliation

Type-computation export/import semantics must be expressed in `ash-core` summary carriers. `ash-engine` owns only parsing, loading, dependency traversal, transport, caching, and errors.

Implementation must reconcile the current fragmented carriers:

- `ash-engine::module_loader::ModuleExports` may cache/transport summaries, but must not be the normative home for type-computation facts;
- parser capability/module export metadata must not gain semantic type-computation facts;
- core `ModuleGraph`/module identities must remain the canonical source of module identity where available;
- summary dedup/reconciliation keys must include sealed-domain and type-function dimensions, not only ordinary type/constructor fields.

## 11. Cache and Invalidation Rules

Any cache or dedup key used for public computation summaries must distinguish old-version summaries from computation-aware V3 summaries. A V1/V2 summary with non-empty computation facts is rejected rather than keyed. Any cache or dedup key used for accepted public computation summaries must include:

- summary schema version;
- module identity and source/module path metadata;
- public type-computation summary content, including equations or mode;
- public sealed-domain summaries used by exported equations;
- imported dependency summary refs/digests where available;
- compiler/type-computation algorithm version when persistent caches are introduced.

The MVP may keep caches in memory, but the summary keys must still be rich enough that selected imports with different computation facts cannot collapse to the same imported-summary key.

## 12. Diagnostics

SPEC-062 introduces these diagnostic families:

- `TypeComputationSummaryUnsupportedVersion` — imported summary version is unsupported.
- `TypeComputationSummaryMalformed` — imported summary content is incompatible with its version or fails import-side SPEC-061 invariant revalidation.
- `TypeFunctionExportPrivateDependency` — a public type function depends on a private type, domain, constructor, projection identity, or type-function head.
- `TypeFunctionExportNotClosed` — public equation closure cannot be proven from public summaries.
- `TypeFunctionImportOrderConflict` — duplicate/conflicting public summary identities or aliases would make imports non-deterministic.
- `TypeFunctionPrivateReductionUnavailable` — downstream code requires a reduction whose equation is unavailable at the current boundary. In the SPEC-062 MVP, ordinary source definitions with private dependencies are rejected during export validation; this diagnostic is reserved for stale/malformed summaries, pre-SPEC-F leaked heads, future opaque public headers/facts, or diagnostic fallback paths where an unavailable reduction reaches a forcing boundary.
- `TypeFunctionSummaryOpaqueNeutral` — diagnostic note/hint for a stable neutral public computation app where reduction is intentionally blocked.

Diagnostics must include the public head name, module identity/path, source anchor where available, and the smallest user action: export the dependency publicly, rewrite the public type function to be export-closed, or keep the computation module-local.

## 13. Acceptance Tests

Required acceptance matrix:

1. Downstream module imports a public sealed-domain and public transparent `pub type fn`; closed applications normalize using only public summaries.
2. Downstream module imports the same summaries in different textual orders; normal forms and diagnostics are identical.
3. Public type function with RHS depending on a private helper type function is rejected at export validation.
4. Public type function with RHS constructing a private marker constructor is rejected at export validation.
5. Named import of one public type function imports only that selected head as source-visible; dependency-closure helper heads may be normalizer-available but unrelated sibling heads and helper aliases do not leak as source-visible names.
6. Glob import imports all public computation heads and their public dependency closures deterministically.
7. `pub use` re-export preserves canonical `TypeComputationHeadId` and equation order.
8. Imported public computation over abstract arguments remains a stable neutral `NormalTypeExpr::NeutralComputationApp` keyed by the imported head.
9. Unknown or future summary versions are rejected before partial registration, and V1/V2 summaries with non-empty computation fields are rejected rather than consumed or ignored.
10. Imported public computation summaries cannot bypass SPEC-061 validation invariants: malformed summaries with bad arity, domain/kind mismatch, overlap/coverage failure, or non-decreasing recursion are rejected before normalizer registration.
11. Existing SPEC-057 ordinary type summaries, SPEC-059 sealed-domain summaries, SPEC-060 normalizer fixture tests, and SPEC-061 module-local type functions remain non-regressed.

TASK-854 must produce a row-by-row acceptance artifact mapping every item above to focused test suites or recorded evidence.

## 14. Non-Goals and Handoff

SPEC-062 completes the public module-summary boundary for direct structural type functions. SPEC-G owns associated recursive type-family computation. SPEC-H owns proposition/constraint solving. Later cache specs may define persistent serialized summary stores; SPEC-062 only defines the semantic data and invalidation inputs those caches must include.
