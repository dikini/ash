# TASK-1815: Close out Phase 177 with gates, review, and status reconciliation

## Status: ✅ Complete

## Description

Close Phase 177 by running focused and broad verification, obtaining independent review, fixing review findings, and reconciling plan/task/spec/changelog status. Do not close the phase by overclaiming full target-Ash implementation.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- TASK-1807 through TASK-1814 complete or explicitly re-scoped with current blockers.

## Requirements

### Functional Requirements

1. Run every focused verification command from TASK-1809 through TASK-1814.
2. Run the broad Phase 177 verification baseline from PLAN-177.
3. Obtain independent review focused on row loss, authority leakage, stale target-spec claims, parser/Core/CPS mismatch, and unsupported-row diagnostics.
4. Fix or explicitly defer every review finding.
5. Reconcile PLAN-177, PLAN-INDEX, task files, CHANGELOG, and any touched specs/notes/indexes.
6. Mark Phase 177 complete only if all acceptance criteria are met or explicitly re-scoped with current blockers.

### Property Requirements

- Closeout must distinguish implemented Phase 177 vertical slice from full target-Ash completion.
- Any unsupported row family must be documented with a fail-closed behavior or a future-phase seed.
- Review findings must be addressed before status surfaces are marked complete.

## TDD Steps

### Step 1: Run focused gates

Execute each implementation task's verification commands and record results.

### Step 2: Run broad gates

Execute the full PLAN-177 verification baseline.

### Step 3: Independent review

Dispatch a reviewer with the changed files, PLAN-177 scope, target spec references, and verification output.

### Step 4: Reconcile closeout

Patch PLAN-177, PLAN-INDEX, task checklists, specs/docs, and CHANGELOG after review remediation.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-core
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Focused tests pass.
  - [x] Broad gates pass.
  - [x] Independent review complete and findings addressed.
  - [x] Phase 177 status surfaces agree.
  - [x] Unsupported target-Ash tails are honestly documented.
```

## Dependencies for Next Task

This task closes Phase 177 and should seed the next target-Ash implementation packet.

## Completion Evidence

- Ran focused Phase 177 gates for TASK-1809 through TASK-1814, including parser, typechecker,
  Core, and engine regression targets.
- Ran the broad Phase 177 verification baseline:
  `cargo fmt --check`, `cargo test -p ash-parser`, `cargo test -p ash-core`,
  `cargo test -p ash-engine`, `cargo test -p ash-typeck`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `git diff --check`, `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  and `bash scripts/check-docs-gate.sh`.
- Independent review completed. Findings were addressed by clarifying that lowercase/source-path
  operation rows remain unresolved requirement metadata in this validation-only slice, by removing
  over-strong source-to-Core/CPS vertical-slice wording, and by reconciling phase status surfaces.
- Phase 177 closes as a bounded parser/validation plus Core/CPS taxonomy alignment slice. It does
  not claim full target-Ash implementation, source-to-Core row lowering, row-polymorphic inference,
  provider/admission runtime wiring, or end-to-end source row preservation into CPS.
