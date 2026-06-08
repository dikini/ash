# TASK-1033: Std Algebra Kleisli Helpers

## Status: ✅ Complete

## Description

Add a Kleisli helper surface over existing Phase 133 `Monad<M>` evidence without changing `do:K` or introducing `std::category`.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031: Audit gate (complete)

## Target Files

- `std/src/algebra/kleisli.ash`
- `std/src/algebra/mod.ash`
- `std/src/algebra/monad.ash` if helper re-exports are audit-approved
- focused Option/Result helper tests named by TASK-1031

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Implement only helper functions expressible in current Ash source.
2. Reuse existing `Monad<M>` evidence and public `unit`/`bind` surfaces.
3. Do not alter generalized `do:K` or comprehension lowering.
4. Do not introduce `Category` or `std::category`.
5. If generic composition cannot be expressed honestly, record a named deferral; do not add concrete Option/Result wrappers to `std::algebra`.

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
| Monad reuse | Helpers call/reuse public Monad evidence rather than hidden dictionaries. |
| No lowering change | `do:K` and comprehension code paths are untouched unless TASK-1031 explicitly requires tests only. |
| Final path | Tests import `std::algebra::kleisli` or record an exact deferral. |
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
  - test -f std/src/algebra/kleisli.ash
  - python3 -c 'from pathlib import Path; text=Path("std/src/algebra/mod.ash").read_text(); assert "pub mod kleisli;" in text'
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --nocapture
  - git diff --check
checklist:
  - [x] TASK-1031 has confirmed these commands are exact and non-zero
  - [x] Kleisli helpers use public Monad evidence
  - [x] No `do:K` lowering or `std::category` changes introduced
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task feeds TASK-1036 law/reference reconciliation and TASK-1037 closeout.
