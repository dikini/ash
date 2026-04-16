# TASK-595: std::regex Interface and Rust Backend

## Status: 📝 Planned

## Description

Add `std::regex` with `find`, `matches`, and `replace` functions, backed by the `regex` Rust crate.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track B
- DESIGN-NOTE-PROCESS-EFFECT.md §6 (auto-registered built-in pattern)

## Dependencies

- None

## Requirements

1. Define Ash interface: `find`, `matches`, `replace`.
2. Implement Rust-backed `regex` capability provider.
3. Return `RuntimeError` on invalid pattern.

## TDD Steps

### Step 1: Write failing test

Rust integration test calling `regex::find` through the engine. Expected: FAIL — capability not found.

### Step 2: Implement

- `std/src/regex.ash`
- `crates/ash-engine/src/capabilities/regex.rs`
- Register provider in engine.

### Step 3: Verify

Test passes with real regex operations.

## Verification Steps

- [ ] Integration tests pass
- [ ] Real callsites exist (spec processor link validation)
- [ ] Codex verification: VERIFIED
