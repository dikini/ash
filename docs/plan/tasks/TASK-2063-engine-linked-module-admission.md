# TASK-2063: Engine Linked-Module Admission

**Status:** In progress
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§8-9 (`M-LINK` and linked-entry admission); PLAN-203
**Owned rule:** MOD-REAL-006
**Run-route impact:** active
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2063 Engine linked-module admission](../SEMANTIC-RULE-COVERAGE.md#task-2063-engine-linked-module-admission)

## Semantic accounting

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** An Engine-sealed linked/admission request over the complete reachable checked Core/CPS dependency closure; canonical dependency identity/version/origin validation; rejection of missing, incomplete, stale, forged, or failed entries before execution; Engine-only consumption with no raw/source/direct-evaluator or alternate module path; and one admitted real program for TASK-2064 file/inline and CLI/daemon normalized-terminal parity.
**Layers:** type `not_applicable`; Core `partial`; CPS `partial`; admission-runtime `not_implemented`; verification `not_implemented`.
**Evidence identifiers:** none are claimed. Future positive, negative, and mutation evidence must be allocated only with the focused implementation test; parity is `not_applicable` until an admitted real program exists.
**Next obligation:** After TASK-2069 supplies the complete non-sealed canonical-keyed Core/CPS
closure, implement one Engine-sealed linked/admission request that consumes only a complete checked
dependency closure, rejects missing, incomplete, stale, forged, or failed entries before execution,
and supplies that admitted request to TASK-2064; no unavailable link/admission stage may select
raw-source, loader, or direct-evaluator authority.
**Non-goals:** Treating TASK-2062 public Core/CPS carriers as sealed authority; raw/source or legacy ModuleGraph/module-loader import authority; parser/source rediscovery, text scans, or filesystem walking; direct-evaluator or alternate execution paths; provider/handler frame authority; dynamic imports, package/cache persistence, runtime module values, or CLI/daemon parity.

## Description

TASK-2063 is activated for an Engine-owned boundary that must link the complete reachable closure
supplied by TASK-2069, mint a separately sealed linked/admission request, and hand that request to
the existing Engine-only execution route. TASK-2069's transport closure is data, not an admission
credential; this task must not treat it as authority before it has verified and sealed the closure.
No linked request or Engine admission exists yet.

## Dependencies

- 📝 TASK-2069 — complete checked definition-body Core/CPS closure plus canonical Engine
  scanner/cache transport fencing. This is the immediate prerequisite.
- ✅ TASK-2062 — bounded provenance-carrier handoff retained as migration/comparison evidence; it
  is not a complete admission input.
- ✅ PLAN-203 shared Engine execution contract.

## Current → target

**Current source paths:** TASK-2069 will supply the complete non-authoritative checked closure from
the module lowering and Engine transport boundary. The candidate Engine boundary paths are
`crates/ash-engine/src/checked_cps_admission.rs`,
`crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/entry.rs`,
`crates/ash-engine/src/module_loader.rs`, and `crates/ash-engine/src/runtime_artifact.rs`.

**Current state:** selected module-file checks and bounded entry paths coexist with loader-specific source/summary behavior.

**Target state:** one Engine-sealed request carries a linked root artifact and verified reachable
module dependency closure. The Engine-only route consumes that sealed request through the checked
CPS executor. TASK-2064 separately proves real-program file/inline and CLI/daemon terminal parity.

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

1. After TASK-2069 completes, first add `task_2063_engine_linked_module_admission` with a bounded linked-program positive,
   missing/failed dependency negative, and forged/stale closure mutation test.
2. Add a negative control that would pass only if a direct evaluator, raw source, or legacy loader
   remained reachable; it must reject before execution.
3. Keep file/inline real-program and client terminal comparison out of this task; TASK-2064 owns
   that parity evidence.

## Completion checklist

- [ ] Admission requires a complete linked reachable module closure.
- [ ] Missing, stale, malformed, or failed module artifacts reject before execution.
- [ ] No direct evaluator or raw-source fallback remains reachable.
- [ ] Focused Engine admission tests, fmt, and clippy pass.

## Remaining target boundary

This activation adds no Engine-linked request, admission implementation, test evidence, source
fingerprint, or terminal result. It does not supply parser/source lowering, full typed imports or
callable authority, parsed visibility/aliases/re-exports/cycles, complete export closure, real
file/inline program parity, or client parity. Until a later implementation validates and seals the
complete checked dependency closure, every raw/source/legacy-loader/direct-evaluator path remains
non-authoritative and must fail closed.

## Handoffs

- **Run-route impact:** `active` once the Engine-sealed request is implemented; this activation
  changes no active route.
- **Consumes:** only TASK-2069's complete but non-sealed canonical-keyed checked Core/CPS closure
  and the existing PLAN-203 Engine-only admitted-program seam; it must not consume raw
  source, a parser graph, a legacy module graph, or a loader-private export table as authority.
- **Produces:** a separately Engine-sealed linked/admission request for one complete reachable
  checked dependency closure. **Current status:** not produced.
- **Downstream owner:** TASK-2064 compares the same admitted real program through CLI and daemon.
- **Integration/proof responsibility:** TASK-2063 owns focused link/admission and rejection
  evidence; TASK-2064 owns file/inline and client normalized-terminal parity.
- **Non-goals:** Treating public carriers as sealed authority; daemon transport redesign, package
  cache persistence, runtime module values, generic dynamic linking, parser/source rediscovery,
  or direct-evaluator/alternate-route execution.

## Files and verification

**Candidate source paths:** `crates/ash-engine/src/checked_cps_admission.rs`,
`crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/entry.rs`,
`crates/ash-engine/src/module_loader.rs`, and `crates/ash-engine/src/runtime_artifact.rs`;
TASK-2069's complete lowering/transport input replaces the bounded TASK-2062-only input. No source
file is modified by this activation.

```text
cargo test -p ash-engine --test task_2063_engine_linked_module_admission
cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
cargo fmt --check
```
