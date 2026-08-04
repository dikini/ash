# TASK-2067: Canonical Module Graph and Structural Diagnostics

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§3-5, §8 (`M-DISCOVER`, `M-PARSE-FILE`, `M-PARSE-INLINE`), §§9 and 11
**Owned rules:** MOD-REAL-001 and MOD-REAL-002
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2067 canonical module graph and structural diagnostics](../SEMANTIC-RULE-COVERAGE.md#task-2067-canonical-module-graph-and-structural-diagnostics)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Complete checked interfaces, parsed imports, visibility, re-exports, import-cycle rejection, and binder integration remain owned by TASK-2068; complete definition-body lowering and the Engine scanner/cache transport fence remain owned by TASK-2069; Engine-linked admission remains owned by TASK-2063; real-program file/inline and CLI/daemon terminal parity remain owned by TASK-2064; and TASK-2065 owns phase closeout.
**Layers:** type `partial`; Core `partial`; CPS/admission-runtime `not_applicable`; verification
`partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-001-CANONICAL-GRAPH` and
`TEST-MOD-REAL-002-REAL-UNIT-TRANSPORT`; negative
`TEST-MOD-REAL-001-STRUCTURAL-DIAGNOSTICS`; mutation
`TEST-MOD-REAL-001-INLINE-SOURCE-REENTRANCY-GUARD`,
`TEST-MOD-REAL-001-GRAPH-KEY-REWRITE`, and
`TEST-MOD-REAL-002-GRAPH-UNIT-PAYLOAD-MUTATION`; source-unit parity
`TEST-MOD-REAL-002-GRAPH-UNIT-PARITY`; architectural fence
`TEST-MOD-REAL-001-LEGACY-ROUTE-FENCE`. The focused parser targets passed:
[`task_2067_canonical_module_graph`](../../../crates/ash-parser/tests/task_2067_canonical_module_graph.rs)
(12), [`task_2067_canonical_identity_fence`](../../../crates/ash-parser/tests/task_2067_canonical_identity_fence.rs)
(3), and [`task_2067_legacy_route_fence`](../../../crates/ash-parser/tests/task_2067_legacy_route_fence.rs)
(2). They prove AST-only canonical `ModuleKey` edges, actual file/inline `ModuleUnit` transport,
the complete `Absent`/`Discovered`/`Parsed`/`Failed` reporting contract, root and nested anchored
missing/duplicate/malformed-inline/cycle rejection, parsed-source invalid-key rejection without a
synthetic child, canonical-key rewrite resistance, root crate metadata retention, complete ordered
payload parity and payload mutation, and a deprecated legacy-route fence. The source-layout portion
of the legacy fence is an architecture regression check, not a semantic mutation proof.
**Next obligation:** TASK-2068 consumes the completed, non-authorizing graph/unit handoff for complete checked interfaces and parsed import/binder semantics; TASK-2069, TASK-2063, TASK-2064, and TASK-2065 retain their separately owned downstream boundaries.

## Description

Replace the legacy `ModuleId`/`ModuleSource` structural path with the parser-owned structural
portion of the SPEC-103 module machine. The graph must use the existing canonical `ModuleKey` and
transport the actual file or inline `ModuleUnit` acquired by TASK-2059; it must not recreate either
from a path, name, or source scan. It owns discovery through parsed source acquisition and failure,
not import binding or checking.

## Dependencies

- ✅ TASK-2057 — AST-derived structural declarations and parser-owned declaration spans.
- ✅ TASK-2058 — canonical `ModuleKey`, `ModuleArtifactOrigin`, and `ModuleArtifact` carrier.
- ✅ TASK-2059 — ordered file/inline `ModuleUnit` acquisition and syntax-phase traversal.

## Requirements

1. Define the canonical structural graph/store carrier around `ModuleKey`; no graph edge, graph
   lookup, duplicate check, or structural cache key may use a bare name, a path string, or legacy
   `ModuleIdentity` as semantic identity.
2. Implement the structural portion of the module machine with `Absent`, `Discovered`, `Parsed`,
   and `Failed` states. It must accept only parsed `ModuleDecl` edges and preserve the source
   origin and declaration anchor used to derive each transition. TASK-2068 owns the later
   expansion, collection, binding, checking, and final-interface transitions.
3. Feed a selected file child and an inline child into the graph as the actual TASK-2059
   `ModuleUnit`; after acquisition, downstream consumers receive the same unit contract rather
   than a second file loader, inline projection, or source reparse.
4. Reject a missing child, duplicate child, malformed inline child, invalid canonical child key,
   or structural cycle before publishing a partial child. Each diagnostic must be anchored at the
   parent `mod` declaration (or the malformed inline declaration) and identify the canonical
   structural path/cycle where applicable.
5. Make structural-cycle failure atomic for the affected dependency closure. A `Failed` child may
   retain diagnostic provenance but may not expose a parsed unit as a usable graph entry.
6. Preserve file/inline semantic parity at this transport layer: source form may affect allowed
   provenance and display paths, never child key, graph topology, state outcome, or delivered
   module-unit semantics.

## TDD steps and reserved evidence

1. ✅ `task_2067_canonical_module_graph` proves a multi-level parsed module tree uses only
   canonical `ModuleKey` graph edges and retains its acquired real file/inline units
   (`TEST-MOD-REAL-001-CANONICAL-GRAPH`, `TEST-MOD-REAL-002-REAL-UNIT-TRANSPORT`).
2. ✅ The focused negatives prove parent-anchored missing, root/nested duplicate, malformed-inline,
   parsed-source invalid-key, and cycle rejection with error-side `Failed` keys and no returned
   partial graph (`TEST-MOD-REAL-001-STRUCTURAL-DIAGNOSTICS`).
3. ✅ The inline-source-reentrancy mutation proves that resolving an inline child cannot erase an
   active root source guard (`TEST-MOD-REAL-001-INLINE-SOURCE-REENTRANCY-GUARD`), and the canonical
   identity target proves graph-key rewrite and wrong-key rejection without synthetic declaration
   authority (`TEST-MOD-REAL-001-GRAPH-KEY-REWRITE`).
4. ✅ Paired file/inline fixtures retain real acquired units, equivalent canonical child topology,
   root metadata, complete ordered item payloads, and payload-only mutation behavior
   (`TEST-MOD-REAL-002-GRAPH-UNIT-PARITY`,
   `TEST-MOD-REAL-002-GRAPH-UNIT-PAYLOAD-MUTATION`).
5. ✅ The parser full suite, formatting, and strict clippy passed after the focused targets.

## Completion checklist

- [x] A parsed `ModuleDecl` is the sole structural-edge authority and every published structural
  key is a canonical `ModuleKey`.
- [x] File and inline children enter the graph as the actual same-contract `ModuleUnit` payload.
- [x] Anchored missing, duplicate, malformed-inline, invalid-key, and structural-cycle diagnostics
  fail atomically without a partial published child.
- [x] Positive, negative, mutation, and file/inline transport-parity evidence is recorded in the
  activated task record and traceability graph.
- [x] TASK-2067 produces the graph handoff for planned TASK-2068 consumption; no graph state
  authorizes imports, bindings, lowering,
  Engine admission, a provider/handler frame, or a direct-evaluator fallback.

## Remaining target boundary

The completed handoff adds a parser-owned canonical graph whose published edges and entries are
`ModuleKey`-keyed and whose values are real acquired `ModuleUnit`s. It is atomic at its public
boundary: anchored missing, duplicate, malformed-inline, invalid-key, and cycle failures report
the affected `Failed` keys on the error and return no partial graph. The graph retains parsed root
crate metadata, complete file/inline ordered payloads, and a strict deprecated legacy-route fence.
This is still not complete SPEC-103 realization: TASK-2068 owns interfaces/imports/visibility and
binder integration; TASK-2069 owns full lowering and Engine transport fencing; TASK-2063 owns
admission; TASK-2064 owns real-program and client parity; and TASK-2065 owns phase closeout. The
predecessor carriers and this graph remain non-authorizing for imports, binding, lowering, Engine
admission, runtime frames, direct evaluation, or client parity.

## Handoffs

- **Consumes:** TASK-2057 parsed declarations/spans, TASK-2058 canonical identity/artifact facts,
  and TASK-2059 acquired ordered module units.
- **Produces:** a canonical-keyed structural graph and parser-state-machine handoff containing real
  source-acquired units, structural edges, source origins, and failed-state diagnostics. It is
  non-authorizing: neither graph membership nor a parsed unit is a checked interface or runtime
  credential.
- **Downstream owner:** TASK-2068 owns expanded collection, parsed imports, visibility, binding,
  complete checked interfaces, and import-cycle atomicity; TASK-2069 later consumes its checked
  module bodies for lowering.
- **Integration/proof responsibility:** TASK-2067 owns focused structural and transport evidence.
  TASK-2064 owns the composed file/inline and CLI/daemon normalized-terminal proof obligation.
- **Run-route impact:** `prerequisite`. This task enables no active Engine or client route.
- **Non-goals:** Import-edge resolution, typechecking, complete public-interface closure,
  aliases/re-exports, typed namespace linkage, Core/CPS lowering, Engine scanner transport,
  admission/execution, import-cycle initialization, runtime module values, or CLI/daemon parity.

## Candidate files and verification

**Source/test paths:** `crates/ash-parser/src/canonical_module_graph.rs`,
`crates/ash-parser/tests/task_2067_canonical_module_graph.rs`,
`crates/ash-parser/tests/task_2067_canonical_identity_fence.rs`, and
`crates/ash-parser/tests/task_2067_legacy_route_fence.rs`.

```text
cargo test -p ash-parser --test task_2067_canonical_module_graph
cargo test -p ash-parser --test task_2067_canonical_identity_fence
cargo test -p ash-parser --test task_2067_legacy_route_fence
cargo test -p ash-parser
cargo clippy -p ash-parser --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
