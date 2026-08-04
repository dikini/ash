# TASK-2066: TypeEnv Module-Unit Interface Finalization

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5-6, §8 (`M-COLLECT`, `M-CHECK`)
**Owned rule:** MOD-REAL-003
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2066 TypeEnv module-unit interface finalization](../SEMANTIC-RULE-COVERAGE.md#task-2066-typeenv-module-unit-interface-finalization)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Checking callable bodies or producing complete callable facts; typed namespace linkage for types, constructors, interfaces, effect rows, and implementations; aliases/re-exports and per-binding source-origin projection; complete export closure and diagnostics; interface-driven imports, visibility, and cycles; module-aware Core/CPS lowering; Engine scanner fencing/transport and linked admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
**Layers:** type `partial`; Core `not_applicable`; CPS `not_applicable`; admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-COLLECTION`; negative `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-KEY-CONTEXT`; mutation `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-DECLARATION-PREFLIGHT`; no proof. Parity is `not_applicable`: this bounded TypeEnv handoff has no paired source, Engine, or client execution relation.
**Next obligation:** TASK-2061 now consumes this bounded `FinalizedModuleInterface` wrapper through a wrapper-only checked store, but its resolver is not parsed import/visibility or full interface authority. Parser/binder integration must add typed linkage, re-export/alias handling, closure, cycles, lowering, Engine transport, and parity without admitting raw Core, parser, Engine, filesystem, legacy graph, or text-scan authority.
**Non-goals:** Treating the wrapper or raw `ash_core::PublicModuleInterface` as an authoritative full interface or import authority; checking bodies or full callable facts; typed namespace linkage, aliases/re-exports, per-binding source-origin projection, complete export closure, imports/visibility/cycles, Core/CPS lowering, Engine transport/admission/execution or scanner fencing, dynamic imports, runtime module values, import-cycle initialization, or client parity.

## Description

Provide a bounded typechecker-owned finalization boundary between parser module units and a future
interface-driven import binder. This task does not change the Core public carrier and does not
publish an authoritative full module interface. It turns one coherent TASK-2059 `ModuleUnit` and
staged TypeEnv declaration-preflight facts into a non-forgeable
`FinalizedModuleInterface` wrapper for a deliberately limited public projection.

## Delivered finalization boundary

`TypeEnvModuleInterfaceCollection::collect` accepts one canonical `ModuleUnit` and a mutable
`TypeEnv`. After structural artifact validation, it clones the environment, claims the module's
canonical `ModuleKey`, prechecks incompatible existing public function/handler markers, and calls
`TypeEnv::register_surface_declarations` on the module definitions. That call is declaration
preflight for public function/handler signatures; it does not check bodies or create complete
callable facts. Its errors are mapped to the task-owned finalization error, then the bounded public
function/handler marker facts are revalidated and the staged environment is committed atomically.

The collection retains parser-visible public child modules, public functions/handlers justified by
the staged declaration markers, and syntax-only public macros. It rejects an incompatible module
key, marker conflict, failed declaration preflight, mismatched artifact/key, or a Core binding not
justified by that bounded collection. Builtins, aliases/re-exports, other typed namespaces, and
per-binding source-origin projection are not collected.

Only the collection can construct `FinalizedModuleInterface`. `finalize` requires full
`ModuleArtifact` equality, not merely equal keys, before it returns the immutable wrapper. The
wrapper is a bounded typechecker handoff, not an authoritative finalized full interface, an import
binder, Engine authority, or runtime authority.

This remains a deliberately partial authority boundary. It does not check bodies, link generic
typed namespaces or complete callable facts, establish re-export/alias provenance or complete
export closure, resolve imports or visibility, lower modules, transport artifacts to Engine, admit
execution, or grant runtime authority.

## Remaining target boundary

The staged preflight is intentionally narrower than production declaration typechecking: it checks
public function/handler declaration signatures only and exposes no body-derived proof, complete
callable fact, typed namespace, alias/re-export, or source-origin binding relation. It therefore
does not establish a complete export closure or install any import/visibility, lowering, Engine,
or runtime authority. Those target clauses remain independently owned and keep this task
`partial / tested / below_spec`.

## Dependencies and handoffs

- **Consumes:** TASK-2059 `ModuleUnit` acquisition; TASK-2058 canonical identity/artifact
  carriers; TASK-2060's bounded Core projection; and staged TypeEnv declaration-preflight facts
  for public function/handler signatures.
- **Produces:** a non-forgeable typechecker `FinalizedModuleInterface` wrapper for the limited
  artifact-equal projection justified by parser and TypeEnv declaration facts.
- **Downstream consumer:** TASK-2061 consumes only this wrapper through its bounded checked store;
  it never consumes a raw `ash_core::PublicModuleInterface` as authority. Parsed imports,
  visibility, aliases/re-exports, typed namespaces, cycles, and binder integration remain separate.
- **Separately owned:** body checking and full callable facts, typed namespace linkage,
  aliases/re-exports, source-origin projection, and complete export closure remain target gaps;
  TASK-2062 owns Core/CPS lowering, TASK-2063 owns linked Engine admission, TASK-2064 owns
  conformance and client parity, and Engine scanner fencing/transport remains separate.

## Requirements

1. Provide a TypeEnv-only finalization operation and a wrapper whose constructor cannot be forged
   outside that collection/finalization path.
2. Require one coherent TASK-2059 module unit and canonical key/artifact, then stage the
   TypeEnv's public function/handler declaration-signature preflight under that key.
3. Reject artifact/key discontinuity, marker conflicts, declaration-preflight failures, or a public
   projection not justified by the bounded collected facts.
4. Preserve the Core carrier as a data input only; do not turn it into import, Engine, or runtime
   authority.
5. Keep body checking, full callable facts, typed namespace linkage, aliases/re-exports,
   source-origin projection, and complete export closure explicit as unsupported target clauses.

## Task-owned evidence

The focused `task_2066_module_interface_finalization` target supplies 11/11 passing tests:

- **Positive:** `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-COLLECTION` exercises staged declaration
  registration and finalization of the matching callable projection.
- **Negative:** `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-KEY-CONTEXT` rejects reuse of the same
  named callable facts from a different canonical module key.
- **Mutation:** `TEST-MOD-REAL-003-TYPEENV-FINALIZATION-DECLARATION-PREFLIGHT` changes a public
  callable signature to an unknown type and rejects collection before the staged environment can
  commit.

The target also covers full artifact mismatch, raw-interface key mismatch, uncollected bindings,
syntax-only macros, child bindings, marker conflicts, immutable wrapper access, and deterministic
finalization. `cargo test -p ash-typeck --lib` passed 477 tests; focused strict clippy, the parser
source-order target, parser lint, docs, formatting, diff, and semantic documentation gates also
passed. No proof or source/Engine/client parity claim is made.

## Completion checklist

- [x] The TypeEnv finalizer stages coherent module-unit, key/artifact, and declaration-preflight
  inputs under one canonical key.
- [x] `FinalizedModuleInterface` cannot be forged from a raw Core public-interface carrier.
- [x] Continuity and bounded public-projection validation have positive, negative, and mutation
  evidence.
- [x] Full typed namespace linkage, body checking/full callable facts, aliases/re-exports,
  source-origin projection, and complete export closure remain explicit target gaps.
- [x] TASK-2061 consumes only this wrapper through its bounded checked store, never raw Core;
  parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, and binder integration
  remain separately owned.

## Verification

```text
cargo test -p ash-typeck --test task_2066_module_interface_finalization
cargo test -p ash-typeck --lib
cargo clippy -p ash-typeck --lib --test task_2066_module_interface_finalization -- -D warnings
cargo test -p ash-parser module_body_from_items_preserves_source_order_and_typed_views
cargo clippy -p ash-parser --lib -- -D warnings
cargo doc -p ash-typeck --no-deps
```
