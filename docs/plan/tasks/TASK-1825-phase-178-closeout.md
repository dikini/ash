# TASK-1825: Close out Phase 178 with gates and review

## Status: ✅ Complete

## Description

Close Phase 178 by running focused and broad verification, obtaining independent review, fixing findings, and reconciling all status surfaces. Do not close by overclaiming row inference, provider/admission runtime wiring, or full target-Ash implementation.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- TASK-1818 through TASK-1824 complete or explicitly re-scoped with current blockers.

## Requirements

### Functional Requirements

1. Run every focused verification command from TASK-1819 through TASK-1823.
2. Run the full PLAN-178 verification baseline.
3. Obtain independent review focused on row loss, authority leakage, stale target-Ash claims, summary/import compatibility, and Core row mismatch.
4. Fix or explicitly defer every review finding.
5. Reconcile PLAN-178, PLAN-INDEX, task files, CHANGELOG, and any touched specs/notes after review remediation.
6. Mark Phase 178 complete only if acceptance criteria are met or explicitly re-scoped with current blockers.

### Property Requirements

- Closeout must distinguish source-to-Core explicit row preservation from row inference.
- Closeout must explicitly preserve provider/admission/handler runtime work as future work unless separate implementation exists.
- Review findings must be addressed before status surfaces are marked complete.

## TDD Steps

### Step 1: Run focused gates

Execute each implementation task's verification commands and record results.

### Step 2: Run broad gates

Execute the full PLAN-178 verification baseline.

### Step 3: Independent review

Dispatch a reviewer with the changed files, PLAN-178 scope, Phase 177 caveats, target specs, and verification output.

### Step 4: Reconcile closeout

Patch PLAN-178, PLAN-INDEX, tasks, docs/specs, and CHANGELOG after review remediation.

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
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo test -p ash-core
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Focused tests pass.
  - [x] Broad gates pass.
  - [x] Independent review complete and findings addressed.
  - [x] Phase 178 status surfaces agree.
  - [x] Future-work caveats remain explicit.
```

## Completion Evidence

- Focused TASK-1821, TASK-1822, and TASK-1823 engine regressions pass after review remediation.
- Broad affected crate suites pass: `cargo test -p ash-parser`, `cargo test -p ash-engine`, `cargo test -p ash-typeck`, and `cargo test -p ash-core`.
- Independent review findings were addressed by preserving rowless callable metadata compatibility for unsupported rowless surface type forms, adding imported row authority-neutrality coverage, and reconciling Phase 178 task counts.
- Full closeout verification baseline passed, including workspace check, clippy, docs index validation, and docs gate.

## Dependencies for Next Task

This task closes Phase 178 and should seed the next target-Ash implementation packet.
