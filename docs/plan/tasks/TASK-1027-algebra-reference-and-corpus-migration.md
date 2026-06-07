# TASK-1027: Algebra Reference and Corpus Migration

## Status: ✅ Complete

## Description

Update reference documentation and reconcile stale generalized-do/Monad/stdlib wording so users and future agents see the new `std::algebra` surface and the honest remaining deferrals.

## Specification Reference

- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [PLAN-128](../PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

Depends on TASK-1025 and TASK-1026 completion.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Public stdlib `Monad<M>` deferred | SPEC-054/SPEC-067 | HKT/evidence incomplete | satisfied by SPEC-067/069 | implement now | final-surface import/evidence tests |
| Hidden tower dictionaries | SPEC-054/SPEC-069 | temporary bridge before visible algebra | public tower ops and do evidence exist | retire/quarantine | negative leakage test |
| Pure `Option`/`Result`/`List` dictionaries deferred | SPEC-055/SPEC-067 | pure container evidence and partial constructor targets incomplete | `Option`, `Result`, `List`, and `Result<_, E>` substrate exist | implement now where source syntax supports it | stdlib evidence tests for pure carriers |
| Law proof/test derivation | SPEC-054/SPEC-067 | no law runner/proof substrate | generated runner exists but law profiles unspecified | concrete follow-up phase/task seed | TASK-1026 owned generated-test handoff |
| Fully self-hosted tower runtime representation | SPEC-047..SPEC-051/SPEC-069 | opaque Act/Proc/Workflow runtime state remains Rust-backed | still true | keep deferred explicitly | opacity/fail-closed tests continue |
## Requirements

1. Add or update reference pages for `std::algebra` interfaces, instances, examples, and do/comprehension usage.
2. Patch stale wording in SPEC-054, SPEC-055, SPEC-067, SPEC-069, and reference docs that still treats stdlib Monad as unavailable without historical qualification.
3. Keep historical docs honest: old deferrals may remain as history only when clearly superseded.
4. Update agent cards/context packs if current reference pages require it.

## File Targets

- Create/modify: `reference/stdlib/algebra.md`
- Modify: `reference/language/generalized-do.md`
- Modify: `docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md` as needed
- Modify: `docs/spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md` as needed
- Modify: `docs/spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md` as needed
- Modify: `docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md` as needed

## TDD / Execution Steps

1. Re-read SPEC-078 and this task file.
2. Write RED tests or audit evidence proving the current gap.
3. Implement only this task's slice without pulling later tasks forward.
4. Run focused non-zero verification.
5. Record RED/GREEN evidence and update task/plan/changelog only when the slice is actually complete.

## Evidence

- RED: Reference docs had no dedicated current `std::algebra` page and generalized-do wording did not mention public `Monad<K>` evidence's canonical `unit` method.
- GREEN: Added `reference/stdlib/algebra.md`, refreshed `reference/language/generalized-do.md`, and ran the scoped stale-wording assertion from this task.

## Sub-Agent Prompts

### Implementer

```text
Repository: /home/dikini/Projects/ash. Read this task file, docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, docs/plan/PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md, and docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md; fail if the audit is missing. Constraints: no new Ash syntax; use final-surface std::algebra/import tests rather than local fixture-only evidence; do not preserve obsolete deferrals unless this task records a concrete current blocker and follow-up.

Implement TASK-1027 after the public surface exists. Update user/reference docs and stale spec wording. Keep the sweep scoped to current normative/reference surfaces; do not rewrite history unless it misleads future implementation.
```

### Spec reviewer

```text
Review TASK-1027 against SPEC-078 and PLAN-128. Check final-surface evidence, deferral disposition, exact file targets, and whether the task overclaims by using bridge-only or fixture-only tests. Return PASS or specific REQUEST_CHANGES findings.
```

### Quality reviewer

```text
Review TASK-1027 for maintainability and Ash project conventions. Look for hidden bridge leakage, stale deferral wording, missing docs/changelog updates, non-zero test coverage, and scope creep. Return APPROVED or REQUEST_CHANGES with concrete fixes.
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
page=Path('reference/stdlib/algebra.md')
assert page.is_file(), page
page_text=page.read_text(errors='ignore')
for s in ['std::algebra','Semigroup','Monoid','Functor','Applicative','Monad','instances','examples','do:','comprehension']:
    assert s in page_text, s
paths=[Path('reference'),Path('docs/spec'),Path('docs/plan')]
terms=['stdlib Monad deferred','future Monad evidence only','hidden Act dictionary','Monad dictionaries deferred','pure List/Option/Result dictionaries remain deferred','Option/Result/List dictionaries deferred','stdlib Monad unavailable','bridge dictionaries','hidden dictionaries','Generalized runtime lowering through arbitrary user-defined Monad']
for root in paths:
    for p in root.rglob('*.md'):
        text=p.read_text(errors='ignore')
        for t in terms:
            if t in text and 'historical' not in text.lower() and 'superseded' not in text.lower():
                raise SystemExit(f'stale unqualified wording: {p}: {t}')
print('scoped stale wording check passed')
PY
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] RED evidence recorded
  - [x] GREEN evidence recorded
  - [x] Reference page artifact assertion passes and stale wording sweep reports no unqualified current deferrals
  - [x] Final-surface or negative-leakage gates satisfied where applicable
  - [x] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs current reference and stale-deferral reconciliation required by TASK-1028. Downstream tasks must not silently expand or preserve old deferrals without updating SPEC-078/PLAN-128.
