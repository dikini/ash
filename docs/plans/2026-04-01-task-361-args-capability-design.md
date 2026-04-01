# TASK-361 Args Capability Design

**Date:** 2026-04-01
**Task:** [TASK-361](../plan/tasks/TASK-361-args-capability.md)

## Goal

Complete and verify the stdlib `Args` capability surface required for entry-point workflows, keeping the canonical declaration-site syntax and explicit observation form defined by Phase 57 and SPEC-017.

## Constraints

- Keep scope limited to TASK-361.
- Preserve the public import surface: `use runtime::Args`.
- Preserve the usage-site capability type: `args: cap Args`.
- Preserve explicit invocation syntax: `observe Args <index>`.
- Follow TDD: add failing tests before implementation changes.
- Update `CHANGELOG.md` for task completion.
- Do not pull runtime injection, supervisor behavior, or CLI plumbing from TASK-362 and later tasks.

## Chosen Approach

Keep the current canonical `Args` interface:

```ash
pub capability Args: observe(index: Int) returns Option<String>;
```

and finish the task by tightening the regression barrier around it:

1. add focused tests that prove `Args` imports and use-site typing compile/typecheck
2. verify the explicit `observe Args 0` form remains canonical
3. align task documentation/status with the verified surface
4. update the changelog

## Why This Approach

- It matches the latest authoritative sources in SPEC-017 and the Phase 57 runtime-capabilities design.
- It avoids inventing a new `Args` variant when the current one is already canonical.
- It keeps the task small and unblocks downstream runtime work by turning the existing stdlib surface into a verified contract.

## Testing Strategy

1. Add failing focused tests around stdlib parsing/typechecking of `Args` imports, `cap Args`, and `observe Args <index>`.
2. Run targeted tests to observe the red phase.
3. Make the minimal source/doc/test changes needed.
4. Re-run the targeted tests and inspect diagnostics.

## Expected Files

- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-typeck/tests/` (targeted TASK-361 coverage file or existing relevant test file)
- Modify: `docs/plan/tasks/TASK-361-args-capability.md`
- Modify: `CHANGELOG.md`
- Possibly modify: `std/src/runtime/args.ash` only if formatting or exact syntax alignment is needed for tests

## Non-Goals

- No runtime capability injection.
- No supervisor execution semantics.
- No CLI argument plumbing.
- No method-style `Args` access surface.
