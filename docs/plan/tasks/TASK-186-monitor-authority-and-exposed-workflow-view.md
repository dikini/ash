# TASK-186: Monitor Authority and Exposed Workflow View

## Status: ✅ Complete

## Description

Define the canonical monitor authority surface for spawned workflows, including an explicit
`exposes { ... }` workflow clause, a first-class `MonitorLink`, and the observable monitor view
that downstream Rust and Lean work must preserve.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-017: Capability Integration
- SPEC-020: ADT Types
- SPEC-021: Runtime Observable Behavior

## Requirements

### Functional Requirements

1. Add an explicit `exposes { ... }` clause to the workflow surface contract
2. Make `MonitorLink<W>` a first-class authority distinct from `InstanceAddr<W>` and `ControlLink<W>`
3. Extend the spawned-instance contract so `Instance` carries `Option<MonitorLink<W>>`
4. Define the exposed monitor view as read-only, policy-gated, and separate from control and messaging
5. Allow exposed monitor metadata such as `monitor_count` without introducing a monitor-specific policy sublanguage
6. Keep monitor grant/delegation explicit and atomic

## TDD Evidence

### Red

Before this task, the canonical workflow surface had `observes`, `receives`, `sets`, and `sends`,
but no explicit monitor view. Spawned-instance contracts still exposed only `InstanceAddr` and
`ControlLink`, and the runtime-observable contract had no first-class `MonitorLink` or exposed view.

### Green

The monitor-authority contract is now explicit:

- [SPEC-002](../../spec/SPEC-002-SURFACE.md) defines `exposes { ... }` as the workflow's
  externally monitorable view
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md) treats `MonitorLink<W>` as a first-class authority
  alongside `InstanceAddr<W>` and `ControlLink<W>`
- [SPEC-021](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) makes the exposed monitor view
  observable, including monitor metadata such as `monitor_count`
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md) keeps monitor governance inside the
  existing policy machinery rather than introducing a monitor-specific policy sublanguage
- the monitor view remains read-only and distinct from control or message-send authority

### Follow-up Clarification

- `SPEC-002` now uses `workflow_obligation_ref` for live workflow obligation state symbols inside
  `exposes { obligations: [...] }`, so the exposed contract is no longer conflated with role-level
  deontic obligation syntax
- `MonitorLink` sharing is non-consuming by default and is distinct from control-link transfer;
  the exposed monitor view remains readable without making monitor authority linear or affine
- `PLAN-INDEX.md` keeps TASK-186 traceable as a monitoring gate without renumbering the later
  convergence phases

## Files

- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no workflow-level monitor view declaration,
- no first-class monitor authority type,
- no canonical exposed-view contract,
- no explicit monitor-grant atomicity or separation from control/messaging.

### Step 2: Verify RED

Expected failure conditions:
- the canonical specs still only expose `InstanceAddr` and `ControlLink` for spawned instances.

### Step 3: Implement the minimal spec fix (Green)

Add only the workflow surface clause and the matching authority/value-shape contracts needed to
make monitor views explicit and observable.

### Step 4: Verify GREEN

Expected pass conditions:
- `exposes { ... }` is canonical,
- `MonitorLink<W>` is first-class,
- exposed monitor state is observable but not conflated with control or messaging.

### Step 5: Commit

```bash
git add docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-020-ADT-TYPES.md docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-186-monitor-authority-and-exposed-workflow-view.md
git commit -m "docs: add monitor authority and exposed workflow view"
```

## Completion Checklist

- [x] monitor authority and exposed view documented
- [x] `MonitorLink<W>` added to spawned-instance contract
- [x] canonical `exposes { ... }` surface clause added
- [x] `CHANGELOG.md` updated

## Non-goals

- No runtime implementation changes
- No new policy sublanguage

## Dependencies

- Depends on: TASK-181, TASK-182, TASK-183, TASK-184, TASK-185
- Blocks: TASK-164 through TASK-176
