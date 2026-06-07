# TASK-1032: Std Algebra Comonad Interface

## Status: ✅ Complete

## Description

Add the `std::algebra::comonad` module and `Comonad` interface only after TASK-1031 freezes exact source syntax. If the audit blocks implementation, record the blocker and keep the source module absent.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031: Audit gate (complete)

## Target Files

- `std/src/algebra/comonad.ash`
- `std/src/algebra/mod.ash`
- carrier impl files approved by TASK-1031, if any
- focused parser/typechecker/engine tests named by TASK-1031

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Create `std/src/algebra/comonad.ash` only with syntax approved by TASK-1031.
2. Export `Comonad` from `std/src/algebra/mod.ash` if implemented.
3. Expose `extract` and `extend` as the first-slice methods, or record the audit-approved primitive alternative.
4. Do not add `Comonad` instances for `Option`, `Result`, ordinary `List`, `Act`, `Proc`, or `Workflow`.
5. Add final-path import/typecheck evidence or a named audit blocker.

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
| Importability | `use algebra::comonad::{Comonad}` works through the real stdlib path, or the audit blocker is recorded. |
| Interface shape | Method names and signatures match TASK-1031 exact syntax. |
| Negative instances | Partial/opaque carriers have no Comonad evidence. |
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
  - test -f std/src/algebra/comonad.ash
  - python3 -c 'from pathlib import Path; text=Path("std/src/algebra/mod.ash").read_text(); assert "pub mod comonad;" in text and "pub use comonad::{Comonad};" in text'
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --nocapture
  - git diff --check
checklist:
  - [x] TASK-1031 has confirmed these commands are exact and non-zero
  - [x] `std::algebra::comonad` final-surface import evidence recorded
  - [x] Negative partial/opaque carrier instance evidence recorded
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task feeds TASK-1036 law/reference reconciliation and TASK-1037 closeout.
