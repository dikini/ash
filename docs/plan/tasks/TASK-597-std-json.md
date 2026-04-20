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
