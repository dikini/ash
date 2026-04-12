# LLM Standard Library

This module provides the LLM standard library for Ash, following the three-tier model
(DESIGN-020): pure functions, capabilities, and workflows.

## Architecture

### Tier 1: Pure Functions (fn)

- **`types.ash`** -- Data type definitions: `Role`, `Message`, `ChatResponse`, `ToolCall`,
  `ToolDef`, `Usage`, `ChatChunk`, `ToolCallDelta`, `Embedding`, `ProviderConfig`,
  `CompletionParams`.
- **`prompt.ash`** -- Pure message constructors (`system`, `user`, `assistant`, `tool_result`),
  inspectors (`has_tool_calls`, `is_final`, `get_tool_calls`), and renderers
  (`render_conversation`, `render_template`).

### Tier 2: Loading Workflows

- **`loading.ash`** -- Workflows for loading prompts from files or literal strings.
  - `load_prompt(source)` -- Load a prompt from `file:path` or literal string.
  - `load_system_prompt(name)` -- Load a named system prompt from the prompt directory.

### Tier 3: Dispatch and Agent Workflows

- **`openai.ash`** -- `Llm` capability declaration with five actions: `chat`, `chat_with_tools`,
  `chat_stream`, `embed`, `list_models`.
- **`dispatch.ash`** -- Thin dispatch workflows wrapping `act` calls: `complete`,
  `complete_with_tools`, `complete_tuned`, `ask`, `stream`, `embed`, `list_models`.
- **`conversation.ash`** -- Multi-turn conversation workflow.
- **`tool_agent.ash`** -- Orient-decide-act tool-use agent loop.
- **`router.ash`** -- Task-complexity classification and multi-model routing.
- **`supervised.ash`** -- Spawn/kill/restart supervised agent pattern.

## Registration

Register LLM providers in the engine builder:

```rust
use ash_engine::providers::llm::{LlmConfig, LlmProvider};
use std::collections::HashMap;

let mut configs = HashMap::new();
configs.insert("openai".to_string(), LlmConfig {
    api_base: "https://api.openai.com/v1".to_string(),
    api_key: env::var("OPENAI_API_KEY").unwrap(),
    default_model: "gpt-4o".to_string(),
    timeout_ms: 30000,
    max_retries: 2,
});

let engine = Engine::new()
    .with_llm_capabilities(configs)
    .build()
    .unwrap();
```

## Multi-Provider Routing

Route to different backends by provider name:

```ash
let local = complete("ollama", "llama3", messages, None);
let remote = complete("openai", "gpt-4", messages, None);
```

## Namespace Layout

```
std/src/llm/
  mod.ash           -- Module root, re-exports
  types.ash         -- Pure type definitions
  prompt.ash        -- Pure prompt functions
  openai.ash        -- Llm capability declaration
  dispatch.ash      -- Dispatch workflows
  loading.ash       -- Loading workflows
  conversation.ash  -- Conversation orchestration
  tool_agent.ash    -- Tool-use agent
  router.ash        -- Multi-model router
  supervised.ash    -- Supervised agent
```

## References

- DESIGN-025: LLM Standard Library (architectural design)
- SPEC-029: LLM Standard Library (normative specification)
- PLAN-025: LLM Standard Library (implementation plan)
