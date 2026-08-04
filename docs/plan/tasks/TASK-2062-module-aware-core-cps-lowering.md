# TASK-2062: Module-Aware Core/CPS Lowering

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 8 (`M-LOWER`, `M-LINK`); SPEC-098c; SPEC-099b
**Owned rule:** MOD-REAL-005
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2062 module-aware Core/CPS lowering](../SEMANTIC-RULE-COVERAGE.md#task-2062-module-aware-corecps-lowering)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Parser/source lowering from a complete ModuleUnit and all reachable definition bodies; full typed imports, callable/type authority, parsed visibility/aliases/re-exports, and import-cycle handling; full type namespace and export-closure validation; file/inline real-program artifact parity; checked dependency-closure linking and Engine-only admission/execution with no direct-evaluator fallback; and CLI/daemon normalized-terminal parity.
**Layers:** type `partial`; Core `partial`; CPS `partial`; admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-005-CORE-CPS-MODULE`; negative `TEST-MOD-REAL-005-UNRESOLVED-IMPORT-LOWER`; mutation `TEST-MOD-REAL-005-IDENTITY-FORGERY`; no proof. Parity is `not_applicable`: the bounded envelope has no real-program source, Engine, or client execution relation.
**Next obligation:** The parser/binder integration owner must supply complete checked definition bodies and typed import facts to a later lowering slice. TASK-2063 must first seal its own dependency-linking/admission input around these non-authoritative public Core/CPS carriers; it cannot treat either carrier as authority. TASK-2064 alone owns real-program file/inline and CLI/daemon parity.
**Non-goals:** Parser/source rediscovery, raw PublicModuleInterface or legacy ModuleGraph authority, parser/source lowering or full definition bodies, typed imports or callable authority, parsed import/visibility/alias/re-export/cycle semantics, complete namespaces or export closure, file/inline real-program parity, Engine linking/admission/execution, direct-evaluator fallback, filesystem or text scans, runtime module values, or CLI/daemon behavior.

## Delivered boundary

`lower_finalized_module_to_core_cps` accepts only a finalizer-issued
`FinalizedModuleInterface`, an `InterfaceImportEnvironment` previously populated by TASK-2061, and
explicit expected local/import defining-identity facts. It snapshots and checks each requested
resolver binding before it validates an already materialized `RawCoreProgram` and delegates only to
the checked Core-to-CPS bridge.

The resulting `ModuleCoreArtifact` and `ModuleCpsArtifact` retain the exact finalizer-owned
`ModuleArtifact`, including canonical module key and origin, plus deterministic cloned import
snapshots containing each binding's defining identity and origin. The snapshots are provenance
metadata only: they do not populate a callable environment, grant runtime authority, admit an
artifact, or execute CPS. No parser, source, filesystem, text scan, legacy graph, Engine, or
client API is consulted by this boundary.

## Type → Core → CPS handoffs

- **Consumes (Type):** TASK-2061's checked environment and finalizer wrapper only, with a caller
  declaring the local aliases and defining identities expected for this lowering request.
- **Produces (Core):** a validated/type-checked `ModuleCoreArtifact` holding the wrapper's exact
  module artifact and deterministic checked-import provenance.
- **Produces (CPS):** a `ModuleCpsArtifact` generated through the existing checked Core-to-CPS
  bridge, retaining the same module artifact and import snapshots without execution authority.
- **Downstream owner:** TASK-2063 alone owns dependency closure linking and admission. It must
  establish a separately sealed input around these non-authoritative public carriers; TASK-2062
  does not issue an admission credential. TASK-2064 alone owns file/inline real-program and
  CLI/daemon parity.

## Task-owned evidence

The focused `task_2062_module_core_cps_lowering` target passes 3/3:

- **Positive:** `TEST-MOD-REAL-005-CORE-CPS-MODULE` retains exact finalizer artifact key/origin and
  an aliased imported declaration's defining identity/origin in both Core and CPS artifacts.
- **Negative:** `TEST-MOD-REAL-005-UNRESOLVED-IMPORT-LOWER` rejects missing and ambiguous requested
  local imports before a deliberately invalid Core program can reach type checking.
- **Mutation:** `TEST-MOD-REAL-005-IDENTITY-FORGERY` substitutes a different defining module in an
  expected import fact and receives `StaleResolvedImportIdentity`, rather than reconstructing an
  identity from the alias.
- **Carrier derivation:** `module_lowering::tests::cps_artifact_derives_metadata_from_its_core_artifact`
  confirms that the CPS carrier copies the Core carrier's exact module artifact and import snapshots.

The independent focused, full `ash-core`, and full `ash-typeck` test runs pass. This is test
evidence for the bounded handoff only; it is not a proof or any source/Engine/client parity claim.

## Remaining target boundary

This handoff starts from an already-materialized `RawCoreProgram`; it does not lower parser source
or module definition bodies. It does not establish full typed imports or callable authority,
parsed aliases/re-exports/visibility/cycles, complete typed namespaces or export closure, file/inline
real-program artifact parity, dependency closure, admission/execution, or CLI/daemon parity. TASK-2063
must seal its own link/admission input around these public non-authoritative carriers. Those omissions
are deliberate and preserve TASK-2063/TASK-2064 ownership; no unavailable or failed later stage may
select a direct-evaluator fallback.

## Completion checklist

- [x] Finalizer wrapper and TASK-2061 resolver facts are the only module/import inputs.
- [x] Core and CPS artifact carriers preserve exact module key/origin and resolved defining
  identity/origin metadata.
- [x] Missing/ambiguous and forged/stale import facts reject before unchecked lowering can publish
  artifacts.
- [x] Focused, `ash-core`, and `ash-typeck` tests; strict clippy; formatting; and diff checks pass.
- [ ] Parser/source lowering, full definitions/typed imports, real-program file/inline parity,
  Engine linking/admission/execution, and client parity remain separately owned.

## Verification notes

`cargo doc -p ash-core -p ash-typeck --no-deps` exits successfully but emits two pre-existing
broken intra-doc-link warnings outside this task's implementation files: `ash_core::core_ash_lower`
references `with_mode_binding_latent_row`, and TASK-2061's `interface_import_resolver` references
`PublicModuleInterface`. They are recorded as warnings, not clean documentation evidence for
TASK-2062.

## Files and verification

**Files:** `crates/ash-core/src/module_lowering.rs`,
`crates/ash-typeck/src/module_core_cps_lowering.rs`, and
`crates/ash-typeck/tests/task_2062_module_core_cps_lowering.rs`.

```text
cargo test -p ash-typeck --test task_2062_module_core_cps_lowering
cargo test -p ash-core
cargo test -p ash-core module_lowering::tests::cps_artifact_derives_metadata_from_its_core_artifact
cargo test -p ash-typeck
cargo clippy -p ash-core -p ash-typeck --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
