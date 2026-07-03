# PLAN-187: Surface Record Expressions

**Status:** Complete
**Depends on:** Phase 186 Surface Function CLI Entry.
**Specs/notes:** `SPEC-095b`, `SPEC-097b`, `SPEC-098c`, `SPEC-100`, and `PLAN-185`.

## Goal

Make structural records usable as ordinary expressions in the function-first target language, so
users can write and project `{ field: expr }` values without falling back to legacy workflow syntax,
nominal constructors, or stdlib helper calls.

## Scope

This phase closes the next surface-function gap after the initial `fn main` entry slices:

- parse bare record expressions such as `{ name: "Ada", age: 41 }` in expression position;
- typecheck and execute record expressions in ordinary `fn` bodies and `do { ... }` sequencing;
- preserve existing nominal constructor syntax such as `User { name: "Ada" }`;
- prove field projection works on structural record values through `ash check`, `ash run --dry-run`,
  and `ash run`;
- update target specs and indexes so “records” are represented as first-class surface expressions,
  not only nominal constructor payloads or stdlib helpers.

## Non-Goals

- No record width subtyping or row-polymorphic record typing.
- No spread/update syntax.
- No changes to nominal ADT constructor syntax.
- No removal of legacy workflow syntax.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1877](tasks/TASK-1877-surface-record-expression-plan-packet.md) | Create the Phase 187 plan packet | Complete |
| [TASK-1878](tasks/TASK-1878-structural-record-expression-execution.md) | Parse, check, and execute structural record expressions | Complete |

## Verification Evidence

- RED: `cargo run -q -p ash-cli -- check` on a function-first source containing `person <- { name: "Ada", age: 41 }; return person.age;` failed with a parse error before implementation.
- GREEN: `cargo test -p ash-engine --test task_1878_surface_record_expressions` passed after structural record expression parsing, checking, lowering, and evaluation were added.
- Parser regression: `cargo test -p ash-parser parse_structural_record_expression` passed.
- CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the structural record fixture, with execution returning `41`.

## Acceptance Criteria

- [x] Phase 187 plan and task packet exist and are indexed.
- [x] Bare structural record expressions parse in ordinary expression position.
- [x] Function-first `fn main` sources can bind a record expression inside `do { ... }` and project a field.
- [x] Nominal constructor syntax remains distinct and covered by existing Phase 185/186 regressions.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the structural record fixture.
- [x] Changelog and target specs record the surface-language change.
