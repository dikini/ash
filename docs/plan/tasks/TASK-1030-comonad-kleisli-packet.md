# TASK-1030: Comonad and Kleisli Helper Planning Packet

## Status: ✅ Complete

## Description

Create the docs-only planning packet for adding `Comonad`, Kleisli helpers, Cokleisli helpers, and a Coapplicative decision gate to `std::algebra`. This task produces implementation-grade specs and task files, but it must not add stdlib source modules or claim runtime behavior.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1028: Standard algebra closeout (complete)
- 📝 TASK-1029: Generated algebra law tests (planned, related follow-up)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Comonad future work | SPEC-078 | Algebra namespace not stable | satisfied | create new packet | SPEC/PLAN/task/index files exist |
| Category hierarchy | SPEC-078 | too broad for algebra MVP | not satisfied | keep deferred | no `std::category` in packet |
| Coapplicative | user request | no precise Ash contract yet | unknown | decision-gated | TASK-1035 owns decision |

## Requirements

1. Create SPEC-079 as a Draft spec.
2. Create PLAN-129 as a Planned phase plan.
3. Create TASK-1030 through TASK-1037 task files.
4. Update `docs/spec/README.md` with SPEC-079.
5. Update `docs/plan/PLAN-INDEX.md` with Phase 134.
6. Update `CHANGELOG.md` under `[Unreleased]`.
7. Add a narrow reference note to `reference/stdlib/algebra.md` describing the planned follow-on without claiming implementation.
8. Patch SPEC-078 future-work wording so Comonad is now owned by SPEC-079 while category-level abstractions remain deferred.

## TDD Steps

### Step 1: Write docs packet

**Files:**

- `docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- `docs/plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md`
- `docs/plan/tasks/TASK-1030-*.md` through `TASK-1037-*.md`
- `docs/spec/README.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`
- `reference/stdlib/algebra.md`

**Target state:** The packet is implementation-grade, but all implementation tasks after TASK-1030 remain planned and audit-gated.

### Step 2: Verify structure

Run structural checks that prove every linked file exists and every new link resolves.

### Step 3: Review

Dispatch independent spec and quality reviewers. Fix any scope overclaims, especially unsupported syntax, fake instances, and broad category claims.
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
  - python3 -c 'from pathlib import Path; files=["docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md","docs/plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md"]+[f"docs/plan/tasks/TASK-{n}-" for n in range(1030,1038)]; missing=[]; import glob; [missing.append(f) for f in files[:2] if not Path(f).exists()]; [missing.append(prefix) for prefix in files[2:] if not glob.glob(prefix+"*.md")]; assert not missing, missing'
  - python3 -c 'from pathlib import Path; text=Path("docs/plan/PLAN-INDEX.md").read_text(); assert "Phase 134" in text and "PLAN-129" in text and "TASK-1037" in text'
  - python3 -c 'from pathlib import Path; text=Path("docs/spec/README.md").read_text(); assert "SPEC-079" in text and "Comonad" in text'
  - python3 -c 'from pathlib import Path; text=Path("CHANGELOG.md").read_text(); assert "TASK-1030" in text and "SPEC-079" in text'
  - git diff --check
checklist:
  - [x] Packet files exist
  - [x] PLAN-INDEX and spec README linked
  - [x] Changelog updated
  - [x] Reference note is planned/future only
```

## Completion Notes

Completed on 2026-06-07. Created SPEC-079, PLAN-129, TASK-1030 through TASK-1037, the Phase 134 PLAN-INDEX row, spec index entry, changelog entry, and reference/SPEC-078 consistency notes. The packet is docs-only: later source implementation remains planned behind TASK-1031's audit gate.

## Dependencies for Next Task

This task outputs the planning packet consumed by TASK-1031 through TASK-1037.
