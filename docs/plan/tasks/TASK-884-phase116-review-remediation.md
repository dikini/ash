# TASK-884: Phase 116 review remediation

## Status: ✅ Complete

## Description

Reserve mandatory independent review remediation for Phase 116 after closeout, with all findings addressed before final completion.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-883 completion

## Files / Ownership

- Modify: files identified by independent review
- Expected review surfaces include SPEC-064, PLAN-112, PLAN-INDEX, docs/spec/README.md, CHANGELOG.md, TASK-871 through TASK-884, audit artifacts TASK-872/TASK-882, and all Phase 116 code/test files.

## Requirements

### Functional Requirements

1. Run independent review across spec, plan, tasks, changed code, acceptance artifact, and verification evidence.
2. Treat blocking, important, minor, and non-blocking findings as work to address unless explicitly documented as out-of-scope by spec authority.
3. Patch docs/code/tests and rerun focused plus broad gates after final change.
4. Update status surfaces and changelog with actual remediation evidence.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Delegate independent review with axes: spec conformance, task/order drift, live-code feasibility, non-inversion, summary opacity, diagnostics, verification honesty.

### Step 2

- Patch every finding or document scoped exception.

### Step 3

- Rerun broad closeout gates after final remediation patch.

### Step 4

- Mark Phase 116 complete only after review remediation is verified.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Independent review findings are addressed.
- [x] Focused remediation tests pass.
- [x] Broad workspace verification passes after final patch.
- [x] Phase 116 status surfaces are complete.


## Completion Evidence

- Independent TASK-884 review axis B found docs/status blockers; remediation updated PLAN-INDEX Phase 116 summary counts from 3/14 to 13/14 before final TASK-884 closeout, checked verification checklist strings in completed TASK-874 through TASK-883 files, and expanded TASK-883 scoped-doc evidence from the narrow 7-file set to the full 21-file Phase 116 review set.
- Independent docs/status re-review passed after remediation: PLAN-INDEX summary rows were corrected, TASK-871 through TASK-883 completed-task verification checklists had no unchecked items, TASK-883 recorded the 21-file scoped-doc evidence, and TASK-884 remained Ready until final status reconciliation.
- Independent semantic/acceptance review passed: no blocking semantic overclaim, no type-function inversion or proof-search overclaim, no parser/runtime constraint conflation, and TASK-882 H1-H12 evidence remained non-zero and honest.
- Scoped Markdown/status check passed over 21 Phase 116 files: 0 trailing-whitespace findings, 0 missing relative links, and 0 completed-task unchecked verification checklist findings.
- Focused remediation verification passed: `git diff --check`, `cargo fmt --check`, `cargo test -p ash-engine --test task_879_proposition_summary_transport`, `cargo check --workspace`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Broad final verification passed after the final TASK-884 status patch: `cargo fmt --check`, `git diff --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase116-doc.log`, and `! grep -i '^warning:' /tmp/ash-phase116-doc.log`.

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
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase116-doc.log
  - ! grep -i '^warning:' /tmp/ash-phase116-doc.log
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-884 for downstream tasks.
