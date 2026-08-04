# TASK-2061: Interface Import Resolution and Visibility

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §6
**Owned rule:** MOD-REAL-004
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2061 interface import resolution and visibility](../SEMANTIC-RULE-COVERAGE.md#task-2061-interface-import-resolution-and-visibility)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Parsed use integration and all parsed visibility forms with inaccessible diagnostics; parsed aliases, pub use, and re-exports; full typed namespaces and binder integration; import-cycle detection; complete final-interface/export-closure validation; module-aware Core/CPS lowering; Engine scanner fencing/transport and linked admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
**Layers:** type `partial`; Core `not_applicable`; CPS `not_applicable`; admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-004-EXPLICIT-CHILD-IDENTITY`; negative `TEST-MOD-REAL-004-MISSING-CHECKED-CHILD`; mutation `TEST-MOD-REAL-004-GROUP-ATOMICITY`; no proof. Parity is `not_applicable`: this bounded checked-interface resolver has no paired source, Engine, or client execution relation.
**Next obligation:** The parser and binder integration owners must consume only this resolver's `FinalizedModuleInterface`-backed checked store. They must add parsed visibility/inaccessible diagnostics, aliases/re-exports, typed namespaces, cycle rejection, complete closure, lowering, Engine transport, and parity without reintroducing raw Core, parser, Engine, filesystem, legacy graph, or text-scan authority.
**Non-goals:** Treating raw `PublicModuleInterface`, parser resolver state, legacy `ModuleGraph`, Engine state, filesystem paths, or text scans as import authority; parsed `use`/visibility/re-export/alias support, full typed namespaces or binder integration, import cycles, complete final-interface/export closure, Core/CPS lowering, Engine transport/admission/execution or scanner fencing, dynamic imports, runtime module values, or client parity.

## Description

Provide a bounded, in-memory import resolver over finalizer-issued interfaces. It resolves explicit,
grouped, and glob requests through canonical checked wrappers only. This is not parsed `use`
resolution, a complete visibility system, a binder integration, or an authoritative full interface.

## Dependencies

- 📝 TASK-2059 — common file/inline module units.
- ✅ TASK-2060 — bounded Core public-interface carrier, not import authority.
- ✅ TASK-2066 — partial/tested/below-spec bounded TypeEnv finalization. This resolver consumes only its `FinalizedModuleInterface` wrapper, never raw Core.

## Current target

**Delivered file:** `crates/ash-typeck/src/interface_import_resolver.rs`.

**Current state:** `CheckedInterfaceStore` accepts only TASK-2066 `FinalizedModuleInterface` wrappers
under canonical module keys and rejects duplicate keys. `InterfaceImportResolver` traverses public
child identities only through that store and never accesses raw Core, parser resolver state, legacy
`ModuleGraph`, Engine state, a filesystem, or a text scan. It preserves provider defining identity
and transports syntax macros as non-runtime metadata.

**Target state:** parser and binder integration consumes the same checked-store identities without
widening this bounded handoff. The Engine never searches the filesystem or raw source to satisfy an
import.

## Activation boundary

The TASK-2060 Core carrier remains a schema/data boundary, and TASK-2066's wrapper remains bounded:
neither is the language-level `PublicInterface` required by SPEC-103. This task accepts only the
wrapper and keeps its store private to wrapper values. It does not claim generic typed namespaces,
complete export closure, parser imports, visibility, Engine transport, or runtime authority.

## Remaining target boundary

The request API is intentionally not the parser's `use` grammar. It does not enforce any parsed
visibility form or inaccessible diagnostic, re-export/`pub use`/parsed alias rule, import cycle,
full typed namespace, binder integration, complete interface closure, lowering, Engine transport,
or client parity. These gaps remain visible deferred clauses; the bounded resolver must not be
used to manufacture a full import or runtime authority.

## Requirements

1. Accept only finalizer-issued wrappers in a canonical-key checked-interface store.
2. Traverse public child-module defining identities through that store before looking up a public
   binding.
3. Resolve bounded explicit, grouped, and glob requests; preserve defining identity and
   syntax-only macro metadata.
4. Stage grouped members before publication and reject duplicate local explicit/group names.
5. Give explicit bindings precedence over globs; leave distinct glob identities ambiguous at lookup
   rather than choosing a winner.
6. Keep every parser, legacy graph, Engine, filesystem, and text-scan route outside this boundary.

## Task-owned evidence

The focused `task_2061_interface_import_resolution` target passes 11/11:

- **Positive/property:** `TEST-MOD-REAL-004-EXPLICIT-CHILD-IDENTITY` proves an explicit child
  import alias retains its provider defining identity for generated canonical paths.
- **Negative:** `TEST-MOD-REAL-004-MISSING-CHECKED-CHILD` rejects a public child identity whose
  finalizer-issued wrapper is absent from the checked store.
- **Mutation:** `TEST-MOD-REAL-004-GROUP-ATOMICITY` substitutes an unknown group member and proves
  that no otherwise resolvable member leaks into the environment.

The same target also proves duplicate checked-store keys and local bindings reject, explicit-over-
glob precedence, deferred glob ambiguity, syntax-only macro transport, and that private Core
bindings cannot become imports. No proof or source/Engine/client parity claim is made.

## Completion checklist

- [x] A finalizer-wrapper-only checked store rejects duplicate canonical module keys and excludes raw
  Core construction.
- [x] Bounded explicit/group/glob requests traverse checked public children, preserve identity, and
  make groups atomic, explicit bindings dominant, and conflicting globs ambiguous.
- [x] Syntax macros remain non-runtime metadata; no parser, legacy graph, Engine, filesystem, or
  text scan is consulted.
- [ ] Parsed `use` forms, all parsed visibility/inaccessible diagnostics, aliases/re-exports,
  typed namespaces, cycles, binder integration, complete closure, lowering, Engine transport, and
  client parity remain separately owned.

## Handoffs

- **Consumes:** TASK-2066 `FinalizedModuleInterface` wrappers only. TASK-2059 units and TASK-2060
  Core carriers are antecedent context, not resolver inputs; parser resolver state, legacy graph,
  Engine, filesystem, and text scans are excluded.
- **Produces:** bounded `InterfaceImportEnvironment` binding facts with explicit/glob precedence
  and preserved defining identities for a later separately owned binder integration.
- **Downstream owner:** parser and binder integration must add parsed import/visibility semantics;
  TASK-2062 owns lowering and TASK-2064 owns conformance. Engine scanner fencing/transport remains
  separately owned.
- **Non-goals:** dynamic imports, runtime capability authority, cross-module initialization, LSP
  workspace indexing, TypeEnv finalization, authoritative full interfaces, and Engine scanner
  authority.

## Files and verification

**Files:** `crates/ash-typeck/src/interface_import_resolver.rs`,
`crates/ash-typeck/tests/task_2061_interface_import_resolution.rs`.

```text
cargo test -p ash-typeck --test task_2061_interface_import_resolution
cargo test -p ash-typeck
cargo clippy -p ash-typeck --all-targets -- -D warnings
cargo fmt --check
cargo doc -p ash-typeck --no-deps
```
