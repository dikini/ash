# TASK-861: Typeck family declaration registration and coherence

## Status: ✅ Complete

## Description

Register `sealed type family` declarations and impl-family bindings in TypeEnv, enforcing family sealing and coherence before normalizer publication.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-860 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/error.rs`
- Modify: `crates/ash-engine/src/module_loader.rs` if new `TypeEnvError` variants require span extraction before a central `TypeEnvError::span()` helper exists
- Create/modify tests: `crates/ash-typeck/tests/task_861_associated_family_registration.rs`

## Requirements

### Functional Requirements

1. Resolve family declarations to core family head identities.
2. Preserve typed interface/impl parameter domain annotations in TypeEnv metadata so `decreases Param` can be validated against sealed-domain-constrained interface arguments.
3. Register dedicated impl-family scheme carriers against exactly one declared sealed family member; do not treat ordinary method/associated-type impl schemes as the final reducible family table.
4. Record defining module identity for family declarations and impl-family schemes, and reject unauthorized downstream extension of a sealed family equation set.
5. Validate scheme overlap/coherence before normalizer registration.
6. Validate family declaration result kind/domain annotations and impl RHS kind/domain conformance before publication.
7. Provide precise diagnostics for missing/extra bindings, duplicate heads, unauthorized extension, overlap, wrong result kind/domain, and module-owner violations.
8. Centralize TypeEnv error span extraction or update `ash-engine/src/module_loader.rs` in the same task when new `TypeEnvError` variants are added.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write typeck RED tests

- Cover typed/domain-constrained parameter registration, missing/extra family bindings, unauthorized sealed-family extension from another module, duplicate family heads, overlapping schemes, wrong result kind/domain annotations, impl RHS result-domain mismatch, module-owner context absence/error, and engine span extraction for any new TypeEnv diagnostics.

### Step 2: Implement registration

- Add TypeEnv registries for family declarations, family impl schemes, and publication state.
- Keep ordinary method impl selection separate.

### Step 3: Verify coherence

- Run focused typeck suites and existing interface/associated-type tests.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Implemented TypeEnv sealed associated-family declaration metadata, result-kind/domain constraint handling, module-owner validation, dedicated impl-family scheme registration, overlap/coherence validation, and precise TypeEnv diagnostics/spans.
- Preserved ordinary SPEC-035 associated-type behavior while routing sealed associated-family bindings through staged family scheme publication. Rejected impls do not publish family schemes; multi-family impl publication rolls back if a later family scheme fails.
- Added module-aware program typechecking through `type_check_program_in_env_for_module` and engine file-program module identity caching so file/module callers do not publish associated-family facts under the standalone synthetic identity.
- Focused verification passed: `cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture` (8 tests), `cargo test -p ash-typeck --test task_859_associated_family_fail_closed -- --nocapture` (3 tests), and `cargo check -p ash-engine`.
- Compatibility verification passed during implementation: `cargo test -p ash-typeck --test closed_world_interfaces_task_422 -- --nocapture` (24 tests), `cargo test -p ash-typeck --test task_800_associated_projection_canonicalization_red -- --nocapture` (18 tests), plus focused `cargo check -p ash-typeck`.
- Independent TASK-861 review after remediation reported PASS with no blocking, important, or non-blocking findings.

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
    cargo test -p ash-typeck --test task_861_associated_family_registration -- --list | tee /tmp/task_861_associated_family_registration-list.txt
    grep -Eq 'associated_family|task_861' /tmp/task_861_associated_family_registration-list.txt
  - cargo test -p ash-typeck --test task_861_associated_family_registration -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Validated family registries consumed by selection and normalizer tasks.
