---
id: plan.ash.cps-ir-runtime-expansion
title: CPS IR Runtime Expansion — Speculative IR Shapes for Upper-Language Lowering
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

Extend the Phase 159 CPS IR interpreter with speculative IR shapes that demonstrate how upper-language features (structured data, pattern matching, mutual recursion) can be represented and executed in the target CPS IR. This phase is **pure IR work**: it adds new IR constructs, implements their evaluation semantics, and provides executable fixtures that serve as proof-of-concept lowering targets for future frontend work.

The resulting artifacts provide:
1. **Working IR constructs** for records, tuples, constructor tags, pattern dispatch, and mutual recursion
2. **Executable test fixtures** demonstrating how upper-language patterns lower to CPS IR
3. **A basis for future design** — frontend teams can use these fixtures as concrete targets when designing lowering passes

## Background

Phase 159 built an isolated CPS IR interpreter with core values, terms, handlers, recursion, and serde-based serialization (via `serde_lexpr`). The interpreter can execute arithmetic, branching, function calls, effect handlers, and single-binding recursion. However, it lacks constructs for:
- Structured data (records, tuples)
- Sum type discrimination (constructor tags + pattern matching)
- Mutual recursion (frontend must desugar to single `LetRec` with tuple dispatch)

This phase adds these constructs to the IR and interpreter, creating executable proof-of-concepts that future frontend lowering passes can target.

## Scope

### In scope (pure IR work)

1. **IR data model extensions**: `Value::Record`, `Value::Tuple`, `Atom::ConstructorName`
2. **Primitive operations**: `PrimOp::RecordGet`, `PrimOp::TupleGet` for field access
3. **Pattern dispatch**: `Term::Match` for multi-way branch on constructor tags
4. **Mutual recursion support**: `LetRec` extended to accept tuple-of-lambdas (documented desugaring pattern)
5. **Serde serialization**: New variants derive `Serialize`/`Deserialize` for `.cps` file round-tripping via `serde_lexpr`
6. **Speculative test fixtures**: Executable `.cps` fixtures demonstrating lowering patterns:
   - Mutual recursion (even/odd via tuple dispatch)
   - Record construction and field access
   - Sum type construction and pattern matching
   - Trait dictionary passing (monomorphized)
7. **Operational semantics**: New document covering all new IR forms
8. **Reference documentation**: For the expanded CPS IR constructs

### Out of scope (not pure IR work)

| Item | Reason |
|------|--------|
| Type checking | Frontend responsibility — interpreter assumes well-formed input |
| Lowering from surface syntax | Future frontend phase — this phase only provides the IR target |
| Migration from old IR | Future concern — this phase establishes new shapes only |
| JIT compilation | Future layer — not needed for fixture execution |
| Bytecode serialization | Future layer — `.cps` files use serde-lexpr |
| Row polymorphism solving | Frontend solves rows before producing IR |
| Effect alias expansion | Frontend expands aliases before producing IR |
| Contract discharge proving | Frontend desugars contracts to `If` + `Trap` |
| Provider frame execution | Requires runtime infrastructure beyond single-file execution |
| Direct-style IR fragments | CPS IR is the target; no convenience nodes |

### Phase 159 boundaries still in force

The following Phase 159 decisions remain unchanged for Phase 160:

| Boundary | Rationale |
|----------|-----------|
| **No bytecode serialization** | `.cps` files remain the fixture format; serde-lexpr handles serialization |
| **No JIT compilation** | Interpreter is the sole execution path |
| **No legacy AST lowering** | No `lower_to_cps` implementation; fixtures are hand-authored or generated |
| **No differential testing against Lean 4** | Lean 4 mechanization remains future work |
| **No mutual recursion beyond tuple-of-lambdas** | Single-binding `LetRec` only; frontend desugars mutual recursion |
| **No row polymorphism** | Concrete rows only; no row variables in IR |
| **No effect aliases** | Aliases are expanded before IR generation |
| **No transparent alias/group expansion** | Concrete effect items only in rows |
| **No full kind-specific discharge** | Interpreter/provider frame boundary only |
| **No direct-style fragments** | CPS IR is the exclusive target representation |

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│  Future Frontend (out of scope for this phase)              │
│  - Parser → Type Checker → Lowering Pass                    │
│  - Produces: CPS IR with concrete rows, no type vars        │
└─────────────────────────────────────────────────────────────┘
                              │
                              v
┌─────────────────────────────────────────────────────────────┐
│  Phase 160: Speculative IR Shapes (this phase)              │
│  - Record/Tuple values in IR data model                     │
│  - Field access primitives (RecordGet, TupleGet)            │
│  - Constructor tags (Atom::ConstructorName)                 │
│  - Pattern dispatch (Match)                                 │
│  - Mutual recursion via tuple-of-lambdas in LetRec          │
│  - Serde-lexpr serialization for .cps fixtures              │
│  - Executable proof-of-concept fixtures                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              v
┌─────────────────────────────────────────────────────────────┐
│  Phase 160 Interpreter (extended from Phase 159)            │
│  - Evaluates new IR constructs                                │
│  - Produces: Result or trap                                 │
└─────────────────────────────────────────────────────────────┘
```

The interpreter is an execution engine for validated IR. It does not type-check, lower, or optimize.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1610](tasks/TASK-1610-cps-ir-record-tuple-values.md) | Add Record and Tuple value variants to CPS IR | 4 | TASK-1590 | 📝 Planned |
| [TASK-1611](tasks/TASK-1611-cps-ir-field-access-primitives.md) | Add RecordGet and TupleGet primitive operations | 3 | TASK-1610 | 📝 Planned |
| [TASK-1612](tasks/TASK-1612-cps-ir-constructor-tags.md) | Add ConstructorName atom variant for sum types | 2 | TASK-1590 | 📝 Planned |
| [TASK-1613](tasks/TASK-1613-cps-ir-match-dispatch.md) | Add Match term for pattern dispatch | 4 | TASK-1612 | 📝 Planned |
| [TASK-1614](tasks/TASK-1614-cps-ir-mutual-recursion-desugaring.md) | Support mutual recursion via tuple-of-lambdas in LetRec | 5 | TASK-1596, TASK-1610 | 📝 Planned |
| [TASK-1615](tasks/TASK-1615-cps-ir-serde-extension.md) | Extend serde-based serialization for new IR forms | 3 | TASK-1610, TASK-1611, TASK-1612, TASK-1613 | 📝 Planned |
| [TASK-1616](tasks/TASK-1616-cps-ir-speculative-fixtures.md) | Write speculative test fixtures for upper-language patterns | 6 | TASK-1614, TASK-1615 | 📝 Planned |
| [TASK-1617](tasks/TASK-1617-cps-ir-expanded-operational-semantics.md) | Write operational semantics for new term forms (new doc) | 4 | TASK-1610, TASK-1611, TASK-1612, TASK-1613 | 📝 Planned |
| [TASK-1618](tasks/TASK-1618-cps-ir-reference-docs-update.md) | Add reference documentation for expanded CPS IR | 3 | TASK-1617 | 📝 Planned |
| [TASK-1619](tasks/TASK-1619-phase-160-closeout.md) | Close out Phase 160 with verification and documentation | 3 | All above | 📝 Planned |

**Total estimated hours:** 37

## Key Risks and Decisions

1. **Record/Tuple representation**: The spec IR grammar (SPEC-098b) uses `Atom` for fields/elements. The interpreter's runtime `Value` type uses `Value` for fields/elements so that records/tuples can contain lambdas and other structured values. This is the same pattern as `LetVal`: the frontend writes `Atom::Var`, the interpreter resolves to `Value`.
2. **Match dispatch**: `Term::Match` is the chosen design. It keeps pattern matching explicit in the term grammar and allows for future extension (guards, nested patterns). `PrimOp::MatchTag` was considered but rejected because match dispatch is control flow, not a pure primitive operation.
3. **Mutual recursion**: Extend `LetRec` AST or add `LetRecMutual`? Decision: Keep single `LetRec` but document the tuple-of-lambdas desugaring pattern. The frontend is responsible for producing the desugared form.
4. **Serde serialization**: New IR variants must derive `Serialize`/`Deserialize` and round-trip through `serde_lexpr` (the established format from Phase 159). No hand-written parser/serializer is needed — serde derives handle the S-expression format automatically. The only concern is ensuring new enum variants are compatible with the serde-lexpr representation.

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

- [ ] `Value::Record` and `Value::Tuple` can be constructed, bound, and passed as arguments
- [ ] `PrimOp::RecordGet` and `PrimOp::TupleGet` correctly extract field/element values
- [ ] `Atom::ConstructorName` enables sum type discrimination in the IR
- [ ] Pattern dispatch (Match) works for 2-way and 3-way cases
- [ ] Mutual recursion (even/odd) executes correctly via tuple-of-lambdas in `LetRec`
- [ ] All new IR forms round-trip through serde-lexpr serialization (`.cps` files)
- [ ] Speculative fixtures demonstrate executable lowering patterns for: mutual recursion, records, sum types, trait dictionaries
- [ ] Operational semantics document covers evaluation rules for all new forms
- [ ] Reference documentation describes new IR constructs and their intended use in lowering
- [ ] CHANGELOG.md updated
- [ ] PLAN-INDEX.md updated

## Recommended Execution Order

```
TASK-1610 (Record/Tuple values)
    |
    v
TASK-1611 (Field access primitives)
    |
    v
TASK-1612 (Constructor tags)
    |
    v
TASK-1613 (Match dispatch)
    |
    v
TASK-1614 (Mutual recursion desugaring)
    |
    v
TASK-1615 (Serde serialization extension)
    |
    v
TASK-1616 (Speculative fixtures) ──► TASK-1617 (Operational semantics)
    |                                    |
    |                                    v
    |                               TASK-1618 (Reference docs)
    |                                    |
    └────────────────────────────────────┘
                                         |
                                         v
                                    TASK-1619 (Closeout)
```

## References

- [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md) — CPS IR syntax and types
- [SPEC-099b: Ash Operational Semantics — Target State](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — Current operational semantics (§1-§7)
- [PLAN-159: CPS IR Interpreter](PLAN-159-CPS-IR-INTERPRETER.md) — Previous phase this builds on
- [TASK-1596](tasks/TASK-1596-cps-ir-letrec-recursion.md) — Single-binding LetRec (prerequisite for TASK-1614)
- [TASK-1599](tasks/TASK-1599-cps-ir-sexpr-parser-hardening.md) — S-expression parser scaffold (prerequisite for TASK-1615)
- [TASK-1610](tasks/TASK-1610-cps-ir-record-tuple-values.md) — Record/Tuple values (prerequisite for TASK-1615)
- [TASK-1615](tasks/TASK-1615-cps-ir-serde-extension.md) — Serde serialization extension for new forms

## Changelog

- 2026-06-19: Created as follow-on to Phase 159. Defines speculative IR shapes for structured data, pattern matching, and mutual recursion as proof-of-concepts for future frontend lowering.
- 2026-06-19: Updated to reflect serde-based serialization (via `serde_lexpr`) as the established format. S-expression parser/serializer work is deferred; new forms will use serde derives instead.
- 2026-06-19: Clarified scope: pure IR work only. Type checking, lowering, migration, and JIT are explicitly out of scope. This phase provides executable IR targets, not a frontend pipeline.
