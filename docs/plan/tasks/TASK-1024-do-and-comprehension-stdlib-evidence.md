# TASK-1024: Do and Comprehension Stdlib Evidence

## Status: 📝 Planned

## Description

Rewire generalized `do:K` and explicit-target comprehensions so they resolve through selected stdlib/prelude `Monad<K>` evidence for pure and tower carriers.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1022 and TASK-1023 completion.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Consume the TASK-1020 `unit` versus `return` method-name decision, then make `do:Option`, `do:Result<_, E>`, `do:Act`, `do:Proc`, and `do:Workflow` use the same selected evidence model.
2. Make comprehension lowering reuse the same evidence path as `do:K`.
3. Keep missing and ambiguous evidence fail-closed.
4. Replace or supplement local fixture tests with stdlib-import final-surface tests.

## File Targets

- Modify: `crates/ash-typeck/src/do_target.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify as needed: `crates/ash-engine/src/monomorphize.rs`
- Modify tests named by TASK-1020, especially generalized-do and comprehension tests

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

Implement TASK-1024 after TASK-1022 and TASK-1023. Write RED tests for final-surface do/comprehension examples. Rewire lowering to selected evidence and preserve fail-closed diagnostics. Do not invent target inference or new syntax.
```

### Spec reviewer

```text
Review TASK-1024 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1024 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --list | tee /tmp/task1024-stdlib-do-evidence.list; grep -E "stdlib_do_evidence" /tmp/task1024-stdlib-do-evidence.list; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --list | tee /tmp/task1024-stdlib-comprehension-evidence.list; grep -E "stdlib_comprehension_evidence" /tmp/task1024-stdlib-comprehension-evidence.list; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --list | tee /tmp/task1024-engine-stdlib-do-evidence.list; grep -E "stdlib_do_evidence" /tmp/task1024-engine-stdlib-do-evidence.list; RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --nocapture'
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
