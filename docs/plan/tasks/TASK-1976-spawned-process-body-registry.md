# TASK-1976: Spawned Process Body Registry

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Refactor the runtime child-entry registry so spawned child execution is modeled as spawned process
body lookup, not as a renamed child workflow or separate entry category.

## Requirements

- Rename active runtime and engine APIs away from child-entry terminology.
- Store registered spawned process bodies keyed by spawn `entry_type`.
- Preserve current spawn execution behavior, including `init` binding and control-link handling.
- Tighten the Phase 201 removal gate so stale child-entry registry names cannot re-enter active
  runtime/engine code.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add or retarget failing gate rows for active child-entry registry terminology.
2. Refactor runtime state, engine helpers, and focused tests to spawned-process body vocabulary.
3. Verify spawn execution and runtime control tests still pass.
4. Run the Phase 201 removal gate and affected crate checks.

## Completion Checklist

- [x] Runtime registry APIs use spawned-process body vocabulary.
- [x] Spawn execution still runs registered bodies with `init` and control bindings.
- [x] Engine integration helpers no longer expose child-entry registry vocabulary.
- [x] Phase 201 removal gate blocks reintroducing stale child-entry registry names.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

- `cargo check -p ash-interp -p ash-engine -p ash-cli --all-targets`
- `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
- `cargo test -p ash-interp spawned_process_body -- --nocapture`
- `cargo test -p ash-interp --test act_env_runtime_boundary function_body_call_inherits_hidden_runtime_act_env -- --nocapture`
- `cargo test -p ash-interp --test runtime_action_control -- --nocapture`
- `cargo test -p ash-interp --test runtime_boundary_visibility -- --nocapture`
- `rg -n "child_entries|register_child_entry|child_entry\\(|run_spawned_child_entry|child-entry|child workflow|spawned child workflow" crates/ash-interp/src crates/ash-engine/src crates/ash-interp/tests crates/ash-engine/tests`
