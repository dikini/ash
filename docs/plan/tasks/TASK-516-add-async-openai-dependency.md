# TASK-516: Add async-openai dependency to ash-engine

## Status: Draft

## Description

Add the `async-openai` crate as a dependency to `ash-engine` so that the LLM provider can use it for OpenAI-compatible HTTP communication. No code changes yet -- just the dependency addition and a compilation check.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS9: Rust Provider Contract)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

None.

## Requirements

1. Add `async-openai` to `[dependencies]` in `crates/ash-engine/Cargo.toml`.
2. Use a version that supports chat completions, streaming, and embeddings.
3. The workspace must compile successfully after adding the dependency.

## Guidance

The `async-openai` crate provides typed clients for the OpenAI chat completions, streaming, and embeddings endpoints. It is added alongside the existing `reqwest`, `serde`, and `serde_json` entries. No feature flags needed for basic chat/embeddings support.

## Likely Files

- Modify: `crates/ash-engine/Cargo.toml`

## TDD Steps

### Red

Not applicable -- this task adds a dependency only.

### Green

1. Add `async-openai` dependency to `crates/ash-engine/Cargo.toml`.
2. Run `cargo check -p ash-engine` to verify compilation.

## Completion Checklist

- [ ] `async-openai` added to `crates/ash-engine/Cargo.toml`
- [ ] `cargo check -p ash-engine` passes
- [ ] No warnings from the new dependency
