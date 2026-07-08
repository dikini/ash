# TASK-1982: Stale Compatibility Tests

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Rewrite tests whose retained purpose or assertion messages still describe old workflow/tower
compatibility under target-shaped names. Tests should assert target primitives, current bridge
behavior, profile boundaries, or be deleted when their only value is preserving removed semantics.

## Requirements

- Remove stale `still_*workflow*` and workflow-context compatibility labels from active tests.
- Retain coverage that proves current computation/profile behavior remains correct.
- Extend the Phase 201 removal gate so stale compatibility labels cannot re-enter active tests.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add failing Phase 201 gate rows for stale compatibility labels in active tests.
2. Retarget affected test names and assertion messages to current computation/profile vocabulary.
3. Run focused tests, the Phase 201 removal gate, formatting, and docs checks.

## Completion Checklist

- [x] Active tests no longer encode old workflow/tower compatibility intent in names/messages.
- [x] Current computation/profile behavior coverage remains green.
- [x] Phase 201 gate blocks the stale labels.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

RED:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Failed after adding rows for stale labels:

- `behavior_still` in `task_803_phase110_non_interference.rs`;
- `do_workflow_still` in `task_909_act_proc_workflow_bridge_non_interference.rs`;
- `workflow-context` / `workflow contexts` in `task_959_pure_closure_arrow.rs`.

Focused compatibility-suite audit:

```bash
cargo test -p ash-typeck --test task_803_phase110_non_interference --quiet
cargo test -p ash-typeck --test task_909_act_proc_workflow_bridge_non_interference --quiet
```

Both suites failed because they asserted implicit Act/Proc/Workflow bridge behavior without
explicit `Monad<K>` evidence. Those suites were deleted as compatibility-only tests. The remaining
closure boundary coverage was retargeted to ambient profile wording.

GREEN:

```bash
cargo test -p ash-typeck --test task_959_pure_closure_arrow --quiet
cargo check -p ash-typeck -p ash-cli --all-targets
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
rg -n "behavior_still|do_workflow_still|workflow-context|workflow contexts|phase109_do_.*still|bridge_targets_resolve_without_source_declared_monad_interface" \
  crates/ash-typeck/tests || true
```

The final scan produced no matches in active typechecker tests.
