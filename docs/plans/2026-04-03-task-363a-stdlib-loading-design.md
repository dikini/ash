# TASK-363a Runtime Stdlib Loading Design

## Goal

Finish the deferred `TASK-363a` work by replacing the current entry-path stdlib preflight shim with an engine-owned runtime stdlib registry that can honestly satisfy canonical `use result::...` and `use runtime::...` imports for the Phase 57 entry path.

## Problem Statement

The current entry bootstrap slice is intentionally narrow:

- `load_runtime_entry_stdlib_sources()` reads raw stdlib source files
- `Engine::parse_entry_source()` strips a leading `use` prelude before parsing a single workflow
- entry verification/bootstrap therefore succeeds without true module registration or import resolution

That is good enough for the completed `TASK-364`, `TASK-363b`, and narrow `TASK-363c` slice, but it does **not** complete `TASK-363a`. The engine still lacks an honest notion of registered stdlib modules, and imported entry files are not validated against a real module table.

## Approved Approach

Implement a small engine-local runtime stdlib registry rather than either:

- extending the current source-stripping shim further, or
- attempting a full general module-graph integration in this task.

This keeps the work aligned with the existing `Engine` abstraction and makes the entry path honest without widening scope into a full compiler/module-system project.

## Scope

In scope:

- add engine-owned storage for registered runtime stdlib modules
- load the canonical runtime stdlib modules by module path:
  - `result`
  - `runtime`
  - `runtime::error`
  - `runtime::args`
- expose explicit engine helpers for runtime stdlib registration/loading
- make the entry path consult registered modules instead of relying only on raw-source preflight
- add focused tests proving runtime stdlib registration and canonical entry import compatibility
- update task docs/changelog accurately

Out of scope:

- general `Engine::load_module()` for arbitrary Ash modules
- full `ModuleResolver` / `ImportResolver` integration across the engine
- full `crate` / dependency graph support
- non-runtime stdlib module resolution

## Constraints

1. Use the existing `ash_engine::Engine`; do not invent a new runtime type.
2. Keep the implementation local to the runtime entry path.
3. Preserve the currently passing entry verification/bootstrap behavior.
4. Do not claim full import resolution beyond the registered runtime stdlib slice.
5. Keep the module registry explicit and inspectable.

## Design

### 1. Engine-owned runtime stdlib module table

`Engine` should gain a small internal registry keyed by canonical module path. Each entry stores the raw Ash source for a registered runtime stdlib module. This is intentionally a source registry, not a full typed module graph.

Proposed behavior:

- `register_runtime_stdlib_module(path, source)` stores a canonical module source
- `load_runtime_stdlib()` populates the registry from `std/src`
- `has_registered_runtime_module(path)` supports tests and internal guards

This makes stdlib loading an engine responsibility instead of an external preflight helper.

### 2. Honest entry-path import precheck

The current parser still cannot parse a full general module file with imports plus arbitrary top-level items, so this task should not pretend to implement that. Instead, the entry path should:

- parse leading `use` items from the entry prelude
- verify that every imported runtime/stdlib path needed by the canonical entry slice is present in the engine registry
- only then strip the prelude and continue through the existing single-workflow parse path

This is narrower than full import resolution but honest about what is actually being checked.

### 3. Entry bootstrap composition

`bootstrap_entry_source()` should call `load_runtime_stdlib()` / the registry-backed helper rather than the current free function preflight. That keeps all runtime stdlib state inside the engine and lets `TASK-363a` complete without changing the observable bootstrap contract.

### 4. Preserve task boundaries

After this task:

- `TASK-363a` becomes complete
- `TASK-363b`, `TASK-363c`, `TASK-364` remain complete
- `TASK-365` and `TASK-366` stay as downstream work

The engine will support honest runtime stdlib registration for entry imports, but not a general-purpose module system.

## Testing Strategy

1. Unit/integration test that engine runtime stdlib loading registers all required module paths.
2. Entry-path test that imported entry source succeeds only after the runtime stdlib is loaded into the engine registry.
3. Negative test for missing registered runtime module, if the current parser boundary allows it cleanly.
4. Re-run existing parser, engine, and CLI suites to ensure no regressions.

## Risks and Mitigations

- **Risk:** the design drifts into a partial module system.
  - **Mitigation:** keep the registry internal to the runtime stdlib slice and avoid arbitrary module loading APIs.
- **Risk:** parser limitations make import validation ambiguous.
  - **Mitigation:** validate only the explicit leading `use` prelude already supported by `parse_entry_source()`.
- **Risk:** duplicated stdlib-loading logic remains between helper layers.
  - **Mitigation:** make engine bootstrap call the registry-backed loader directly and treat old free helpers as compatibility wrappers or remove them if no longer needed.
