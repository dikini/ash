# TASK-2059: File/Inline Module-Unit Parity

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§3-5, §8 (`M-PARSE-FILE`, `M-PARSE-INLINE`)
**Owned rule:** MOD-REAL-002
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2059 file/inline module-unit parity](../SEMANTIC-RULE-COVERAGE.md#task-2059-fileinline-module-unit-parity)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Structural-cycle/CircularDependency rejection and malformed-inline parent-anchor/error-atomicity remain outside this one-unit source-acquisition route; resolver ModuleGraph and legacy semantic_summary::ModuleIdentity migration/persistence; checked export-closed interfaces; interface-driven import binding and visibility; module-aware Core/CPS lowering; linked Engine admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
**Layers:** type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-002-FILE-INLINE-UNIT` and `TEST-MOD-REAL-002-INLINE-NESTED-MOD-PARSE`; negative `TEST-MOD-REAL-002-SOURCE-DIAGNOSTICS` and `TEST-MOD-REAL-002-DUPLICATE-CHILD`; mutation `TEST-MOD-REAL-002-SOURCE-KIND-ERASURE`; source-unit parity `TEST-MOD-REAL-002-FILE-INLINE-PARITY`; no proof.
**Next obligation:** TASK-2060 must consume the completed TASK-2059 units for checked export-closed interfaces; TASK-2061 owns interface-driven import binding and visibility; TASK-2062 module-aware Core/CPS lowering; TASK-2063 Engine-only linked admission with no direct-evaluator fallback; TASK-2064 owns source-diagnostic conformance, structural-cycle coverage, and CLI/daemon parity; TASK-2065 closes the phase.

## Delivered handoff

TASK-2059 completes a bounded, parser-owned source-acquisition handoff:

- `ModuleItem` and `ModuleBody` preserve ordered `use`, definition, and nested-module items.
- File and inline declarations use the same item dispatcher; `ModuleUnit` is the common
  source-kind-independent carrier after acquisition.
- `ModuleUnitResolver` consumes TASK-2058 `ModuleKey`/`ModuleArtifact` facts. It prefers
  `child.ash`, falls back to `child/mod.ash`, reads and parses the chosen file once, and performs
  zero filesystem operations for an inline body.
- Artifact construction occurs only after duplicate-child and canonical-key checks. Missing and
  invalid-key errors retain the enclosing source path and declaration span.
- Macro/notation expansion, expanded-boundary diagnostics, and hygiene traversal recurse through
  nested inline bodies with isolated local scopes.

The source form is erased from the unit body and canonical child keys. `ModuleArtifactOrigin` and
the diagnostic source path remain intentionally observable provenance, not alternate semantic
authority. Parsed `use` items remain syntax: this handoff neither binds imports nor grants
visibility, lowering, admission, or execution authority.

## Remaining target boundary

The handoff does not traverse a transitive graph, so it cannot reject a structural cycle or prove
graph-level failure atomicity. A malformed inline declaration fails while parsing its enclosing
source before a `ModuleUnit` exists; a parent-path-bearing malformed-inline diagnostic remains
unimplemented. The legacy resolver `ModuleGraph`, `semantic_summary::ModuleIdentity`, persistent
caches, checked interfaces, import/visibility binding, Core/CPS lowering, Engine admission and
execution, and CLI/daemon parity are unchanged. No failed or absent downstream stage may select a
direct-evaluator fallback.

## Task-owned evidence

**Canonical traceability rule:** `SEM-MODULE-REALIZATION-002`, the traceability alias for
`MOD-REAL-002` in SPEC-103. The primary implementation is
`ash_parser::resolver::ModuleUnitResolver::acquire_child`, fingerprint
`sha256:abeb18b2a182154fc9bbf0abb34a659ec31c60851d46841edd7e596b6cb71954`.

| Axis | Traceability witness | Focused evidence |
|---|---|---|
| Positive | `TEST-MOD-REAL-002-FILE-INLINE-UNIT` | File and inline children retain the same ordered body and canonical child keys. |
| Positive | `TEST-MOD-REAL-002-INLINE-NESTED-MOD-PARSE` | Depth-two inline macro/notation scopes expand recursively and remain isolated. |
| Negative | `TEST-MOD-REAL-002-SOURCE-DIAGNOSTICS` | Missing and genuinely invalid child keys retain their parent declaration anchor. |
| Negative | `TEST-MOD-REAL-002-DUPLICATE-CHILD` | Duplicate file and inline children reject before a unit returns. |
| Mutation | `TEST-MOD-REAL-002-SOURCE-KIND-ERASURE` | Direct-file preference, directory fallback, and inline zero-FS behavior do not alter the unit identity/body handoff. |
| Parity | `TEST-MOD-REAL-002-FILE-INLINE-PARITY` | The paired source fixtures establish source-unit parity only, not runtime or client parity. |

The focused evidence is `cargo test -p ash-parser --test
task_2059_file_inline_module_unit_parity` (8 tests). The consumed canonical-name grammar evidence
is `cargo test -p ash-parser --test task_2058_module_key_identifier_parity` (2 tests).

## Description

Construct one parser module-unit route for file-backed and inline modules. Source acquisition is
the only permitted difference; ordered item syntax, canonical identity, artifact topology, and
syntax-phase traversal share the handoff. Later checking, import binding, lowering, admission, and
execution remain separately authorized work.

## Dependencies

- ✅ TASK-2057 — AST-driven discovery.
- ✅ TASK-2058 — canonical module identity and artifact substrate.

## Current → target

**Implemented files:** `crates/ash-parser/src/module.rs`, `crates/ash-parser/src/parse_module.rs`,
`crates/ash-parser/src/resolver.rs`, `crates/ash-parser/src/lib.rs`, and
`crates/ash-parser/src/surface.rs`.

**Completed handoff:** parsed file bodies and parsed inline bodies now arrive as the same ordered
`ModuleBody` within a `ModuleUnit`; only artifact origin and diagnostic source anchoring differ.

**Still target-only:** conversion of units into checked interfaces/import bindings, legacy graph
migration, Core/CPS artifacts, Engine linking/admission, and client parity.

## Requirements and closure

1. **Delivered:** inline parsing accepts the existing `use`, definition, and nested-module item
   forms through the shared dispatcher, and both source forms acquire one `ModuleUnit` carrier.
2. **Delivered for acquisition:** identities normalize through `ModuleKey` and `ModuleArtifact`;
   file and inline diagnostics retain their permitted source anchors.
3. **Deferred:** Engine ordinary-definition guards are not changed by this parser handoff; ordinary
   checking diagnostics belong to the checked-interface/Engine owners.
4. **Partially delivered:** missing files and duplicate children reject before returning a unit.
   Malformed-inline diagnostics, structural cycles, and graph-level failure atomicity are deferred.
5. **Delivered for syntax phase:** recursive macro/notation scopes, expanded-boundary diagnostics,
   and hygiene traversal are isolated and source-form-independent for the parser carrier.

## Completion checklist

- [x] File and inline declarations accept the same module-item domain and construct one common
  source-acquisition/module-unit representation.
- [x] Paired source forms have tested ordered-body, canonical-child-key, origin, and source-anchor
  evidence at the module-unit boundary.
- [x] Focused parser tests, full parser tests, Engine guard regression, formatting, and clippy are
  recorded below.
- [ ] Checked interface/export closure, import/visibility binding, Core/CPS lowering, Engine
  admission/execution, graph-cycle conformance, and CLI/daemon parity remain later tasks.

## Handoffs

- **Consumes:** TASK-2057 parser-owned structural declarations and TASK-2058 `ModuleKey`,
  `ModuleArtifactOrigin`, and `ModuleArtifact` carriers.
- **Produces:** an ordered parser `ModuleBody`/`ModuleUnit` source-acquisition handoff with
  canonical child keys, provenance, source anchors, and recursive syntax-phase scope traversal.
  It is non-authorizing: it publishes no checked interface, import binding, Core/CPS artifact, or
  Engine frame/admission fact.
- **Downstream owners:** TASK-2060 owns checked interface creation; TASK-2061 owns
  import/visibility resolution; TASK-2062 owns Core/CPS lowering; TASK-2063 owns Engine-only
  linked admission; TASK-2064 owns structural diagnostic conformance and client parity; TASK-2065
  owns closeout.
- **Run-route impact:** prerequisite. TASK-2064 is the consuming integration owner; this task does
  not make a CLI or daemon route runnable and cannot authorize a direct evaluator fallback.
- **Non-goals:** Structural-cycle graph traversal and malformed-inline parse diagnostics beyond this parser unit handoff; resolver ModuleGraph and legacy semantic_summary::ModuleIdentity migration/persistence; checked interface/export closure, import binding, visibility enforcement, Core/CPS lowering, Engine admission/execution or a direct-evaluator fallback, dynamic loading, import-cycle initialization, runtime module values, or client parity.

## Verification

```text
cargo test -p ash-parser --test task_2058_module_key_identifier_parity
cargo test -p ash-parser --test task_2059_file_inline_module_unit_parity
cargo test -p ash-parser
cargo test -p ash-engine --test module_file_check_tests
cargo clippy -p ash-parser -p ash-engine --all-targets -- -D warnings
cargo fmt --check
```
