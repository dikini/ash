# TASK-531: Create loading workflows

## Status: Draft

## Description

Create the loading workflows that read prompts from file, cache, or environment: `load_prompt` and `load_system_prompt`. These are workflows (Tier 2) because they perform IO, not Tier 1 fn.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS7: Loading Workflows)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-529](TASK-529-create-openai-capability.md)

## Requirements

1. `workflow load_prompt(source: String) -> Message` -- loads prompt from file path, env var, cache key, or raw string.
2. `workflow load_system_prompt(name: String) -> Message` -- loads named system prompt from configured directory.
3. Both use IO capabilities (fs.read) -- hence workflow, not fn.
4. Source form detection per SPEC-029 SS7:
   - `file:path` -> read file, return `system(content)`
   - `env:VAR` -> read env var, return `system(content)`
   - `cache:key` -> lookup cached prompt, return `system(content)`
   - Other (literal string) -> return `system(content)`

## Guidance

`load_prompt` detects the source form by prefix: `file:` for file paths, `env:` for environment variables, `cache:` for cached prompts. All forms return a `system` message. `load_system_prompt` looks up named prompts in a standard directory.

## Likely Files

- Modify: `std/src/llm/openai/mod.ash` (add loading workflows)

## TDD Steps

### Red

1. Write test: `load_prompt` with `file:path` reads file and returns system message.
2. Write test: `load_prompt` with `env:VAR` resolves env var and returns system message.
3. Write test: `load_prompt` with `cache:key` looks up cached prompt and returns system message.
4. Write test: `load_prompt` with raw string returns system message directly.

### Green

Implement both loading workflows.

## Completion Checklist

- [ ] `load_prompt` handles file:/env:/cache:/raw sources
- [ ] All forms return `system(content)` messages
- [ ] `load_system_prompt` loads named prompts
- [ ] Both are workflows (not fn)
- [ ] File parses without errors
