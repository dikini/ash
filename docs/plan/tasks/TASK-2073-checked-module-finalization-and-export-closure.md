# TASK-2073: Checked Module Finalization and Export Closure

**Status:** Complete for the frozen callable-module completion domain
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§6–8 (`M-CHECK`, final export closure)
**Owned rule:** MOD-REAL-003
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2073](../SEMANTIC-RULE-COVERAGE.md#task-2073-checked-module-finalization-and-export-closure)

## Semantic accounting

**Implementation:** implemented
**Evidence:** tested
**Parity:** matches_spec
**Missing target-spec clauses:** None within the frozen finalization/export-closure domain.

The finalizer consumes checker-owned module snapshots and staged import bindings, validates all
public dependency and visibility/export paths, and publishes an atomic export-closed interface.
Every public declaration required by an importer retains canonical identity, defining origin,
visibility, and checked metadata. Callable declarations provide the metadata needed by the
downstream Core/CPS route; non-callable declarations remain importer-visible metadata and are not
standalone runtime entries.

Dedicated Ash role and policy declarations, carriers, and semantics are removed by TASK-2077.
Dynamic module loading is outside this task; static file-backed and inline acquisition remain.
Raw synthesized-pattern compatibility APIs, package/registry resolution, incremental workspaces,
and generalized macro/runtime behavior are also outside this task.

**Layers:** Type `implemented`; Core/CPS/admission-runtime `not_applicable`; verification
`implemented`.
**Evidence:** `crates/ash-typeck/tests/task_2073_checked_module_finalization.rs` passes 104/104,
including public callable/type-bearing dependency closure, visibility-path rejection, staged
re-export validation, constructor and structural-child identity checks, mutation rejection, and
file/inline projection.

## Description

Finalize a module only after its internal declarations and parsed imports have been checked as a
single atomic unit. The final interface is the only input available to downstream lowering and
importing modules; no name-only view or source rescan may supply missing facts.

## Requirements

1. Consume the collected snapshot and staged import result without recovering declarations from
   source text or the provisional name-only view.
2. Validate public defining-module paths, visibility, imported type/callable dependencies,
   constructors, re-exports, structural children, and checked metadata before publication.
3. Reject missing, private, forged, drifted, cyclic, or inconsistent dependencies atomically.
4. Preserve public declaration identity, origin, visibility, and checked metadata for importers.
5. Publish no dedicated role/policy or dynamic-module-loading surface.

## TDD steps and evidence

1. Run the focused finalizer corpus before status changes.
2. Keep positive, negative, mutation, and file/inline projection witnesses for each owned closure
   rule.
3. Run the affected Type/core/Engine tests and repository documentation gates.

## Completion checklist

- [x] Checker-owned snapshots and staged bindings are finalized atomically.
- [x] Public declarations propagate through imports with identity, origin, visibility, and
  checked metadata intact.
- [x] Public dependency, re-export, structural-child, and mutation checks are tested.
- [x] Dedicated role/policy machinery is removed and is not a completion criterion.
- [x] Dynamic module loading is recorded as excluded; static file/inline acquisition remains.
- [x] `CHANGELOG.md`, PLAN-INDEX, semantic coverage, and traceability are updated.

## Handoffs

- **Consumes:** TASK-2075 collected snapshots and TASK-2072 parsed import bindings.
- **Produces:** export-closed checked module interfaces for TASK-2069 and downstream importers.
- **Downstream owner:** TASK-2069 owns Core/CPS lowering and Engine transport; TASK-2063 owns
  Engine sealing/admission; TASK-2064 owns CLI/daemon parity.
- **Non-goals:** runtime authority, dynamic module loading, package resolution, incremental
  workspaces, and generalized macro/runtime behavior.

## Verification

```text
cargo test -p ash-typeck --test task_2073_checked_module_finalization
cargo fmt --check
git diff --check
```
