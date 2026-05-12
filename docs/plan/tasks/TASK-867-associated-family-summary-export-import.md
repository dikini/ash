# TASK-867: Associated family summary export/import

## Status: 🟡 Ready

## Description

Export/import public associated-family summaries through V4 semantic summaries with private-opacity and import-order guarantees.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-866 completion

## Files / Ownership

- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-engine/src/module_loader.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs` only to expose imported validated family tables to the lookup path after import validation
- Create/modify tests: `crates/ash-core/tests/task_867_associated_family_summary.rs`
- Create/modify tests: `crates/ash-engine/tests/task_867_associated_family_summary_transport.rs`
- Create/modify tests: `crates/ash-typeck/tests/task_867_associated_family_import.rs`

## Requirements

### Functional Requirements

1. Export only public, export-closed associated-family summaries whose entire validated closed equation set and every dependency are public-summary-visible.
2. Reject public exports with private/incomplete dependencies instead of exporting partial reducible equation tables.
3. Reject V1/V2/V3 summaries with non-empty family facts.
4. Carry V4 fields required by SPEC-063 §11: family head identity, interface/member identities, visible names, result kind/domain, export mode, ordered scheme patterns, RHS result expressions, decreases metadata, source anchors, dependency closure, and helper-family source-visible vs normalizer-available status.
5. Batch-declare imported family heads before validation and normalizer registration.
6. Revalidate imported kind/domain, coverage/overlap, coherence, selected-scheme uniqueness, recursion/decreases metadata, result expressions, public dependency closure, and import-order stability before normalizer registration.
7. Preserve canonical identities through named/glob/pub-use imports and dependency helper closures.
8. Make imported family tables normalizer-available only after successful V4 validation, extending TASK-866's local-only lookup path.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write RED tests

- Version rejection, private dependency rejection for every §11 dependency category, export-not-closed rejection, named import, glob import, pub-use identity, helper non-leakage, import-order independence, malformed decreases metadata, result-domain mismatch, selected-scheme ambiguity, dependency-closure conflict, and downstream imported-family reduction after validated V4 import.

### Step 2: Implement summary transport

- Extend core summaries, engine transport/reconciliation, and TypeEnv imported summary registration.
- Revalidate imported family invariants before normalizer registration.

### Step 3: Verify public/private boundary

- Run engine/typeck summary suites and SPEC-062 non-regression tests.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Requirements above are satisfied.
- [ ] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [ ] Negative leakage/non-interference behavior is covered for this task's surface.
- [ ] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [ ] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Completion evidence must be recorded by the implementing agent before marking this task complete.

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
    cargo test -p ash-core --test task_867_associated_family_summary -- --list | tee /tmp/task_867_core-list.txt
    grep -Eq 'associated_family|task_867' /tmp/task_867_core-list.txt
  - cargo test -p ash-core --test task_867_associated_family_summary -- --nocapture
  - |
    cargo test -p ash-engine --test task_867_associated_family_summary_transport -- --list | tee /tmp/task_867_engine-list.txt
    grep -Eq 'associated_family|task_867' /tmp/task_867_engine-list.txt
  - cargo test -p ash-engine --test task_867_associated_family_summary_transport -- --nocapture
  - |
    cargo test -p ash-typeck --test task_867_associated_family_import -- --list | tee /tmp/task_867_typeck-list.txt
    grep -Eq 'associated_family|task_867' /tmp/task_867_typeck-list.txt
  - cargo test -p ash-typeck --test task_867_associated_family_import -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Cross-module public associated-family computation with private-opacity guarantees.
