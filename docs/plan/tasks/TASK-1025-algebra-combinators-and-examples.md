# TASK-1025: Algebra Combinators and Examples

## Status: ✅ Complete

## Description

Add useful `std::algebra` helper functions and examples so the library is practically usable, not only a set of interfaces.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1024 completion.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Add helpers such as `functor::void`, `functor::replace`, `applicative::lift2`, `applicative::then`, `monad::then`, `monad::join`, `monad::compose`, and `monoid::concat` where expressible honestly.
2. Add executable/checkable examples using `Option`, `Result`, at least one monoid, and one tower carrier.
3. Trim any helper that current Ash cannot express and record a follow-up row rather than adding a Rust builtin unnecessarily.
4. Prefer Ash source implementations; use Rust only for already-existing opaque runtime primitives.

## File Targets

- Modify: `std/src/algebra/functor.ash`
- Modify: `std/src/algebra/applicative.ash`
- Modify: `std/src/algebra/monad.ash`
- Modify: `std/src/algebra/monoid.ash`
- Add: `crates/ash-engine/tests/task_1025_algebra_combinators.rs`
- Add: `crates/ash-cli/tests/task_1025_algebra_examples.rs`

## TDD / Execution Steps

1. Re-read SPEC-078 and this task file.
2. Write RED tests or audit evidence proving the current gap.
3. Implement only this task's slice without pulling later tasks forward.
4. Run focused non-zero verification.
5. Record RED/GREEN evidence and update task/plan/changelog only when the slice is actually complete.

## Evidence

- RED: `std::algebra` exposed interfaces and carrier impls, but no practical helper function surface or CLI-checked example importing the algebra helpers with Option/Result/monoid/tower modules.
- GREEN: Added honest carrier-owned helper wrappers for currently expressible Option, Result, List, and String operations, plus non-zero engine and CLI coverage using final stdlib imports.
- Phase 135 cleanup correction: carrier impls and concrete helper wrappers now live only with the carrier modules (`option.ash`, `result.ash`, `list.ash`, `string.ash`) instead of in `std/src/algebra/*.ash`; the algebra modules own interface surfaces only. Removed prior `std::algebra`-owned `concat_string`/`concat_list` examples from the final surface. More general higher-rank helpers such as `then`, `join`, `compose`, and `lift2` remain follow-up material until the current surface can express them without fake builtins.
- Codex delegation: TASK-1025 was delegated to `codex exec`, but the spawned process was killed after it surfaced an unrelated hard-gate prompt instead of implementing; the work was completed manually in the phase worktree.

## Sub-Agent Prompts

### Implementer

```text
Repository: /home/dikini/Projects/ash. Read this task file, docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, docs/plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, and docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md; fail if the audit is missing. Constraints: no new Ash syntax; use final-surface std::algebra/import tests rather than local fixture-only evidence; do not preserve obsolete deferrals unless this task records a concrete current blocker and follow-up.

Implement TASK-1025. Add only helpers that can be expressed using current Ash. Write examples first and keep them final-surface: imports from std::algebra, real Option/Result/tower usage, no local fixture interfaces.
```

### Spec reviewer

```text
Review TASK-1025 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1025 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --list | tee /tmp/task1025-algebra-combinators.list; matches=$(grep -E "(^|::)algebra_combinators[^[:space:]]*: test$" /tmp/task1025-algebra-combinators.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine algebra_combinators tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --list | tee /tmp/task1025-algebra-examples.list; matches=$(grep -E "(^|::)algebra_examples[^[:space:]]*: test$" /tmp/task1025-algebra-examples.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-cli algebra_examples tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --nocapture'
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] RED evidence recorded
  - [x] GREEN evidence recorded
  - [x] Focused test commands have recorded non-zero test counts or an explicit artifact-check proof
  - [x] Final-surface or negative-leakage gates satisfied where applicable
  - [x] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs algebra helper functions and examples required by TASK-1026 through TASK-1028. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
