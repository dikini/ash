# TASK-2060: Checked Module Interface and Export Closure

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 7-8
**Owned rule:** MOD-REAL-003
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2060 checked module interface and export closure](../SEMANTIC-RULE-COVERAGE.md#task-2060-checked-module-interface-and-export-closure)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Complete TypeEnv-private/interface collection beyond TASK-2066's staged public function/handler declaration-signature preflight; body/full-callable facts, typed summary-binding linkage, aliases/re-exports, per-binding source origins, and closure finalization; Engine export-scanner retirement or disagreement-only fencing and interface transport; interface-driven import binding and visibility; module-aware Core/CPS lowering; Engine-only linked admission/execution with no direct-evaluator fallback; structural/import-cycle conformance; and CLI/daemon normalized-terminal parity.
**Layers:** type `partial`; Core `partial`; CPS/admission-runtime `not_applicable`; verification
`partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-003-EXPORT-CLOSURE`; negative
`TEST-MOD-REAL-003-PRIVATE-LEAK`; mutation `TEST-MOD-REAL-003-INTERFACE-SCHEMA`; no proof. Parity
is `not_applicable`: this Core carrier has no paired source, Engine, or client execution relation.
**Next obligation:** TASK-2066 now supplies a bounded TypeEnv wrapper after staged declaration-signature preflight and full artifact equality, and TASK-2061 consumes only that wrapper in a bounded checked store. Neither supplies complete private facts, typed summaries, closure, parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, or binder integration. TASK-2062 then owns lowering, TASK-2063 consumes only linked artifacts, TASK-2064 owns conformance/parity, and TASK-2065 closes Phase 207.

**Non-goals:** AST-derived ModuleUnit collection, private typechecking state, typed-summary identity linkage, import/visibility binding, Core/CPS lowering, Engine scanner fencing/transport, Engine admission/execution or a direct-evaluator fallback, dynamic imports, package resolution, runtime module values, structural/import-cycle conformance, or client parity.

## Delivered handoff

TASK-2060 completes a bounded, non-authorizing Core public-interface carrier:

- `PublicModuleInterface` is a durable V1 schema over a canonical `ModuleArtifact`, public
  bindings, schema-versioned dependencies, and an optional compatibility
  `ModuleSemanticSummary` payload.
- `ModuleInterfaceBinding` preserves visible spelling, stable defining identity, checked
  visibility, and diagnostic source origin. Re-exporting changes only the visible name.
- Publication rejects non-public or duplicate visible bindings, child identities absent from the
  artifact, mismatched inline-child parents, unsupported interface/dependency schemas, malformed
  compatibility summaries, forged generic child identities, and unknown wire fields at every
  interface nesting level.
- Generic Core binding identities are deliberately limited to values, callables, and syntax
  macro/notation metadata. Type, constructor, interface, and effect-row identities must continue
  to use their existing semantic-summary carriers; implementation publication is explicitly
  deferred until it has a dedicated checked identity and closure contract.
- Compatibility calls the existing semantic-summary V1--V8 validation contract. It neither adds a
  V9 summary version nor migrates legacy `ModuleIdentity`.
- Syntax macro and notation bindings are syntax metadata only. This carrier has no parser AST,
  callable body, binding environment, Engine cache, provider/handler authority, admission fact, or
  runtime behavior.

This is not yet the language-level checked `PublicInterface` from SPEC-103. The core schema can be
constructed and validated, but it is not collected from parser `ModuleUnit` values, paired with a
private TypeEnv view, or finalized against typed public closure facts.

## Remaining target boundary

TASK-2060 does not collect AST declarations, resolve `use`, maintain a private interface, link a
generic binding to a typed summary identity, or prove that a public signature references only
reachable facts. It does not change `ash-engine` export scanners, raw-source collectors, cache
transport, lowering, admission, execution, or any client route. The existing Engine scanner seams
remain non-authoritative planning debt; this Core handoff neither removes nor fences them.

The next consumer has a bounded handoff: TASK-2061 consumes only TASK-2066's
`FinalizedModuleInterface` wrapper, never an arbitrary `PublicModuleInterface::new` value. It does
not provide parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, binder
integration, complete interface authority, or Engine scanner-fence ownership.

## Task-owned evidence

**Canonical traceability rule:** `SEM-MODULE-REALIZATION-003`, the traceability alias for
`MOD-REAL-003` in SPEC-103. The primary implementation is
`ash_core::module_interface::{PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION, PublicModuleInterface,
ModuleInterfaceBinding, ModuleInterfaceDefiningIdentity}`, fingerprint
`sha256:9a4d8f162c6d51946e619f1109e3ef326b9023888e4332c6b9805b84e2c10b2e`.

| Axis | Traceability witness | Focused evidence |
|---|---|---|
| Positive | `TEST-MOD-REAL-003-EXPORT-CLOSURE` | Public child and declaration bindings retain canonical identity, visibility, and source origin. |
| Negative | `TEST-MOD-REAL-003-PRIVATE-LEAK` | The property target rejects every generated private binding before interface publication. |
| Mutation | `TEST-MOD-REAL-003-INTERFACE-SCHEMA` | Unsupported/malformed cache payloads and forged nested identities reject through the common deserialization boundary. |

`cargo test -p ash-core --test task_2060_public_module_interface` passes 14/14 tests. It includes
alias preservation, duplicate rejection, structural-child and inline-origin validation, generic
typed-identity rejection, private-binding property coverage, syntax-only metadata, strict nested
serde, deterministic normalization, and schema-version rejection. `cargo test -p ash-core` passes
the recorded 222-test Core suite.

## Deferred downstream witness

`TEST-MOD-REAL-003-ENGINE-EXPORT-SCAN` remains deferred. The audited Engine export scanners have
not been removed or fenced as disagreement-only checks, and no test here asserts otherwise. TASK-2066
does not transport interfaces to Engine consumers; a later separately owned Engine-fence change
must update AUDIT-207 before that can happen.

## Description

Provide the Core-owned, versioned public-interface schema required by later checking and lowering
work. It validates the bounded structural/public projection but does not itself implement the
parser-to-TypeEnv collection or full export-closure rule.

## Dependencies

- ✅ TASK-2058 — canonical module artifacts.
- ✅ TASK-2059 — common module-unit route.

## Requirements and closure

1. **Delivered for the Core carrier:** versioned artifact identity, public binding provenance,
   visibility, dependency references, strict durable serde, and V1--V8 summary compatibility.
2. **Delivered for the bounded projection:** public-only and duplicate checks, structural child and
   inline-origin validation, alias identity preservation, and syntax-only macro/notation metadata.
3. **Explicitly partial:** existing typed identities are not recreated. Generic typed namespaces
   reject pending a collector that supplies their existing summary identities; implementation
   bindings also reject pending a dedicated contract.
4. **Deferred:** parser `ModuleUnit` collection, private/provisional interfaces, typed public
   closure validation, imports/visibility, and all Core/CPS/Engine/client work.
5. **Deferred:** every Engine scanner in AUDIT-207 remains outside this Core-only handoff and must
   be removed or fenced before final interface transport.

## Completion checklist

- [x] The bounded public Core carrier validates version, artifact identity, public bindings,
  structural children, aliases, dependencies, and strict wire payloads.
- [x] Focused Core evidence covers positive, negative, mutation, and property cases; full Core,
  formatting, and clippy evidence is recorded below.
- [x] Existing summary schema validation remains V1--V8 only; syntax metadata remains non-runtime.
- [x] TASK-2066 supplies a bounded TypeEnv wrapper; complete private/finalized interface state,
  body/full-callable facts, typed-summary linkage, aliases/re-exports, origins, and closure remain
  separately owned. TASK-2061 consumes only that wrapper in a bounded checked store; parsed
  imports/visibility, aliases/re-exports, typed namespaces, cycles, and binder integration remain
  separate.
- [ ] Engine scanner fencing/transport, import/visibility binding, lowering, admission, and client
  parity remain separately owned.

## Handoffs

- **Consumes:** TASK-2058 `ModuleKey`, `ModuleArtifactOrigin`, and `ModuleArtifact` carriers; an
  optional existing `ModuleSemanticSummary` only through its current V1--V8 validator. Although
  TASK-2059 produces `ModuleUnit`, this task does not yet collect it.
- **Produces:** a non-authorizing Core public-interface schema and validation boundary. It publishes
  no parser-derived final interface, TypeEnv binding fact, Core/CPS artifact, Engine frame,
  admission authority, or runtime capability.
- **Bounded downstream handoff:** TASK-2066 now turns one `ModuleUnit` plus staged declaration
  preflight into a bounded wrapper. TASK-2061 consumes only that wrapper in a bounded checked store;
  raw Core-constructor values are not import authority. Complete private facts, typed summaries,
  aliases/re-exports, origins, closure, parsed imports/visibility, typed namespaces, cycles, and
  binder integration remain separate work.
- **Downstream owners:** TASK-2062 owns module-aware Core/CPS lowering; TASK-2063 owns Engine-only
  linked admission; TASK-2064 owns structural/import-cycle conformance and CLI/daemon parity;
  TASK-2065 owns closeout. A separate Engine scanner-fence/transport activation remains required.
- **Run-route impact:** prerequisite. This carrier does not make a CLI or daemon route runnable and
  cannot authorize a direct-evaluator fallback.

## Verification

```text
cargo test -p ash-core --test task_2060_public_module_interface
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json
python3 -m unittest tools.docs.tests.test_validate_semantic_task_records
python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```
