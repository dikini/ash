# TASK-807: Sealed Domain Audit Gate

**Status:** Complete
**Date:** 2026-05-05
**Auditor:** Hermes Agent (Phase 111 substrate audit)

## Purpose

Authoritative audit of the live parser/core/engine/typechecker substrate before Phase 111 (SPEC-059, sealed type-level domains) implementation begins. Documents what Phase 111 may change, what must remain compatible, and what is explicitly deferred.

---

## 1. Parser Baseline

### Current state

- `crates/ash-parser/src/surface.rs` line 76: `pub enum Definition` has variants: `Capability`, `CapabilityInterface`, `CapabilityImplementation`, `ResourceType`, `Type`, `Policy`, `Role`, `Proxy`, `Interface`, `Impl`, `Function`, `BuiltinFn`.
- **No `SealedDomain` variant exists.** The only type-declaration carrier is `Definition::Type(TypeDef)`.
- `TypeDef` (line 105) carries visibility, name, generic params, body (`TypeBody`), and a `builtin` flag. No sealed-domain fields.

### Lowering

- `crates/ash-parser/src/lower.rs` line 1257: `pub fn lower_module_type_metadata(...)` produces `LoweredTypeMetadata` from a parsed `ModuleFile`.
- Only processes `Definition::Type` entries. No sealed-domain lowering path exists.

### Required changes (TASK-808)

- Add `Definition::SealedDomain(SealedDomainDef)` to `surface::Definition`.
- Add `SealedDomainDef`, `DomainConstructor`, `DomainField`, `DomainSlot` surface AST types.
- Add parser production for `sealed type domain Ident { ... }`.
- Add explicit rejection boundaries for: inline-module sealed domains, per-constructor visibility, arbitrary type expressions in field slots, tuple fields, generic domain params.
- Parse tests for accepted and rejected forms.

---

## 2. Core Baseline

### Kind ownership

- `crates/ash-core/src/kind.rs` line 13: `pub enum Kind { Type, Arrow(Box<Kind>, Box<Kind>) }` is core-owned (Phase 110 moved Kind here from ash-typeck).

### Semantic summary

- `crates/ash-core/src/semantic_summary.rs`:
  - `SummaryVersion(pub u16)` at line 375, single const: `SPEC057_ORDINARY_TYPE_V1 = Self(1)`.
  - `ReservedSemanticIdentitySlots` at line 383 with `sealed_domains: Vec<String>` -- currently a placeholder string list, NOT a typed identity carrier.
  - `ModuleSemanticSummary` at line 453 carries: `module`, `version`, `exported_types`, `exported_constructors`, `re_exports`, `imported_summary_refs`, `interface_identities`, `associated_member_identities`, `reserved_identity_slots`, `diagnostic_anchors`.
  - **No `exported_sealed_domains` field exists.**
  - `TypeDeclSummary` and `ConstructorSummary` are ordinary-type/runtime-constructor carriers.

### Required changes (TASK-809)

- Add `SealedDomainId` and `DomainConstructorId` identity types (distinct from `TypeDeclId`/`ConstructorId`).
- Add `SealedDomainSummary`, `DomainConstructorSummary`, `DomainFieldSummary`, `StructuralFieldStatus` summary carriers.
- Add `exported_sealed_domains: Vec<SealedDomainSummary>` to `ModuleSemanticSummary`.
- Advance `SummaryVersion` to a new constant (e.g., `SPEC059_SEALED_DOMAIN_V2 = Self(2)`).
- Keep backward compatibility: old-version summaries without sealed-domain fields must still be readable.

---

## 3. Engine Baseline

### Module loader

- `crates/ash-engine/src/module_loader.rs` line 1555: `collect_module_type_metadata_from_module_file(...)` parses a module file, calls `reject_inline_module_ordinary_types(...)`, then lowers via `ash_parser::lower::lower_module_type_metadata`.
- `reject_inline_module_ordinary_types` (line 1567) checks inline module declarations for `Definition::Type` and rejects them. Does NOT check for sealed-domain definitions (since they don't exist yet).

### Required changes (TASK-811)

- Extend `collect_module_type_metadata_from_module_file` (or add a parallel path) to extract sealed-domain metadata from parsed module files.
- Add inline-module sealed-domain rejection (mirroring the ordinary-type rejection pattern).
- Transport public sealed-domain summaries through the import/export pipeline.
- Ensure private domains export opaquely (no constructor metadata leaks).

---

## 4. Typechecker Baseline

### TypeEnv

- `crates/ash-typeck/src/type_env.rs` line 1199: `validate_summary_visibility_and_duplicates(...)` validates imported module summaries.
- Line 1202: **Hard-rejects any summary version that is not `SPEC057_ORDINARY_TYPE_V1`**. This is a blocking contradiction: sealed-domain summaries with a new version will be rejected.
- Validation checks ordinary type visibility, representation exposure, and constructor visibility. No sealed-domain awareness.

### Kind in typeck

- `crates/ash-typeck/src/kind.rs` re-exports or mirrors `ash_core::kind::Kind`.

### Required changes (TASK-812)

- Accept the new summary version in `validate_summary_visibility_and_duplicates` (or add a parallel validation path).
- Add sealed-domain registration: store domain metadata, constructor metadata, and field metadata in TypeEnv.
- Add declare-then-validate two-pass flow for domain registration.
- Validate: domain name uniqueness, constructor name uniqueness within domain, field domain references resolve to known domains, structural-status derivation, visibility/anti-leak rules.
- Enforce: at most one structural self-domain field per constructor, no mutual recursion between local domains.

---

## 5. Contradictions and Blockers

| # | Contradiction | Location | Resolution |
|---|---------------|----------|------------|
| C1 | No `SealedDomain` variant in `surface::Definition` | `surface.rs:76` | TASK-808 adds variant |
| C2 | No `SealedDomainId` / `DomainConstructorId` identity types | `ash-core` | TASK-809 adds them |
| C3 | `ReservedSemanticIdentitySlots.sealed_domains` is `Vec<String>`, not typed | `semantic_summary.rs:385` | TASK-809 replaces with typed carriers in `exported_sealed_domains` |
| C4 | `SummaryVersion` only has `SPEC057_ORDINARY_TYPE_V1` | `semantic_summary.rs:378` | TASK-809/810 advances version |
| C5 | TypeEnv hard-rejects non-V1 summaries | `type_env.rs:1202` | TASK-812 widens accepted versions |
| C6 | Engine inline-module rejection only checks `Definition::Type` | `module_loader.rs:1577` | TASK-811 adds sealed-domain check |
| C7 | No lowering path for sealed domains | `lower.rs` | TASK-810 adds domain lowering |

### Feasibility blockers for TASK-812

1. **Domain vs ordinary-constructor registries**: Must use separate identity types and storage. Marker constructors MUST NOT go into the ordinary `ConstructorId` registry. TypeEnv needs a dedicated `sealed_domains` map keyed by `SealedDomainId`.
2. **Summary-version evolution**: TypeEnv must accept both V1 (ordinary only) and V2 (ordinary + sealed-domain) summaries. V2 summaries without sealed domains are valid (empty `exported_sealed_domains`).
3. **Declare-then-validate**: For local domains, first declare all domain identities (pass 1), then validate field references and structural status (pass 2). This handles forward references like `Cons<tail: TypeList>` where `TypeList` is the enclosing domain. For imported domains, validation happens during summary import.

---

## 6. Exact File Targets by Task

### TASK-808: Parser surface for sealed type domains

| File | Action |
|------|--------|
| `crates/ash-parser/src/surface.rs` | Add `SealedDomainDef` struct, `DomainConstructor`, `DomainField`, `DomainSlot` types, add `Definition::SealedDomain` variant |
| `crates/ash-parser/src/parse_module.rs` | Add parser production for sealed-domain declarations |
| `crates/ash-parser/src/parse_type_def.rs` | Possibly add sealed-domain parsing helpers (or new file) |
| Test file (new or existing) | Parser acceptance and rejection tests |

### TASK-809: Core domain kind, IDs, and summary carriers

| File | Action |
|------|--------|
| `crates/ash-core/src/lib.rs` | Re-export new types |
| `crates/ash-core/src/semantic_summary.rs` | Add `SealedDomainId`, `DomainConstructorId`, `SealedDomainSummary`, `DomainConstructorSummary`, `DomainFieldSummary`, `StructuralFieldStatus`, add `exported_sealed_domains` field to `ModuleSemanticSummary`, advance `SummaryVersion` |
| `crates/ash-core/src/kind.rs` | No changes needed (Kind is already core-owned and sufficient) |
| Test file | Unit tests for new identity and summary types |

### TASK-810: Domain lowering and summary versioning

| File | Action |
|------|--------|
| `crates/ash-parser/src/lower.rs` | Add sealed-domain lowering function, produce `SealedDomainSummary` carriers |
| `crates/ash-core/src/semantic_summary.rs` | Version advancement, builder methods |
| Test file | Lowering tests |

### TASK-811: Engine domain summary export/import

| File | Action |
|------|--------|
| `crates/ash-engine/src/module_loader.rs` | Extend `collect_module_type_metadata_from_module_file` or add sealed-domain extraction path, add inline-module sealed-domain rejection |
| `crates/ash-engine/src/lib.rs` | Expose sealed-domain metadata in engine API |
| Test file | Engine transport tests |

### TASK-812: TypeEnv domain registration and validation

| File | Action |
|------|--------|
| `crates/ash-typeck/src/type_env.rs` | Accept V2 summaries, add sealed-domain registration, declare-then-validate flow, field/reference validation |
| `crates/ash-typeck/src/lib.rs` | Re-export new types |
| Test file | Registration and validation tests |

### TASK-813: Diagnostics and non-interference

| File | Action |
|------|--------|
| Multiple crates | Negative test cases, diagnostic quality, non-interference with Phase 109/110 behavior |

---

## 7. Explicitly Deferred Work (SPEC-D/E/F/G/H)

The following are **out of scope** for Phase 111 and belong to future design/spec packets:

| Item | Belongs to | Notes |
|------|------------|-------|
| Normalization / definitional equality | SPEC-D+ | No reduction/forcing in this phase |
| Public `type fn` syntax and equation tables | SPEC-D+ | Deferred explicitly |
| Promoted data kinds / DataKinds-style promotion | SPEC-E+ | Marker constructors are type-level only |
| Type-level pattern matching / coverage checking | SPEC-F+ | Substrate only (closed constructor set) |
| Per-constructor visibility | Future packet | Constructor visibility inherits domain |
| Generic sealed domain parameters | SPEC-G+ | No higher-kinded domains |
| Mutual recursive domain SCCs | Future packet | Rejected in first slice |
| Constructor-only imports/re-exports | SPEC-H+ | Domain-scoped only |
| Associated type-family computation | SPEC-D+ | Explicitly deferred |
| Arbitrary type expressions in field slots | Future packet | Only `Type` and domain-ref allowed |
| Inline-module sealed-domain declarations | Future packet | Explicitly rejected |
| Structural recursion checking (runtime) | SPEC-F+ | Only structural-status metadata recorded |
| Totality proofs / overlap checking | Future packet | Not in scope |

---

## 8. Execution Gate

Before any Rust implementation task (TASK-808+) begins:

- [x] SPEC-059 is written and registered (TASK-806 complete)
- [x] PLAN-107 exists with task table and execution order
- [x] All task files TASK-807 through TASK-815 exist with dispatch metadata
- [x] This audit documents the live substrate state, contradictions, and file targets
- [x] Ordinary-type/parser/core/engine/typeck baseline confirmed clean (`cargo check --all` passes)

**The audit gate is satisfied. Phase 111 implementation may proceed.**
