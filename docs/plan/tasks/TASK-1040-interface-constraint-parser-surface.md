# TASK-1040: Parse and preserve interface-level `where` evidence constraints with positive/negative tests

## Status: ✅ Complete

## Description

Parse and preserve interface-level `where` evidence constraints with positive/negative tests.

## Specification Reference

- [SPEC-080](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [PLAN-130](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Task Type

Parser

## Dependencies

- Depends on TASK-1038 completion
- Depends on TASK-1039 audit gate completion and exact verification-command replacement.

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
  - RUSTC_WRAPPER= cargo test -p ash-parser --test task_1040_interface_constraint_surface -- --nocapture
  - RUSTC_WRAPPER= cargo check --workspace
checklist:
- [x] Focused tests/artifact are non-zero and pass
- [x] `cargo fmt --check` passes for touched Rust files or is not applicable
- [x] `git diff --check` passes
- [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Audit-gate implementation seams

- Audit artifact: `../audits/TASK-1039-interface-evidence-constraints-audit.md`.
- Primary parser files: `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`.
- Add focused parser test binary: `crates/ash-parser/tests/task_1040_interface_constraint_surface.rs`.
- Preserve constraints on `InterfaceDef` as interface-owned evidence constraints; do not conflate them with `ImplDef::where_bounds` semantics.

## Verification evidence

- `cargo fmt --check`
- `git diff --check`
- `RUSTC_WRAPPER= cargo test -p ash-parser --test task_1040_interface_constraint_surface -- --nocapture` (4 tests passed)
- `RUSTC_WRAPPER= cargo check --workspace`

## Notes

This task is part of the interface evidence-constraint phase. The motivating final syntax is:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```
