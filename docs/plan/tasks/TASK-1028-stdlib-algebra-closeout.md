# TASK-1028: Stdlib Algebra Closeout

## Status: 📝 Planned

## Description

Run broad verification, independent review, status reconciliation, and closeout for SPEC-078/PLAN-128 only after all final-surface and negative-leakage gates pass.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1020 through TASK-1027 completion.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Run focused gates from TASK-1020 through TASK-1027 and broad workspace gates.
2. Run independent spec and quality review of the full phase delta.
3. Promote SPEC-078/PLAN-128/PLAN-INDEX/task statuses only if acceptance matrix A78-1 through A78-12 is satisfied or explicitly scoped with follow-up rows.
4. Update CHANGELOG.md under `[Unreleased]` and reconcile docs/spec index.

## File Targets

- Modify: `docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md`
- Modify: `docs/plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/README.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/tasks/TASK-102[0-8]-*.md`

## TDD / Execution Steps

1. Re-read SPEC-078 and this task file.
2. Write RED tests or audit evidence proving the current gap.
3. Implement only this task's slice without pulling later tasks forward.
4. Run focused non-zero verification.
5. Record RED/GREEN evidence and update task/plan/changelog only when the slice is actually complete.

## Sub-Agent Prompts

### Implementer

```text
Repository: /home/dikini/Projects/ash. Read this task file, docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, docs/plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, and the TASK-1020 audit artifact if it exists. Constraints: no new Ash syntax; use final-surface std::algebra/import tests rather than local fixture-only evidence; do not preserve obsolete deferrals unless this task records a concrete current blocker and follow-up.

Implement TASK-1028 closeout only after TASK-1020 through TASK-1027 are complete. Re-run all focused and broad gates, dispatch independent review, reconcile all status surfaces, and record exact evidence. Do not promote status if any final-surface or negative-leakage gate is missing.
```

### Spec reviewer

```text
Review TASK-1028 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1028 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
```

## Dispatch

```yaml
agent: codex
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - RUSTC_WRAPPER= cargo check --workspace
  - RUSTC_WRAPPER= cargo test -p ash-typeck --all-targets
  - RUSTC_WRAPPER= cargo test -p ash-engine --all-targets
  - RUSTC_WRAPPER= cargo test -p ash-cli --all-targets
  - RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
  - RUSTC_WRAPPER= cargo test --workspace
  - RUSTC_WRAPPER= cargo doc --workspace --no-deps
  - git diff --check
checklist:
  - [ ] RED evidence recorded
  - [ ] GREEN evidence recorded
  - [ ] Focused test commands have recorded non-zero test counts or an explicit artifact-check proof
  - [ ] Final-surface or negative-leakage gates satisfied where applicable
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs the verified slice required by downstream TASK-1020..TASK-1028 work. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
