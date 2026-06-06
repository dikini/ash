# TASK-1020: Stdlib Algebra Audit Gate

## Status: ✅ Complete

## Description

Audit the live stdlib, parser, typechecker, do-target, module-loader, and test-runner seams before any std::algebra implementation starts. Freeze exact syntax, exact file targets, and exact focused commands for TASK-1021 through TASK-1028.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

None. This is the hard pre-implementation audit gate.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Create `docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md` mapping live seams and stale deferrals.
2. Verify exact Ash syntax and lowering support for interface declarations, impl bodies, constructor-kinded parameters, method signatures, partial constructor impl heads, and std submodule imports.
3. Decide whether the canonical Monad method is `unit` or `return`, based on current evidence-lowering constraints.
4. Replace downstream placeholder verification with exact non-zero tests/commands, including `-- --list`/count guards or artifact assertions so filtered cargo tests cannot pass with zero matches.
5. List any real blockers as split follow-up tasks rather than preserving old MVP deferrals by inertia.

## File Targets

- Create: `docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md`
- Inspect: `std/src/*.ash`, `std/src/*/*.ash`
- Inspect: `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/lower.rs`
- Inspect: `crates/ash-typeck/src/do_target.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/check_expr.rs`
- Inspect: `crates/ash-engine/src/module_loader.rs` and stdlib import tests

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

Implement the TASK-1020 audit gate. Do not implement std::algebra yet. Produce an audit artifact with live file/callsite mappings, exact syntax decisions, deferral-retirement decisions, and downstream focused commands. Verify the artifact and run the existing do-target tests.
```

### Spec reviewer

```text
Review TASK-1020 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1020 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
p=Path('docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md')
assert p.is_file(), p
text=p.read_text()
for s in ['Live Seams','Deferral Retirement','Focused Commands','TASK-1021','TASK-1028','non-zero','hidden bridge leakage','stale deferral sweep']:
    assert s in text, s
# Downstream task verification must include non-zero guards for focused filtered tests.
for task in sorted(Path('docs/plan/tasks').glob('TASK-102[1-8]-*.md')):
    t = task.read_text()
    if 'cargo test -p' in t and '-- --nocapture' in t:
        assert '-- --list' in t, f'filtered cargo tests need -- --list non-zero guard in {task}'
        assert 'grep -E' in t or 'artifact assertion' in t, f'filtered cargo tests need count/assertion guard in {task}'
print('TASK-1020 audit artifact and downstream non-zero command guards exist')
PY
  - cargo test -p ash-typeck do_target
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

- `python3` audit-artifact guard failed before implementation with `AssertionError: docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md`, proving the required audit artifact was missing.

### GREEN

- `python3` audit-artifact/downstream non-zero guard: exit 0, printed `TASK-1020 audit artifact and downstream non-zero command guards exist`.
- `RUSTC_WRAPPER= cargo test -p ash-typeck do_target`: exit 0, ran 9 matching `do_target` unit tests in `ash-typeck` with 9 passed and 0 failed.
- `cargo fmt --check`: exit 0.
- `git diff --check`: exit 0.

### Scope Confirmation

- No `std::algebra` source modules, algebra interfaces, algebra instances, `do:K` evidence rewiring, helper combinators, law-test runner behavior, or reference migration behavior were implemented. TASK-1021 through TASK-1028 remain planned downstream work.

## Dependencies for Next Task

This task outputs the verified audit gate required by downstream TASK-1021 through TASK-1028 work. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
