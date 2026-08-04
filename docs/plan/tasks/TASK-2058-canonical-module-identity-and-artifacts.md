# TASK-2058: Canonical Module Identity and Artifacts

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§2, 5, 7-8
**Owned rule:** MOD-REAL-001
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2058 canonical module identity and artifacts](../SEMANTIC-RULE-COVERAGE.md#task-2058-canonical-module-identity-and-artifacts)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Resolver graph construction does not yet consume ModuleKey/ModuleArtifact; legacy semantic_summary::ModuleIdentity remains unchanged; source-kind-independent module units and file/inline parity; identity-preserving aliases/re-exports, checked export-closed interfaces, interface-driven imports and visibility; module-aware Core/CPS lowering; linked Engine admission; source-anchored ModuleNotFound and CircularDependency diagnostics; and CLI/daemon terminal parity.
**Layers:** type `partial`; Core `partial`; CPS/admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-001-CANONICAL-KEY` and `TEST-MOD-REAL-001-KEY-GRAMMAR-PARITY`; negative `TEST-MOD-REAL-001-DUPLICATE-ORIGIN`; mutation `TEST-MOD-REAL-001-CACHE-KEY-FORGERY`; parity `not_applicable`.
**Next obligation:** TASK-2059 must consume ModuleKey/ModuleArtifact for source-kind-independent module units and structural diagnostics; TASK-2060/2061 consume stable identities for interfaces/imports; TASK-2062/2063 consume checked artifacts for lowering/admission; TASK-2064 owns conformance/parity; TASK-2065 closes the phase.

## Task-owned evidence

**Canonical traceability rule:** `SEM-MODULE-REALIZATION-001`, the traceability alias for
`MOD-REAL-001` in SPEC-103.

`ash_core::module_graph` now publishes `ModuleKey`, `ModuleArtifactOrigin`, `ModuleArtifact`, and
`MODULE_ARTIFACT_SCHEMA_VERSION`. A key is crate-qualified, independent of file layout and source
origin, displayable, serializable, and cache-keyable. The artifact records a file or inline origin,
rejects invalid parents/origins/duplicate child keys, stores children in key order, and rejects
unknown, malformed, or unsupported wire values. Its source fingerprint is
`sha256:8f33f5195cccc26d9b6b80e8e076750a76e5bd454fab2c5c26ebc6a94012eef8`.

- Positive: `TEST-MOD-REAL-001-CANONICAL-KEY` covers crate-qualified, layout-independent keys;
  crate-root domain; parent/serde/cache-key round trips; and schema/origin round trips.
- Positive: `TEST-MOD-REAL-001-KEY-GRAMMAR-PARITY` is the parser-to-Core guard for every
  canonical reserved keyword plus representative parser-accepted child names. It does not create
  or migrate a resolver graph edge.
- Negative: `TEST-MOD-REAL-001-DUPLICATE-ORIGIN` rejects mismatched structural parents,
  non-direct and duplicate children, mismatched inline parents, malformed key segments, and
  unsupported schema versions.
- Mutation: `TEST-MOD-REAL-001-CACHE-KEY-FORGERY` rejects forged unknown top-level and nested
  wire fields, so a serialized key or artifact cannot carry a substituted cache-key field or
  source payload.
- Parity: not_applicable. This non-authorizing Core carrier has no paired execution relation.

The carrier is deliberately not wired into resolver graph construction or
`semantic_summary::ModuleIdentity`; it neither supplies a common file/inline module route nor
authorizes interfaces, imports, lowering, admission, execution, or client parity.

## Description

Provide a Core-owned, crate-qualified module-key and durable-artifact carrier that later stages can
adopt without parser-private or Engine-private rediscovery. This handoff deliberately does not
migrate the existing resolver graph or semantic-summary identity implementations.

## Dependencies

- ✅ TASK-2056 — target contract and seam audit.
- 📝 TASK-2057 — AST-derived structural declarations; may be integrated after its public handoff is stable.

## Current → target

**Current files:** `crates/ash-core/src/module_graph.rs`, `crates/ash-parser/src/resolver.rs`, semantic-summary modules in `crates/ash-core`.

**Current state:** graph nodes still use allocation-backed `ModuleId`/`ModuleSource`, and semantic
summaries still use their existing `ModuleIdentity`. The new independent Core carrier provides a
tested stable identity/artifact contract but is not yet consumed by either existing route.

**Target state:** `ash-core` owns canonical module path/key, source origin, structural/import dependency facts, parsed/expanded/check-state references, interface schema identity, and diagnostic anchors. The exact Rust shapes may vary, but no downstream semantic interface uses a bare name or filesystem path as identity. This task introduces a new `ModuleKey` contract; it does not change existing `ModuleIdentity` semantics.

## Requirements

1. Define canonical equality, display, serialization, and cache-key behavior for crate-qualified paths.
2. Preserve defining identity through aliases and re-exports without minting a second identity.
   This target clause is unmet by the carrier and is assigned to TASK-2060/2061 once they consume
   it through checked interfaces and imports.
3. Make structural-parent and child-key queries deterministic.
4. Represent file and inline source origins without requiring inline text reconstruction.
5. Completed subset: reject duplicate canonical child identities before **Core artifact**
   publication. Interface publication, including the alias/re-export identity clause, remains a
   later checked-interface obligation.

## TDD Steps and evidence

1. [x] Add unit tests for canonical nested paths, `mod.ash` directory identity, file identity,
   inline identity, crate distinction, parser-valid spelling, and parent/child round trips.
2. [x] Add proptest coverage for path segment construction and structural-tree invariants.
3. [x] Add serde/cache-key and schema/origin round trips, including malformed/forged wire
   rejection.
4. [x] Keep existing graph consumers untouched until a separately owned migration consumes the
   carrier.

## Completion checklist

- [x] Canonical module identity is crate-qualified and independent of filesystem spelling and
  source origin.
- [x] File and inline origins are represented without rebuilding inline text.
- [x] Duplicate child identity, topology, wire-validation, and property tests pass.
- [x] Focused core tests, legacy module-graph tests, fmt, and strict clippy pass.

## Handoffs

- **Consumes:** the SPEC-103 identity/origin contract and the forward-compatible structural
  handoff from TASK-2057; no existing resolver graph value is consumed yet.
- **Produces:** a tested, schema-versioned Core `ModuleKey`/`ModuleArtifact` carrier for
  TASK-2059 through TASK-2063. It supplies no resolver graph migration or module unit.
- **Downstream owners:** TASK-2059 owns source acquisition and module units; TASK-2060 owns
  checked interfaces; TASK-2061 owns interface-driven imports/visibility; TASK-2062 owns
  lowering; TASK-2063 owns admission; TASK-2064 owns conformance/parity; TASK-2065 owns closeout.
- **Non-goals:** Migrating resolver graph construction or existing semantic_summary::ModuleIdentity; source-kind parity or source-anchored structural diagnostics; checked interfaces, import binding, visibility enforcement, Core/CPS lowering, Engine admission, persistent disk cache, runtime module values, or client parity.

## Files and verification

**Files:** `crates/ash-core/src/module_graph.rs`,
`crates/ash-core/tests/task_2058_canonical_module_identity.rs`,
`crates/ash-parser/tests/task_2058_module_key_identifier_parity.rs`.

```text
cargo test -p ash-core --test task_2058_canonical_module_identity
cargo test -p ash-parser --test task_2058_module_key_identifier_parity
cargo test -p ash-core module_graph
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
```
