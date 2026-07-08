# TASK-1979: Ambient Effect Context

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Finish the Phase 201 cleanup for ambient effect context. The retained effect context must read as
target row/profile typing support, not as a workflow/tower-specific effect category.

## Requirements

- Keep `ambient_effect` only as the target lexical/profile effect context used by current
  expression typing.
- Remove stale active comments or diagnostics that describe the effect context as a workflow
  effect context.
- Remove stale active diagnostics that describe ambient target contract statements as workflow
  contract statements.
- Extend the Phase 201 removal gate so these stale strings cannot re-enter active typechecker
  paths.
- Update task, audit, and changelog evidence.

## TDD Steps

1. Add failing Phase 201 gate rows for workflow-scoped effect/contract wording in active
   typechecker paths.
2. Retarget affected comments, diagnostics, and focused tests to target profile/application
   vocabulary.
3. Run focused ambient-do and pure-closure tests, the Phase 201 removal gate, and affected checks.

## Completion Checklist

- [x] Active typechecker source has no workflow-scoped effect-context wording.
- [x] Ambient target contract diagnostics use target profile vocabulary.
- [x] Focused ambient-do and closure/effect tests pass.
- [x] Phase 201 removal gate blocks stale workflow-scoped effect/contract wording.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

RED:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Failed after adding forbidden-token rows for:

- `Workflow effect context` in `crates/ash-typeck/src/check_expr/mod.rs`;
- `workflow contract statement` in active typechecker source/tests.

GREEN:

```bash
cargo test -p ash-typeck --test task_1841_ambient_do --quiet
cargo test -p ash-typeck task558 --lib --quiet
cargo check -p ash-typeck -p ash-cli --all-targets
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
rg -n "Workflow effect context|workflow contract statement|workflow_effect|set_workflow_effect" \
  crates/ash-typeck/src crates/ash-typeck/tests || true
```

The final scan produced no matches in active typechecker source/tests.

Notes:

- This slice retargeted active effect-context wording and ambient target contract diagnostics.
- Broader runtime `entry_effect` naming remains a target application/runtime metadata concern for
  Phase 201 closeout proof; no workflow-scoped effect carrier remains in the scanned active
  typechecker paths.
