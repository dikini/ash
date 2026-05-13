# TASK-866: Normalizer projection-family integration

## Status: ✅ Complete

## Description

Integrate validated associated-family reduction into the SPEC-060 normalizer and definitional equality APIs.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-865 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Modify: `crates/ash-typeck/src/type_env.rs` for local validated-family table lookup APIs
- Modify: `crates/ash-core/src/type_ir.rs` if normal-form blocker reasons or projection helper carriers need extension
- Create/modify tests: `crates/ash-typeck/tests/task_866_associated_family_normalizer.rs`

## Requirements

### Functional Requirements

1. Normalize associated-family projections by consulting validated local family tables only; imported family table availability is owned by TASK-867.
2. Normalize projection interface arguments according to the current SPEC-060 demand mode before local family lookup.
3. Preserve rigid/neutral projections with precise blocker reasons when reduction is unavailable.
4. Cover blocker categories for not sealed, ambiguous selection, generic-bound rigidity, private opacity/local unavailable, fuel/cycle exhaustion, and unsupported imported-family availability before TASK-867.
5. Respect fuel/cycle controls for recursive families.
6. Keep definitional equality normalize-and-compare and non-inverting.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write RED tests

- Normalizer reduces local `Iterator<List<A>>::Item` and recursive `Append` family cases.
- Demand-mode argument normalization is observable in projection spines.
- Not-sealed, ambiguous-selection, generic-bound rigid, private/local-unavailable, unsupported-imported, and fuel/cycle cases produce blocker evidence.
- Equality does not invert family outputs.

### Step 2: Implement normalizer lookup

- Add family table lookup to projection handling only.
- Do not widen direct `type fn` computation semantics.

### Step 3: Verify equality

- Run normalizer/equality suites including SPEC-060/SPEC-061/SPEC-062 regressions.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Added `crates/ash-typeck/tests/task_866_associated_family_normalizer.rs` with 8 focused tests covering local `Iterator<List<T>>::Item` reduction, projection-argument normalization before lookup, recursive Append-family reduction, fuel exhaustion, open-input non-inversion, rigid generic preservation, ordinary associated-type not-sealed blockers, and pre-TASK-867 local-only imported-family unsupported blockers.
- Implemented local-only normalizer projection-family reduction through `TypeEnv::reduce_local_associated_family_projection_from_normal_args`, normal-form associated-family pattern matching, associated-family result normalization, and typed normal-form blocker reasons without widening direct `type fn` computation or adding proof search/output inversion.
- Fresh verification passed: `cargo fmt --all`; `cargo test -p ash-typeck --test task_866_associated_family_normalizer -- --nocapture` (8 passed); `cargo test -p ash-typeck --test task_863_associated_family_selection -- --nocapture` (10 passed); prerequisite suites TASK-864 (8), TASK-865 (6), TASK-861 (8), TASK-837 (11); `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.
- Independent TASK-866 review reported PASS with no blocking, important, or non-blocking findings.

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
    cargo test -p ash-typeck --test task_866_associated_family_normalizer -- --list | tee /tmp/task_866_associated_family_normalizer-list.txt
    grep -Eq 'normalizer|associated_family|task_866' /tmp/task_866_associated_family_normalizer-list.txt
  - cargo test -p ash-typeck --test task_866_associated_family_normalizer -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Normalizer projection-family reduction consumed by public summary and acceptance tasks.
