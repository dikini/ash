# TASK-1630: Document Core Ash implementation boundaries

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add reference documentation for the implemented Core Ash subset, `.core` fixture format, validation boundary, and Core-to-CPS lowering boundary.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)

## Dependencies

- [TASK-1629](TASK-1629-core-end-to-end-fixtures.md)

## Requirements

### Functional Requirements

1. Update `docs/reference/core-ash-text-format.md` with implemented examples.
2. Add or update a reference page for Core-to-CPS lowering.
3. Clearly state that `.core` is not surface Ash.
4. List deferred features: surface lowering, typeclass solving, user-defined algebraic effects, `MultiShotPure`, Core `Match`, full type checker.

### Property Requirements

- Reference docs must not overclaim implementation status.
- Examples must correspond to committed fixtures or tests.

## TDD Steps

### Step 1: Write docs consistency check

**Files:** `crates/ash-core/tests/task_1630_core_docs_consistency.rs`

Add a lightweight test that checks referenced fixture files exist for examples named by docs.

Run:

```bash
cargo test -p ash-core --test task_1630_core_docs_consistency
```

Expected: fail until docs/examples are reconciled.

### Step 2: Update docs

**Files:** `docs/reference/core-ash-text-format.md`, optional `docs/reference/core-ash-lowering.md`

Keep documentation factual and bounded.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1630_core_docs_consistency
git diff --check -- docs/reference crates/ash-core/tests/task_1630_core_docs_consistency.rs
```

Expected: docs consistency check passes.

## Completion Evidence

- Updated `docs/reference/core-ash-text-format.md` with the Phase 161 fixture/golden corpus and explicit `.core` fixture/debug boundary.
- Added `docs/reference/core-ash-lowering.md` for the implemented validated Core-to-CPS lowering boundary, row synthesis rules, effect/handler/contract behavior, and deferred features.
- Added `task_1630_core_docs_consistency.rs` to keep the reference pages tied to committed fixtures and bounded implementation claims.

Verified on 2026-06-20:

```bash
cargo test -p ash-core --test task_1630_core_docs_consistency
```
