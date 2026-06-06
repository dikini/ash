# TASK-1026: Algebra Law Profile Generated-Test Handoff

## Status: 📝 Planned

## Description

Create the concrete follow-up task/phase seed for derived/generated algebra law tests, explicitly deferring law proof/checking and law-test execution from this stdlib-surface phase to an owned generated-test implementation packet while preserving law metadata requirements.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1021 through TASK-1025 completion or explicit TASK-1020 audit approval that enough instance decisions are known to create concrete law-test follow-up ownership.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Document law profiles for Semigroup, Monoid, Functor, Applicative, and Monad as normative contracts.
2. Create a mandatory follow-up plan/task seed for generated law tests integrated with the SPEC-077 runner framework; a prose-only audit note is not sufficient.
3. Define what metadata/generators/equivalence relations pure and tower instances need for future law tests.
4. Do not implement law-test runner execution in this task.
5. Create a concrete future task file or PLAN-INDEX reserved phase row for generated algebra law tests; keyword-only audit notes are insufficient.

## File Targets

- Create: `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`
- Modify: `docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md` if audit finds missing law-profile requirements
- Create: `docs/plan/tasks/TASK-XXXX-generated-algebra-law-tests.md` or a reserved PLAN-INDEX phase row named Generated Algebra Law Tests, with acceptance rows and ownership recorded in the audit artifact

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

Implement TASK-1026 as docs/planning only. Create a law-profile handoff artifact plus a concrete follow-up task/phase seed for a later generated-test phase. Do not add law proof syntax, do not implement runner law generation, and do not describe laws as documentation-only.
```

### Spec reviewer

```text
Review TASK-1026 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1026 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
p=Path('docs/plan/audits/TASK-1026-algebra-law-test-handoff.md')
assert p.is_file(), p
text=p.read_text()
for s in ['Semigroup','Monoid','Functor','Applicative','Monad','generated test','SPEC-077','follow-up task','acceptance rows','owner','pure instances','tower carriers']:
    assert s in text, s
assert any(Path('docs/plan/tasks').glob('TASK-*-generated-algebra-law-tests.md')) or 'Generated Algebra Law Tests' in Path('docs/plan/PLAN-INDEX.md').read_text()
print('law-test handoff artifact and concrete follow-up owner exist')
PY
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] RED evidence recorded
  - [ ] GREEN evidence recorded
  - [ ] Audit artifact verification command is non-zero and passes
  - [ ] Final-surface or negative-leakage gates satisfied where applicable
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs the verified slice required by downstream TASK-1020..TASK-1028 work. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
