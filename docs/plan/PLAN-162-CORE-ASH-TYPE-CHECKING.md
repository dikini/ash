---
id: plan.ash.core-ash-type-checking
title: Core Ash Type Checking
kind: plan
audience: [human, agent]
authority: design
status: planned
stability: alpha
owner: language
last_verified: 2026-06-20
verified_against:
  specs:
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
---

# Core Ash Type Checking Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the first annotation-led Core Ash type checker described by SPEC-100, producing typed Core facts, row summaries, obligations, and diagnostics before Core-to-CPS lowering.

**Architecture:** Add a new `ash-core::core_ash_typecheck` boundary that consumes representation-validated Core programs from Phase 161. The checker remains annotation-led: it validates explicit Core types and rows, synthesizes only local facts needed for diagnostics and CPS lowering, and records refinement/discharge obligations without running proof search or surface elaboration.

**Tech Stack:** Rust 2024, `ash-core`, Phase 161 `core_ash`/`core_ash_validate`/`core_ash_lower`, focused integration tests in `crates/ash-core/tests/task_164x_*.rs`, property tests with `proptest` only where row normalization benefits from generated inputs.

---

## Phase: 162

## Status

In progress: 5/12 tasks complete.

## Background

Phase 161 implemented Core Ash AST carriers, strict `.core` fixture parsing/serialization, representation validation, and minimal Core-to-CPS lowering. SPEC-100 now defines the missing type-checking contract for Core Ash.

Phase 162 turns SPEC-100 into an implementation-ready checker slice. It does not implement surface-to-Core lowering, full Hindley-Milner inference, proof solving, ad-hoc polymorphism, arbitrary user-defined algebraic effects, or `MultiShotPure` semantics.

The checker must preserve the row-accounting distinctions already established by SPEC-098b and SPEC-099:

- `Jump` has Core local row `{}`; CPS `Jump.row` stores the target continuation row.
- `Raise` local row is the operation row; resume/continuation rows are accounted separately.
- `Handle` residual rows preserve captured resume effects for resumptive handlers.
- `ContractViolation` remains trap metadata, not a row item.

## Scope

### In scope

1. Core type-checker API and typed-program wrappers.
2. Type, row, value, continuation, operation, and discharge environments.
3. Type well-formedness for all Phase 161 `CoreType` variants.
4. Row normalization, exact duplicate removal, structural row inclusion, and conservative row-variable solving.
5. Atom and value typing.
6. Expression typing for lets, primitives, calls, conditionals, jumps, raise, handle, discharge, and trap.
7. Operation signature checks for capability, channel, process, and failure operations.
8. Conservative affine resume checks integrated into the type-checking boundary.
9. Refinement predicate well-formedness scaffolding and obligation recording.
10. Static/evidence/dynamic discharge metadata shape checks.
11. Public row/type summary scaffolding for future export/import work.
12. Documentation and closeout verification.

### Out of scope

| Item | Reason |
|------|--------|
| Surface Ash to Core lowering | Owned by a future elaboration phase. |
| Full HM-style inference | SPEC-100 initial profile is annotation-led. |
| Complete row-polymorphic inference | This phase supports structural row solving only where explicit row variables appear. |
| Proof search, SMT, QuickCheck, or SmallCheck discharge | This phase records obligations and consumes shaped evidence only. |
| Ad-hoc polymorphism/typeclass solving | Upper-layer elaboration feature. |
| Arbitrary user-defined algebraic effects | Not admitted by SPEC-096b/SPEC-098b. |
| `MultiShotPure` semantics | Future hook only; handler resumes remain affine. |
| Session-type or MPST channel checking | Future channel/protocol phase. |

## Implementation Notes

- Prefer a new module `crates/ash-core/src/core_ash_typecheck.rs`.
- Keep the public entrypoint explicit, for example `type_check_core_program(raw_or_valid, env) -> Result<TypedCoreProgram, CoreTypeError>`.
- The checker should accept `ValidCoreProgram` from `core_ash_validate` first. If a convenience API accepts raw Core, it must validate before type checking.
- Keep proof obligations and discharge records as compiler metadata, not ordinary user-visible values.
- Reuse Phase 161 AST carriers. Do not replace the Core AST in this phase.
- Add test modules per task rather than one large test file.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1640](tasks/TASK-1640-core-typecheck-api-and-environments.md) | Add Core type-checker API, environments, typed program wrappers, and diagnostics | 3 | Phase 161 | Complete |
| [TASK-1641](tasks/TASK-1641-core-type-wellformedness.md) | Check Core type well-formedness and nominal/type-app shape | 3 | TASK-1640 | Complete |
| [TASK-1642](tasks/TASK-1642-core-row-normalization-solving.md) | Normalize rows, remove duplicates, compare rows, and solve explicit row variables | 4 | TASK-1640 | Complete |
| [TASK-1643](tasks/TASK-1643-core-atom-value-typing.md) | Type Core atoms and values, including lambdas, records, tuples, and discharge markers | 4 | TASK-1641, TASK-1642 | Complete |
| [TASK-1644](tasks/TASK-1644-core-expression-basics-typecheck.md) | Type Atom, LetVal, LetRec, LetPrim, If, and Trap expressions | 4 | TASK-1643 | Complete |
| [TASK-1645](tasks/TASK-1645-core-call-jump-row-accounting.md) | Type LetCall, Call, and Jump with SPEC-098b row-accounting facts | 4 | TASK-1644 | Planned |
| [TASK-1646](tasks/TASK-1646-core-effect-operation-typing.md) | Type capability/channel/process/failure Raise operations and operation signatures | 4 | TASK-1645 | Planned |
| [TASK-1647](tasks/TASK-1647-core-handle-affine-resume-typecheck.md) | Type Handle clauses with affine resume and captured-resume row preservation | 5 | TASK-1646 | Planned |
| [TASK-1648](tasks/TASK-1648-core-refinement-obligations-discharge.md) | Record refinement obligations and validate discharge metadata shape | 5 | TASK-1647 | Planned |
| [TASK-1649](tasks/TASK-1649-core-public-summary-scaffold.md) | Add public type/row summary scaffolding and private alias diagnostics | 3 | TASK-1648 | Planned |
| [TASK-1650](tasks/TASK-1650-core-typecheck-integration-fixtures.md) | Add `.core` parse -> validate -> type-check -> lower integration fixtures | 4 | TASK-1649 | Planned |
| [TASK-1651](tasks/TASK-1651-core-typecheck-reference-closeout.md) | Document Core type-checker behavior and close out Phase 162 | 3 | TASK-1650 | Planned |

**Total estimated hours:** 46.

## TDD Policy

Every implementation task must follow a fast red/green loop:

1. Add a focused failing test in `crates/ash-core/tests/task_164x_*.rs`.
2. Run the exact focused test and record the expected failure.
3. Implement the minimal code slice.
4. Run the focused test until it passes.
5. Run the affected crate gate.
6. Update the task file, PLAN-INDEX status, and CHANGELOG.
7. Commit before starting the next task.

For row normalization/solving, add property tests only after the first concrete examples are green.

## Verification Gates

### Focused per-task gates

```bash
cargo test -p ash-core --test task_1640_core_typecheck_api
cargo test -p ash-core --test task_1641_core_type_wellformedness
cargo test -p ash-core --test task_1642_core_row_normalization
cargo test -p ash-core --test task_1643_core_atom_value_typing
cargo test -p ash-core --test task_1644_core_expression_basics_typecheck
cargo test -p ash-core --test task_1645_core_call_jump_row_accounting
cargo test -p ash-core --test task_1646_core_effect_operation_typing
cargo test -p ash-core --test task_1647_core_handle_affine_resume
cargo test -p ash-core --test task_1648_core_refinement_discharge
cargo test -p ash-core --test task_1649_core_public_summary
cargo test -p ash-core --test task_1650_core_typecheck_integration
```

### Affected-crate gate

```bash
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

### Documentation gate

```bash
cargo test -p spec_processor spec_links
```

## Acceptance Criteria

- [ ] Core type-checker module is exported from `ash-core`.
- [ ] Type-checker entrypoint consumes validated Core and returns a typed Core program or structured diagnostics.
- [ ] Type, row, value, continuation, operation, and discharge environments are represented.
- [ ] Core type well-formedness covers all Phase 161 `CoreType` variants.
- [ ] Rows normalize by namespace identity, remove exact duplicates, preserve open tails, and solve explicit row variables conservatively.
- [ ] Atom and value typing covers variables, literals, primitive names, constructors, lambdas, records, tuples, and discharge markers.
- [ ] Expression typing covers all Phase 161 Core expression variants.
- [ ] `Jump` keeps Core local row `{}` while preserving the target continuation row for lowering facts.
- [ ] `Raise` checks operation arity/types and records operation-local row only.
- [ ] `Handle` checks operation parameters, affine resume type, handler body row, and residual row including `resume_row`.
- [ ] Refinement checks emit obligations in the base-to-refinement direction and allow refinement-to-base use without new obligations.
- [ ] Static/evidence/dynamic discharge metadata is checked for coherent shape without running proof search.
- [ ] Public summary scaffold preserves normalized row/type facts needed by future export/import work.
- [ ] Integration fixtures prove `.core` parse -> validate -> type-check -> lower works for representative programs.
- [ ] Reference documentation explains the initial algorithmic profile and deferred features.
- [ ] PLAN-INDEX and CHANGELOG are reconciled.

## Recommended Execution Order

```text
TASK-1640 -> TASK-1641 -> TASK-1642 -> TASK-1643 -> TASK-1644
                                                       |
                                                       v
             TASK-1645 -> TASK-1646 -> TASK-1647 -> TASK-1648
                                                       |
                                                       v
                         TASK-1649 -> TASK-1650 -> TASK-1651
```

## References

- [SPEC-100: Ash Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-099: Ash Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [PLAN-161: Core Ash IR Foundation](PLAN-161-CORE-ASH-IR-FOUNDATION.md)

## Changelog

- 2026-06-20: Created Phase 162 plan for implementing SPEC-100 Core Ash type checking with annotation-led TDD tasks.
