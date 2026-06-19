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
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
---

# PLAN-159: CPS IR Interpreter

**Status:** 📝 Planned; 0/14 implemented — spec/plan only, no implementation
**Spec:** [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)
**Task range:** TASK-1590 through TASK-1603

## Overview

Builds an isolated prototype CPS IR interpreter for the target Ash language with gradual feature addition, thorough testing, and formal operational semantics developed in parallel. The interpreter prototype executes hand-authored Target CPS IR fixtures directly. Legacy lowering, differential testing against Lean 4, bytecode serialization, and JIT compilation are future concerns outside this phase.

## Design Principles

1. **Interpreter is the core:** The CPS IR interpreter is the primary execution path. Bytecode and JIT are future layers.
2. **Vertical slices:** Each phase extends the interpreter, the minimal S-expression grammar, and the formal semantics for a feature subset. The parser/serializer scaffold starts in Phase 1 and is hardened in Phase 6.
3. **Future comparison ready:** The S-expression format is the prototype fixture contract. It must encode the SPEC-098b `Atom` / `Value` / `Term` split directly, not a friendlier direct-style surface, so a future Lean 4 implementation can reuse the same corpus.
4. **Maintainability first:** Immutable environments, explicit handler chains, clear separation of concerns.
5. **Formal semantics in lockstep:** Operational semantics are developed alongside implementation, not after.

## Architecture

```text
S-expression .cps ── parse ──→ Target CPS IR ──→ Rust prototype interpreter ──→ Result or trap
```

PLAN-159 does not replace the existing parser, type checker, workflow interpreter, or legacy
lowering pipeline. During this phase, `.cps` files are the direct test/debug path into Target
CPS IR. Legacy AST lowering and Lean 4 differential testing remain useful context for later
phases, but they are not implemented here.

## Phases

### Phase 1: Core values, terms, and format scaffold (TASK-1590, TASK-1591, TASK-1601)

**Features:** Values `Lam` and `Cont`; terms `LetVal`, `LetPrim`, `LetCont`, `Jump`, `Call`; minimal `.cps` parser/serializer scaffold for these forms.

**Interpreter:** Basic eval for arithmetic, variable binding, continuation invocation.

**S-expression format:**
```lisp
(letval x 42
  (letcont k [v] (jump exit v {})
    (call f [x] k {})))
```

**Formal semantics:** Syntax (§1) and inference rules for core terms (§2).

**Testing:** Unit tests for each value and term form. Round-trip S-expression tests must use the Phase 1 grammar subset.

---

### Phase 2: Conditionals and Data (TASK-1592, TASK-1601)

**Features:** `If`, `Record`, `Tuple`

**Interpreter:** Branching and structured data evaluation.

**S-expression format:**
```lisp
(letprim positive (gt x 0)
  (if positive
    (letval result (record ((ok true) (value x)))
      (jump k result {}))
    (letval result (record ((ok false) (value 0)))
      (jump k result {}))))
```

`Record` and `Tuple` are `Value` constructors. They appear under `LetVal`; branch bodies remain `Term`s.

**Formal semantics:** Inference rules for `If`, records, tuples (§3).

**Testing:** Conditional logic, record/tuple construction, round-trip S-expressions.

---

### Phase 3: Handlers and Effects (TASK-1593, TASK-1594, TASK-1595, TASK-1602)

**Features:** `Raise`, `Handle`, `HandlerChain`

**Interpreter:** Full handler dispatch with shallow vs provider semantics.

**S-expression format:**
```lisp
(handle
  (clause
    (op (cap db.read) ((String) (Int)) String)
    (params (table id))
    (resume resume)
    (body (jump resume "alice" {}))
    (row {}))
  (body
    (raise
      (op (cap db.read) ((String) (Int)) String)
      (args "users" user_id)
      (resume k)
      (row {cap db.read})))
  (cont k)
  (row {}))
```

This syntax mirrors SPEC-098b's `HandlerClause { op, params, resume, body, row }` and
`Handle { clause, body, cont, row }`. `Raise.row` is the operation-local row; `Handle.row`
is the local residual row after the handler transformation.

**Formal semantics:** Inference rules for `Raise` dispatch, `Handle` installation, resume construction, chain capture (§4).

**Testing:** Retry pattern, rollback pattern, nested handlers, provider persistence.

---

### Phase 4: Recursion (TASK-1596, TASK-1601)

**Features:** `LetRec`

**Interpreter:** Recursive functions with backfill.

**S-expression format:**
```lisp
(letrec factorial
  (lam [n] k
    (letprim is_zero (eq n 0)
      (if is_zero
        (jump k 1 {})
        (letprim n_minus_one (sub n 1)
          (letcont k_mul [rec_result]
            (letprim product (mul n rec_result)
              (jump k product {}))
            (call factorial [n_minus_one] k_mul {}))))))
  (call factorial [5] exit {}))
```

**Formal semantics:** Inference rules for `LetRec` backfill (§5).

**Testing:** Recursive factorial, Fibonacci, retry loops.

---

### Phase 5: Advanced Features (TASK-1597, TASK-1598, TASK-1601)

**Features:** `RecordDischarge`, `Trap`, row checker

**Interpreter:** Administrative terms and error handling.

**S-expression format:**
```lisp
(recorddischarge (contract "requires {b != 0}" Dynamic)
  (jump k result {}))

(trap ContractViolation)
```

**Formal semantics:** Inference rules for `RecordDischarge` (administrative no-op), `Trap` (bottom-typed diagnostic abort), and row checker validation (§6).

**Testing:** Contract discharge example, error handling, row checker validation.

---

### Phase 6: S-expression format hardening (TASK-1599, TASK-1600)

**Features:** Complete parser and serializer coverage for `.cps` files.

**Interpreter:** Can load and save every Phase 1 through Phase 5 IR form in S-expression format.

**Testing:** Round-trip tests, property tests for parser/serializer consistency, and negative grammar tests for values used where terms or continuation references are required.

---

## Task Breakdown

| Task | Description | Phase | Status |
|------|-------------|-------|--------|
| [TASK-1590](tasks/TASK-1590-cps-ir-core-data-structures.md) | Define core data structures: Atom, Value, Term, Env, HandlerChain | 1 | 📝 Planned |
| [TASK-1591](tasks/TASK-1591-cps-ir-core-evaluator.md) | Implement eval for LetVal, LetPrim, LetCont, Jump, Call | 1 | 📝 Planned |
| [TASK-1592](tasks/TASK-1592-cps-ir-conditionals-data.md) | Implement If, Record, Tuple evaluation | 2 | 📝 Planned |
| [TASK-1593](tasks/TASK-1593-cps-ir-raise-handle-dispatch.md) | Implement Raise, Handle with handler chain walking | 3 | 📝 Planned |
| [TASK-1594](tasks/TASK-1594-cps-ir-handler-provider-persistence.md) | Implement shallow handler vs provider frame persistence | 3 | 📝 Planned |
| [TASK-1595](tasks/TASK-1595-cps-ir-resume-continuations.md) | Implement resume continuation construction with env + chain capture | 3 | 📝 Planned |
| [TASK-1596](tasks/TASK-1596-cps-ir-letrec-recursion.md) | Implement LetRec with placeholder backfill for recursion | 4 | 📝 Planned |
| [TASK-1597](tasks/TASK-1597-cps-ir-discharge-trap.md) | Implement RecordDischarge (no-op) and Trap (abort) | 5 | 📝 Planned |
| [TASK-1598](tasks/TASK-1598-cps-ir-row-validation-scaffold.md) | Implement row representation and local/total row validation scaffold | 5 | 📝 Planned |
| [TASK-1599](tasks/TASK-1599-cps-ir-sexpr-parser-hardening.md) | Harden S-expression parser for full .cps files | 6 | 📝 Planned |
| [TASK-1600](tasks/TASK-1600-cps-ir-sexpr-serializer-hardening.md) | Harden S-expression serializer for IR | 6 | 📝 Planned |
| [TASK-1601](tasks/TASK-1601-cps-ir-core-operational-semantics.md) | Write formal operational semantics for syntax, core terms, conditionals/data, recursion, and advanced terms (§1-§3, §5-§6) | 1/2/4/5 | 📝 Planned |
| [TASK-1602](tasks/TASK-1602-cps-ir-handler-operational-semantics.md) | Write formal operational semantics for handlers (§4) | 3 | 📝 Planned |
| [TASK-1603](tasks/TASK-1603-phase-159-closeout.md) | Close out Phase 159 with verification, documentation, and changelog | Closeout | 📝 Planned |

TASK-1598 is intentionally a scaffold task. It must implement typed row carriers, namespaced
effect-item identity, local-vs-total validation, duplicate elimination, and fail-closed residual
row diagnostics for the interpreter slice. Full transparent alias/group expansion, public module
summary export, and all kind-specific discharge rules remain separate follow-up work unless the
TASK-1598 file explicitly absorbs them with tests.

TASK-1590 owns the minimal parser/serializer scaffolding needed by Phase 1 examples. TASK-1599
and TASK-1600 harden that scaffold after all term forms exist; they are not prerequisites for
writing early `.cps` fixtures.

TASK-1601 owns the architecture/operational-semantics document advertised by PLAN-INDEX.
It must create a concrete document path rather than leaving the design note as an unowned
placeholder.

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

## Deliverables

| Milestone | Criteria |
|-----------|----------|
| M1 (Phase 1) | `cargo test` passes, simple arithmetic works, minimal `.cps` scaffold round-trips, formal semantics §1-§2 complete |
| M2 (Phase 2) | Branching and records work, formal semantics §3 complete |
| M3 (Phase 3) | Retry/rollback handler patterns work, formal semantics §4 complete |
| M4 (Phase 4) | Recursive functions work, formal semantics §5 complete |
| M5 (Phase 5) | All term forms + row checker scaffold work, formal semantics §6 complete |
| M6 (Phase 6) | `.cps` test suite runs, round-trip tests pass |

## What to Skip (For Now)

- No bytecode serialization (S-expression is the format)
- No JIT compilation
- No legacy AST lowering or `lower_to_cps` implementation
- No differential testing against Lean 4
- No mutual recursion (single-variable `LetRec` only)
- No row polymorphism (concrete rows only)
- No effect aliases
- No transparent alias/group expansion in TASK-1598 unless explicitly promoted by a later review
- No full kind-specific discharge implementation beyond the interpreter/provider frame boundary
- No direct-style fragments

## Resolved and Open Decisions

1. **Environment representation:** start with immutable environment frames using structural sharing. `LetRec` may use a narrow placeholder/backfill cell only for the recursive binding under construction.
2. **Handler chain representation:** start with an explicit frame stack (`Vec` or small wrapper over `Vec`) because handler/provider lookup order must be observable in tests. A linked representation can be introduced later if profiling justifies it.
3. **Answer type discipline:** the interpreter does not need a runtime `Ans` value, but task tests, row/type checks, and formal semantics must model a fixed answer type for each CPS region. The phase must not accept examples that bypass this discipline.
4. **Affine continuations:** TASK-1595 must enforce one-shot resume/continuation use. A runtime consumed-state trap is acceptable for the initial interpreter, but task files must preserve the SPEC-098b target of static affine typing.
5. **Formal semantics format:** write LaTeX-style inference rules in repository Markdown first. Lean 4 mechanization remains future work outside this isolated prototype phase.

## See Also

- [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099b: Ash Operational Semantics — Target State](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [PLAN-INDEX](PLAN-INDEX.md) — Master task index
