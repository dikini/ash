# TASK-1031: Comonad and Kleisli Audit Gate

## Status: ✅ Complete

## Description

Audit the live Ash algebra, parser, typechecker, stdlib, module-loading, and evidence-selection seams before any Comonad/Kleisli/Cokleisli/Coapplicative source implementation starts. This task freezes exact accepted Ash syntax, classifies lawful carrier candidates, and patches downstream task verification placeholders with concrete non-zero commands or artifact assertions.

## Specification Reference

- [SPEC-079](../../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [PLAN-129](../PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)

## Dependencies

- ✅ TASK-1030: Planning packet (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Logical generic signatures | SPEC-079 | Current source interfaces may be monomorphic MVP shapes | unknown | audit exact syntax before source work | audit section with accepted/rejected syntax |
| Coapplicative | SPEC-079/user request | no precise laws or carrier chosen | unknown | classify before TASK-1035 | decision matrix row |
| Lawful Comonad carriers | SPEC-079 | extraction must be total | unknown | classify candidates | negative rows for partial/opaque carriers |

## Requirements

1. Create `docs/plan/audits/TASK-1031-comonad-kleisli-audit.md`.
2. Inspect current `std/src/algebra/*.ash`, `std/src/option.ash`, `std/src/result.ash`, `std/src/list.ash`, `std/src/act.ash`, `std/src/proc.ash`, and `std/src/workflow.ash`.
3. Inspect parser/typechecker/module-loader seams relevant to interfaces, impls, constructor-kinded parameters, function types, helper functions, selected evidence, and stdlib imports.
4. Freeze exact accepted source syntax for `Comonad`, Kleisli helpers, Cokleisli helpers, and any Coapplicative candidate.
5. Classify candidate Comonad carriers, including required negative rows for `Option`, `Result`, ordinary `List`, `Act`, `Proc`, and `Workflow`.
6. Decide whether TASK-1032 through TASK-1036 are implementable source tasks or must remain named deferrals.
7. Patch TASK-1032 through TASK-1037 verification blocks with exact non-zero focused commands or artifact assertions.

## TDD / Audit Steps

### Step 1: Inspect live seams

**Files to inspect:**

- `std/src/algebra/mod.ash`
- `std/src/algebra/functor.ash`
- `std/src/algebra/applicative.ash`
- `std/src/algebra/monad.ash`
- `std/src/option.ash`
- `std/src/result.ash`
- `std/src/list.ash`
- `std/src/act.ash`
- `std/src/proc.ash`
- `std/src/workflow.ash`
- parser/typechecker/module-loader/evidence files discovered by search

### Step 2: Write audit artifact

**File:** `docs/plan/audits/TASK-1031-comonad-kleisli-audit.md`

The artifact must contain sections named:

- `## Live syntax findings`
- `## Interface and impl registration seams`
- `## Module loading and stdlib import seams`
- `## Evidence selection and helper-function seams`
- `## Comonad carrier classification`
- `## Coapplicative decision inputs`
- `## Downstream verification replacements`

### Step 3: Patch downstream task commands

Review the preliminary commands in TASK-1032 through TASK-1037 and patch them if the live audit finds better exact non-zero focused commands. If a downstream row remains unimplementable, use an artifact assertion proving the named deferral exists rather than a command that can pass vacuously.

### Step 4: Independent review

Dispatch spec and quality review. Reviewers must check that the audit does not invent syntax, does not approve unsound instances, and does not leave downstream placeholders.

## Acceptance Rows

| Area | Acceptance |
|---|---|
| Audit artifact | `docs/plan/audits/TASK-1031-comonad-kleisli-audit.md` exists with all required sections. |
| Syntax | Exact accepted source spelling is frozen or blocked with file/symbol evidence. |
| Carrier policy | Required negative carrier rows are present. |
| Downstream commands | TASK-1032 through TASK-1037 no longer contain fail-closed placeholder commands. |
| Reviews | Spec and quality reviews pass after audit patching. |

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
  - test -f docs/plan/audits/TASK-1031-comonad-kleisli-audit.md
  - python3 -c 'from pathlib import Path; text=Path("docs/plan/audits/TASK-1031-comonad-kleisli-audit.md").read_text(); required=["Live syntax findings","Interface and impl registration seams","Module loading and stdlib import seams","Evidence selection and helper-function seams","Comonad carrier classification","Coapplicative decision inputs","Downstream verification replacements"]; missing=[r for r in required if r not in text]; assert not missing, missing'
  - python3 -c 'from pathlib import Path; paths=list(Path("docs/plan/tasks").glob("TASK-103[2-7]-*.md")); bad=[str(p) for p in paths if "false # TASK-1031 audit" in p.read_text()]; assert not bad, bad'
  - git diff --check
checklist:
  - [x] Audit artifact exists
  - [x] Exact syntax frozen or blocked
  - [x] Carrier classification complete
  - [x] Downstream commands patched
  - [x] Independent spec review complete
  - [x] Independent quality review complete
```

## Dependencies for Next Task

TASK-1032 through TASK-1037 may not start until this audit completes and patches their verification blocks.
