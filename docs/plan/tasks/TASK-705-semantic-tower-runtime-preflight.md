# TASK-705: Semantic tower runtime preflight

## Status: 🟠 Preflight recorded; baseline drift blocks completion

## Description

Perform the implementation preflight for PLAN-098 before any runtime code is changed.

## Specification Reference

- SPEC-047
- SPEC-048
- SPEC-049
- SPEC-050
- SPEC-051

## Dependencies

- None

## Requirements

### Functional Requirements

1. Confirm Phase 97 Act tasks required by the selected slice are complete or explicitly not required.
2. Inventory live parser/typechecker/runtime surfaces that changed since PLAN-098 was written.
3. Check for stale `yield`, `panic`, `Workflow::Spawn`, and `ControlLink` behavior that could be confused with PLAN-098 semantics.
4. Record any semantic fork as a plan/task amendment instead of implementation.

### Property Requirements (proptest)

No property tests are expected for this preflight-only task. If preflight discovers implementation drift, create or amend the downstream implementation task that should own the tests.

## Preflight Steps

### Step 1: Inspect Live Code

Re-run prerequisite discovery against the current branch before choosing an implementation task.

### Step 2: Verify Phase 97 Dependencies

Check whether the selected PLAN-098 slice depends on completed Act substrate work; if it does, verify the relevant Phase 97 tasks first.

### Step 3: Document Drift or Forks

If code/spec drift or a semantic fork is found, amend PLAN-098 and affected task files before implementation.

### Step 4: No Runtime Implementation

Do not implement runtime behavior in this preflight task. Its output is verified readiness or an explicit plan/task amendment.

## Verification Steps

- [x] No runtime implementation is performed in this task.
- [x] PLAN-098 dependency assumptions are validated against live code.
- [x] Any changed prerequisite order is documented in PLAN-INDEX and task dependencies.
- [x] Baseline `cargo fmt --check` was attempted; pre-existing formatting drift is recorded below.
- [x] Baseline `cargo test --all` was sampled through the known failing Act module-resolution test; the pre-existing failure is recorded below.
- [ ] `cargo clippy --all-targets --all-features` passes cleanly; not run because baseline fmt/test gates are already failing outside TASK-705 scope.

## Dependencies for Next Task

This task outputs the preflight readiness decision and any plan/task amendments needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.


## Preflight Findings (2026-04-24)

TASK-705 performed a read-only substrate survey before Phase 98 runtime implementation. No runtime implementation was performed.

### Readiness decision

TASK-706 may proceed if it stays limited to runtime identity and structured failure carrier skeletons. It does not depend on completed Phase 97 expression-level `Act` execution or `from_act` behavior.

Act-dependent Phase 98 work remains deferred until the relevant Phase 97 Act substrate is implemented and verified. In particular, downstream work must not assume `from_act`, expression-level `Act` execution, or public Proc combinators over Act unless the selected task explicitly avoids that dependency.

### Live substrate inventory

- `WorkflowId` exists, but no live `RunId`, `ProcessId`, or internal `BranchId` carrier was found.
- No live structured `OperationalFailure`, `ProcessFailure`, aggregate child-process failure, handle-consumed failure, `WorkflowFailure`, or `WorkflowReport` carrier was found.
- Existing `Workflow::Yield` is workflow/proxy suspension, not `yield : Proc<Unit>`.
- Existing `Workflow::Spawn`, `Instance`, `InstanceAddr`, and `ControlLink` model workflow-instance/control authority, not affine `P<A>` result-observation handles.
- Existing `Context::extend()` is cloneable lexical/runtime context extension, not the SPEC-049 component-wise child process environment projection model.
- Existing parser/typechecker/core surfaces have no implemented `Proc`, `P`, `fail`, `with_error`, process `yield`, `await`, `join`, or `gather` support. `Type::Constructor` can likely host `Proc`/`P` first, matching PLAN-098 D2.
- Existing surface `panic` remains a separate adjacent feature: it is parsed and treated bottom-like in typechecking, but lowering rejects it. TASK-708 must keep `panic` distinct from SPEC-050 operational `fail`.

### Phase 97 / Act dependency status

Phase 97 and TASK-672 through TASK-691 are still marked Ready in PLAN-INDEX in this worktree. The survey found no completed expression-level Act substrate carriers such as `ActBlock`, `ActStmt`, `ActEnv`, `Act` type constructor registration, or `invoke` as `Act<Value>`. This does not block TASK-706, but it blocks or constrains Act-dependent later slices.

### Baseline verification drift

The preflight intentionally did not repair unrelated baseline drift. Current baseline issues observed in this worktree:

- `cargo fmt --check` reports formatting drift in existing files including `crates/ash-cli/src/commands/run.rs`, `crates/ash-engine/src/module_loader.rs`, `crates/ash-engine/src/providers/http.rs`, `crates/ash-engine/src/providers/process.rs`, `crates/ash-engine/src/providers/time.rs`, and several `ash-engine` tests.
- Targeted test command `cargo test --all --test module_resolution stdlib_act_module_resolution -- --exact` fails in `ash-engine --test module_resolution`: `stdlib_act_module_resolution` reports `Parse("item 'unit' not found in module 'act'")`.
- Full clippy was not run because the read-only baseline already has fmt/test failures outside this task's scope.

These baseline failures prevent claiming the whole workspace gate is green, but they do not expose a semantic fork in PLAN-098 and do not block TASK-706 if TASK-706 remains carrier/substrate-only.
