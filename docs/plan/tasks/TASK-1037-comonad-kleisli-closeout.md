# TASK-1037: Comonad and Kleisli Closeout

## Status: ✅ Complete

## Description

Close Phase 134 only after implemented surfaces pass focused and broad gates, deferred surfaces are named honestly, and independent review approves status promotion.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)
- ✅ TASK-1031 through TASK-1036 (complete; TASK-1034/TASK-1035 are explicit deferrals)

## Target Files

- `docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- `docs/plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/plan/tasks/TASK-1030-*.md` through `TASK-1037-*.md`
- `docs/spec/README.md`
- `reference/stdlib/algebra.md`
- `CHANGELOG.md`

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Unsupported generic algebra syntax | TASK-1031/SPEC-079 | live source may not accept logical signatures | pending audit | use audit-approved syntax or defer | exact focused command/artifact assertion |
| Broad category hierarchy | SPEC-078/SPEC-079 | out of phase scope | no | keep deferred | no `std::category`/`Category` source |
| Unsound Comonad instances | SPEC-079 | extraction must be total | carrier-specific | reject unless lawful | negative evidence/audit row |

## Requirements

1. Run all TASK-1031-selected focused gates.
2. Run broad cargo/doc/format/clippy/test gates required by PLAN-129.
3. Reconcile SPEC-079, PLAN-129, PLAN-INDEX, task statuses, changelog, reference docs, and closeout evidence.
4. Run independent spec and quality review after reconciliation.
5. Promote statuses only for surfaces actually implemented and verified.

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
| Gates | Focused and broad commands are recorded with exit status. |
| Status | SPEC/PLAN/tasks/PLAN-INDEX statuses agree. |
| Deferred rows | Category, Coapplicative, laws, or helper gaps remain explicitly deferred where not implemented. |
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
  - cargo fmt --check
  - RUSTC_WRAPPER= cargo check --workspace
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --list
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --list
  - RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --list
  - RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --nocapture
  - RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --list
  - RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --nocapture
  - git diff --check
  - python3 -c 'from pathlib import Path; pi=Path("docs/plan/PLAN-INDEX.md").read_text(); spec=Path("docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md").read_text(); plan=Path("docs/plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md").read_text(); assert "Phase 134" in pi and "SPEC-079" in spec and "TASK-1037" in plan'
checklist:
  - [x] Focused TASK-1031-selected gates recorded
  - [x] Broad cargo gates pass
  - [x] SPEC/PLAN/PLAN-INDEX/task/changelog/reference statuses reconciled
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

This task closes Phase 134 and reconciles SPEC-079, PLAN-129, PLAN-INDEX, task statuses, changelog, reference docs, and closeout evidence.

## Completion Notes

Completed on 2026-06-07. Closeout evidence is recorded in `docs/plan/audits/TASK-1037-comonad-kleisli-closeout.md`. Phase 134 is complete for the current source slice: `Comonad` interface and concrete Option/Result Kleisli helpers are implemented; Cokleisli, Coapplicative, generated law execution, and broad category hierarchy work remain explicit deferrals.
