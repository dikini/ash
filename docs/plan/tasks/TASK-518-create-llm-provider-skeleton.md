# TASK-518: Create LlmProvider skeleton and list_models implementation

## Status: ✅ Complete

## Description

Create the `LlmProvider` struct implementing `CapabilityProvider`, with a multi-provider registry, action dispatch, and a working list_models execute action. This follows the `McpProvider` pattern and is the main entry point for LLM capability execution in the engine.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS9: Rust Provider Contract)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-517](TASK-517-create-llm-config.md)

## Requirements

1. Create `LlmProvider` with `configs: HashMap<String, LlmConfig>` for multi-provider registry.
2. Lazy client creation per provider name using `async-openai::Client<OpenAIConfig>`.
3. Implement `CapabilityProvider`: `name() -> "llm"`, `effect() -> Deliberative`.
4. `observe()` is unused for the Phase 77 LLM action surface and returns `NotAvailable`.
5. `execute()` dispatches on `"chat"`, `"chat_with_tools"`, `"chat_stream"`, `"embed"`, and `"list_models"`.
6. Constructor validates all configs.
7. Tests for construction, provider name, effect level, unknown action handling.

## Guidance

Follow `McpProvider` in `crates/ash-engine/src/providers/mcp.rs` for the `CapabilityProvider` impl pattern. Use `tokio::sync::Mutex` for lazy client map since client creation is async. The `list_models` implementation in `execute("list_models", &[provider])` should use `async-openai`'s model listing API.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/mod.rs`
- Create: `crates/ash-engine/src/providers/llm/models.rs` (list_models implementation)
- Modify: `crates/ash-engine/src/providers/mod.rs` (add `pub mod llm;`)

## TDD Steps

### Red

1. Write test: `LlmProvider::new()` with valid config map succeeds.
2. Write test: `LlmProvider::new()` with invalid config returns error.
3. Write test: `name()` returns `"llm"`.
4. Write test: `effect()` returns `Deliberative`.
5. Write test: `execute()` with unknown action returns `NotAvailable`.
6. Write test: `observe()` returns `NotAvailable` for the unused observe entry point.
7. Write test: `execute("list_models", &[provider])` returns list of model names from mock.

### Green

Implement the provider skeleton with stub action handlers and a working `list_models` execute handler.

## Completion Checklist

- [ ] `LlmProvider` struct with multi-provider registry
- [ ] Lazy client creation per provider
- [ ] `CapabilityProvider` impl with correct name and effect
- [ ] Action dispatch for execute actions (chat, chat_with_tools, chat_stream, embed)
- [ ] `observe()` returns `NotAvailable` for the unused observe entry point
- [ ] Execute dispatch for list_models using async-openai API
- [ ] `models.rs` created with list_models implementation
- [ ] All tests pass: `cargo test -p ash-engine --lib providers::llm`
