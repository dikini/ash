# TASK-517: Create LlmConfig struct

## Status: Draft

## Description

Define the `LlmConfig` struct that holds per-provider connection settings for the LLM provider, with validation and defaults. This is the configuration type that the Rust-side `LlmProvider` will use to connect to different OpenAI-compatible backends.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS9.3: LlmConfig)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-516](TASK-516-add-async-openai-dependency.md)

## Requirements

1. Create `LlmConfig` struct with fields: `api_base: String`, `api_key: String`, `default_model: String`, `timeout_ms: u64`, `max_retries: u32`.
2. Implement `Default` with sensible values (api_base: OpenAI default, timeout: 30s, retries: 2).
3. Implement `validate(&self) -> Result<(), String>` checking api_base is a valid URL and api_key is non-empty.
4. Implement `Debug` that redacts `api_key`.
5. Unit tests for construction, validation, and debug redaction.

## Guidance

Follow the pattern of `McpConfig` in `crates/ash-engine/src/providers/mcp.rs`. The struct is `#[derive(Debug, Clone)]` with a manual Debug impl that masks the API key.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/config.rs`
- Modify: `crates/ash-engine/src/providers/llm/mod.rs` (re-export)

## TDD Steps

### Red

1. Write test: `LlmConfig::default()` has expected field values.
2. Write test: `LlmConfig::validate()` accepts valid config.
3. Write test: `LlmConfig::validate()` rejects empty api_base.
4. Write test: `LlmConfig::validate()` rejects invalid URL.
5. Write test: `LlmConfig::validate()` rejects empty api_key.
6. Write test: Debug output does not contain the api_key value.

### Green

Implement `LlmConfig` with all required impls to pass tests.

## Completion Checklist

- [ ] `LlmConfig` struct created with all fields
- [ ] `Default` impl with sensible values
- [ ] `validate()` method rejects invalid configs
- [ ] `Debug` impl redacts api_key
- [ ] All tests pass: `cargo test -p ash-engine --lib providers::llm::config`
