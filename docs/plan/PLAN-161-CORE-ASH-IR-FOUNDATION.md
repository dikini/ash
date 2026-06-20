---
id: plan.ash.core-ash-ir-foundation
title: Core Ash IR Foundation
kind: plan
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-20
verified_against:
  specs:
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
---

# Core Ash IR Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first executable Core Ash layer from SPEC-099: a small Core AST, canonical Core text fixture format, parser/serializer, validator, and minimal Core-to-CPS lowering.

**Architecture:** Core Ash is a separate direct-style IR layer in `ash-core`, not a replacement for the existing CPS IR. Hand-authored `.core` fixtures parse to a raw Core AST, pass through a validator that enforces SPEC-099 invariants, and then lower mechanically into the existing `crate::cps` AST. The parser/serializer exist for fixtures, golden tests, and debugging only; they are not surface Ash syntax.

**Tech Stack:** Rust 2024, `ash-core`, existing `crate::cps` carriers, focused integration tests, property tests with `proptest` where structure generation is useful.

---

## Phase: 161

## Status

In progress: 3/12 tasks complete.

## Background

SPEC-099 defines Core Ash as the canonical direct-style IR between surface language elaboration and SPEC-098b CPS IR. Phase 159 and Phase 160 already provide a CPS IR and interpreter substrate. Phase 161 adds the missing Core layer without implementing surface-to-Core lowering.

The phase intentionally includes a basic Core text format. That format gives developers and agents a fast fixture loop:

```text
Core text fixture -> raw Core AST -> validated Core program -> CPS IR
```

This is distinct from surface Ash parsing. The Core text format is strict, explicit, and close to the AST.

## Scope

### In scope

1. Core Ash AST carriers for the SPEC-099 subset needed by this phase.
2. A canonical `.core` text fixture format.
3. Parser from `.core` text to raw Core AST.
4. Serializer from Core AST to canonical `.core` text.
5. Core validator for syntactic and representation invariants.
6. Minimal Core-to-CPS lowering for values, lets, primitive calls, conditionals, calls, jumps, raises, handles, record discharge, and traps.
7. Golden and round-trip tests.
8. PLAN-INDEX, changelog, and closeout documentation.

### Out of scope

| Item | Reason |
|------|--------|
| Surface Ash to Core lowering | Future frontend/elaboration phase. |
| Ad-hoc polymorphism or typeclass solving | Upper-layer type-system feature, not Core implementation. |
| Arbitrary user-defined algebraic effects | SPEC-096b/SPEC-098b do not admit these yet. |
| `MultiShotPure` semantics | SPEC-099 keeps this as a future hook only. |
| Direct Core `Match` | SPEC-099 currently lowers matching through primitives and `If`. |
| Full type checker and row solver | This phase validates representation shape, not full semantic typing. |
| Replacing the CPS interpreter | Core lowers into the existing CPS substrate. |

## Core Text Format Decision

Use a small S-expression-like fixture syntax with lowercase node names and explicit fields. This keeps parsing deterministic and avoids pretending the format is user-facing surface Ash.

Example shape:

```text
(let-val x : Int (lit-int 1)
  (let-prim y add (x (lit-int 2))
    (jump (label exit) y)))
```

The serializer must produce one canonical spelling so fixture diffs are stable. The parser may accept only the canonical spelling in Phase 161.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1620](tasks/TASK-1620-core-ash-ast-carriers.md) | Add Core Ash AST carriers and module exports | 3 | SPEC-099 | Complete |
| [TASK-1621](tasks/TASK-1621-core-text-format-fixtures.md) | Freeze minimal `.core` text grammar and golden fixtures | 2 | TASK-1620 | Complete |
| [TASK-1622](tasks/TASK-1622-core-text-parser-atoms-values.md) | Parse Core atoms, rows, types, and values | 3 | TASK-1621 | Complete |
| [TASK-1623](tasks/TASK-1623-core-text-parser-expressions.md) | Parse Core expressions and effect/discharge forms | 4 | TASK-1622 | Planned |
| [TASK-1624](tasks/TASK-1624-core-text-serializer.md) | Add canonical Core AST serializer and round-trip tests | 3 | TASK-1623 | Planned |
| [TASK-1625](tasks/TASK-1625-core-validator-basic-invariants.md) | Validate basic SPEC-099 Core invariants | 3 | TASK-1623 | Planned |
| [TASK-1626](tasks/TASK-1626-core-validator-affine-resume.md) | Validate handler resume affine-position restrictions | 3 | TASK-1625 | Planned |
| [TASK-1627](tasks/TASK-1627-core-to-cps-lowering-basic.md) | Lower values, lets, primitives, conditionals, calls, and jumps | 5 | TASK-1625 | Planned |
| [TASK-1628](tasks/TASK-1628-core-to-cps-lowering-effects.md) | Lower raise, handle, discharge, and trap forms | 5 | TASK-1627 | Planned |
| [TASK-1629](tasks/TASK-1629-core-end-to-end-fixtures.md) | Add `.core` -> validate -> CPS golden fixtures | 4 | TASK-1624, TASK-1628 | Planned |
| [TASK-1630](tasks/TASK-1630-core-ash-reference-docs.md) | Document Core text and implementation boundaries | 2 | TASK-1629 | Planned |
| [TASK-1631](tasks/TASK-1631-phase-161-closeout.md) | Close out Phase 161 with verification and review | 3 | All above | Planned |

**Total estimated hours:** 40.

## TDD Policy

Every implementation task must follow a fast red/green loop:

1. Add a focused failing test in `crates/ash-core/tests/task_16xx_*.rs`.
2. Run the exact focused test and record the expected failure.
3. Implement the minimal slice.
4. Run the focused test until it passes.
5. Run the affected crate gate.
6. Commit that task before starting the next task.

Prefer small tests over broad fixture suites until TASK-1629.

## Verification Gates

### Focused per-task gates

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
cargo test -p ash-core --test task_1621_core_text_format
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1625_core_validator_basic
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo test -p ash-core --test task_1628_core_to_cps_effects
cargo test -p ash-core --test task_1629_core_end_to_end
```

### Affected-crate gate

```bash
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

### Baseline note

At worktree creation on 2026-06-20, `cargo test -p ash-core -p ash-interp` passed `ash-core` and most `ash-interp` tests but failed the pre-existing `ash-interp --test builtin_dispatch` case `dispatch_table::stdlib_pub_builtin_declarations_have_honest_dispatch_entries`, which reports missing `test::quickcheck::*` dispatch entries. Phase 161 tasks should use `ash-core` gates unless they deliberately touch interpreter behavior.

## Acceptance Criteria

- [ ] Core AST types are separate from CPS AST types and exported through `ash-core`.
- [ ] `.core` fixtures parse into raw Core AST.
- [ ] Core AST serializes to canonical `.core` text.
- [ ] Parser/serializer round-trip tests are stable.
- [ ] Validator rejects non-ANF or illegal Core shapes before lowering.
- [ ] Validator rejects unsupported effect operation kinds.
- [ ] Handler resume continuation restrictions are represented and checked at the Core boundary.
- [ ] Basic Core terms lower into existing `crate::cps::Term` shapes with SPEC-098b row field conventions.
- [ ] Raise, Handle, RecordDischarge, and Trap lower without reintroducing `ContractViolation` as an effect row item.
- [ ] End-to-end `.core` fixtures produce expected CPS golden output.
- [ ] Reference documentation states that Core text is a fixture/debug format, not surface Ash.
- [ ] PLAN-INDEX and CHANGELOG are reconciled.

## Recommended Execution Order

```text
TASK-1620 -> TASK-1621 -> TASK-1622 -> TASK-1623 -> TASK-1624
                                      |
                                      v
                         TASK-1625 -> TASK-1626
                                      |
                                      v
                         TASK-1627 -> TASK-1628 -> TASK-1629
                                                           |
                                                           v
                                      TASK-1630 -> TASK-1631
```

## References

- [SPEC-099: Ash Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [PLAN-159: CPS IR Interpreter](PLAN-159-CPS-IR-INTERPRETER.md)
- [PLAN-160: CPS IR Runtime Expansion](PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Changelog

- 2026-06-20: Completed TASK-1620 by adding Core Ash AST carriers and module exports.
- 2026-06-20: Completed TASK-1621 by freezing the first `.core` fixture grammar documentation and golden fixture corpus.
- 2026-06-20: Completed TASK-1622 by adding the first Core text parser slice for atoms, rows, types, and values.
- 2026-06-20: Created Phase 161 plan for the Core Ash IR foundation, including `.core` text parser/serializer, Core validation, and Core-to-CPS lowering.
