# TASK-718: Proc library core combinators

## Status: 🟡 Ready

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

- [ ] `Proc` unit/bind/then type-check as SPEC-048 library functions.
- [ ] `from_act` is either implemented behind verified Phase 97 substrate or explicitly deferred in docs/tests.
- [ ] No task in this slice creates child `ProcessId`s or public `P<A>` handles.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs the non-concurrent `Proc` construction/sequencing surface needed by later process-runtime tasks in PLAN-098.

## Notes

- This task fills the SPEC-048 `unit`/`bind`/`then` surface before scheduler/concurrency tasks.
- Preserve `Proc<A>` as distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
