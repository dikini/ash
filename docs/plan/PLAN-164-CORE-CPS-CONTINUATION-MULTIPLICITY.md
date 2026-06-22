---
id: plan.ash.core-cps-continuation-multiplicity
title: Core CPS Continuation Multiplicity
kind: plan
audience: [human, agent]
authority: design
status: complete
stability: alpha
owner: language
last_verified: 2026-06-22
verified_against:
  specs:
    - docs/spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md
    - docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
---

# Core CPS Continuation Multiplicity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement SPEC-102 continuation multiplicity for Core Ash and CPS IR, preserving affine
behavior by default while allowing explicitly typed pure multi-shot continuations.

**Architecture:** Build the feature in vertical slices over the current Phase 159-163 substrate:
spec/text alignment first, CPS IR/runtime second, Core validation/type checking third, Core/CPS
answer-binding continuation invocation fourth, then lowering and fixtures. The implementation must
keep user-facing surface syntax informational only and must use hand-authored Core/CPS fixtures as
the acceptance boundary.

**Tech Stack:** Rust 2024, `ash-core` Core AST/text/validate/typecheck/lower modules,
`ash-core::cps` IR carriers, `ash-interp` CPS runtime, focused tests in
`crates/ash-core/tests/task_168x_*.rs` and `crates/ash-interp/tests/task_168x_*.rs`, and
`.core` fixtures under `crates/ash-core/tests/fixtures/core`.

---

## Phase: 164

## Status

Complete: 12/12 tasks complete. Verification passed for focused Phase 164 suites, full workspace
tests, and workspace clippy on 2026-06-22.

## Background

The current CPS runtime enforces affine continuation use with `ConsumedFlag`. Core already has a
future hook named `CoreMultiplicity::MultiShotPure`, and the Core text parser already accepts
`multi-shot-pure` in continuation types, but Phase 162 intentionally rejected non-affine handler
resumes. SPEC-102 promotes that hook into an operational feature with one strict rule:
multi-shot continuations are explicit and require a normalized closed empty row.

The design notes that motivated this phase are:

- `docs/design/multi-shot-continuations.md`
- `docs/notes/NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md`

Use those notes as rationale, not as normative syntax. Surface examples in those notes are
informational only.

## Scope Locks

Agents implementing this phase must follow these constraints exactly:

1. Work only in Core Ash, CPS IR, CPS operational semantics, Core validation, Core type checking,
   and Core-to-CPS lowering.
2. Do not add or change user-facing Ash surface syntax.
3. Do not implement surface-to-Core lowering for multi-shot continuations.
4. Do not build a Choice/Search standard library or new effect declaration surface.
5. Do not infer multi-shot behavior from empty rows. Empty row is a legality condition only.
6. Preserve affine behavior as the default for existing Core/CPS programs and old serde input.
7. Use current `.core` spelling: `(cont A Ans Row affine)` and
   `(cont A Ans {} multi-shot-pure)`.
8. Use motivational examples as tests, encoded in Core/CPS syntax introduced or preserved by this
   phase.
9. Do not add a new recursion mechanism. Use existing `LetRec`/tuple-of-lambdas support where
   recursive fixtures need it.
10. Do not change SPEC-101 lazy/memo runtime semantics in this phase.

## Implementation Decisions

### Multiplicity Names

The spec term is "multi-shot-pure". The current Core enum variant is
`CoreMultiplicity::MultiShotPure`. Keep that spelling unless a task explicitly updates all
fixtures, docs, and serializers with backward compatibility. For CPS IR, add a matching
`ContMultiplicity` enum rather than overloading `ThunkMode` or any lazy/memo type.

### Default Compatibility

All existing CPS continuation values should behave as affine. Add serde defaults so historical
fixtures and tests do not need mechanical churn beyond places where explicit construction requires
a new field.

Recommended shape:

```rust
pub enum ContMultiplicity {
    Affine,
    MultiShotPure,
}
```

Use `Affine` as `Default`.

### Runtime Behavior

In `ash-interp/src/cps/mod.rs`, continuation invocation is centralized around `Jump` and handler
resume construction. The first runtime task should add the multiplicity field without changing
semantics; the second should alter the jump path:

- affine continuations keep current `ConsumedFlag` behavior;
- multi-shot-pure continuations skip consumed-flag rejection and do not set the flag;
- invalid non-empty-row multi-shot-pure CPS values are rejected by validation or runtime fail-closed
  checks.

Extend ordinary CPS continuation binding to carry row and multiplicity:
`Term::LetCont { name, param, cont_body, row, multiplicity, body }`. Runtime `LetCont` evaluation
must copy those fields into the created `Value::Cont`; it must not infer multiplicity from row.

Add a non-tail answer-binding CPS term, `Term::LetContCall { name, cont, arg, row, body }`, for
handler bodies that must invoke a continuation, bind the answer, and keep evaluating. `Jump`
remains the terminal continuation transfer. `LetContCall` uses the same affine versus multi-shot
invocation rules as `Jump`, carries the continuation-invocation row in its `row` field, then
resumes the caller-side `body` with `name` bound to the continuation answer.

CPS `HandlerClause` must carry both the dynamic resume row and multiplicity, for example
`resume_row: ResumeRowMetadata` and `resume_multiplicity: ContMultiplicity`. Core lowering cannot
directly construct the dynamic resume `Value::Cont`; `ash-interp` constructs it when a handler
catches an operation. The interpreter must therefore copy or derive the row and copy multiplicity
into the `Value::Cont` it binds for the resume parameter. Without these carriers, checked Core
multi-shot resumes are either silently downgraded to affine or constructed with an untrustworthy
default row at runtime.

`ResumeRowMetadata` must distinguish a known row from a legacy omitted row. Checked lowering emits
only a known row. Old serialized handler clauses that omit the field deserialize to a legacy
compatibility state, not to a real `{}` row, and that state is valid only for affine resumes.
Handler dispatch must resolve the `Raise.resume` continuation target row before constructing the
dynamic resume. For a known row, dispatch compares it with the resolved target row. For the legacy
state, dispatch derives the affine resume row from the resolved target row. If the target row cannot
be resolved, or if a known row differs from the target row, dispatch must trap or otherwise fail
closed. Validation should catch statically resolvable mismatches, but runtime still owns the
dynamic fail-closed check.

CPS validation must also validate continuation bodies, not only declared row fields. A
`MultiShotPure` `Value::Cont` or `Term::LetCont` is legal only when the declared row is closed empty
and the effective row of its continuation body matches that declared empty row. Declaring `row = {}`
on an effectful body must be rejected before the continuation can become reusable.

### Type Checking

`check_handler_resume` currently rejects every non-affine resume. Replace that with:

- accept `Affine`;
- accept `MultiShotPure` only when the row structurally normalizes to closed `{}`;
- reject non-empty/open/ambiguous multi-shot rows;
- return the same `(resume_row, answer_ty)` facts for residual-row checking.

The affine use checker must keep rejecting two jumps to an affine resume and must stop treating
two jumps to a legal multi-shot-pure resume as misuse.

### Lowering

Core-to-CPS lowering must preserve explicit multiplicity. Do not infer multi-shot-pure from an
empty row while lowering.

Handler lowering must write resume row and multiplicity onto CPS `HandlerClause`, not only onto
`Value::Cont`, because the dynamic resume continuation is built later by `ash-interp`. Untyped
fallback lowering may remain affine if no checked type fact exists, but checked lowering must
preserve the explicit row and multi-shot-pure multiplicity through the handler-clause metadata path.

Lower Core answer-binding continuation invocation to CPS `Term::LetContCall`. Do not emulate it
with terminal `Jump`; the motivational examples need to inspect the continuation answer and then
continue within the handler body. Lowering must populate `Term::LetContCall.row` from the checked
continuation row.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1680](tasks/TASK-1680-continuation-multiplicity-spec-plan-packet.md) | Freeze SPEC-102 and Phase 164 planning packet | 2 | Phase 163 | Done |
| [TASK-1681](tasks/TASK-1681-cps-cont-multiplicity-carrier.md) | Add CPS continuation, LetCont, LetContCall, and handler row/multiplicity carriers | 3 | TASK-1680 | Done |
| [TASK-1682](tasks/TASK-1682-cps-multishot-runtime.md) | Implement affine vs multi-shot CPS jump and LetContCall behavior | 4 | TASK-1681 | Done |
| [TASK-1683](tasks/TASK-1683-cps-multishot-validation.md) | Validate CPS multi-shot row legality and malformed unchecked input | 3 | TASK-1681 | Done |
| [TASK-1684](tasks/TASK-1684-core-cont-multiplicity-wellformedness.md) | Type-check Core continuation multiplicity well-formedness | 3 | TASK-1680 | Done |
| [TASK-1685](tasks/TASK-1685-core-handler-multishot-resume-typecheck.md) | Accept legal multi-shot handler resumes and reject illegal ones | 4 | TASK-1684 | Done |
| [TASK-1686](tasks/TASK-1686-core-affine-use-discipline-with-multishot.md) | Add Core LetContCall and preserve affine use discipline with multi-shot | 4 | TASK-1685 | Done |
| [TASK-1687](tasks/TASK-1687-core-to-cps-multiplicity-lowering.md) | Preserve multiplicity and LetContCall through Core-to-CPS lowering | 4 | TASK-1682, TASK-1685, TASK-1686 | Done |
| [TASK-1688](tasks/TASK-1688-core-text-fixtures-for-continuation-multiplicity.md) | Add Core text fixtures and golden coverage for multiplicity and LetContCall | 3 | TASK-1684, TASK-1686, TASK-1687 | Done |
| [TASK-1689](tasks/TASK-1689-motivational-multishot-fixtures.md) | Add Choice/backtracking/nested/discard motivational fixtures | 5 | TASK-1686, TASK-1687 | Done |
| [TASK-1690](tasks/TASK-1690-continuation-multiplicity-reference-docs.md) | Add reference docs and non-normative commentary links | 3 | TASK-1689 | Done |
| [TASK-1691](tasks/TASK-1691-phase-164-closeout.md) | Close out Phase 164 with verification, changelog, and index reconciliation | 2 | TASK-1690 | Done |

## Required Test Families

### CPS Runtime Tests

Add focused tests in `crates/ash-interp/tests/task_1682_cps_multishot_runtime.rs`:

1. affine continuation first jump succeeds and second jump traps;
2. multi-shot-pure continuation can be jumped to twice;
3. multi-shot-pure continuation preserves captured env on each invocation;
4. multi-shot-pure continuation preserves captured handler chain on each invocation;
5. `LetContCall` binds an affine continuation answer and consumes the continuation;
6. `LetContCall` binds multi-shot answers repeatedly without consuming the continuation;
7. handler dispatch builds resume `Value::Cont` with resolved/known row metadata and
   `HandlerClause.resume_multiplicity`;
8. legacy omitted handler rows inherit the resolved affine target row instead of comparing as `{}`;
9. affine defaults are preserved for serde/deserialized old-style continuation and handler values.

### Core Type Tests

Add focused tests in:

- `crates/ash-core/tests/task_1684_core_cont_multiplicity_wellformedness.rs`
- `crates/ash-core/tests/task_1685_core_handler_multishot_resume_typecheck.rs`
- `crates/ash-core/tests/task_1686_core_affine_use_discipline_with_multishot.rs`

Required cases:

1. `(cont Int Unit {} multi-shot-pure)` is well formed.
2. `(cont Int Unit {cap db.read} multi-shot-pure)` is rejected.
3. `(cont Int Unit {tail r} multi-shot-pure)` is rejected.
4. affine handler resume behavior remains unchanged.
5. repeated jumps to multi-shot resume type check.
6. repeated jumps to affine resume remain rejected.
7. discarded multi-shot resume type checks.
8. repeated `LetContCall` uses of a multi-shot resume type check.
9. repeated `LetContCall` uses of an affine resume are rejected.

### Lowering Tests

Add focused tests in `crates/ash-core/tests/task_1687_core_to_cps_multiplicity_lowering.rs`:

1. checked lowering emits CPS `Term::LetCont { row, multiplicity: MultiShotPure, ... }`;
2. checked handler lowering emits `HandlerClause { resume_row: Known(...), resume_multiplicity: MultiShotPure, ... }`;
3. affine lowering remains affine;
4. lowering never infers multi-shot-pure from empty row alone;
5. Core `LetContCall` lowers to CPS `Term::LetContCall` with the checked continuation row;
6. lowered multi-shot fixture invokes a resume twice and runs in `ash-interp` without an affine trap.

### Motivational Fixtures

Add tests in `crates/ash-core/tests/task_1689_motivational_multishot_fixtures.rs` and fixtures in
`crates/ash-core/tests/fixtures/core/`:

- `multishot_choice_all_outcomes.core`
- `multishot_backtracking_find_first.core`
- `multishot_nested_choice.core`
- `multishot_discard_resume.core`
- `invalid_affine_choice_double_resume.core`
- `invalid_multishot_effectful_resume.core`

The fixtures must use Core syntax available after TASK-1686, especially the Core answer-binding
continuation invocation form. Do not copy surface examples from
`docs/design/multi-shot-continuations.md`. Do not move Choice/backtracking/nested examples out of
Core solely because pre-Phase-164 Core lacks non-tail resume invocation; adding that Core form is
part of this phase.

## Verification Gates

Every task must run its focused tests. The closeout task must run:

```bash
cargo fmt --check
cargo test -p ash-core --test task_1681_cps_cont_multiplicity_carrier
cargo test -p ash-interp --test task_1682_cps_multishot_runtime
cargo test -p ash-core --test task_1683_cps_multishot_validation
cargo test -p ash-core --test task_1684_core_cont_multiplicity_wellformedness
cargo test -p ash-core --test task_1685_core_handler_multishot_resume_typecheck
cargo test -p ash-core --test task_1686_core_affine_use_discipline_with_multishot
cargo test -p ash-core --test task_1687_core_to_cps_multiplicity_lowering
cargo test -p ash-core --test task_1688_core_text_continuation_multiplicity
cargo test -p ash-core --test task_1689_motivational_multishot_fixtures
cargo test -p ash-core --test task_1690_continuation_multiplicity_docs_consistency
cargo test -p spec_processor spec_links
cargo test --all
cargo clippy --all-targets --all-features
git diff --check
```

If full workspace gates are too slow during intermediate tasks, record the narrower evidence in the
task file and leave full gates for TASK-1691.

## Agent Handoff Notes

Use `rust-skills` for Rust code work and `task-development-using-tdd` before implementation. If
subagents are available, split by task:

- test agent writes failing focused tests and fixtures;
- code agent implements the minimum feature slice;
- QA agent runs the task gates;
- review agent checks spec drift and compatibility.

Subagents must receive the exact task file path and this plan path. Do not ask subagents to infer
scope from the original design notes.

## History

- 2026-06-22: Created Phase 164 plan and SPEC-102 packet for Core/CPS continuation multiplicity.
