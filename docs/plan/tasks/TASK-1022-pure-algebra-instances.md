# TASK-1022: Pure Algebra Instances

## Status: 📝 Planned

## Description

Add source-level algebra instances for ordinary pure carriers: `Option`, `Result<_, E>`, `List`, and string/list semigroup/monoid surfaces. Prelude-backed fallback for pure carriers is allowed only if TASK-1020 records a concrete blocker, ties the fallback to importable stdlib symbols, and creates a named replacement follow-up.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1020 and TASK-1021 completion.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Implement `Functor`, `Applicative`, and `Monad` evidence for `Option` and `Result<_, E>` through source-level stdlib surfaces unless TASK-1020 records a concrete source-syntax blocker.
2. Implement `Semigroup`/`Monoid` evidence for `String` and `List<A>` where current syntax supports it.
3. Implement `Functor<List>` and only implement `Applicative<List>`/`Monad<List>` if the audit freezes honest list semantics and required helpers.
4. Tests must import stdlib interfaces/instances, not define local fixture interfaces.

## File Targets

- Modify or create: `std/src/algebra/*.ash` instance sections or companion files chosen by TASK-1020
- Modify as needed: `std/src/option.ash`, `std/src/result.ash`, `std/src/list.ash`, `std/src/string.ash`
- Add focused tests named by TASK-1020

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

Implement TASK-1022. Use TDD with final-surface stdlib imports. Add pure instances only; do not touch Act/Proc/Workflow or do-target special cases except where tests expose a necessary interface registration hook owned by this task.
```

### Spec reviewer

```text
Review TASK-1022 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1022 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --list | tee /tmp/task1022-ash-typeck-pure-algebra-instances.list; grep -E "pure_algebra_instances" /tmp/task1022-ash-typeck-pure-algebra-instances.list; RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --list | tee /tmp/task1022-ash-engine-pure-algebra-instances.list; grep -E "pure_algebra_instances" /tmp/task1022-ash-engine-pure-algebra-instances.list; RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --nocapture'
  - cargo fmt --check
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
