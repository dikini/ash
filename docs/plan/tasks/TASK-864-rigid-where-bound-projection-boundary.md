# TASK-864: Rigid where-bound projection boundary

## Status: ✅ Complete

## Description

Enforce the boundary that where-bound evidence creates rigid projections but never selects family equations by itself.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-861 completion
- Depends on TASK-862 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs` if rigid projection blocker/equality classification changes
- Modify: `crates/ash-typeck/src/error.rs` if new rigid-projection notes/hints require structured diagnostics
- Create/modify tests: `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs`

## Requirements

### Functional Requirements

1. Keep `T::Item` rigid under only `T: Iterator`.
2. Ensure generic bounds do not trigger speculative impl search.
3. Make rigid projection equality structural and non-inverting.
4. Emit clear notes/hints at forcing points requiring concrete family reduction.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write RED tests

- Generic function signatures with `T: Iterator` keep `T::Item` rigid.
- Equality does not collapse `T::Item` to an arbitrary concrete type.
- Adding a bound does not reduce without explicit family head/argument selection.

### Step 2: Implement boundary

- Audit where-bound lookup paths and separate evidence from equation selection.

### Step 3: Verify no hidden solver

- Run focused typeck/equality tests and non-inversion regressions.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Added `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs` with 8 focused tests covering real `T: Iterator` / `T::Item` lowering to rigid canonical projection, rigid projection normalization, non-selection from where-bound evidence, structural-only rigid projection equality, non-collapse to concrete types, explicit concrete family reduction, forcing-point diagnostics, and normalization notes.
- Updated `TypeEnv::lower_type_to_canonical_for_equality` / equality-boundary canonicalization so in-bounds `Type::Associated { base: Type::Var(..), .. }` lowers to a `ProjectionRigidity::Rigid` canonical projection while legacy unbounded TASK-798 lowering remains neutral.
- Updated `Normalizer::require_concrete_normal_form` forcing-point diagnostics to name the concrete family-reduction/non-inversion boundary.
- Verification passed: `cargo fmt --all --check`; `git diff --check`; `cargo check --workspace`; `cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --list` (8 tests); `cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --nocapture` (8 passed); TASK-798, TASK-824, TASK-825, TASK-829, and TASK-863 focused regressions; `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.
- Independent review PASS after remediation; no remaining blocking, important, or non-blocking findings.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - |
    cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --list | tee /tmp/task_864_rigid_where_bound_projection-list.txt
    grep -Eq 'rigid|where_bound|task_864' /tmp/task_864_rigid_where_bound_projection-list.txt
  - cargo test -p ash-typeck --test task_864_rigid_where_bound_projection -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Rigid projection boundary required before recursive/summary reductions can be trusted.
