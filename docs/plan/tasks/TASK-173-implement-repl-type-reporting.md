# TASK-173: Implement REPL Type Reporting

## Status: ✅ Complete

## Description

Replace placeholder REPL `:type` reporting with canonical inferred-type reporting.

This task routes REPL type reporting through the canonical parse and type-check pipeline so
the user-visible output matches the frozen REPL contract.

## Specification Reference

- SPEC-003: Type System
- SPEC-005: CLI
- SPEC-011: REPL

## Reference Contract

- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/surface-guidance-boundary.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`

## Requirements

### Functional Requirements

1. Make `:type` report canonical inferred types rather than placeholders
2. Route reporting through the canonical parse/type-check pipeline
3. Add tests proving the canonical REPL type output
4. Preserve `:type` as runtime-observable behavior; any future advisory-versus-authoritative stage
   wording remains explanatory only and must not change runtime semantic authority

## Files

- Modify: `crates/ash-repl/src/lib.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/parse.rs`
- Test: `crates/ash-repl/tests/repl_type_reporting.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving `:type` reports the canonical inferred type output instead of a placeholder.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-repl repl_type_reporting -- --nocapture
```

Expected: fail because type reporting is placeholder-level today.

### Step 3: Implement the minimal fix (Green)

Route type reporting through the canonical parse/type-check pipeline.

### Step 4: Verify focused GREEN

Run the same command again.

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-repl/src/lib.rs crates/ash-engine/src/lib.rs crates/ash-engine/src/parse.rs crates/ash-repl/tests/repl_type_reporting.rs CHANGELOG.md
git commit -m "fix: implement repl type reporting"
```

## Completion Checklist

- [x] failing REPL type-reporting tests added
- [x] failure verified
- [x] canonical type reporting implemented
- [x] focused verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No output-format redesign outside the canonical spec
- No new REPL features
- No reinterpretation of REPL type output as projected reasoner context

## Dependencies

- Depends on: TASK-163, TASK-172
- Blocks: TASK-176, TASK-208

This task is the second implementation step in the tooling observable convergence extension defined
by `docs/plan/2026-03-20-tooling-observable-convergence-plan.md`.
