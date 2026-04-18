# TASK-595: std::regex Interface and Rust Backend

## Status: 🟡 Partial

## Completion Note

The Rust-side `regex` provider is functional with 12 passing tests in
`crates/ash-engine/`. However, the Ash-language import surface (`use regex::{find,matches,replace}`)
is **not** proven end-to-end. The `std/src/regex.ash` file defines `pub fn` wrappers
that use `act execute` inside `fn` bodies, which the parser cannot handle at expression
level. Attempting `use regex::{find}` fails at module load time with:

```
item 'find' not found in module 'regex'
```

This error occurs because `regex.ash` cannot be fully parsed — the module loader
collects `pub fn` exports but rejects the `act execute` body syntax, so no callable
items are exported from the module.

The task remains Partial until the Ash surface can be exercised through the runtime.

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
3. Return `CapabilityError::InvalidArgument` on invalid pattern (not `RuntimeError` — the provider boundary uses the capability error type, not the runtime error tuple).

## TDD Steps

### Step 1: Write failing test

Rust integration test calling `regex::find` through the engine. Expected: FAIL — capability not found.

### Step 2: Implement

- `std/src/regex.ash` (aspirational — not yet parseable by current substrate)
- `crates/ash-engine/src/providers/regex.rs` (landed, tested)
- Register provider in engine (landed).

### Step 3: Verify

Rust-side provider tests pass. Ash-language surface does not yet work.

## Verification Steps

- [x] Rust provider integration tests pass (`cargo test -p ash-engine --test regex_capability`)
- [ ] Ash-language `use regex::{find}` import succeeds and function is callable
- [x] Limitation regression test documents current import failure (`crates/ash-engine/tests/regex_import_limitation.rs`)
