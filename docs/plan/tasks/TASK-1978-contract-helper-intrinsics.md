# TASK-1978: Contract Helper Intrinsics

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Retarget compiler-known contract helper intrinsics away from workflow-scoped names. Target Ash
contracts should be represented as contract/evidence helpers over checked target functions, not as
`workflow::requires` or `workflow::ensures` operations.

## Requirements

- Replace active `workflow::requires` and `workflow::ensures` intrinsic identities with
  `contract::requires` and `contract::ensures`.
- Keep raw requirement/postcondition classification behavior for current contract helper calls.
- Update manifest and misuse tests to assert contract helper identity.
- Tighten the Phase 201 removal gate so workflow-scoped contract intrinsic names cannot re-enter
  active typechecker code and tests.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add failing Phase 201 gate rows for `workflow::requires` and `workflow::ensures` in active
   typechecker paths.
2. Retarget type environment intrinsic registration and descriptors to `contract::*`.
3. Update focused manifest, elaboration, and misuse tests.
4. Run focused typechecker tests, Phase 201 removal gate, and affected crate checks.

## Completion Checklist

- [x] TypeEnv registers `contract::requires` and `contract::ensures`.
- [x] Public computation manifest reports contract helper identities, not workflow helper names.
- [x] Focused tests prove standalone contract helpers are not ordinary first-class calls.
- [x] Phase 201 removal gate blocks workflow-scoped contract intrinsic names.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

RED:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Failed after adding forbidden-token rows for `workflow::requires` and `workflow::ensures` in active
typechecker source and test paths.

GREEN:

```bash
cargo test -p ash-typeck --test task_778_workflow_contract_intrinsic_misuse --quiet
cargo check -p ash-typeck -p ash-cli --all-targets
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Notes:

- `crates/ash-typeck/tests/task_773_workflow_algebra_calls.rs` still has unrelated failing
  `do:Workflow` cases because current `TypeEnv::with_builtin_types()` lacks explicit
  `Monad<Workflow>` evidence; the same failure reproduces in
  `task_772_workflow_do::do_workflow_return_has_workflow_type_and_unit_form_artifact`.
- This task did not fix that broader workflow-algebra evidence regression; it removed the stale
  contract helper spellings and preserved the Phase 201 source gate.
