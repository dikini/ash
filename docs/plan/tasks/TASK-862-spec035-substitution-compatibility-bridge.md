# TASK-862: SPEC-035 substitution compatibility bridge

## Status: ✅ Complete

## Description

Preserve current SPEC-035 selected-impl substitution while bridging computable family declarations to the new family infrastructure.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-861 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs` only if compatibility projections need blocker classification at the normalizer boundary
- Create/modify tests: `crates/ash-typeck/tests/task_862_spec035_associated_compat.rs`

## Requirements

### Functional Requirements

1. Prove non-family `type Name` associated types keep selected concrete impl substitution.
2. Route family-declared associated outputs through family projection identities without regressing simple substitution.
3. Keep ambiguous `T::Assoc` diagnostics stable.
4. Add negative leakage tests proving ordinary associated types do not become reducible families accidentally.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write compatibility tests

- Cover concrete selected impl substitution for ordinary associated types.
- Cover family member explicit projection.
- Cover ambiguous `T::Assoc` and non-family projection neutrality.

### Step 2: Implement bridge

- Refactor only the compatibility seam needed to share identities or lookup helpers.
- Do not replace ordinary associated behavior wholesale.

### Step 3: Verify regression surface

- Run SPEC-035/SPEC-058 associated projection suites plus new bridge tests.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Added `crates/ash-typeck/tests/task_862_spec035_associated_compat.rs` with eight focused compatibility/negative-leakage tests covering SPEC-035 selected concrete impl substitution, explicit family projection lowering, family RHS projection publication, ambiguous `T::Assoc` stability, mixed ordinary/family interface non-leakage, explicit family syntax rejection for ordinary associated members, concrete `String` argument preservation in Type-kind family projections, and neutral ordinary `S::Ok` lowering.
- Implemented the bridge in `crates/ash-typeck/src/type_env.rs` by lowering explicit `<Interface<Args>>::Assoc` family syntax to canonical family projection identities, lowering family RHS projections to associated-family result expressions, registering local interface/member identities for ordinary interfaces when module identity is available without declaring ordinary members as family heads, and preserving concrete Type-kind projection arguments unless they are scheme-owned variables.
- Verification passed: `cargo fmt --all`; `cargo test -p ash-typeck --test task_862_spec035_associated_compat -- --nocapture` (8 passed); `cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture` (8 passed); `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`; independent final re-review also ran `cargo fmt --all --check`, `git diff --check`, `cargo check --workspace`, focused TASK-862/TASK-861 tests, and clippy with no blocking, important, or non-blocking findings.

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
    cargo test -p ash-typeck --test task_862_spec035_associated_compat -- --list | tee /tmp/task_862_spec035_associated_compat-list.txt
    grep -Eq 'spec035|associated|task_862' /tmp/task_862_spec035_associated_compat-list.txt
  - cargo test -p ash-typeck --test task_862_spec035_associated_compat -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Compatibility-safe bridge from SPEC-035 behavior to SPEC-063 family infrastructure.
