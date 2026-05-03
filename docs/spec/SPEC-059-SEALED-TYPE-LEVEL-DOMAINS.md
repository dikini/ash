# SPEC-059: Sealed Type-Level Domains

**Status:** Draft
**Date:** 2026-05-03
**Promotes:** [DESIGN-034 §16.3](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Builds on:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
**Related:** [SPEC-020](SPEC-020-ALGEBRAIC-DATA-TYPES.md), [SPEC-030](SPEC-030-MODULE-TYPE-RESOLUTION.md), [SPEC-042](SPEC-042-ASH-SOURCE-FORMATTER.md)
**Plan:** [PLAN-107](../plan/PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
**Implementation Tasks:** [TASK-806](../plan/tasks/TASK-806-spec-c-spec-plan-packet.md) through [TASK-815](../plan/tasks/TASK-815-phase111-review-remediation.md)

## 1. Summary

SPEC-059 is DESIGN-034 SPEC-C. It defines the first normative Ash contract for closed, compiler-known sets of type-level marker constructors.

The packet exists to answer one question that Phase 110 intentionally deferred: when later packets talk about structural recursion, coverage checking, or type-level constructor matching, what is the closed constructor set they are allowed to inspect?

The required end state is:

```text
sealed type domain surface declarations
  -> core-owned sealed-domain and marker-constructor identities
  -> public semantic-summary transport for visible domain constructor sets
  -> TypeEnv registration and validation of imported/local domain metadata
  -> future coverage / equality / structural-recursion consumers keyed by domain identity
```

In this packet, sealed domains are metadata and validation only. They do not create runtime value constructors, do not promote existing ADT variants, do not expose public `type fn` syntax, and do not implement normalization or type-level pattern evaluation.

## 2. Motivation

[SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) created the canonical type-expression IR, shared `Kind`, and projection identities required for honest later type computation. It explicitly deferred sealed domains.

Without sealed domains, later packets still lack the substrate needed to say:

1. which constructors belong to a closed type-level datatype such as `TypeList`;
2. which fields are structurally recursive versus merely ordinary `Type`-kind parameters;
3. which constructor sets may cross module boundaries as public computation metadata;
4. which imported constructor names are valid inputs to future coverage and equality consumers;
5. how private constructor sets stay private across import/export.

The current live code confirms that this gap is real:

- `ash-parser` and `ash-parser::lower` currently lower only ordinary `surface::Definition::Type` metadata;
- `ash-core::semantic_summary::ModuleSemanticSummary` and `SummaryVersion::SPEC057_ORDINARY_TYPE_V1` currently model only ordinary type and runtime-constructor exports;
- `ash-engine::module_loader::collect_module_type_metadata_from_module_file(...)` transports only ordinary type metadata;
- `ash-typeck::TypeEnv::validate_summary_visibility_and_duplicates(...)` accepts only the ordinary-type summary version and ordinary type/constructor contracts.

SPEC-059 closes that substrate gap without pretending that public structural `type fn`, normalization, or type-level coverage engines already exist.

## 3. Scope

In scope for SPEC-059:

- explicit sealed `type domain` declarations as the first user-facing carrier for closed type-level constructor sets;
- dedicated canonical identities for sealed domains and type-level marker constructors;
- constructor field metadata carrying field name, `Kind`, optional domain constraint, and structural status;
- conservative recursive-domain validation rules suitable for future structural recursion checking;
- public semantic-summary transport of sealed-domain metadata across modules;
- TypeEnv registration and validation of local/imported sealed-domain metadata;
- explicit visibility and privacy rules for domain metadata;
- diagnostics for malformed field kinds/domains, duplicate constructor sets, unsupported shapes, and privacy leaks.

Out of scope:

- promoted value/data constructors or full promoted data kinds;
- public `type fn` syntax, equation tables, reduction rules, overlap checking, or totality proofs;
- definitional equality, normalization, forcing-point rollout, or comparison under neutral computation heads;
- runtime value inhabitants or term-level construction for marker constructors;
- type-level pattern syntax or coverage/exhaustiveness rollout in user-facing language surfaces;
- constructor-only imports, standalone constructor re-exports, or a separate public constructor namespace for sealed-domain markers;
- generic sealed domains, partial type-constructor application, holes, or new public kind-binder syntax;
- mutually recursive local sealed-domain strongly connected components.

## 4. Live Baseline and Boundary Audit

SPEC-059 is intentionally written against the current repository boundary rather than an abstract future compiler.

### 4.1 Parser baseline

At the start of this packet, ordinary metadata lowering is anchored on `surface::Definition::Type` and `lower_module_type_metadata(...)`. No sealed-domain declaration carrier exists yet.

### 4.2 Core baseline

`ash-core::semantic_summary` already owns canonical module identities, ordinary `TypeDeclId`, ordinary `ConstructorId`, and the reserved `sealed_domains` identity slot list. That reserved slot is not yet a semantic contract.

### 4.3 Engine baseline

`ash-engine` currently parses module files for ordinary type metadata and explicitly rejects inline-module ordinary type lowering in the current path. Sealed domains must start from the same honest boundary rather than silently widening inline-module support.

### 4.4 Typechecker baseline

`ash-typeck::TypeEnv` currently validates and registers only ordinary-type public semantic summaries and rejects unknown summary versions. Sealed domains therefore require an explicit summary-version and validation contract.

## 5. Sealed Domain Surface Contract

### 5.1 Declaration form

The first-slice source declaration is:

```ash
sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}
```

A public domain may be declared with the existing type-level visibility prefix:

```ash
pub sealed type domain TypeList {
    Nil;
    Cons<head: Type, tail: TypeList>;
}
```

The surface grammar is intentionally narrow:

```text
sealed-domain-decl   = visibility? "sealed" "type" "domain" ident "{" domain-ctor* "}"
domain-ctor          = ident ctor-fields? ";"
ctor-fields          = "<" domain-field ("," domain-field)* ">"
domain-field         = ident ":" domain-slot
domain-slot          = "Type" | domain-ref
```

`domain-ref` is a resolved sealed-domain name/path, not an arbitrary type expression.

### 5.2 Meaning of the declaration

A sealed-domain declaration introduces a closed set of type-level marker constructors owned by the declared domain. It does not register existing ADT variants, does not alias ordinary nominal types, and does not create runtime constructors.

For the example above:

- `TypeList` is the closed domain identity;
- `Nil` and `Cons` are marker constructors belonging to `TypeList`;
- `head` is an unconstrained `Type`-kind slot;
- `tail` is a `Type`-kind slot constrained to the sealed domain `TypeList` and is structurally recursive.

### 5.3 Visibility

The first slice supports domain visibility only.

Rules:

1. the domain declaration may be `pub`, `pub(crate)`, or private according to the existing visibility lattice;
2. every constructor in the domain inherits the domain's visibility;
3. explicit per-constructor visibility syntax is deferred;
4. a non-public domain exports no public constructor metadata.

This inherited-visibility rule is the normative meaning of “private domain constructors” in this packet.

### 5.4 Rejection boundary

The parser/type surface for SPEC-059 must reject, rather than silently reinterpret:

- inline-module `sealed type domain` declarations;
- explicit per-constructor visibility modifiers;
- arbitrary type expressions in constructor field slots;
- tuple-style or unnamed constructor fields;
- generic domain parameters or higher-kinded domain heads;
- syntax that looks like promoted ADT/data kinds rather than marker-domain declarations.

## 6. Domain and Constructor Identity Model

### 6.1 Dedicated identity carriers

Sealed domains and marker constructors must use dedicated computation-grade identity carriers owned by `ash-core`.

At minimum the core contract requires identities equivalent to:

```text
SealedDomainId {
  module: ModuleIdentity,
  name: Name,
}

DomainConstructorId {
  domain: SealedDomainId,
  name: Name,
}
```

The exact Rust names may vary, but the contract does not.

### 6.2 Separation from ordinary runtime identities

Sealed-domain identities must not be represented as ordinary `TypeDeclId` / `ConstructorId` in a way that makes marker constructors appear to be ordinary runtime declarations.

That separation is required because:

- ordinary `ConstructorId` currently denotes runtime constructors/variants derived from an ordinary parent type;
- marker constructors are type-level only in this packet;
- promoted data kinds are explicitly deferred.

### 6.3 Namespace rules

The first slice uses two namespaces:

1. domain names participate in the module-level type/domain declaration namespace and therefore must not collide with other domain names or ordinary type declarations in the same module;
2. constructor names are scoped by their parent domain identity.

Future packets may expose constructor references more directly, but in SPEC-059 the canonical matching key is `(domain identity, constructor local name)`, not the local name alone.

## 7. Constructor Field Model

### 7.1 Field summary contract

Every marker-constructor field must lower to metadata equivalent to:

```text
DomainFieldSummary {
  name: Name,
  kind: Kind,
  domain_constraint: Option<SealedDomainId>,
  structural_status: StructuralFieldStatus,
}
```

`kind` is always explicit semantic metadata even when the written slot annotation is simple.

### 7.2 Field slot classes

The first slice allows exactly two field-slot classes:

1. `Type`
   - semantic meaning: an unconstrained `Kind::Type` slot;
   - `domain_constraint = None`;
   - `structural_status = NonStructural`.
2. `SomeDomain`
   - semantic meaning: a `Kind::Type` slot constrained to a resolved sealed-domain identity;
   - `domain_constraint = Some(resolved_domain_id)`;
   - `structural_status` determined by §7.3.

No other field-slot forms are valid in SPEC-059.

### 7.3 Structural status

`StructuralFieldStatus` is derived, not user-controlled, in the first slice.

Rules:

1. if `domain_constraint` resolves to the enclosing domain identity, the field is `StructuralSelfDomain`;
2. otherwise the field is `NonStructural`.

### 7.4 Recursive-domain restrictions

SPEC-059 chooses a conservative first-slice rule set:

1. a constructor may have at most one `StructuralSelfDomain` field;
2. self-recursive use of the enclosing domain is allowed only through a field whose slot annotation names that enclosing domain directly;
3. self-recursive use of the enclosing domain through an unconstrained `Type` slot is invalid;
4. mutual recursion between two or more local sealed domains is deferred and must be rejected;
5. nested or disguised recursion through ordinary nominal types is not structural in this packet and must not be accepted as if it were.

These rules intentionally bias toward a substrate that is easy to validate and safe for later structural recursion work.

## 8. Core Semantic Summary and Metadata Contract

### 8.1 Public summary carriers

`ash-core` must extend the semantic-summary contract with public carriers for sealed domains and their constructors.

At minimum the module summary contract requires fields equivalent to:

```text
ModuleSemanticSummary {
  ...existing Phase 110 fields...,
  exported_sealed_domains: Vec<SealedDomainSummary>,
}

SealedDomainSummary {
  id: SealedDomainId,
  exported_name: Name,
  visibility: Visibility,
  constructors: Vec<DomainConstructorSummary>,
  anchor: SourceAnchor,
}

DomainConstructorSummary {
  id: DomainConstructorId,
  exported_name: Name,
  visibility: Visibility,
  fields: Vec<DomainFieldSummary>,
  anchor: SourceAnchor,
}
```

Constructors are nested beneath their parent domain summary in this first slice. Constructor-only export/import surfaces are deferred.

### 8.2 Summary versioning

This packet requires a new summary version contract beyond `SummaryVersion::SPEC057_ORDINARY_TYPE_V1`.

Normative rules:

1. a module summary containing any sealed-domain metadata must use a SPEC-059-era summary version;
2. import consumers must accept both the older ordinary-only version and the new sealed-domain version during migration;
3. the sealed-domain version must preserve all Phase 110 ordinary-type semantics rather than replacing them with a separate transport path.

### 8.3 Visibility and leak prevention

A public summary must not expose:

- a private domain;
- the constructor set of a non-public domain;
- field domain references to non-visible domains.

Any such leak is an invalid-definition error at summary validation time.

## 9. Parser and Lowering Contract

### 9.1 Surface carriers

`ash-parser` must add a dedicated surface carrier for sealed-domain declarations rather than encoding them as ordinary `TypeDef` values.

The exact Rust type name may vary, but the surface layer must distinguish:

- ordinary `type` declarations;
- sealed `type domain` declarations.

### 9.2 Parser parity

`parse_type_def.rs` and `parse_module.rs` must remain aligned on the supported subset and explicit rejection boundary.

TASK ownership rule for this packet:

- the parser surface task owns acceptance and explicit-rejection evidence for the SPEC-059 subset;
- later tasks may cite that suite but must not create a second parser-boundary owner.

### 9.3 Lowering

The ordinary `lower_module_type_metadata(...)` contract must be widened, or a clearly named successor introduced, so that module lowering yields both:

1. ordinary type metadata from SPEC-057 / SPEC-058; and
2. sealed-domain metadata from SPEC-059.

Lowering requirements:

- module identities remain module-anchored and canonical;
- domain identities are derived from the resolved module identity plus declared name;
- constructor identities are derived from parent domain identity plus local constructor name;
- source anchors exist for the domain and each constructor;
- field metadata resolves `Type` versus domain-constrained slots explicitly.

### 9.4 Inline-module boundary

Inline-module sealed-domain declarations remain out of scope for this packet and must be rejected explicitly by the current engine/parser integration path.

## 10. Engine Transport Contract

`ash-engine` must extend its current module-file metadata collection path to transport public sealed-domain summaries together with the existing ordinary-type summaries.

Required behavior:

1. file-module sealed-domain metadata is parsed and lowered through the authoritative module-file path;
2. imported public sealed-domain summaries are stored and re-registered through the same imported-summary lifecycle used by Phase 109/110 summary transport, or a clearly named sibling path with equivalent guarantees;
3. visible aliases may change display names, but not origin identities;
4. private domains and constructor sets must not leak through import/export or re-export transport;
5. engine transport must not fabricate runtime constructors, value-level types, or pattern metadata for sealed-domain markers.

## 11. TypeEnv Registration and Validation Contract

`ash-typeck::TypeEnv` must gain explicit registration and validation rules for sealed-domain metadata.

Validation requirements:

1. reject unsupported summary versions clearly;
2. reject duplicate exported domain names with conflicting identities;
3. reject the same domain identity appearing under incompatible visible contracts;
4. reject duplicate constructor names within the same domain;
5. reject constructor summaries whose parent-domain identity does not match the enclosing domain summary;
6. reject field domain references to missing or non-visible domains;
7. reject non-public domain summaries presented as public import metadata.

Registration requirements:

- imported domain identities and constructor sets must be available before dependent typechecking begins;
- visible aliases must not create new origin identities;
- registration must preserve enough source/anchor information for diagnostics.

## 12. Typechecker Semantic Contract

SPEC-059 requires typechecker-side semantic validation even before public `type fn` exists.

Mandatory checks:

1. every field slot annotation resolves either to `Type` or a visible sealed domain;
2. field kinds are explicit and match the allowed first-slice slot classes;
3. structural status is computed according to §7.3 rather than guessed later;
4. direct self recursion through unconstrained `Type` slots is rejected;
5. more than one structural self-domain field per constructor is rejected;
6. mutually recursive local domain SCCs are rejected;
7. unrelated ordinary nominal constructors are never registered as members of a sealed domain.

This packet does not require normalization, unification over domain constructors, or public type-level pattern checking.

## 13. Coverage and Equality Exposure Contract

Although the public consumers land later, SPEC-059 fixes the metadata contract they must consume.

Future coverage/equality/structural-recursion consumers must treat a sealed domain as:

- a closed set keyed by `SealedDomainId`;
- containing exactly the constructor summaries nested in that domain's validated public/local metadata;
- excluding ordinary ADT constructors, constructors from other domains, and private/import-hidden constructors.

Therefore:

1. `TypeList` exposes exactly `Nil` and `Cons` to future coverage consumers;
2. an ordinary enum variant named `Cons` is not a `TypeList` constructor;
3. a private domain's constructor set is not part of another module's public equality/coverage surface.

## 14. Diagnostics

SPEC-059 requires domain-aware diagnostics for at least the following failures:

- duplicate sealed domain declaration in the same visible namespace;
- duplicate constructor name within a sealed domain;
- unsupported sealed-domain syntax shape in a parser path that otherwise accepts the packet subset;
- unresolved field domain reference;
- malformed field kind/domain slot;
- direct recursive use of the enclosing domain through a non-domain-constrained slot;
- more than one structural self-domain field in a constructor;
- mutually recursive local domains unsupported in the first slice;
- imported/public summary leak of a non-public domain or constructor set;
- mismatched constructor/domain identity inside imported summary metadata.

Diagnostics should name the domain and constructor involved whenever available.

## 15. Required Invariants

1. Sealed domains are closed constructor sets, not ordinary ADTs.
2. Marker constructors are type-level metadata only in this packet.
3. Domain and constructor matching keys are canonical identities, not visible names alone.
4. Constructor visibility inherits the containing domain visibility in the first slice.
5. Field domain constraints are explicit semantic metadata, not recovered heuristically later.
6. Structural self-recursion is conservative: at most one self-domain field per constructor.
7. Mutual recursive domain SCCs remain rejected.
8. Public semantic summaries never leak private domain constructor sets.
9. Phase 110 ordinary type-expression IR, projection identities, and ordinary summary transport remain intact.
10. SPEC-059 does not implement promoted data kinds, normalization, public `type fn`, runtime inhabitants, or pattern/exhaustiveness rollout.

## 16. Out of Scope and Deferred Follow-Ups

Deferred to later packets:

- promoted data kinds or reusing ADT runtime constructors as type-level constructors;
- normalization and definitional equality over domain constructors;
- user-facing type-level pattern matching and structural `type fn` equations;
- constructor-only imports or re-exports;
- runtime value inhabitants for marker constructors;
- mutual recursive domain SCCs and richer positivity analysis;
- generalized kinded field annotations beyond `Type` or visible sealed-domain names.

## 17. Acceptance Criteria

SPEC-059 is satisfied when the implementation can demonstrate all of the following:

1. file-module `sealed type domain` declarations lower to canonical domain and constructor identities with source anchors;
2. `ash-core` exposes public summary carriers for sealed domains and their constructor sets;
3. `ash-engine` transports visible sealed-domain summaries across module boundaries without leaking private domains;
4. `ash-typeck` registers imported/local domain summaries and validates domain/constructor/field consistency;
5. `TypeList` exposes exactly `Nil` and `Cons` to future coverage/equality consumers keyed by domain identity;
6. unrelated nominal constructors cannot be matched as `TypeList` constructors;
7. malformed field domains/kinds are rejected with domain-aware diagnostics;
8. inline-module sealed-domain declarations remain explicitly unsupported rather than silently accepted;
9. Phase 109/110 ordinary type, interface, workflow, capability, resource, `do`, and comprehension behavior remains non-regressed.

## 18. Implementation Tasks

- [TASK-806](../plan/tasks/TASK-806-spec-c-spec-plan-packet.md) — create the SPEC-C docs/planning packet.
- [TASK-807](../plan/tasks/TASK-807-sealed-domain-audit-gate.md) — audit the live parser/core/engine/typeck boundary before code changes.
- [TASK-808](../plan/tasks/TASK-808-parser-surface-for-sealed-type-domains.md) — land the restricted parser surface and ModuleFile carriers.
- [TASK-809](../plan/tasks/TASK-809-core-domain-kind-ids-and-summary-carriers.md) — add core domain identities, kinds, and summary carriers.
- [TASK-810](../plan/tasks/TASK-810-domain-lowering-and-summary-versioning.md) — lower sealed-domain declarations into versioned core metadata.
- [TASK-811](../plan/tasks/TASK-811-engine-domain-summary-export-import.md) — transport public domain summaries through engine import/export paths.
- [TASK-812](../plan/tasks/TASK-812-typeenv-domain-registration-and-validation.md) — register and validate imported/local domain summaries.
- [TASK-813](../plan/tasks/TASK-813-sealed-domain-diagnostics-and-non-interference.md) — add diagnostics and non-interference coverage.
- [TASK-814](../plan/tasks/TASK-814-spec-c-closeout-docs-and-verification.md) — reconcile docs/status/changelog and record verification evidence.
- [TASK-815](../plan/tasks/TASK-815-phase111-review-remediation.md) — remediate post-closeout review findings.
