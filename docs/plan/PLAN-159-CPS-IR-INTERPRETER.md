---
id: plan.ash.cps-ir-interpreter
title: CPS IR Interpreter Implementation Plan
description: Phased implementation of the Ash CPS IR interpreter with S-expression testing format and formal operational semantics
kind: plan
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
---

# PLAN-159: CPS IR Interpreter

**Status:** 📝 Planned; 0/15 implemented — spec/plan only, no implementation
**Spec:** [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)
**Task range:** TASK-1590 through TASK-1604

## Overview

Builds a CPS IR interpreter for the target Ash language with gradual feature addition, thorough testing, and formal operational semantics developed in parallel. The interpreter is the core execution engine; bytecode serialization and JIT compilation are future concerns. The S-expression textual format serves as the common interface for testing and differential testing against a future Lean 4 implementation.

## Design Principles

1. **Interpreter is the core:** The CPS IR interpreter is the primary execution path. Bytecode and JIT are future layers.
2. **Vertical slices:** Each phase delivers interpreter + S-expression format + formal semantics for a feature subset.
3. **Differential testing ready:** The S-expression format is the contract between Rust and future Lean 4 implementations.
4. **Maintainability first:** Immutable environments, explicit handler chains, clear separation of concerns.
5. **Formal semantics in lockstep:** Operational semantics are developed alongside implementation, not after.

## Architecture

```
S-expression file (.cps)
    ↓
Rust interpreter ←── differential testing ──→ Lean 4 interpreter (future)
    ↓
Result (value or trap)
```

## Phases

### Phase 1: Core Terms (TASK-1590, TASK-1591, TASK-1601)

**Features:** `LetVal`, `LetPrim`, `LetCont`, `Jump`, `Call`, `Lam`, `Cont`

**Interpreter:** Basic eval for arithmetic, variable binding, continuation invocation.

**S-expression format:**
```lisp
(letval x 42
  (letcont k [v] (jump exit v {})
    (call f [x] k {})))
```

**Formal semantics:** Inference rules for core terms (§1).

**Testing:** Unit tests for each term form. Round-trip S-expression tests.

---

### Phase 2: Conditionals and Data (TASK-1592)

**Features:** `If`, `Record`, `Tuple`

**Interpreter:** Branching and structured data evaluation.

**S-expression format:**
```lisp
(if (> x 0)
  (record [ok true value x])
  (record [ok false value 0]))
```

**Formal semantics:** Inference rules for `If`, records, tuples (§2).

**Testing:** Conditional logic, pattern matching, round-trip S-expressions.

---

### Phase 3: Handlers and Effects (TASK-1593, TASK-1594, TASK-1595, TASK-1602)

**Features:** `Raise`, `Handle`, `HandlerChain`

**Interpreter:** Full handler dispatch with shallow vs provider semantics.

**S-expression format:**
```lisp
(handle DbReadOp
  (clause [resume] (jump resume 42 {}))
  (body (raise db.read [] k {}))
  (cont k {})
  (row {}))
```

**Formal semantics:** Inference rules for `Raise` dispatch, `Handle` installation, resume construction, chain capture (§3).

**Testing:** Retry pattern, rollback pattern, nested handlers, provider persistence. Differential tests.

---

### Phase 4: Recursion (TASK-1596)

**Features:** `LetRec`

**Interpreter:** Recursive functions with backfill.

**S-expression format:**
```lisp
(letrec factorial
  (lam [n] k
    (if (= n 0)
      (jump k 1 {})
      ...))
  (call factorial [5] exit {}))
```

**Formal semantics:** Inference rules for `LetRec` backfill (§4).

**Testing:** Recursive factorial, Fibonacci, retry loops. Differential tests.

---

### Phase 5: Advanced Features (TASK-1597, TASK-1598)

**Features:** `RecordDischarge`, `Trap`, row checker

**Interpreter:** Administrative terms and error handling.

**S-expression format:**
```lisp
(recorddischarge (contract "requires {b != 0}" Dynamic)
  (jump k result {}))

(trap ContractViolation)
```

**Formal semantics:** Inference rules for `RecordDischarge` (no-op), `Trap` (abort), row checker validation (§5).

**Testing:** Contract discharge example, error handling, row checker validation. Differential tests.

---

### Phase 6: S-expression Format (TASK-1599, TASK-1600)

**Features:** Parser and serializer for `.cps` files

**Interpreter:** Can load and save programs in S-expression format.

**Testing:** Round-trip tests, property tests for parser/serializer consistency.

---

### Phase 7: Differential Testing (TASK-1603)

**Features:** Test harness comparing Rust and future Lean 4 implementations

**Testing:** Both implementations read the same `.cps` file and produce identical results.

---

## Task Breakdown

| Task | Description | Phase | Status |
|------|-------------|-------|--------|
| TASK-1590 | Define core data structures: Atom, Value, Term, Env, HandlerChain | 1 | 📝 Planned |
| TASK-1591 | Implement eval for LetVal, LetPrim, LetCont, Jump, Call | 1 | 📝 Planned |
| TASK-1592 | Implement If, Record, Tuple evaluation | 2 | 📝 Planned |
| TASK-1593 | Implement Raise, Handle with handler chain walking | 3 | 📝 Planned |
| TASK-1594 | Implement shallow handler vs provider frame persistence | 3 | 📝 Planned |
| TASK-1595 | Implement resume continuation construction with env + chain capture | 3 | 📝 Planned |
| TASK-1596 | Implement LetRec with placeholder backfill for recursion | 4 | 📝 Planned |
| TASK-1597 | Implement RecordDischarge (no-op) and Trap (abort) | 5 | 📝 Planned |
| TASK-1598 | Implement row checker pass for local/total row validation | 5 | 📝 Planned |
| TASK-1599 | Implement S-expression parser for .cps files | 6 | 📝 Planned |
| TASK-1600 | Implement S-expression serializer for IR | 6 | 📝 Planned |
| TASK-1601 | Write formal operational semantics for core terms (§1) | 1 | 📝 Planned |
| TASK-1602 | Write formal operational semantics for handlers (§3) | 3 | 📝 Planned |
| TASK-1603 | Build differential test harness comparing Rust and future Lean 4 | 7 | 📝 Planned |
| TASK-1604 | Close out Phase 159 with verification, documentation, and changelog | 7 | 📝 Planned |

## Formal Semantics Document Structure

```text
Operational Semantics for Ash CPS IR
=====================================

§1: Syntax
  Atom, Value, Term definitions (inductive types)

§2: Core Terms (Phase 1)
  LetVal, LetPrim, LetCont, Jump, Call

§3: Conditionals and Data (Phase 2)
  If, Record, Tuple

§4: Handlers (Phase 3)
  Raise, Handle, HandlerChain, Resume

§5: Recursion (Phase 4)
  LetRec, Backfill

§6: Advanced (Phase 5)
  RecordDischarge, Trap, Row Checker

§7: Metatheory (Future)
  Type safety
  Handler chain invariants
  Row consistency
```

## Testing Strategy

### Unit Tests
- One test per term form
- One test per value constructor
- One test per handler chain operation

### Integration Tests
- Full programs: factorial, Fibonacci, retry, rollback
- Handler patterns: nested handlers, provider persistence
- Error cases: unhandled operations, contract violations

### Property Tests
- Round-trip: parse → serialize → parse produces same IR
- Row checker: valid rows pass, invalid rows fail
- Handler chain: capture and restore invariants

### Differential Tests
- Same `.cps` file through both implementations
- Compare results and execution traces

## Deliverables

| Milestone | Criteria |
|-----------|----------|
| M1 (Phase 1) | `cargo test` passes, simple arithmetic works, formal semantics §1 complete |
| M2 (Phase 2) | Branching and records work, formal semantics §2 complete |
| M3 (Phase 3) | Retry/rollback handler patterns work, formal semantics §3 complete |
| M4 (Phase 4) | Recursive functions work, formal semantics §4 complete |
| M5 (Phase 5) | All term forms + row checker work, formal semantics §5 complete |
| M6 (Phase 6) | `.cps` test suite runs, round-trip tests pass |
| M7 (Phase 7) | Differential test harness ready for Lean 4 integration |

## What to Skip (For Now)

- No bytecode serialization (S-expression is the format)
- No JIT compilation
- No mutual recursion (single-variable `LetRec` only)
- No row polymorphism (concrete rows only)
- No effect aliases
- No direct-style fragments

## Open Decisions

1. Whether to use `Rc<<RefCell<<...>>` or immutable structures with structural sharing for environments
2. Whether to represent `HandlerChain` as `Vec` or linked list
3. Whether to include answer type `Ans` in the runtime interpreter or only in the type checker
4. Whether the formal semantics should be written in Lean 4 directly or as LaTeX-style inference rules

## See Also

- [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [PLAN-INDEX](PLAN-INDEX.md) — Master task index
