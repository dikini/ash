# PLAN-185: Surface Function Language

**Status:** Complete
**Depends on:** Phase 184 Handler / Provider Semantics.
**Specs/notes:** `SPEC-095b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, and `SPEC-100`.

## Goal

Make target Ash feel like one coherent surface language: ordinary `fn` declarations are the user-facing computation unit, computation rows remain requirement sets, `do { ... }` is direct-style sequencing sugar, and runtime entry/profile concepts do not introduce a second source-language semantic path.

## Scope

This phase focuses on the first executable surface-function slice:

- accept `fn main(...) -> {row} T { ... }` as the target entry shape for ordinary engine parsing/checking;
- preserve existing row metadata and direct-style `do` behavior on that path;
- prove ordinary function bodies can combine records, ADTs, pattern matching, calls, and `do { ... }` without requiring privileged workflow syntax;
- reconcile docs and indexes so `workflow` is described as a compatibility/runtime profile rather than the core language path.

## Non-Goals

- No broad removal of legacy `workflow` parsing or runtime entry compatibility.
- No new tower runtime semantics.
- No row inference beyond the existing explicit row bridge.
- No new pattern exhaustiveness or ADT typing model beyond existing parser/typechecker support.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1863](tasks/TASK-1863-surface-function-language-plan-packet.md) | Create the Phase 185 plan packet | Complete |
| [TASK-1864](tasks/TASK-1864-surface-function-boundary-audit.md) | Audit current `fn`/row/`do`/workflow boundaries | Complete |
| [TASK-1865](tasks/TASK-1865-fn-main-entry-adapter.md) | Accept `fn main` as target entry syntax | Complete |
| [TASK-1866](tasks/TASK-1866-function-body-language-fixture.md) | Add cohesive ordinary function body conformance fixture | Complete |
| [TASK-1867](tasks/TASK-1867-surface-function-spec-reconciliation.md) | Reconcile target specs and indexes | Complete |
| [TASK-1868](tasks/TASK-1868-surface-function-closeout.md) | Close out Phase 185 | Complete |
| [TASK-1869](tasks/TASK-1869-surface-function-do-return-and-execution.md) | Accept semicolon `do` return and execute `fn main` sources | Complete |

## Verification Evidence

- RED: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` failed before implementation because function-only source returned `Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` passed after the `fn main` adapter, local type-definition registration, and nominal record field-access changes.
- Continuation RED: the same focused target failed for `do { return ...; }` and inline-row `fn main` execution before TASK-1869.
- Continuation GREEN: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` passed with 4/4 tests after TASK-1869.

## Acceptance Criteria

- A source file with top-level `fn main` and no `workflow` parses through the ordinary engine path.
- `fn main` preserves explicit inline and `where row` requirement metadata through callable summaries and Core callable types.
- `fn main` bodies can use target `do { ... }` without choosing `Act`, `Proc`, or `Workflow`.
- A canonical fixture covers ordinary expressions, records, ADTs, pattern matching, and calls inside `fn` bodies.
- Target docs/indexes route surface-language work through `fn`, rows, direct-style `do`, and runtime profiles/library concepts, with no new legacy authority or tower-as-core wording.
- Changelog and task evidence record verification commands.
