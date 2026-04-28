# PLAN-102: Monad Comprehension Syntax

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 106 is a follow-on to Phase 105 and must preserve SPEC-054 typed-do semantics. Do not implement parser/typechecker work without the corresponding task file.

**Goal:** Implement [SPEC-055](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) by adding bracket comprehension syntax as a source-level, container-view spelling of generalized typed do-notation from [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md).

**Architecture:** Phase 106 is syntax-substrate-first. It introduces a source-fidelity comprehension surface node, parses `[result | qualifiers]: K`, rejects parser-only lowering, then reuses the Phase 105 typed-do elaboration path by normalizing qualifiers to do statements during type checking. The MVP requires explicit targets and Act/Proc dictionary reuse; pure `List`/`Option`/`Result<_, E>` comprehension targets remain deferred until their Monad dictionaries and constructor-hole support exist.

**Tech Stack:** Ash parser/typechecker/lowerer/lint/REPL surfaces, Rust 2024, existing `DoTarget`, `DoStmt`, `Expr::DoBlock`, `resolve_do_target`, and `elaborate_typed_do_block` infrastructure from Phase 105.

---

## Phase 106: Monad Comprehension Syntax

**Status:** ✅ Complete
**Spec:** [SPEC-055](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
**Design:** [DESIGN-032](../design/DESIGN-032-MONAD-COMPREHENSION-SYNTAX.md)
**Depends on:** Phase 105 / [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), especially target-preserving `DoBlock` parsing, MVP Act/Proc dictionaries, typed elaboration, and diagnostics.

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-754](tasks/TASK-754-monad-comprehension-spec-plan-packet.md) | Promote DESIGN-032 into SPEC-055/PLAN-102 and register Phase 106 | Docs/Planning | 3 | ✅ Complete |
| [TASK-755](tasks/TASK-755-comprehension-surface-ast-and-parser.md) | Add comprehension surface AST and parser support for `[result | qualifiers]: K` | Substrate | 7 | ✅ Complete |
| [TASK-756](tasks/TASK-756-comprehension-lowering-boundary-and-cross-crate-visitors.md) | Wire non-typechecking visitors and enforce parser-only lowering rejection | Substrate | 5 | ✅ Complete |
| [TASK-757](tasks/TASK-757-comprehension-typed-elaboration.md) | Reuse typed-do target resolution and elaboration for comprehensions | Semantic | 8 | ✅ Complete |
| [TASK-758](tasks/TASK-758-comprehension-diagnostics.md) | Add comprehension-specific diagnostics and non-fatal teaching diagnostics | Semantic | 5 | ✅ Complete |
| [TASK-759](tasks/TASK-759-monad-comprehension-docs-examples-closeout.md) | Add examples, reconcile docs/status/changelog, and run final verification | Docs/Planning | 4 | ✅ Complete |

Estimated total: 32 hours.

## Tracks

### Track A: Spec and Surface Substrate

- TASK-754 establishes the normative packet and Phase 106 traceability.
- TASK-755 adds parser/surface carriers with span preservation and no semantic lowering.

### Track B: Integration Boundary

- TASK-756 updates visitor-style surfaces that must remain exhaustive when `Expr::Comprehension` exists: lowering, lint, name resolution, capability checks, and REPL rendering. Purity handling should mirror the current `Expr::DoBlock` boundary unless the task explicitly fixes both typed-do and comprehension purity traversal together.

### Track C: Typed Elaboration and Diagnostics

- TASK-757 normalizes comprehension qualifiers into the existing typed-do checker and proves Act/Proc equivalence.
- TASK-758 hardens errors and non-fatal diagnostics with comprehension-specific wording.

### Track D: Closeout

- TASK-759 updates examples/docs/changelog, runs verification, and requests independent review.

## Implementation Constraints

1. Do not fork SPEC-054 semantics. Comprehensions are syntax over typed do.
2. Require explicit target annotations in the MVP unless target inference is implemented with focused tests.
3. Do not lower parser-surface comprehensions directly to untyped `bind` / `return` calls.
4. Do not implicitly lift between Act, Proc, Result, Option, List, or workflow contexts.
5. Do not import target-specific operations; `guard`, `proc::from_act`, and similar names are ordinary scope-resolved calls.
6. Do not implement bare boolean guards, pattern binders, applicative/zip/parallel comprehensions, or collection-builder semantics in Phase 106.
7. Preserve source spans for the whole comprehension, result expression, target annotation, qualifiers, and binders.
8. Keep pure `List`/`Option`/`Result` examples clearly marked as future semantic targets unless dictionaries are implemented in a later phase.

## Verification Strategy

Every implementation task must include:

1. focused parser/typechecker/lowering tests for its changed layer;
2. negative tests for missing target, wrong target kind, wrong qualifier RHS constructor, pure RHS with `<-`, bare boolean qualifiers, and no implicit lifts;
3. equivalence tests comparing comprehension elaboration with an explicit `do:K` block for Act/Proc MVP targets;
4. regression checks proving Phase 105 `do:Act`, `do:Proc`, and new-form `act { ... }` behavior remains unchanged;
5. `cargo fmt --check`;
6. `cargo test -p ash-parser` and `cargo test -p ash-typeck` focused as appropriate;
7. affected-crate `cargo check` / `cargo clippy --all-targets --all-features -- -D warnings`;
8. independent subagent verification before marking a task complete.

## Deferred Follow-on Phase Candidates

A later phase should own:

- user-defined constructor-kinded `Monad<M>`;
- target inference for unannotated comprehensions;
- one-hole constructor targets such as `Result<_, ParseError>`;
- pure `List`, `Option`, and `Result` Monad dictionaries;
- guard/filter syntax through an explicit algebra;
- applicative, zip, or parallel comprehensions with distinct syntax and interface requirements;
- formatter support for bracket comprehensions.
