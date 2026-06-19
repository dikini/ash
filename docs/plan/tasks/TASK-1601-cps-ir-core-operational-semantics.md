# TASK-1601: Write architecture and non-handler operational semantics

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Write the repository Markdown architecture/operational-semantics document for non-handler CPS IR terms: `Atom`, `Value`, `Term`, `LetVal`, `LetPrim`, `LetCont`, `Jump`, `Call`, `If`, `Record`, `Tuple`, `LetRec`, `RecordDischarge`, `Trap`, and row-checker validation. TASK-1602 owns the handler-specific §4 rules.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590: Define CPS IR core data structures.
- 📝 TASK-1591: Evaluate core CPS values and terms.
- 📝 TASK-1592: Evaluate conditionals and structured data.
- 📝 TASK-1596: Implement single-binding LetRec recursion.
- 📝 TASK-1597: Implement RecordDischarge and Trap.
- 📝 TASK-1598: Implement row representation and local/total row validation scaffold.

## Requirements

### Functional Requirements

1. Create or update `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` as the concrete document path named by PLAN-159 and PLAN-INDEX.
2. Define syntax in §1 and core term rules in §2.
3. Define conditionals/data rules in §3, recursion rules in §5, and advanced/row-checker rules in §6.
4. State that legacy AST lowering and Lean 4 differential testing are future work outside PLAN-159.
5. State fixed answer-type discipline explicitly.
6. Include at least one worked normalized CPS example.
7. Cross-link SPEC-098b and PLAN-159.

### Property Requirements

- Rule numbering in the semantics document matches PLAN-159.
- Examples obey the declared grammar.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** Documentation verification script plus any doctest-like fixture checks introduced by the task.

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`, `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-INDEX.md`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Update PLAN-159 and PLAN-INDEX to point at `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` once this task creates it.

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
  - git diff --check -- docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md docs/plan/PLAN-159-CPS-IR-INTERPRETER.md docs/plan/PLAN-INDEX.md
  - python3 -c 'from pathlib import Path; p=Path("docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md"); text=p.read_text(); required=["§1", "§2", "§3", "§5", "§6", "Answer type", "LetPrim", "LetRec", "RecordDischarge", "Trap"]; missing=[s for s in required if s not in text]; assert not missing, missing'
checklist:
  - [ ] Semantics document path exists
  - [ ] §1, §2, §3, §5, and §6 are present and match PLAN-159
  - [ ] Examples are normalized CPS IR
```

## Dependencies for Next Task

- Provides the normative core semantics required by closeout review.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
