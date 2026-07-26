# TASK-597: std::json Hybrid Interface

## Status: ✅ Complete

## Description

Implement `std::json` with Rust-backed `parse`/`stringify` and a pure-Ash `JsonValue` AST.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track B
- DESIGN-NOTE-JSON-STRATEGY.md

## Dependencies

- None

## Requirements

1. Define `JsonValue` enum: `Null | Bool | Number | String | Array | Object`.
2. Implement Rust-backed `parse` and `stringify` via `serde_json`.
3. Implement pure-Ash accessors: `is_null`, `as_string`, `get`, `get_index`.

## TDD Steps

### Step 1: Write failing test

Property test: `parse(stringify(v)) == v` for generated `JsonValue`.

### Step 2: Implement

- `std/src/json.ash`
- `crates/ash-engine/src/capabilities/json.rs`

### Step 3: Verify

Round-trip property tests pass.

## Verification Steps

- [ ] Property tests pass (100+ iterations)
- [ ] `JsonValue` shape matches design note
- [ ] Codex verification: VERIFIED

## Current Production-Route Evidence

`json_stdlib_e2e` retains file-based parser and typechecker coverage for `parse`, `stringify`,
and `stringify_pretty`, including the combined import and a malformed JSON argument. Under
TASK-2014 Path B it deliberately asserts the exact closed-admission error after checking:
`checked Core/CPS admission rejected: no validated production typed lowering is available`.
That rejection occurs before JSON host dispatch, so it is not evidence that malformed JSON has
been evaluated. The former direct-evaluator result assertions are superseded and must not return
as a compatibility fallback; runtime JSON execution awaits validated typed lowering and the
checked Core/CPS host path.
