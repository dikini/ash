# TASK-1602: Write handler operational semantics

**Status:** 📝 Planned
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Write the operational semantics for `Raise`, `Handle`, handler/provider chains, resume construction, one-shot use, and row transformation.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1593: Implement Raise and Handle dispatch.
- 📝 TASK-1594: Separate shallow handlers from provider frames.
- 📝 TASK-1595: Construct and enforce resume continuations.
- 📝 TASK-1601: Write architecture and non-handler operational semantics.

## Requirements

### Functional Requirements

1. Define handler frame installation and chain walking.
2. Define resume construction with environment and chain capture.
3. State shallow handler versus provider-frame persistence rules.
4. State one-shot resume behavior and the initial runtime-trap stopgap.
5. Define local/residual row transformation for handlers and provider frames.

### Property Requirements

- Handler rules agree with SPEC-098b `Raise.row` and `Handle.row` meanings.
- Resume examples do not rely on implicit direct-style returns.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** Documentation verification script plus handler fixture cross-checks introduced by the task

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Update PLAN-159 and task links if handler semantics live in a new document.

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
  - git diff --check -- docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md docs/plan/PLAN-159-CPS-IR-INTERPRETER.md
  - python3 -c 'from pathlib import Path; text=Path("docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md").read_text(); required=["§4", "Raise", "Handle", "handler frame", "resume", "shallow", "provider", "one-shot", "row transformation"]; missing=[s for s in required if s not in text]; assert not missing, missing'
checklist:
  - [ ] Handler dispatch rules present
  - [ ] Resume construction rules present
  - [ ] Shallow/provider distinction present
  - [ ] One-shot behavior present
```

## Dependencies for Next Task

- Provides the normative handler semantics required by closeout review.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
