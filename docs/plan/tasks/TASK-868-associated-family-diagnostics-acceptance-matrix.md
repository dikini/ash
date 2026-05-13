# TASK-868: Associated family diagnostics and acceptance matrix

## Status: ✅ Complete

## Description

Add structured diagnostics and a row-by-row SPEC-063 acceptance/non-interference matrix.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-867 completion

## Files / Ownership

- Create: `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md`
- Modify: `crates/ash-typeck/src/error.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs`
- Modify: `crates/ash-engine/src/module_loader.rs` if diagnostic/span routing is affected
- Create/modify tests: `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs`
- Create/modify tests: `crates/ash-engine/tests/task_868_associated_family_diagnostics.rs` if engine projection/summary diagnostics are emitted through engine paths

## Requirements

### Functional Requirements

1. Implement or route every SPEC-063 §12 diagnostic family with stable spans/codes where current diagnostic infrastructure supports them.
2. Create an acceptance/non-interference artifact mapping every SPEC-063 §13 row to focused tests or evidence.
3. Add negative leakage assertions for SPEC-035, SPEC-058, SPEC-060, SPEC-061, and SPEC-062 boundaries.
4. Add an associated-family-specific non-inversion acceptance row, such as `<Append<Xs, Ys>>::Out == Cons<A, Nil>` not solving `Xs` or `Ys`.
5. Ensure focused tests do not pass with zero-test filters; record test counts for every cited command.
6. Assert structured diagnostic identity, span/source anchor, message/hint, and non-fatal/fatal severity as applicable, not only substring text.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write diagnostics tests

- Add focused tests for every SPEC-063 §12 family: syntax unsupported, not sealed, ambiguous member, unauthorized extension, missing binding, extra binding, overlap, unreachable row, non-exhaustive, missing/invalid decreases, non-sealed decreases parameter, non-decreasing recursion, result kind mismatch, result domain mismatch, mutual recursion unsupported, selection ambiguous, rigid projection note, private reduction unavailable, export private dependency, export not closed, import-order/dependency-closure conflict, malformed summary, and unsupported summary version.

### Step 2: Create matrix artifact

- Add `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md` with row/evidence mapping.

### Step 3: Verify matrix

- Run every cited focused command and record counts in the artifact.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Added `docs/plan/audits/TASK-868-associated-family-acceptance-matrix.md` mapping every SPEC-063 §13 row to exact focused evidence and non-zero target counts.
- Added `crates/ash-typeck/tests/task_868_associated_family_diagnostics.rs` with 7 focused tests for SPEC-063 §12 diagnostic route inventory, structured codes/spans/severity/message tokens, blocker reasons, associated-family non-inversion, and behavioral non-interference for SPEC-035, SPEC-058, SPEC-060, SPEC-061, and SPEC-062.
- Verification:
  - `cargo fmt --check` — passed.
  - `cargo test -p ash-typeck --test task_868_associated_family_diagnostics -- --list` — 7 tests, 0 benchmarks.
  - `cargo test -p ash-typeck --test task_868_associated_family_diagnostics -- --nocapture` — 7 passed.
  - `cargo clippy -p ash-typeck --test task_868_associated_family_diagnostics --all-features -- -D warnings` — passed.
- Independent review initially found the negative leakage row too synthetic; remediation made it behavioral for the predecessor-spec boundaries and updated the audit limitations. Final re-review verdict: PASS with no important findings.

## Dispatch

```yaml
agent: hermes
reasoning: high
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
    cargo test -p ash-typeck --test task_868_associated_family_diagnostics -- --list | tee /tmp/task_868_typeck-list.txt
    grep -Eq 'associated_family|diagnostic|task_868' /tmp/task_868_typeck-list.txt
  - cargo test -p ash-typeck --test task_868_associated_family_diagnostics -- --nocapture
  - |
    if [ -f crates/ash-engine/tests/task_868_associated_family_diagnostics.rs ]; then
      cargo test -p ash-engine --test task_868_associated_family_diagnostics -- --list | tee /tmp/task_868_engine-list.txt
      grep -Eq 'associated_family|diagnostic|task_868' /tmp/task_868_engine-list.txt
      cargo test -p ash-engine --test task_868_associated_family_diagnostics -- --nocapture
    else
      echo 'No TASK-868 ash-engine diagnostic target: engine projection/summary diagnostics were not emitted through engine paths.'
    fi
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Diagnostics coverage and acceptance artifact required for closeout.
