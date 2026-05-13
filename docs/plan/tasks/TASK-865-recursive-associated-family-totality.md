# TASK-865: Recursive associated family totality

## Status: ✅ Complete

## Description

Validate recursive associated-family coverage, overlap, ordered residual defaults, and structural decreasingness over sealed domains.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-863 completion
- Depends on TASK-864 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify/refactor reusable SPEC-061 coverage/decreases helpers in `crates/ash-typeck/src/type_env.rs` or adjacent helper modules identified by TASK-858
- Modify: `crates/ash-typeck/src/error.rs` for missing/invalid decreases, non-sealed decreases parameter, result-domain, and mutual-recursion diagnostics
- Create/modify tests: `crates/ash-typeck/tests/task_865_recursive_associated_family.rs`

## Requirements

### Functional Requirements

1. Adapt SPEC-061 finite residual coverage/overlap semantics to family impl-scheme heads.
2. Require every recursive family to declare an explicit `decreases Param`, and require that parameter to be a sealed-domain-constrained interface argument preserved from TASK-859/TASK-861 typed-parameter metadata.
3. Accept Append-style structural recursion and reject same/rebuilt/computed recursive arguments.
4. Reject mutual recursion in MVP.
5. Validate result kind/domain conformance in recursive RHSs.
6. Emit precise diagnostics for missing decreases, invalid decreases, non-sealed decreases parameters, non-exhaustive/overlap/unreachable/default rows, non-decreasing recursion, mutual recursion, and result-domain mismatch.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write RED tests

- Positive `Append` Nil/Cons family coverage and reduction setup with `Xs: TypeList` and `decreases Xs`.
- Negative non-exhaustive, overlap, unreachable, empty default, missing decreases, invalid decreases parameter, non-sealed decreases parameter, non-decreasing, result-domain mismatch, and mutual-recursion cases.

### Step 2: Implement validation

- Reuse or factor SPEC-061 residual coverage/decreases helpers where possible.
- Preserve source-order diagnostics and anchors.

### Step 3: Verify totality

- Run direct type-function totality suites plus new family totality suites.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Added `crates/ash-typeck/tests/task_865_recursive_associated_family.rs` with 6 focused tests covering direct closed-table associated-family totality and structural recursion: Append-style Nil/Cons acceptance, missing decreases, non-sealed/non-structural decreases, non-exhaustive/unreachable rows, same/rebuilt/computed recursive arguments, cross-family recursion rejection, and result-domain mismatch.
- Implemented TypeEnv associated-family totality/decreasingness validation by adapting SPEC-061 residual coverage to direct closed family tables while preserving the production `register_impl` one-row publication boundary through an internal non-closed-totality registration path.
- Fresh verification passed: `cargo fmt --all`; `cargo test -p ash-typeck --test task_865_recursive_associated_family -- --nocapture` (6 passed); `cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture` (8 passed); `cargo test -p ash-typeck --test task_837_type_function_recursion -- --nocapture` (11 passed); `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.
- Independent TASK-865 re-review after remediation reported PASS with no blocking, important, or non-blocking findings.

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
    cargo test -p ash-typeck --test task_865_recursive_associated_family -- --list | tee /tmp/task_865_recursive_associated_family-list.txt
    grep -Eq 'recursive|associated_family|task_865' /tmp/task_865_recursive_associated_family-list.txt
  - cargo test -p ash-typeck --test task_865_recursive_associated_family -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Total validated recursive family tables available for normalizer integration.
