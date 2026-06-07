# TASK-1036: Comonad Law Profile and Reference Docs

## Status: ✅ Complete

## Description

Extend law-profile/generated-test handoff for Comonad, Kleisli, and Cokleisli laws, and update reference/corpus docs to distinguish implemented, planned, and deferred surfaces.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031: Audit gate (complete)
- ✅ TASK-1032: Comonad interface (complete)
- ✅ TASK-1033: Kleisli helpers (complete)
- ✅ TASK-1034: Cokleisli helper deferral (complete)
- ✅ TASK-1035: Coapplicative decision (complete)

## Target Files

- `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md` or a new `docs/plan/audits/TASK-1036-comonad-law-test-handoff.md`
- `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md` (extend this existing generated-law owner; do not create a parallel owner unless TASK-1029 is explicitly superseded)
- `reference/stdlib/algebra.md`
- `reference/stdlib/README.md`
- `CHANGELOG.md`

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Extend TASK-1029 as the concrete generated-law owner for Comonad, Kleisli, and Cokleisli law profiles.
2. Record Comonad, Kleisli, and Cokleisli law profiles with generator/equivalence prerequisites.
3. Record Coapplicative law status from TASK-1035.
4. Update reference docs to describe only actual implemented APIs as implemented.
5. Sweep stale wording around Comonad/category/helper deferrals.

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
| Law ownership | TASK-1029 is explicitly extended for Comonad/Kleisli/Cokleisli generated law tests. |
| Law profiles | Comonad/Kleisli/Cokleisli laws are listed with generated-test prerequisites. |
| Reference truth | Reference docs distinguish implemented, planned, and deferred surfaces. |
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
  - python3 -c 'from pathlib import Path; text=Path("docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md").read_text(); required=["Comonad", "Kleisli", "Cokleisli"]; missing=[r for r in required if r not in text]; assert not missing, missing'
  - python3 -c 'from pathlib import Path; text=Path("reference/stdlib/algebra.md").read_text(); assert "Comonad" in text and "Kleisli" in text and "SPEC-079" in text'
  - git diff --check
checklist:
  - [x] TASK-1029 explicitly extended for Comonad/Kleisli/Cokleisli generated law tests
  - [x] Comonad/Kleisli/Cokleisli law profiles recorded with generator/equivalence prerequisites
  - [x] Reference docs distinguish implemented, planned, and deferred surfaces
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task feeds TASK-1036 law/reference reconciliation and TASK-1037 closeout.
