# TASK-2063: Engine Linked-Module Admission

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 8-9; PLAN-203
**Owned rule:** MOD-REAL-006
**Run-route impact:** active

## Description

Link every module artifact reachable from a checked `fn main` entry, then admit and execute only the resulting Engine-owned artifact. The Engine must reject absent, failed, forged, or stale module artifacts and must never fall back to source scanning or direct evaluation.

## Dependencies

- 📝 TASK-2062 — module-aware checked Core/CPS artifacts.
- ✅ PLAN-203 shared Engine execution contract.

## Current → target

**Current files:** `crates/ash-engine/src/module_loader.rs`, Engine entry/admission modules, CLI/daemon adapters covered by PLAN-203.

**Current state:** selected module-file checks and bounded entry paths coexist with loader-specific source/summary behavior.

**Target state:** the admission request carries one linked root artifact and its verified module dependency closure. Engine execution consumes it through the same checked CPS executor as other admitted programs.

## Requirements

1. Implement deterministic reachable-module linking from canonical entry identity.
2. Verify interface versions, dependency identities, source/digest provenance, and failed/incomplete status before admission.
3. Reject every missing or malformed linked dependency with an anchored module diagnostic.
4. Keep provider/handler authority separate: linked modules transport requirements but do not install frames.
5. Prove that entry execution cannot select a direct evaluator, raw import loader, or alternative module path.

## TDD Steps and evidence

1. Build a multi-module `fn main` fixture with file and inline variants and a selected normal terminal result.
2. Add tampered artifact, missing child, stale digest, failed dependency, private-import, and unadmitted-entry controls.
3. Add a negative test that would pass only if a direct evaluator or raw-source fallback remained; it must reject.
4. Verify entry module, imported module, and diagnostic origins appear in the admitted artifact/terminal envelope as defined by existing owners.

## Completion checklist

- [ ] Admission requires a complete linked reachable module closure.
- [ ] Missing, stale, malformed, or failed module artifacts reject before execution.
- [ ] No direct evaluator or raw-source fallback remains reachable.
- [ ] Focused Engine admission tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** linked Core/CPS module artifacts from TASK-2062 and PLAN-203 admitted-program seam.
- **Produces:** one Engine-owned linked module request and terminal evidence for TASK-2064.
- **Downstream owner:** TASK-2064 compares the same request through CLI and daemon.
- **Non-goals:** daemon transport redesign, package cache persistence, runtime module values, or generic dynamic linking.

## Files and verification

**Files:** `crates/ash-engine/src/module_loader.rs`, Engine entry/admission modules, `crates/ash-cli` and daemon adapter test surfaces only as required by existing PLAN-203 seam.

```text
cargo test -p ash-engine module
cargo test -p ash-engine admission
cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
cargo fmt --check
```
