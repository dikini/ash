# TASK-522: Implement embeddings

## Status: ✅ Complete

## Description

Implement the `"embed"` action for text embedding via `async-openai`. Constructs embedding requests and converts responses to Ash Values.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS5.2.3: Embed action, SS9.4: Error mapping)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-518](TASK-518-create-llm-provider-skeleton.md)

## Requirements

1. Construct `CreateEmbeddingRequest` from action args (provider, model, texts).
2. Convert `CreateEmbeddingResponse` to Ash `Value::List` of Embedding records.
3. Error mapping same as chat (SPEC-029 SS9.4).
4. Provider routing by name.
5. Tests for request construction and response conversion.

## Guidance

Use `async-openai` builder pattern for embedding requests. Response has `data: Vec<Embedding>` with `index` and `embedding` fields.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/embeddings.rs`
- Modify: `crates/ash-engine/src/providers/llm/mod.rs` (wire embed action)

## TDD Steps

### Red

1. Write test: request construction from args.
2. Write test: response conversion preserves index ordering.
3. Write test: result length matches input length.

### Green

Implement embedding request construction and response conversion.

## Completion Checklist

- [ ] Embedding request construction
- [ ] Response conversion to Ash Value
- [ ] Error mapping
- [ ] Tests pass: `cargo test -p ash-engine --lib providers::llm::embeddings`
