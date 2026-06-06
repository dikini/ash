# TASK-1021: std::algebra Namespace and Interfaces

## Status: ✅ Complete

## Description

Create the source-visible `std::algebra` namespace and add importable `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` interface modules using the syntax frozen by TASK-1020.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1020 completion and its focused command handoff.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Create `std/src/algebra/mod.ash` and interface modules under `std/src/algebra/` using the exact accepted source syntax frozen by TASK-1020, not unchecked logical pseudocode.
2. Export `pub mod algebra;` from `std/src/lib.ash` without root-level algebra re-exports in this slice.
3. Add final-surface tests proving a user module can import each interface from `algebra::*` through the real engine/stdlib path.
4. Own any minimal parser-lowering/type-summary changes required to make the source-visible algebra interfaces importable; do not add instances or do-lowering rewrites in this task.

## File Targets

- Create: `std/src/algebra/mod.ash`
- Create: `std/src/algebra/semigroup.ash`
- Create: `std/src/algebra/monoid.ash`
- Create: `std/src/algebra/functor.ash`
- Create: `std/src/algebra/applicative.ash`
- Create: `std/src/algebra/monad.ash`
- Modify: `std/src/lib.ash`
- Modify if required by the audit: `crates/ash-parser/src/lower.rs` to stop rejecting constructor-kinded interface parameters on the final stdlib path.
- Add: `crates/ash-engine/tests/task_1021_std_algebra_namespace_and_interfaces.rs`
- Add: `crates/ash-typeck/tests/task_1021_algebra_interface_registration.rs`

## TDD / Execution Steps

1. Re-read SPEC-078 and this task file.
2. Write RED tests or audit evidence proving the current gap.
3. Implement only this task's slice without pulling later tasks forward.
4. Run focused non-zero verification.
5. Record RED/GREEN evidence and update task/plan/changelog only when the slice is actually complete.

## Sub-Agent Prompts

### Implementer

```text
Repository: /home/dikini/Projects/ash. Read this task file, docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, docs/plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, and docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md; fail if the audit is missing. Constraints: no new Ash syntax; use final-surface std::algebra/import tests rather than local fixture-only evidence; do not preserve obsolete deferrals unless this task records a concrete current blocker and follow-up.

Implement TASK-1021 after reading the TASK-1020 audit artifact. Add only the namespace and interfaces. Write failing import/check tests first, then create the std::algebra files and lib export. Do not add instances or change do lowering.
```

### Spec reviewer

```text
Review TASK-1021 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1021 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
  - python3 - <<'PY'
from pathlib import Path
for rel in ['std/src/algebra/mod.ash','std/src/algebra/semigroup.ash','std/src/algebra/monoid.ash','std/src/algebra/functor.ash','std/src/algebra/applicative.ash','std/src/algebra/monad.ash']:
    assert Path(rel).is_file(), rel
lib=Path('std/src/lib.ash').read_text()
assert any(line.strip() == 'pub mod algebra;' for line in lib.splitlines()), 'non-comment pub mod algebra;'
mod=Path('std/src/algebra/mod.ash').read_text()
for module in ['semigroup','monoid','functor','applicative','monad']:
    assert f'pub mod {module};' in mod, module
for rel, iface in [
    ('std/src/algebra/semigroup.ash','Semigroup'),
    ('std/src/algebra/monoid.ash','Monoid'),
    ('std/src/algebra/functor.ash','Functor'),
    ('std/src/algebra/applicative.ash','Applicative'),
    ('std/src/algebra/monad.ash','Monad'),
]:
    assert f'interface {iface}' in Path(rel).read_text(), rel
print('std::algebra files and lib export exist')
PY
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --list | tee /tmp/task1021-ash-engine-algebra-interface.list; matches=$(grep -E "(^|::)algebra_interface[^[:space:]]*: test$" /tmp/task1021-ash-engine-algebra-interface.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine algebra_interface tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --nocapture'
  - bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --list | tee /tmp/task1021-ash-typeck-algebra-interface.list; matches=$(grep -E "(^|::)algebra_interface[^[:space:]]*: test$" /tmp/task1021-ash-typeck-algebra-interface.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck algebra_interface tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --nocapture'
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] RED evidence recorded
  - [x] GREEN evidence recorded
  - [x] Focused test commands have recorded non-zero test counts or an explicit artifact-check proof
  - [x] Final-surface or negative-leakage gates satisfied where applicable
  - [x] Docs/status/changelog updated if public behavior changed
```

## Evidence

### RED

- `bash -lc 'RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --list ...'` initially had zero matching tests before `crates/ash-engine/tests/task_1021_std_algebra_namespace_and_interfaces.rs` existed.
- `bash -lc 'RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --list ...'` initially had zero matching tests before `crates/ash-typeck/tests/task_1021_algebra_interface_registration.rs` existed.

### GREEN

- Artifact assertion for `std/src/algebra/*.ash` files and `std/src/lib.ash` `pub mod algebra;`: exit 0, printed `std::algebra files and lib export exist`.
- `RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --list` recorded 2 matching tests, then `RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --nocapture` passed 2/2.
- `RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --list` recorded 2 matching tests, then `RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --nocapture` passed 2/2.
- `RUSTC_WRAPPER= cargo clippy -p ash-engine -p ash-typeck --all-targets -- -D warnings`: exit 0 after fixing the new engine test's implicit clone warning.
- `cargo fmt --check`: exit 0 after formatting the new Rust tests/module-loader edits.
- `git diff --check`: exit 0.

### Scope Confirmation

- Added only namespace/interface source files, final-surface import/registration tests, and the minimal module-loader semantic-summary selection needed for interface imports. No pure instances, tower evidence, `do:K`/comprehension rewiring, combinators, law-test support, or reference migration behavior were implemented.

## Dependencies for Next Task

This task outputs the importable `std::algebra` namespace required by TASK-1022 through TASK-1028. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
