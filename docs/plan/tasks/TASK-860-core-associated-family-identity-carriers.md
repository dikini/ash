# TASK-860: Core associated-family identity carriers

## Status: ✅ Complete

## Description

Add core-owned associated-family head/projection helpers and V4 semantic-summary carriers without typechecker or engine semantics.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-859 completion

## Files / Ownership

- Modify: `crates/ash-core/src/type_ir.rs`
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Create/modify tests: `crates/ash-core/tests/task_860_associated_family_carriers.rs`
- Modify: `crates/ash-typeck/src/type_env.rs` if V4/core summary validation errors or supported-version handling require TypeEnv diagnostic mapping updates
- Modify: `crates/ash-engine/src/module_loader.rs` if summary version matching or TypeEnv error-span extraction becomes non-exhaustive

## Requirements

### Functional Requirements

1. Add or reuse core identity carriers for associated family heads keyed by interface/member identities.
2. Add named helper APIs that distinguish ordinary associated projection identity, reducible sealed family head, rigid where-bound projection, and neutral blocked/unavailable projection.
3. Add checked family scheme/result carriers capable of representing sealed-domain constructor patterns/results and recursive associated-family projection RHSs without encoding marker constructors as ordinary nominal types.
4. Add concrete V4 summary carriers for `AssociatedFamilySummary`, scheme summaries, result expressions, validated decreases metadata, dependency closure, source anchors, and helper-family source-visible vs normalizer-available status.
5. Add V4 summary version contract for associated-family facts.
6. Add serde/hash/equality tests for new carriers.
7. Reject V1/V2/V3 summaries carrying associated-family facts.
8. Update `ash-typeck::TypeEnv` summary-validation error mapping and `ash-engine` version/error-span seams in the same task if new core validation variants or V4 supported-version checks make the workspace non-exhaustive.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write core carrier tests

- Add serde roundtrip/equality/hash/version validation tests for family identities, projections, helper classification APIs, scheme/result carriers, V4 family summaries, validated decreases metadata, dependency closures, and malformed older-version summaries with family facts.

### Step 2: Implement core carriers

- Keep public semantic carriers in `ash-core`.
- Do not add parser or engine-private semantic structs.

### Step 3: Verify version contract

- Test malformed older-version summaries with non-empty family facts reject before consumption.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Implemented core-owned associated-family head/projection carriers, helper classification APIs, scheme/result/validated-decreases/dependency-closure carriers, and V4 semantic-summary payload/version validation in `ash-core`.
- Updated `ash-typeck::TypeEnv` validation diagnostic mapping for V4 associated-family summaries and `ash-engine` imported-summary merge compatibility so identical family payloads deduplicate while divergent same-head payloads are preserved rather than silently dropped.
- Focused verification recorded after final remediation:
  - `cargo test -p ash-engine module_loader::tests::task_860_imported_summary_merge_preserves_associated_family_payloads -- --nocapture` — passed, 1 test.
  - `cargo test -p ash-core --test task_860_associated_family_carriers -- --nocapture` — passed, 6 tests.
  - `cargo check --workspace` — passed.
  - Independent final re-review reported no remaining blocking or non-blocking findings after remediation.
- Final closeout verification after status/changelog reconciliation is recorded by the implementing controller in the session/commit evidence.

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
    cargo test -p ash-core --test task_860_associated_family_carriers -- --list | tee /tmp/task_860_associated_family_carriers-list.txt
    grep -Eq 'associated_family|task_860' /tmp/task_860_associated_family_carriers-list.txt
  - cargo test -p ash-core --test task_860_associated_family_carriers -- --nocapture
  - cargo test -p ash-engine module_loader::tests::task_860_imported_summary_merge_preserves_associated_family_payloads -- --nocapture
checklist:
  - "[x] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[x] Focused tests/evidence for this task pass with non-zero test counts"
  - "[x] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Core family identity and summary schema used by TypeEnv and engine tasks.
