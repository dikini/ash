# TASK-718: Proc library core combinators

## Status: ✅ Complete

## Description

Implement the basic `Proc<A>` construction and sequencing surface required by SPEC-048 before enabling process scheduling/concurrency operations.

## Specification Reference

- SPEC-048
- SPEC-049

## Dependencies

- 📝 TASK-707: prerequisite task

## Requirements

### Functional Requirements

1. Add or register the proc library surface for `unit : A -> Proc<A>`, `bind : Proc<A> -> (A -> Proc<B>) -> Proc<B>`, and `then : Proc<A> -> Proc<B> -> Proc<B>`.
2. Type-check the core combinators against `Proc` constructor types without enabling `par`, `await`, `join`, `gather`, or process scheduling behavior.
3. Decide and document whether `from_act` is included in this task or deferred until the required Phase 97 Act runtime substrate is complete.
4. Preserve the distinction between trivial `Proc` construction/sequencing and child process creation.

### Property Requirements (proptest)

```rust
// Add property-based tests for Proc unit/bind/then identity and sequencing
// invariants where the implementation exposes enough structure to test them.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests for well-typed `Proc` unit/bind/then declarations/usages and malformed type-arity diagnostics.

### Step 2: Implement (Green)

Implement the minimal proc-library registration/typechecking/runtime stubs needed for non-concurrent Proc construction and sequencing.

### Step 3: Integration (Green)

Wire the combinators through parser/typechecker/runtime/library paths without adding process child admission or handle observation.

### Step 4: Property Tests (Verify)

Add or extend tests for sequencing identity/associativity where representable without overcommitting the runtime model.

## Verification Steps

- [x] `Proc` unit/bind/then type-check as SPEC-048 library functions.
- [x] `from_act` is either implemented behind verified Phase 97 substrate or explicitly deferred in docs/tests.
- [x] No task in this slice creates child `ProcessId`s or public `P<A>` handles.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

Verification evidence recorded during completion:

- Red: `cargo test -p ash-interp --test task_718_proc_runtime -- --nocapture` failed before runtime registration with `UnknownFunction("unit")` for qualified `proc::unit` calls.
- Red: `cargo test -p ash-engine --test task_718_proc_stdlib -- --nocapture` failed before adding `std/src/proc.ash` with `module 'proc' not found`.
- Green targeted runtime: `cargo test -p ash-interp --test task_718_proc_runtime -- --nocapture` passed with 3 tests.
- Green targeted stdlib/import: `cargo test -p ash-engine --test task_718_proc_stdlib -- --nocapture` passed with 2 tests.
- Green targeted type signatures/proptest: `cargo test -p ash-typeck --test task_718_proc_combinators -- --nocapture` passed with 4 tests.

Completion notes:

- Added `std/src/proc.ash` with `pub builtin fn unit`, `bind`, and `then` over `Proc<A>` and exposed the namespace from `std/src/lib.ash` as `pub mod proc` without re-exporting the names unqualified, preserving the existing top-level `act::{unit, bind, then}` surface.
- Added interpreter runtime stubs for qualified `proc::unit`, `proc::bind`, and `proc::then`. They are closure-shaped, non-concurrent `Proc` values with an opaque `__proc_env` parameter and do not allocate `ProcessId`, return `P<A>`, schedule children, or observe handles.
- `from_act` is explicitly deferred from TASK-718. This slice does not add the Act-to-Proc embedding surface; later Phase 98 work should verify the exact Phase 97 Act force/hidden-carrier paths before introducing it.

## Dependencies for Next Task

This task outputs the non-concurrent `Proc` construction/sequencing surface needed by later process-runtime tasks in PLAN-098.

## Notes

- This task fills the SPEC-048 `unit`/`bind`/`then` surface before scheduler/concurrency tasks.
- Preserve `Proc<A>` as distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
