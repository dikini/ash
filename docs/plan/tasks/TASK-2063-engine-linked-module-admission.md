# TASK-2063: Engine Linked-Module Admission

**Status:** Complete for the frozen callable-module completion domain
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§8-9 (`M-LINK` and linked-entry admission); PLAN-203
**Owned rule:** MOD-REAL-006
**Run-route impact:** active
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2063 Engine linked-module admission](../SEMANTIC-RULE-COVERAGE.md#task-2063-engine-linked-module-admission)

## Semantic accounting

**Implementation:** implemented
**Evidence:** tested
**Parity:** matches_spec
**Missing target-spec clauses:** None within the frozen Engine linked-module admission domain.
  Dynamic loading and broader runtime features remain outside Phase 207; CLI/daemon terminal parity
  is evidenced by TASK-2064.
**Layers:** type `not_applicable`; Core `implemented`; CPS `implemented`;
  admission-runtime `implemented`; verification `implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-006-LINKED-CLOSURE`; negative
`TEST-MOD-REAL-006-MISSING-FAILED-CLOSURE`, `TEST-MOD-REAL-006-MALFORMED-CPS`, and
`TEST-MOD-REAL-006-ROOT-ENTRY-METADATA-FENCE`, and
`TEST-MOD-REAL-006-SELECTED-LOCAL-CLOSURE-FENCE`, and
`TEST-MOD-REAL-006-SELECTED-PARAMETER-CLOSURE-FENCE`; mutation
`TEST-MOD-REAL-006-PROVENANCE-MUTATION`; authority fence
`TEST-MOD-REAL-006-PROVIDER-AUTHORITY-FENCE`; non-callable invocation fence
`TEST-MOD-REAL-006-NON-CALLABLE-IMPORT-INVOCATION-FENCE`. The focused target passes 9/9 and executes its
root only through the existing checked-CPS terminal route. The bounded linker now resolves
checked imported callable entries by defining identity, selected entry name, and parameter
metadata, including per-function local closure entries in a multi-callable provider; TASK-2064
supplies the real file/inline and CLI/daemon witness. The frozen-domain parity requirement is
therefore complete; broader runtime and dynamic-loading parity is outside Phase 207.
The checked transport additionally rejects non-empty selected-entry metadata that is absent from
the supplied checker-lowered local callable closure, and rejects selected parameter metadata that
disagrees with the matched local callable entry. It also rejects any local closure entry whose
namespace kind is not callable before linking can inspect or substitute its CPS body.
**Next obligation:** None within Phase 207. The Engine-sealed canonical closure remains the only
admission input; raw-source, loader, direct-evaluator, and provider-authority alternatives remain
rejected.
**Non-goals:** Treating TASK-2062 public Core/CPS carriers as sealed authority; raw/source or legacy ModuleGraph/module-loader import authority; parser/source rediscovery, text scans, or filesystem walking; direct-evaluator or alternate execution paths; provider/handler frame authority; dynamic imports, package/cache persistence, runtime module values, or CLI/daemon parity.

## Description

TASK-2063 is activated for an Engine-owned boundary that must link the complete reachable closure
supplied by TASK-2069, mint a separately sealed linked/admission request, and hand that request to
the existing Engine-only execution route. TASK-2069's transport closure is data, not an admission
credential; this task must not treat it as authority before it has verified and sealed the closure.
The bounded linked request now exists, including checked selected-entry inlining for resolved
cross-module callable calls, including binding finalized arguments into non-root selected callable
bodies before root validation. Real-program file/inline and CLI/daemon parity is supplied by
TASK-2064; broader runtime and dynamic-loading behavior is outside Phase 207.

## Dependencies

- ✅ TASK-2069 — complete checked definition-body Core/CPS closure plus canonical Engine
  scanner/cache transport fencing. This is the immediate prerequisite.
- ✅ TASK-2062 — bounded provenance-carrier handoff retained as migration/comparison evidence; it
  is not a complete admission input.
- ✅ PLAN-203 shared Engine execution contract.

## Current → target

**Current source paths:** TASK-2069 supplies the complete non-authoritative checked closure from
the module lowering and Engine transport boundary. The Engine boundary paths are
`crates/ash-engine/src/checked_cps_admission.rs`,
`crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/entry.rs`,
`crates/ash-engine/src/module_loader.rs`, and `crates/ash-engine/src/runtime_artifact.rs`.

**Current state:** the canonical checked closure has a separate Engine seal and checked-CPS
execution route; loader-specific source/summary behavior cannot authorize this route.

**Target state:** achieved. One Engine-sealed request carries a linked root artifact and verified
reachable module dependency closure, and the Engine-only route consumes it through the checked CPS
executor. TASK-2064 supplies the real-program file/inline and CLI/daemon terminal parity.

## Requirements

1. Define deterministic reachable-module linking from TASK-2069's canonical checked entry
   identity and complete non-sealed closure.
2. Verify module-artifact/interface versions, dependency identities, origins, and failed/incomplete
   status before minting the sealed request.
3. Reject every missing, stale, forged, malformed, or failed linked dependency before execution
   with a diagnostic owned by the Engine boundary.
4. Keep provider/handler authority separate: linked modules transport requirements but do not
   install frames.
5. Route only the Engine-sealed request to the checked CPS executor; no direct evaluator,
   raw-source loader, or alternative module route may be selected.

## TDD Steps and evidence

1. Add `task_2063_engine_linked_module_admission` with a bounded linked-program positive,
   missing/failed dependency negative, and forged/stale closure mutation test.
2. Add a negative control that would pass only if a direct evaluator, raw source, or legacy loader
   remained reachable; it must reject before execution. The focused target now covers malformed CPS
   and provider-authority rejection as this control.
3. Keep file/inline real-program and client terminal comparison out of this task; TASK-2064 owns
   that parity evidence.

The focused target passes 9/9: complete shuffled closure execution, missing and stale dependency rejection,
failed dependency rejection, Core/CPS provenance mutation rejection, malformed CPS rejection,
provider-authority rejection without frame installation, non-callable metadata import invocation
rejection, and root selected-entry metadata rejection.

## Completion checklist

- [x] Admission requires a complete linked reachable module closure.
- [x] Missing, stale, malformed, or failed module artifacts reject before execution in the
  bounded Engine transport/admission route.
- [x] The linked route has no direct evaluator or raw-source fallback and rejects provider/handler
  CPS authority.
- [x] Engine admission rejects a root transport input without a non-empty selected checked-entry
  identity before sealing an execution route.
- [x] Focused Engine admission tests, fmt, and clippy pass.
- [x] Real file/inline linked-program and CLI/daemon parity is supplied by TASK-2064's 59/59
  conformance corpus.

## Remaining target boundary

The bounded implementation supplies an Engine-linked request, admission implementation, focused
test evidence, and a canonical terminal result. The parser/source, typed import, visibility,
re-export, export-closure, and real file/inline parity clauses listed in the older boundary
description are now supplied by the upstream/following frozen-route owners. The only remaining
boundary is the explicit follow-on set: dynamic loading/package resolution, generalized runtime,
and other non-frozen language forms. The linked route rejects raw/source/legacy-loader/direct-
evaluator authority and must remain fail-closed.

## Handoffs

- **Run-route impact:** `active`; the bounded linked closure reaches the Engine-only checked-CPS
  route, and TASK-2064 supplies the real file/inline and client parity evidence.
- **Consumes:** only TASK-2069's complete but non-sealed canonical-keyed checked Core/CPS closure
  and the existing PLAN-203 Engine-only admitted-program seam; it must not consume raw
  source, a parser graph, a legacy module graph, or a loader-private export table as authority.
- **Produces:** a separately Engine-sealed linked/admission request for one complete reachable
  checked dependency closure. **Current status:** bounded closure produced and tested; real-program
  parity is evidenced by TASK-2064.
- **Downstream owner:** TASK-2064 compares the same admitted real program through CLI and daemon.
- **Integration/proof responsibility:** TASK-2063 owns focused link/admission and rejection
  evidence; TASK-2064 owns file/inline and client normalized-terminal parity.
- **Non-goals:** Treating public carriers as sealed authority; daemon transport redesign, package
  cache persistence, runtime module values, generic dynamic linking, parser/source rediscovery,
  or direct-evaluator/alternate-route execution.

## Files and verification

**Implemented source paths:** `crates/ash-engine/src/checked_cps_admission.rs`,
`crates/ash-engine/src/lib.rs`, and `crates/ash-engine/src/module_transport.rs`; the focused
contract is in `crates/ash-engine/tests/task_2063_engine_linked_module_admission.rs`.
TASK-2069's complete lowering/transport input is consumed as non-authorizing data and revalidated
before this task mints the Engine seal.

```text
cargo test -p ash-engine --test task_2063_engine_linked_module_admission
cargo test -p ash-engine --lib module_transport::tests::local_callable_identity_rejects_a_forged_non_callable_namespace_kind
cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
cargo fmt --check
```

The focused target passes 9/9; workspace check, formatting, diff hygiene, orientation indexes,
semantic traceability, and the docs gate pass. An older package-suite note recorded six
effect-row/domain-summary failures outside this linked route. It is historical, not a TASK-2063
blocker; current workspace verification is green.
