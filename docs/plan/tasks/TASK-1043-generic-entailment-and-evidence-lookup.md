# TASK-1043: Make constrained evidence entail required evidence in generic contexts without reverse derivation

## Status: ✅ Complete

## Description

Make constrained evidence entail required evidence in generic contexts without reverse derivation.

## Specification Reference

- [SPEC-080](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [PLAN-130](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Task Type

Typeck

## Dependencies

- Depends on TASK-1038 completion
- Depends on TASK-1039 audit gate completion and exact verification-command replacement.
- Depends on TASK-1042 completion.

## Requirements

1. Preserve SPEC-080's core rule: `M: Monad` entails `M: Applicative`, but Ash does not automatically derive implementations.
2. Use “requires”, “entails”, “evidence constraint”, or “required evidence”; do not use object-hierarchy wording in user-facing docs or diagnostics.
3. Keep interface-level constraints distinct from generic impl `where` constraints.
4. Add focused non-zero tests or an explicit audit artifact matching this task type.
5. Avoid broadening generalized proposition syntax, proof search, implementation derivation, or overlap/specialization semantics.

## File Targets

- Spec/plan: `docs/spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md`, `docs/plan/PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md`
- Parser: `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/surface.rs`, focused parser tests
- Type checker: `crates/ash-typeck/src/type_env.rs`, related evidence lookup/checking modules, focused typeck tests
- Engine/stdlib if needed: `crates/ash-engine/src/module_loader.rs`, `std/src/algebra/`, engine final-path tests
- Docs/status: `docs/plan/PLAN-INDEX.md`, `docs/spec/README.md`, `CHANGELOG.md`, reference pages touched by the stdlib migration

## TDD / Execution Steps

1. Re-read SPEC-080 and PLAN-130 before editing code.
2. Add the smallest failing focused test or audit row for this task's exact boundary.
3. Implement only this task's boundary; do not add automatic derivation, object-hierarchy semantics, or generalized proposition syntax.
4. Run the focused command set recorded for this task.
5. Update task status, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment -- --list
  - RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment -- --nocapture
  - RUSTC_WRAPPER= cargo check --workspace
checklist:
- [x] Focused tests/artifact are non-zero and pass
- [x] `cargo fmt --check` passes for touched Rust files or is not applicable
- [x] `git diff --check` passes
- [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Audit-gate implementation seams

- Audit artifact: `../audits/TASK-1039-interface-evidence-constraints-audit.md`.
- Primary generic-evidence seams: `TypeEnv::type_var_interface_bounds`, `bind_type_var_interface_bound`, proposition solving for `InterfaceBoundProposition`, and impl where-bound assumption recording.
- Add focused typeck test binary: `crates/ash-typeck/tests/task_1043_interface_constraint_entailment.rs`.
- Directional only: `M: Monad` entails required `M: Applicative`; reverse entailment remains rejected.

## Notes

This task is part of the interface evidence-constraint phase. The motivating final syntax is:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

## Completion notes

- Added directional generic entailment over interface-owned required evidence constraints for in-scope type-variable bounds and impl `where`-bound proposition assumptions.
- Integrated the same directional check into generic method lookup so `M: Strong` can call required-interface methods such as `Weak::weak_id` when `Strong<M> where M: Weak` is registered.
- Preserved the no-reverse/no-derivation boundary: `M: Weak` does not satisfy `M: Strong`, and concrete `Applicative<Option>` does not synthesize concrete `Monad<Option>` evidence.
- Verification: `cargo fmt --check`; `git diff --check`; `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment -- --list` (10 tests); `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment -- --nocapture` (10 passed); `RUSTC_WRAPPER= cargo check --workspace`; `RUSTC_WRAPPER= cargo clippy -p ash-typeck --all-targets -- -D warnings`.
