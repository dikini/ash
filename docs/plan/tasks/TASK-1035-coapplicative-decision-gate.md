# TASK-1035: Coapplicative Decision Gate

## Status: ✅ Complete

## Description

Choose whether `Coapplicative` has a precise, lawed, lawful first slice for Ash. If not, defer it explicitly and keep source modules absent.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031: Audit gate (complete)

## Target Files

- `docs/plan/audits/TASK-1035-coapplicative-decision.md`
- `docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md` if the decision amends scope
- `docs/plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md` if the decision amends tasks
- `std/src/algebra/coapplicative.ash` only if implementation is approved

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Define the intended meaning of Coapplicative in Ash or reject the name for this phase.
2. If implementing, specify method names, laws, and at least one lawful carrier.
3. If deferring, state the blocker and do not create a source module that looks implemented.
4. Update SPEC-079/PLAN-129 if the decision changes scope.
5. Add final-path tests only for implemented surfaces; otherwise add deferral artifact checks.

## TDD / Execution Steps

### Step 1: Red evidence or decision artifact

Create the failing final-surface test, audit artifact, law-profile row, or status assertion that proves the current gap this task owns. Do not use local fixture-only evidence when SPEC-079 requires final stdlib paths.

### Step 2: Minimal implementation or honest deferral

Implement the task's scoped source/docs surface using only TASK-1031-approved syntax. If the approved syntax or lawful carrier does not exist, patch the task/plan/spec with a named deferral instead of adding placeholder APIs.

### Step 3: Integration and corpus update

Wire only the files listed in `Target Files`. Update reference/changelog/status surfaces if this task changes source, docs policy, or phase status.

### Step 4: Review

Run independent spec and quality review. Fix category overclaims, unsupported syntax, fixture-only tests, stale status rows, and unsound instance claims before marking complete.

## Acceptance Rows

| Area | Acceptance |
|---|---|
| Decision record | `docs/plan/audits/TASK-1035-coapplicative-decision.md` records implement vs defer. |
| Law clarity | Implemented path names laws; deferred path names missing law/carrier reason. |
| No placeholder API | No vague source `Coapplicative` module is added. |
| Status/docs | Changelog and tracking surfaces are updated if this task changes them. |

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - test -f docs/plan/audits/TASK-1035-coapplicative-decision.md
  - python3 -c 'from pathlib import Path; text=Path("docs/plan/audits/TASK-1035-coapplicative-decision.md").read_text(); assert ("Decision: implement" in text and "Lawful carrier" in text and "Laws" in text) or ("Decision: defer" in text and "No source module" in text)'
  - python3 -c 'from pathlib import Path; p=Path("std/src/algebra/coapplicative.ash"); decision=Path("docs/plan/audits/TASK-1035-coapplicative-decision.md").read_text(); assert p.exists() == ("Decision: implement" in decision)'
  - git diff --check
checklist:
  - [x] Coapplicative decision record exists
  - [x] Implemented path names laws and a lawful carrier, or deferred path keeps source absent
  - [x] SPEC/PLAN scope patched if decision changes phase scope
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task feeds TASK-1036 law/reference reconciliation and TASK-1037 closeout.
