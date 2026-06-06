# TASK-1023: Tower Algebra Instances and Bridge Remediation

## Status: 📝 Planned

## Description

Add `Act`, `Proc`, and `Workflow` algebra evidence tied to public stdlib operations, preserving opaque Rust runtime carriers while removing or quarantining anonymous hidden sequencing authority.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1020 and TASK-1021 completion; may run after or alongside TASK-1022 only if test files do not conflict.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Install `Monad<Act>`, `Monad<Proc>`, and `Monad<Workflow>` evidence through source-level impls if possible, otherwise through named compiler-prelude evidence tied to `act::unit/bind`, `proc::unit/bind`, and `workflow::unit/bind`.
2. Add `Functor`/`Applicative` tower evidence only if expressible honestly; otherwise record explicit follow-up rows.
3. Preserve hidden `ActEnv`, process identity, workflow admission, and failure/report boundaries.
4. Add negative leakage tests proving old anonymous hidden operations are not independent authority.

## File Targets

- Modify: `crates/ash-typeck/src/do_target.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `std/src/act.ash`, `std/src/proc.ash`, `std/src/workflow.ash` as needed
- Modify/create: `std/src/algebra/*` tower instance surfaces
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

Implement TASK-1023. Start with RED tests showing tower do/evidence still depends on hidden bridge authority. Then add named stdlib/prelude evidence tied to public tower operations. Do not expose ActEnv or invent new runtime syntax.
```

### Spec reviewer

```text
Review TASK-1023 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1023 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck tower_algebra_instances -- --list | tee /tmp/task1023-tower-algebra-instances.list; grep -E "tower_algebra_instances" /tmp/task1023-tower-algebra-instances.list; RUSTC_WRAPPER= cargo test -p ash-typeck tower_algebra_instances -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck hidden_bridge_leakage -- --list | tee /tmp/task1023-hidden-bridge-leakage.list; grep -E "hidden_bridge_leakage" /tmp/task1023-hidden-bridge-leakage.list; RUSTC_WRAPPER= cargo test -p ash-typeck hidden_bridge_leakage -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp act -- --list | tee /tmp/task1023-ash-interp-act.list; grep -E "act" /tmp/task1023-ash-interp-act.list; RUSTC_WRAPPER= cargo test -p ash-interp act -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp proc -- --list | tee /tmp/task1023-ash-interp-proc.list; grep -E "proc" /tmp/task1023-ash-interp-proc.list; RUSTC_WRAPPER= cargo test -p ash-interp proc -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp workflow -- --list | tee /tmp/task1023-ash-interp-workflow.list; grep -E "workflow" /tmp/task1023-ash-interp-workflow.list; RUSTC_WRAPPER= cargo test -p ash-interp workflow -- --nocapture'
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
