# TASK-523: Wire up engine builder method

## Status: ✅ Complete

## Description

Add `with_llm_capabilities()` to `EngineBuilder`, update module re-exports, and add integration tests that the full engine lifecycle works with an LLM provider registered.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS9: Rust Provider Contract)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-519](TASK-519-implement-chat-completion.md)
- [TASK-520](TASK-520-implement-streaming-adapter.md)
- [TASK-521](TASK-521-implement-tool-dispatch-helpers.md)
- [TASK-522](TASK-522-implement-embeddings.md)

## Requirements

1. Add `pub mod llm;` and re-exports to `crates/ash-engine/src/providers/mod.rs`.
2. Add `with_llm_capabilities(HashMap<String, LlmConfig>) -> Self` to EngineBuilder.
3. Integration test: engine with valid LLM provider builds successfully.
4. Integration test: engine with multi-provider config registers correctly.
5. Integration test: engine with LLM provider can execute a workflow using LLM capability.

## Guidance

Follow the pattern of `with_stdio_capabilities()` and `with_fs_capabilities()` in `crates/ash-engine/src/lib.rs`.

## Likely Files

- Modify: `crates/ash-engine/src/providers/mod.rs`
- Modify: `crates/ash-engine/src/lib.rs`

## TDD Steps

### Red

1. Write test: engine with valid LLM config builds.
2. Write test: engine with invalid LLM config handles gracefully.
3. Write test: multi-provider config (two entries) registers correctly.

### Green

Add the builder method and re-exports.

## Completion Checklist

- [ ] `pub mod llm` in providers/mod.rs
- [ ] Re-exports of LlmConfig and LlmProvider
- [ ] `with_llm_capabilities()` builder method
- [ ] Integration tests pass: `cargo test -p ash-engine --lib tests::llm_integration`
