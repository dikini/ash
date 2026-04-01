# TASK-359 Stdlib Initialization Design

**Date:** 2026-04-01

## Goal

Extend the existing Ash standard library rooted at `std/src/` with the minimum entry-point foundation required by Phase 57B: a `runtime` root module, a `RuntimeError` type, an `Args` capability declaration, and a supervisor module scaffold that later runtime/bootstrap tasks can import.

## Validated Spec Inputs

This design is constrained by the completed Phase 57A spec work:

- [docs/spec/SPEC-009-MODULES.md](../spec/SPEC-009-MODULES.md) defines standard-library root modules as compiler-provided top-level namespaces rooted at `std/src/`.
- [docs/spec/SPEC-012-IMPORTS.md](../spec/SPEC-012-IMPORTS.md) requires `::` import syntax, so entry-point code must import `runtime::...` directly.
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md) distinguishes declaration-site `capability` syntax from usage-site `cap X` typing.
- [docs/spec/SPEC-022-WORKFLOW-TYPING.md](../spec/SPEC-022-WORKFLOW-TYPING.md) fixes the canonical entry workflow signature to `workflow main(...cap params...) -> Result<(), RuntimeError>`.

## Problem Statement

`TASK-359` is the stdlib foundation task for Phase 57B. Downstream tasks need a canonical standard-library place to import:

- `runtime::RuntimeError`
- `runtime::Args`
- `runtime::supervisor` surface

The repository already has a working stdlib layout with `option`, `result`, `prelude`, and `lib`. The task should extend that layout without prematurely wiring runtime bootstrap, CLI behavior, or Engine internals that belong to later blocked tasks.

## Chosen Scope

`TASK-359` will do exactly four things:

1. Create `std/src/runtime/mod.ash` as the root `runtime` module.
2. Create `std/src/runtime/error.ash` with the canonical `RuntimeError` type.
3. Create `std/src/runtime/args.ash` with the canonical `Args` capability declaration.
4. Create `std/src/runtime/supervisor.ash` with a stdlib-visible supervisor scaffold aligned to the downstream runtime/bootstrap public contract.
5. Update `std/src/lib.ash` so the new runtime surface is exported from the stdlib root.
6. Add stdlib-focused tests proving the new module tree and declarations exist and remain discoverable.

## Explicit Non-Goals

This task will **not**:

- inject runtime capabilities from Rust;
- load stdlib modules through the Engine API;
- verify the `main` workflow signature dynamically;
- implement OS exit-code propagation;
- define final supervisor execution semantics beyond the stdlib surface contract;
- change CLI behavior.

Those concerns remain in `TASK-360` through `TASK-367`.

## Module Design

### `std/src/runtime/mod.ash`

Acts as the root `runtime` module and re-exports:

- `RuntimeError`
- `Args`
- supervisor-facing items from `runtime/supervisor.ash`

This preserves the standard-library root namespace rule from `SPEC-009`: user code imports `runtime::RuntimeError` and `runtime::Args`, not `std::runtime::...`.

### `std/src/runtime/error.ash`

Defines the entry-point error carrier as a public record-style ADT:

```ash
pub type RuntimeError = RuntimeError {
    exit_code: Int,
    message: String
};
```

This matches the current task descriptions for downstream `TASK-360` and `TASK-364` and gives the type checker a concrete stdlib symbol to resolve.

### `std/src/runtime/args.ash`

Defines the runtime-provided capability interface using declaration-site capability syntax and standard-library imports:

```ash
use option::Option;

pub capability Args: observe(index: Int) returns Option<String>;
```

This is intentionally declaration-only. Runtime injection remains a later Rust task.

### `std/src/runtime/supervisor.ash`

Defines a stdlib-visible supervisor scaffold that downstream runtime/bootstrap tasks can target. For `TASK-359`, the supervisor module only needs to exist, type-check as standard-library source, and expose a canonical symbol rather than full execution behavior.

To reduce later public API churn, the preferred shape is the downstream-facing public contract from `TASK-362`/`TASK-363c`: `system_supervisor(args: cap Args) -> Int`. `TASK-359` keeps only a placeholder `0` body; the real spawn/observation semantics still belong to the blocked runtime tasks.

## Testing Strategy

This task follows TDD with stdlib-focused tests first.

### Red tests to add

1. Existence tests for:
   - `std/src/runtime/mod.ash`
   - `std/src/runtime/error.ash`
   - `std/src/runtime/args.ash`
   - `std/src/runtime/supervisor.ash`
2. Surface-content tests proving:
   - `runtime/error.ash` declares `pub type RuntimeError`
   - `runtime/args.ash` declares `pub capability Args`
   - `runtime/mod.ash` re-exports the runtime symbols
   - `std/src/lib.ash` exposes the runtime surface
3. Keep tests textual/file-based unless an existing compile-with-stdlib harness already exists in the relevant crate. `TASK-359` should not invent a large new Rust harness if file/surface tests are sufficient to prove the stdlib extension.

## Implementation Strategy

1. Add failing parser-side stdlib tests.
2. Create the new `runtime` source tree.
3. Update `std/src/lib.ash` exports.
4. Run focused stdlib tests until green.
5. Run a focused build for `ash-std` plus the touched parser tests.
6. Update `CHANGELOG.md` for completed `TASK-359` work.

## Risks and Mitigations

### Risk: over-implementing supervisor semantics too early

Mitigation: keep `supervisor.ash` as a stdlib contract scaffold only; leave bootstrap/runtime behavior to blocked follow-up tasks.

### Risk: mismatching current parser surface syntax

Mitigation: use only already-canonicalized forms validated by S57-4/S57-5 and mirror existing stdlib style.

### Risk: exporting the runtime surface inconsistently

Mitigation: verify both `std/src/runtime/mod.ash` and `std/src/lib.ash` via focused textual tests.

## Expected Files

### Create

- `std/src/runtime/mod.ash`
- `std/src/runtime/error.ash`
- `std/src/runtime/args.ash`
- `std/src/runtime/supervisor.ash`
- `docs/plans/2026-04-01-task-359-stdlib-initialization-design.md`
- `docs/plans/2026-04-01-task-359-stdlib-initialization-plan.md`

### Modify

- `std/src/lib.ash`
- `crates/ash-parser/tests/stdlib_parsing.rs`
- `crates/ash-parser/tests/stdlib_surface.rs`
- `CHANGELOG.md`

## Success Criteria

`TASK-359` is complete when:

- the `runtime` stdlib module tree exists under `std/src/`;
- `runtime::RuntimeError` and `runtime::Args` are defined using canonical syntax;
- a supervisor module scaffold exists for downstream tasks;
- root stdlib exports include the new runtime surface;
- focused stdlib tests pass;
- changelog bookkeeping is updated for `TASK-359`.
