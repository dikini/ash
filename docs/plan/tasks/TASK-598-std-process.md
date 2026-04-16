# TASK-598: std::process Interface and Rust Backend

## Status: 📝 Planned

## Description

Implement `std::process` as a built-in, auto-registered `Capability` with the `Operational` effect.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track B
- DESIGN-NOTE-PROCESS-EFFECT.md

## Dependencies

- None

## Requirements

1. Define `ProcessOutput`, `ProcessOptions`, `run`, `run_with_options`.
2. Implement Rust backend using `tokio::process::Command`.
3. Auto-register `Process` capability in `Engine::default()` / CLI builder.
4. Enforce timeout (default 30s).

## TDD Steps

### Step 1: Write failing test

Ash workflow calls `process::run("echo", ["hello"])`; assert stdout contains `"hello"`.

### Step 2: Implement

- `std/src/process.ash`
- `crates/ash-engine/src/capabilities/process.rs`
- Update engine builder.

### Step 3: Verify

Subprocess execution works from Ash source.

## Verification Steps

- [ ] Integration test passes
- [ ] Auto-registration confirmed in CLI and `Engine::default()`
- [ ] Timeout handling verified
- [ ] Codex verification: VERIFIED
