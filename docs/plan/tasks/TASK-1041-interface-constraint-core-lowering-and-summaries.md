# TASK-1041: Carry interface constraints through lowering/core summaries or prove no summary change is needed

## Status: ✅ Complete

## Description

Carry interface constraints through lowering/core summaries or prove no summary change is needed.

## Specification Reference

- [SPEC-080](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [PLAN-130](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Task Type

Core/Engine

## Dependencies

- Depends on TASK-1038 completion
- Depends on TASK-1039 audit gate completion and exact verification-command replacement.
- Depends on TASK-1040 parser surface completion.

## Requirements

1. Preserve SPEC-080's core rule: `M: Monad` entails `M: Applicative`, but Ash does not automatically derive implementations.
2. Use “requires”, “entails”, “evidence constraint”, or “required evidence”; do not use object-hierarchy wording in user-facing docs or diagnostics.
3. Keep interface-level constraints distinct from generic impl `where` constraints.
4. Add focused non-zero tests or an explicit audit artifact matching this task type.
5. Avoid broadening generalized proposition syntax, proof search, implementation derivation, or overlap/specialization semantics.

## TASK-1041 Decision

Interface evidence constraints must cross the core/module-summary boundary. TASK-1041 therefore adds an explicit transport carrier instead of proving no summary change is needed:

- `ash_core::ast::InterfaceDef` owns `evidence_constraints: Vec<InterfaceEvidenceConstraint>` for core lowering.
- `ash_core::semantic_summary::InterfaceIdentitySummary` owns `evidence_constraints: Vec<InterfaceEvidenceConstraintSummary>` for imported interface summaries.
- `ash-engine` lowers public interface-owned constraints into interface identity summaries so named and glob imports carry required evidence metadata for later TypeEnv enforcement.

This task does not add required-evidence verification, entailment lookup, automatic derivation, proof search, blanket impl synthesis, or stdlib algebra migrations. Those remain owned by TASK-1042 through TASK-1046.

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
  - RUSTC_WRAPPER= cargo test -p ash-engine --test task_1041_interface_constraint_summary_transport -- --list
  - RUSTC_WRAPPER= cargo test -p ash-engine --test task_1041_interface_constraint_summary_transport -- --nocapture
  - RUSTC_WRAPPER= cargo check --workspace
checklist:
- [x] Focused tests/artifact are non-zero and pass
- [x] `cargo fmt --check` passes for touched Rust files or is not applicable
- [x] `git diff --check` passes
- [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Audit-gate implementation seams

- Audit artifact: `../audits/TASK-1039-interface-evidence-constraints-audit.md`.
- Decide explicitly whether interface evidence constraints must be transported in `ModuleSemanticSummary`.
- If no summary test binary is added, replace the conditional command with a task-owned audit artifact proving final import-path preservation.

## Notes

This task is part of the interface evidence-constraint phase. The motivating final syntax is:

```ash
interface Monad<M : * -> *> where M: Applicative {
    unit(Int) -> M<Int>
    bind(M<Int>, (Int) -> M<Int>) -> M<Int>
}
```

## Completion notes

- Added core `InterfaceEvidenceConstraint` metadata and lowered parser-owned interface evidence constraints into core interface definitions.
- Added summary `InterfaceEvidenceConstraintSummary` metadata on `InterfaceIdentitySummary` so imported public interfaces can carry required evidence metadata.
- Added focused engine coverage for core lowering, named-import transport, and glob-import transport that stays distinct from impl `where` bounds.
- Verification: `cargo fmt --check`; `git diff --check`; `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1041_interface_constraint_summary_transport -- --list` (3 tests); `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1041_interface_constraint_summary_transport -- --nocapture` (3 passed); `RUSTC_WRAPPER= cargo check --workspace`.
