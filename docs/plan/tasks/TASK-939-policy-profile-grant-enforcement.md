# TASK-939: Policy-profile grant enforcement across runtime execution

## Status: 📝 Planned

## Description

Broaden RuntimeKernel admission from focused provider fallback checks to policy-profile grants that constrain capability/resource/action use across Act, Proc, and Workflow execution.

## Specification Reference

- SPEC-070 A70-6
- SPEC-070 §7 Authority and admission
- NI-4 visible authority/admission boundary

## Dependencies

- TASK-937 completion
- TASK-938 completion

## Requirements

### Functional Requirements

1. Define minimal alpha policy-profile grant model: capability binding, resource binding if available, and allowed action surface.
2. RuntimeKernel admission must produce projected grants before execution.
3. Act capability calls must use the projected grants.
4. Proc child processes must inherit or derive authority only through explicit policy.
5. Reports must record admission/grant facts needed for audit.

Property invariant: a granted provider with an ungranted action fails closed; a child process cannot gain authority absent inherited/derived grant.

## TDD Steps

1. Write RED tests in capability/admission suites.
2. Implement grant model/projection in `crates/ash-core/src/runtime_kernel.rs`, `crates/ash-interp/src/runtime_state.rs`, `crates/ash-interp/src/execute.rs`, and `crates/ash-interp/src/eval.rs` as needed.
3. Verify all existing fail-closed suites.

## Dispatch

```yaml
agent: codex
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

Codex instructions:
- Work in a dedicated worktree.
- Do not spawn nested agents.
- Follow RED-GREEN-REFACTOR for code tasks.
- Keep the task scope narrow; do not implement later tasks early.
- Return exact files changed, focused commands run, and any remaining blockers.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-interp --test task_736_capability_binding_admission -- --nocapture
  - cargo test -p ash-interp --test runtime_action_control -- --nocapture
  - cargo test -p ash-interp --test invoke_runtime_dispatch -- --nocapture
  - CARGO_BUILD_JOBS=1 PROPTEST_CASES=2048 cargo test -p ash-interp --test task_712_par_scatter_child_admission proc_scatter_preserves_input_order -- --test-threads=1 --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task.
  - [ ] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task.
  - [ ] cargo fmt --check passes when Rust code changed.
  - [ ] git diff --check passes.
  - [ ] cargo check --workspace passes if shared carriers or public APIs changed.
  - [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [ ] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [ ] Codex verification reports no blockers.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
