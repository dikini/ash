---
id: plan.ash.cps-ir-runtime-expansion
title: CPS IR Runtime Expansion — Records, Tuples, and Speculative Upper-Language Testing
kind: plan
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-19
verified_against:
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
---

# PLAN-160: CPS IR Runtime Expansion

> **For Hermes:** Use ash-phase-implementation skill to execute this plan task-by-task. Load rust-skills for all Rust code work. Do not modify pre-existing specs, plans, or task files from Phase 159 — only create new artifacts.

## Phase: 160

## Status: 📝 Planned

## Goal

Extend the Phase 159 CPS IR interpreter with the minimal runtime features needed to execute correctly-lowered programs that use structured data (records, tuples, constructor tags), pattern matching, and mutual recursion desugaring. The resulting interpreter becomes an **objective testing ground** for speculative upper-language features — frontend lowering can be tested by writing `.cps` fixtures that represent the lowered form and verifying execution produces the expected result.

## Background

Phase 159 built an isolated CPS IR interpreter with core values, terms, handlers, recursion, and S-expression round-tripping. However, the interpreter cannot execute programs that:
- Construct or destructure records and tuples
- Use declared product types with named constructors
- Pattern-match on sum types
- Express mutual recursion (frontend must desugar to single `LetRec` with tuple dispatch)

This phase closes those gaps at the **runtime level only**. Type checking, row solving, alias expansion, and contract proving remain frontend responsibilities. The interpreter assumes well-formed, correctly-lowered input.

## Scope

### In scope

1. **Record and tuple values** (`Value::Record`, `Value::Tuple`) in the CPS IR data model
2. **Field access primitives** (`PrimOp::RecordGet`, `PrimOp::TupleGet`) for destructuring
3. **Constructor tags** (`Atom::ConstructorName`) for sum type discrimination
4. **Pattern-match term** (`Term::Match`) or `PrimOp::MatchTag` for multi-way dispatch
5. **Mutual recursion desugaring support** — extend `LetRec` to accept tuple-of-lambdas patterns
6. **S-expression parser/serializer** updates for all new forms
7. **Speculative test fixtures** demonstrating upper-language lowering patterns:
   - Mutual recursion (even/odd via tuple dispatch)
   - Record construction and field access
   - Sum type construction and pattern matching
   - Trait dictionary passing (monomorphized)
8. **Operational semantics updates** (new document, not modifying SPEC-099b)
9. **Reference documentation** for the expanded CPS IR

### Out of scope

| Item | Reason |
|------|--------|
| Type checking | Frontend responsibility — interpreter assumes well-typed input |
| Row polymorphism solving | Frontend solves all row variables before lowering |
| Effect alias expansion | Frontend expands aliases before lowering |
| Contract discharge predicates | Frontend desugars dynamic contracts to `If` + `Trap` |
| Provider frame execution | Requires runtime infrastructure (provider registry, capability bindings) beyond single-file execution |
| Bytecode serialization | Future layer, not needed for `.cps` fixture testing |
| JIT compilation | Future layer |
| Direct-style fragments | CPS IR is the target; no direct-style convenience nodes |

## Architecture

```text
Frontend (parser, type checker, lowering)
    |
    v
Lowered CPS IR (.cps files with concrete rows, expanded aliases, no type vars)
    |
    v
Phase 160 Interpreter (this phase)
    - Record/Tuple values
    - Field access primitives
    - Constructor tags
    - Match dispatch
    - Mutual recursion via tuple LetRec
    |
    v
Result or trap
```

The interpreter is **not** a type checker. It is an execution engine for validated IR.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1600](tasks/TASK-1600-cps-ir-record-tuple-values.md) | Add Record and Tuple value variants to CPS IR | 4 | TASK-1590 | 📝 Planned |
| [TASK-1601](tasks/TASK-1601-cps-ir-field-access-primitives.md) | Add RecordGet and TupleGet primitive operations | 3 | TASK-1600 | 📝 Planned |
| [TASK-1602](tasks/TASK-1602-cps-ir-constructor-tags.md) | Add ConstructorName atom variant for sum types | 2 | TASK-1590 | 📝 Planned |
| [TASK-1603](tasks/TASK-1603-cps-ir-match-dispatch.md) | Add Match term or MatchTag primitive for pattern dispatch | 4 | TASK-1602 | 📝 Planned |
| [TASK-1604](tasks/TASK-1604-cps-ir-mutual-recursion-desugaring.md) | Support mutual recursion via tuple-of-lambdas in LetRec | 5 | TASK-1596, TASK-1600 | 📝 Planned |
| [TASK-1605](tasks/TASK-1605-cps-ir-sexpr-parser-new-forms.md) | Update S-expression parser/serializer for new forms | 4 | TASK-1600, TASK-1601, TASK-1602, TASK-1603 | 📝 Planned |
| [TASK-1606](tasks/TASK-1606-cps-ir-speculative-fixtures.md) | Write speculative test fixtures for upper-language patterns | 6 | TASK-1604, TASK-1605 | 📝 Planned |
| [TASK-1607](tasks/TASK-1607-cps-ir-expanded-operational-semantics.md) | Write operational semantics for new term forms (new doc) | 4 | TASK-1600, TASK-1601, TASK-1602, TASK-1603 | 📝 Planned |
| [TASK-1608](tasks/TASK-1608-cps-ir-reference-docs-update.md) | Add reference documentation for expanded CPS IR | 3 | TASK-1607 | 📝 Planned |
| [TASK-1609](tasks/TASK-1609-phase-160-closeout.md) | Close out Phase 160 with verification and documentation | 3 | All above | 📝 Planned |

**Total estimated hours:** 38

## Key Risks and Decisions

1. **Record/Tuple representation**: Should `Record` store `Vec<(Name, Atom)>` or `Vec<(Name, Value)>`? Decision: `Atom` for fields — records are inert data, no nested computation. (SPEC-098b §2.2 already specifies `Atom` for fields.)
2. **Match dispatch**: Dedicated `Term::Match` or primitive `PrimOp::MatchTag`? Decision: `PrimOp::MatchTag` is simpler and keeps the term grammar small. A dedicated `Term::Match` can be added later if pattern matching becomes complex enough to warrant it.
3. **Mutual recursion**: Extend `LetRec` AST or add `LetRecMutual`? Decision: Keep single `LetRec` but document the tuple-of-lambdas desugaring pattern. The frontend is responsible for producing the desugared form.
4. **S-expression syntax**: New syntax must not conflict with existing forms. Proposed:
   - `(record ((name value) ...))` for records
   - `(tuple (value ...))` for tuples
   - `(record_get name record)` for field access
   - `(tuple_get index tuple)` for element access
   - `(constructor "Name")` for constructor tags
   - `(match_tag scrutinee ("Tag1" branch1) ("Tag2" branch2) ...)` for dispatch

## Verification Gates

### Per-task gates

```bash
# After each implementation task
cargo test -p ash-core -p ash-interp
cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
cargo fmt --check
```

### Phase closeout gate

```bash
# Full verification
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps

# Speculative fixture execution
# (run all .cps fixtures in test corpus)
```

## Acceptance Criteria

- [ ] Record and tuple values can be constructed, bound, and passed as arguments
- [ ] Field access primitives correctly extract values
- [ ] Constructor tags enable sum type discrimination
- [ ] Match dispatch works for at least 2-way and 3-way cases
- [ ] Mutual recursion (even/odd) executes correctly via tuple-of-lambdas desugaring
- [ ] All new forms round-trip through S-expression parse → serialize → parse
- [ ] Speculative fixtures demonstrate: mutual recursion, records, sum types, trait dictionaries
- [ ] Operational semantics document covers all new forms
- [ ] Reference documentation updated
- [ ] CHANGELOG.md updated
- [ ] PLAN-INDEX.md updated

## Recommended Execution Order

```
TASK-1600 (Record/Tuple values)
    |
    v
TASK-1601 (Field access primitives)
    |
    v
TASK-1602 (Constructor tags)
    |
    v
TASK-1603 (Match dispatch)
    |
    v
TASK-1604 (Mutual recursion desugaring)
    |
    v
TASK-1605 (S-expression parser updates)
    |
    v
TASK-1606 (Speculative fixtures) ──► TASK-1607 (Operational semantics)
    |                                    |
    |                                    v
    |                               TASK-1608 (Reference docs)
    |                                    |
    └────────────────────────────────────┘
                                         |
                                         v
                                    TASK-1609 (Closeout)
```

## References

- [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md) — CPS IR syntax and types
- [SPEC-099b: Ash Operational Semantics — Target State](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — Current operational semantics (§1-§7)
- [PLAN-159: CPS IR Interpreter](PLAN-159-CPS-IR-INTERPRETER.md) — Previous phase this builds on
- [TASK-1596](tasks/TASK-1596-cps-ir-letrec-recursion.md) — Single-binding LetRec (prerequisite for TASK-1604)
- [TASK-1599](tasks/TASK-1599-cps-ir-sexpr-parser-hardening.md) — S-expression parser scaffold (prerequisite for TASK-1605)

## Changelog

- 2026-06-19: Created as follow-on to Phase 159. Defines runtime expansion for structured data and speculative upper-language testing.
