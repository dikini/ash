# TASK-751: Proc Do Integration and Tower Behavior

## Status: 📝 Planned

## Description

Validate `do:Proc` end to end and enforce tower behavior: sequential Proc bind, explicit process operations, explicit `proc::from_act` for Act-to-Proc crossing, and operational `fail` remaining operational bottom.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §§10-12
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-049](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-050](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)

## Dependencies

- 📝 TASK-749: typed do elaboration.
- 📝 TASK-750: Act compatibility migration.
- ✅ Phase 99: `proc::from_act` embedding boundary.
- ✅/🟢 Phase 104 closeout for normal sequencing.

## Requirements

1. `do:Proc { return v }` produces `Proc<A>`.
2. `do:Proc { x <- proc::unit(v); return x }` uses sequential Proc bind.
3. `do:Proc` does not import `proc::par`, `proc::await`, `proc::join`, etc.
4. `do:Proc` rejects `Act<A>` RHS in `<-` unless wrapped with `proc::from_act`.
5. `Proc<Act<A>>` is not flattened.
6. `fail` inside `do:Proc` remains operational bottom and is not converted to a domain value.
7. Existing Proc runtime/resource split/join tests keep passing.

## TDD Steps

### Step 1: Add integration tests

**Files:**

- Modify: `crates/ash-typeck/src/check_expr.rs` tests or integration test modules.
- Modify: `crates/ash-interp/src/` tests only if runtime forcing needs coverage.
- Consider end-to-end tests in `crates/ash-engine` if existing Proc examples use engine paths.

Tests:

- successful `do:Proc` with `proc::unit`.
- successful `do:Proc` with `proc::from_act(do:Act { ... })`.
- rejected `do:Proc` binding raw `do:Act`.
- rejected unqualified `par(...)` unless imported or already in scope through normal rules.
- process failure attribution remains process/tower scoped.

### Step 2: Wire Proc dictionary paths

Ensure elaboration uses `proc::unit` / `proc::bind` (or equivalent internal operations) and does not call Act operations.

### Step 3: Verify no implicit lifting

Add a focused assertion that the elaborated or checked representation still distinguishes:

- `Proc<A>`;
- `Act<A>`;
- `Proc<Act<A>>`;
- `Proc<A>` produced by `proc::from_act`.

### Step 4: Verify

Run:

```bash
cargo test --workspace proc_do -- --nocapture
cargo test --workspace proc::from_act -- --nocapture
cargo test --workspace operational_fail -- --nocapture
cargo fmt --check
```

## Verification Steps

- [ ] Proc do positive tests pass.
- [ ] No implicit Act-to-Proc lift exists.
- [ ] No implicit target-specific imports exist.
- [ ] Operational failure remains bottom.
- [ ] Existing Proc/resource split/join tests pass.
- [ ] Independent review confirms Phase 104 runtime authority semantics were not changed.

## Dependencies for Next Task

Required by:

- TASK-752 diagnostics.
- TASK-753 closeout.
