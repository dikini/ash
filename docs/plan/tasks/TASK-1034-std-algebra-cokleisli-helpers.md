# TASK-1034: Std Algebra Cokleisli Helpers

## Status: ✅ Complete

## Description

Add a Cokleisli helper surface over `Comonad<W>` evidence after the Comonad interface exists or is otherwise audit-approved.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031: Audit gate (complete)
- ✅ TASK-1032: Comonad interface (complete)

## Target Files

- `std/src/algebra/cokleisli.ash`
- `std/src/algebra/mod.ash`
- `std/src/algebra/comonad.ash` if helper colocating is audit-approved
- focused helper tests named by TASK-1031

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Depend on real or audit-approved `Comonad<W>` evidence.
2. Expose only Cokleisli helper functions expressible in current Ash.
3. Do not introduce `Category` or `std::category`.
4. Do not fabricate Comonad evidence just to test Cokleisli helpers.
5. If no lawful Comonad carrier exists, record an honest deferral or interface-only test boundary.

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
| Comonad dependency | Helpers use final Comonad surface or are blocked until it exists. |
| No category overclaim | No `Category` interface or `std::category` source is added. |
| Final path | Tests import `std::algebra::cokleisli` or record an exact deferral. |
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
  - python3 -c 'from pathlib import Path; audit=Path("docs/plan/audits/TASK-1031-comonad-kleisli-audit.md").read_text(); assert "Cokleisli helpers are not source-implemented" in audit and not Path("std/src/algebra/cokleisli.ash").exists()'
  - git diff --check
checklist:
  - [x] TASK-1031 has confirmed this deferral assertion is exact
  - [x] Cokleisli helpers are deferred until public Comonad evidence has a lawful carrier or source evidence-method dispatch exists
  - [x] No `Category` interface or `std::category` source introduced
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task feeds TASK-1036 law/reference reconciliation and TASK-1037 closeout.
