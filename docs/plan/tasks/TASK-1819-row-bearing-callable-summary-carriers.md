# TASK-1819: Add row-bearing callable summary carriers

## Status: ✅ Complete

## Description

Add the minimal row-bearing callable summary carriers needed to transport explicit parsed rows beyond Phase 177 validation. This task should not change row inference or runtime behavior.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-021](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)

## Dependencies

- TASK-1818 row-loss audit complete.

## Requirements

### Functional Requirements

1. Add or extend a source callable row summary carrier that can represent explicit inline rows and `where row` rows.
2. Preserve source spans/origins needed for diagnostics.
3. Preserve whole-row variables, open-row tails, impl-qualified operation identities, and unresolved source-path row metadata from Phase 177.
4. Avoid replacing unrelated type infrastructure or broad `Type::Fn` semantics in this task.
5. Add focused tests proving a parsed function's row appears in the new summary carrier.
6. Add serialization/import-summary tests if the carrier crosses module boundaries in this task.

### Property Requirements

- Carriers are metadata/requirements only; they do not install providers, roles, resources, handlers, or admission.
- Rowless callables must retain existing summary behavior.
- Unsupported row forms must remain fail-closed as established in Phase 177.

## TDD Steps

### Step 1: Write failing carrier tests

Add tests in the owner crate identified by TASK-1818 proving inline and expanded rows appear in row-bearing callable summaries.

### Step 2: Verify RED

Run focused tests and confirm row metadata is absent or lost.

### Step 3: Implement minimal carriers

Add the row-bearing summary shape and populate it at the audited source boundary.

### Step 4: Verify GREEN

Run focused tests and affected crate tests.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - git diff --check
checklist:
  - [ ] Row-bearing callable summary carrier exists.
  - [ ] Inline rows populate the carrier.
  - [ ] Expanded `where row` rows populate the carrier.
  - [ ] Rowless callables retain existing behavior.
```

## Dependencies for Next Task

This task feeds TASK-1820.

## Completion Evidence

- Added `InlineCallable::row_requirement` with `CallableRowRequirementSummary` and `CallableRowRequirementSource` in `crates/ash-engine/src/module_loader.rs`.
- Populated row metadata from ordinary `pub fn` inline rows, ordinary `pub fn where row { ... }`, and `pub builtin fn where row { ... }` in `crates/ash-engine/src/module_loader/callable_exports.rs`.
- Preserved rowless callable behavior by leaving `row_requirement` empty when no explicit callable row is present.
- Added TASK-1819 module-loader tests covering inline rows, expanded rows, builtin rows, and rowless callables.
- Verification:
  - `cargo fmt --check`
  - `cargo test -p ash-engine task1819 -- --nocapture`
  - `cargo test -p ash-parser`
  - `cargo test -p ash-engine`
  - `git diff --check`
