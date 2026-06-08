# TASK-1048: Run diagnostics, broad verification, independent review, and status reconciliation

## Status: ✅ Complete

## Description

Run diagnostics, broad verification, independent review, and status reconciliation.

## Specification Reference

- [SPEC-080](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [PLAN-130](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Task Type

Closeout

## Dependencies

- Depends on TASK-1038 completion
- Depends on TASK-1039 audit gate completion and exact verification-command replacement.
- Depends on TASK-1040 through TASK-1046 completion.

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
  - RUSTC_WRAPPER= cargo check --workspace
checklist:
- [x] Focused tests/artifact are non-zero and pass
- [x] `cargo fmt --check` passes for touched Rust files or is not applicable
- [x] `git diff --check` passes
- [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Notes

This task is part of the interface evidence-constraint phase. The motivating final syntax is:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

## Completion notes

- Reconciled final stdlib algebra source imports for constrained interfaces:
  `monad.ash` imports `Applicative`, `applicative.ash` imports `Functor`, and
  `monoid.ash` imports `Semigroup` through final stdlib paths.
- Fixed final module export checking for constrained public interfaces by seeding
  imported interface definitions before validating interface-owned evidence
  constraints during `collect_module_exports`, including the public interface
  identity and public associated-family summary paths.
- Preserved compiler-prelude tower Monad evidence after `Monad` gained an
  `Applicative` prerequisite by synthesizing matching tower `Functor` and
  `Applicative` evidence before registering tower `Monad` evidence.
- Updated legacy focused fixtures affected by the new stdlib constraints without
  weakening the asserted behavior.
- Focused verification passed:
  - `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1024_stdlib_do_evidence -- --nocapture`
  - `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1022_pure_algebra_instances -- --nocapture`
  - `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1023_tower_algebra_instances_and_bridge_remediation -- --nocapture`
  - `RUSTC_WRAPPER= cargo run -q -p ash-cli -- check std/src/algebra/mod.ash --format human`
  - `RUSTC_WRAPPER= cargo test -p ash-cli --test stdlib_corpus_check stdlib_corpus_cli_check_baseline_is_classified_and_honest -- --nocapture`
- Broad verification passed:
  - `cargo fmt --check`
  - `git diff --check`
  - `RUSTC_WRAPPER= cargo check --workspace`
  - `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `RUSTC_WRAPPER= cargo test --workspace`
