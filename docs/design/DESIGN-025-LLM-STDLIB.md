# DESIGN-025: LLM Standard Library

## Status: Draft

## Overview

Design an LLM standard library for Ash that communicates with OpenAI-compatible providers via the
`async-openai` Rust crate. The library provides pure data types, prompt construction, loading
workflows, LLM dispatch workflows, and agent orchestration patterns -- all structured as protocol
modules that sit alongside peer modules `mcp/` and `a2a/`, composed by an `agent/` layer above them.

This design follows the three-vertex model from DESIGN-020: pure functions (`fn`) for data types and
prompt construction, capabilities for effect contracts, and workflows for effectful loading and LLM
dispatch. It extends the unified action system (DESIGN-015, PLAN-015) with a new `LlmProvider` that
implements the same `ash_core::CapabilityProvider` trait used by `FsProvider`, `StdioProvider`, and
`McpProvider`.

## Problem Statement

Ash currently has no standard library support for LLM interaction. The `McpProvider` in
`crates/ash-engine/src/providers/mcp.rs` demonstrates that a Rust-side capability provider using an
HTTP client can be wired into the engine via `with_custom_provider()`, but there is no
OpenAI-compatible LLM provider, no Ash-level types for messages and responses, no prompt
construction vocabulary, and no agent orchestration patterns.

What is missing:

1. No data types for messages, roles, tool calls, embeddings, or chat responses.
2. No pure prompt construction functions (system/user/assistant message builders, formatters,
   renderers).
3. No capability contract for LLM actions (chat, stream, embed, list models).
4. No Rust-side provider bridging `async-openai` into Ash's capability system.
5. No loading workflows for prompt sources (file, cache, environment).
6. No dispatch workflows wrapping capability `act` calls.
7. No agent orchestration patterns (tool-use loops, multi-model routing, supervised agents).
8. No namespace structure that accommodates multiple LLM providers or peer protocols (MCP, A2A).

## Goals

1. Define a provider-agnostic vocabulary of pure types and prompt functions in `std/src/llm/`.
2. Implement an OpenAI-specific capability and dispatch layer in `std/src/llm/` (flat layout).
3. Provide a Rust-side `LlmProvider` using `async-openai` that plugs into the existing engine
   registration system.
4. Support multi-provider routing (different `api_base` per provider) for ollama, vLLM,
   together.ai, fireworks, litellm, and any OpenAI-compatible endpoint.
5. Lay out agent orchestration patterns as first-class workflows in separate files under `std/src/llm/`.
6. Structure namespaces so future providers (`llm/anthropic/`) and peer protocols (`mcp/`, `a2a/`)
   coexist naturally, with `agent/` composing across all of them.

## Non-Goals

This design does not attempt to:

- Implement Anthropic, Google, or other non-OpenAI-compatible providers.
- Define a training or fine-tuning API.
- Provide a chat UI or REPL.
- Embed a local inference engine.
- Solve prompt-versioning or prompt-registry concerns beyond basic file/env loading.
- Implement persistent conversation storage (provider does not own state).

## Design Decisions

### D1: Protocol Module Architecture

```
std/src/
  llm/              -- protocol module: LLM shared vocabulary
    mod.ash           -- module root, re-exports
    types.ash         -- pure: Message, Role, ChatResponse, ToolCall, ToolDef, Usage, ChatChunk,
                       --        Embedding, ProviderConfig, CompletionParams
    prompt.ash        -- pure fn: constructors, formatters, filters, renderers
    openai.ash        -- capability declaration (Llm)
    dispatch.ash      -- dispatch workflows (complete, stream, embed, list_models, etc.)
    loading.ash       -- loading workflows (load_prompt, load_system_prompt)
    conversation.ash  -- orchestration: multi-turn conversation
    tool_agent.ash    -- orchestration: tool-use agent loop
    router.ash        -- orchestration: multi-model routing
    supervised.ash    -- orchestration: spawn/kill/restart supervised agent
  mcp/              -- peer protocol module (Model Context Protocol)
  a2a/              -- peer protocol module (Agent-to-Agent)
  agent/            -- composes across all protocols (patterns, skills, types)
```

Principle: **protocol modules declare capabilities, agent modules compose them.**

`llm/` owns the shared vocabulary that any LLM provider can use. `llm/openai.ash` owns the
OpenAI-specific capability declaration. `llm/dispatch.ash` provides thin dispatch workflows
wrapping `act` calls. `llm/loading.ash` provides prompt-loading workflows. Agent orchestration
patterns are each in their own file (`conversation.ash`, `tool_agent.ash`, `router.ash`,
`supervised.ash`). `mcp/` and `a2a/` are peers at the same level. `agent/` sits above all
protocols and composes them into higher-order patterns. Agent loops that use LLM + MCP + A2A
are general agent patterns, not tied to any single protocol.

### D2: Three-Tier Layering

Consistent with the three-vertex model (DESIGN-020, D1), the LLM stdlib is organized in three
strictly separated tiers:

**Tier 1: fn (pure)** -- `types.ash`, `prompt.ash`

These contain only `fn` definitions. No `act`, no `ret`, no workflow constructs.

- `types.ash`: data type definitions (`Message`, `Role`, `ChatResponse`, `ToolCall`, `ToolDef`,
  `Usage`, `ChatChunk`, `Embedding`, `ProviderConfig`).
- `prompt.ash`: pure constructors (`system()`, `user()`, `assistant()`, `tool_result()`), pure
  inspectors (`append_response()`, `has_tool_calls()`, `is_final()`, `get_tool_calls()`), pure
  renderers (`render_conversation()`, `render_template()`).

These functions compose freely with each other and with any other pure fn in the stdlib.

**Tier 2: workflow (effectful loading)** -- loading workflows in `llm/loading.ash`

These are workflows because they perform IO (read files, check caches, read environment variables)
but do not dispatch to the LLM capability itself.

- `load_prompt(source)`: load a prompt from file, cache, or environment variable.
- `load_system_prompt(name)`: load a named system prompt from a configured directory.

**Tier 3: workflow (LLM dispatch)** -- dispatch workflows in `llm/dispatch.ash`

These are thin workflows that wrap `act` calls to the LLM capability.

- `complete(provider, model, messages, params)`: single chat completion.
- `complete_with_tools(provider, model, messages, tools, params)`: completion with tool
  definitions.
- `complete_tuned(provider, model, messages, params)`: completion with tuning parameters
  (temperature, top_p, etc.).
- `ask(provider, model, question)`: convenience single-turn completion.
- `stream(provider, model, messages, params)`: streaming chat completion returning `Stream<ChatChunk>`.
- `embed(provider, model, texts)`: text embedding.
- `list_models(provider)`: list available models for a provider.

**Strict constraint: NO fn wrappers around act.** Anything using `act` is a workflow. This is the
three-vertex boundary enforcement from DESIGN-020: `fn` never invokes `workflow` or `cap`. Any
function that dispatches an LLM call must be a `workflow`, not a `fn`.

Composition follows the rules from DESIGN-020:
- fn -> fn (freely) -- prompt construction chains pure functions
- workflow -> fn (workflow calls fn for data transforms) -- dispatch workflows call prompt.ash fns
- workflow -> cap (workflow uses capabilities for effects) -- dispatch workflows use `act` on LLM
- fn -X-> workflow (functions never invoke workflows)
- fn -X-> cap (functions never use capabilities)

### D3: Capability Contract

The LLM capability is declared in `llm/openai.ash`:

```ash
pub capability Llm: execute chat(provider: String, model: String, messages: List<Message>,
                                  params: Option<CompletionParams>) -> ChatResponse
               | execute chat_with_tools(provider: String, model: String, messages: List<Message>,
                                          tools: List<ToolDef>,
                                          params: Option<CompletionParams>) -> ChatResponse
               | execute chat_stream(provider: String, model: String, messages: List<Message>,
                                     params: Option<CompletionParams>) -> Stream<ChatChunk>
               | execute embed(provider: String, model: String, texts: List<String>)
                              -> List<Embedding>
               | execute list_models(provider: String) -> List<String>;
```

**Tool passing model:** Tools are a first-class parameter on the `chat_with_tools` action, not
embedded in `CompletionParams`. The `complete_with_tools` dispatch workflow calls
`act llm:chat_with_tools(...)` which passes tools as a separate field. The Rust provider
converts the `List<ToolDef>` into `async-openai`'s `Vec<ChatCompletionTool>` at the wire
boundary. This keeps `CompletionParams` as purely tuning parameters and avoids conflating
tool definitions with generation controls.

Each action takes a **provider name** as its first argument, enabling multi-provider routing. The
provider name maps to an `LlmConfig` entry in the Rust-side registry, while `ProviderConfig` remains
the Ash-level pure type. The Rust registry entry determines the `api_base`, `api_key`, and other
connection details. This means the same `Llm` capability can route to different backends:

```ash
let local = complete("ollama", "llama3", messages, None);
let remote = complete("openai", "gpt-4", messages, None);
```

The provider name is a string key, not a capability instance. The Rust-side `LlmProvider` holds a
registry of named configurations and dispatches to the correct `api_base` based on this key.

### D4: Rust Provider Layer

A new `LlmProvider` in `crates/ash-engine/src/providers/llm/` implements
`ash_core::capability::CapabilityProvider`, following the precedent of `McpProvider` in
`crates/ash-engine/src/providers/mcp.rs`.

```
crates/ash-engine/src/providers/llm/
  mod.rs            -- LlmProvider, re-exports
  chat.rs           -- chat completion + chat_with_tools + streaming logic
  config.rs         -- LlmConfig, multi-provider registry
  embeddings.rs     -- embedding logic
  models.rs         -- model listing logic (list_models execute action)
  stream_adapter.rs -- async-openai Stream -> Ash Stream<ChatChunk>
  tool_dispatch.rs  -- tool call parsing and result formatting
```

**LlmConfig** per named provider:

```rust
pub struct LlmConfig {
    pub api_base: String,       // e.g. "https://api.openai.com/v1" or "http://localhost:11434/v1"
    pub api_key: String,        // API key (or "dummy" for local providers)
    pub default_model: String,  // default model if none specified
    pub timeout_ms: u64,        // request timeout
    pub max_retries: u32,       // retry count for transient failures
}
```

The `api_base` override makes the provider **provider-agnostic**. Any service that exposes an
OpenAI-compatible `/v1/chat/completions` endpoint works without code changes:

- ollama (`http://localhost:11434/v1`)
- vLLM (`http://localhost:8000/v1`)
- together.ai (`https://api.together.xyz/v1`)
- fireworks (`https://api.fireworks.ai/inference/v1`)
- litellm proxy (`http://localhost:4000/v1`)

**Effect level: Operational.** LLM inference involves external HTTP calls with side effects (API
usage billing, potential state changes on remote services). Although the model itself does not
mutate local state, the provider's effect classification is Operational because it performs
network IO and interacts with external systems. This supersedes an earlier Deliberative
classification.

**Streaming:** The `async-openai` crate returns a stream of `ChatChunk` objects from its streaming
API. `stream_adapter.rs` adapts this into Ash's `Stream<ChatChunk>` type. Each chunk contains a
delta content fragment and optional tool call delta, matching the `ChatChunk` type in `types.ash`.

**Provider does NOT own conversation state.** The `LlmProvider` is stateless with respect to
conversations. It receives a full message list on each call and returns a response. State
management (accumulating conversation history, tracking tool results) belongs to Ash workflows.
This is a deliberate design choice: state belongs in the orchestration layer, not the provider
layer.

**Engine registration** follows the existing pattern:

```rust
// Via the general mechanism:
engine.with_custom_provider("llm", Arc::new(LlmProvider::new(configs)?))

// Or via a dedicated builder method (convenience):
engine.with_llm_capabilities(provider_configs)
```

The `with_llm_capabilities()` builder method accepts a `HashMap<String, LlmConfig>` mapping
provider names to their configurations, registers the `LlmProvider` once, and the provider
internally routes by name. The engine registration key for the capability provider is `"llm"`.
Endpoint routing keys such as `"openai"` or `"ollama"` remain data inside the provider registry and
are passed as the first workflow argument to `chat`, `chat_with_tools`, `embed`, or `list_models`.

### D5: Namespace Future-Proofing

The namespace layout is designed to accommodate future growth without restructuring:

```
std/src/llm/           -- shared vocab (types + prompt) usable by any LLM provider
std/src/llm/openai.ash -- OpenAI-specific capability declaration
std/src/llm/dispatch.ash -- dispatch workflows wrapping act calls
std/src/llm/loading.ash  -- loading workflows for prompt sources
std/src/llm/conversation.ash -- orchestration: multi-turn conversation
std/src/llm/tool_agent.ash   -- orchestration: tool-use agent loop
std/src/llm/router.ash       -- orchestration: multi-model routing
std/src/llm/supervised.ash   -- orchestration: supervised agent
std/src/mcp/           -- peer: MCP protocol
std/src/a2a/           -- peer: A2A protocol
std/src/agent/         -- composes across protocols (patterns, skills, types)
```

Key invariants:

- `llm/` types and prompt functions are **provider-agnostic**. They define `Message`, `Role`,
  `ChatResponse`, etc. -- concepts universal to all LLM providers.
- `llm/openai.ash` declares the `Llm` capability. Future provider-specific files (e.g.,
  `llm/anthropic.ash`) would declare their own capabilities with provider-specific actions.
- Agent orchestration workflows (`conversation.ash`, `tool_agent.ash`, `router.ash`,
  `supervised.ash`) use the `Llm` capability declared in `openai.ash`. Each pattern is its own
  file for modularity.
- `agent/` (at the top level, not under any protocol) composes across protocols. A multi-protocol
  agent that uses LLM + MCP + A2A lives here, not under `llm/`.

### D6: Agent Orchestration

Agent loops are **first-class workflows**, not callbacks. They use Ash's
full orchestration vocabulary: `receive`, `spawn`, `send`, `kill`, `check_health`.
Each agent pattern is its own file under `llm/`: `conversation.ash`, `tool_agent.ash`,
`router.ash`, `supervised.ash`.

**Tool-use agent loop** -- the orient-decide-act cycle:

```text
High-level pseudocode for the tool-use agent pattern:

1. Call complete_with_tools(provider, model, messages, tools, None).
2. If the response is final, return it.
3. Otherwise append the assistant response to the conversation.
4. For each tool call in the response:
   - dispatch through a workflow helper with statically declared branches
   - append the tool result as a tool message
5. Recur with the extended conversation and an incremented round count.
6. Stop once max_rounds is reached and return the latest response.

Companion helper idea:
- dispatch_tool_call(call) matches on call.name
- known names route to explicit act ToolHost:... branches
- unknown names return a rendered tool error
```

**Multi-model routing** -- classify task complexity, route to appropriate model:

```text
High-level pseudocode for multi-model routing:

1. Render a classification prompt from the incoming messages.
2. Ask a cheaper routing model for a complexity label.
3. Parse that label into a routing decision such as Simple, Moderate, or Complex.
4. Map the decision to the target model.
5. Call complete(provider, selected_model, messages, None).
```

**Supervised agents** -- spawn/kill/restart pattern:

```text
High-level pseudocode for supervised agents:

1. Spawn a tool_agent workflow from the supplied config.
2. Check the child health and wait for either:
   - a ChatResponse result, which is returned to the caller, or
   - a failure signal, which triggers supervision.
3. On failure, kill the failed child, increment the restart count, and compare it with max_restarts.
4. If the restart budget is exhausted, return AgentError.
5. Otherwise spawn a fresh tool_agent and continue supervising.
```

These patterns are workflows because they use `act` (via dispatch workflows), `spawn`, `kill`,
`check_health`, and `receive`. They cannot be `fn` by the three-vertex constraint.

## File Structure

```
std/src/llm/
  mod.ash             -- module root, re-exports from types and prompt
  types.ash           -- pure fn: type definitions
                        Role = System | User | Assistant | Tool
                        Message { role: Role, content: String, tool_calls: Option<List<ToolCall>>, tool_call_id: Option<String> }
                        ChatResponse { content: Option<String>, tool_calls: Option<List<ToolCall>>, finish_reason: Option<String>, usage: Option<Usage>, model: String, id: String }
                        ToolCall { id: String, name: String, arguments: String }
                        ToolDef { name: String, description: String, parameters: String }
                        Usage { prompt_tokens: Int, completion_tokens: Int, total_tokens: Int }
                        ChatChunk { delta_content: Option<String>, delta_tool_calls: Option<List<ToolCallDelta>>, finish_reason: Option<String> }
                        ToolCallDelta { index: Int, id: Option<String>, name: Option<String>, arguments: Option<String> }
                        Embedding { index: Int, embedding: List<Float> }
                        ProviderConfig { name: String, api_base: String, api_key: String, default_model: String }
                        CompletionParams { temperature: Option<Float>, top_p: Option<Float>, max_tokens: Option<Int>, stop: Option<List<String>>, seed: Option<Int> }
  prompt.ash          -- pure fn:
                        -- Constructors
                        system(content) -> Message
                        user(content) -> Message
                        assistant(content) -> Message
                        tool_result(call_id, content) -> Message
                        -- Inspectors
                        append_response(messages, response) -> List<Message>
                        has_tool_calls(response) -> Bool
                        is_final(response) -> Bool
                        get_tool_calls(response) -> List<ToolCall>
                        -- Renderers
                        render_conversation(messages) -> String
                        render_template(template, vars) -> String
  openai.ash          -- capability Llm declaration
  dispatch.ash        -- Dispatch workflows (Tier 3):
                        complete(provider, model, messages, params) -> ChatResponse
                        complete_with_tools(provider, model, messages, tools, params) -> ChatResponse
                        complete_tuned(provider, model, messages, params) -> ChatResponse
                        ask(provider, model, question) -> ChatResponse
                        stream(provider, model, messages, params) -> Stream<ChatChunk>
                        embed(provider, model, texts) -> List<Embedding>
                        list_models(provider) -> List<String>
  loading.ash         -- Loading workflows (Tier 2):
                        load_prompt(source) -> Message
                        load_system_prompt(name) -> Message
  conversation.ash    -- Orchestration: multi-turn conversation
  tool_agent.ash      -- Orchestration: tool-use agent loop
  router.ash          -- Orchestration: multi-model routing
  supervised.ash      -- Orchestration: supervised agent

crates/ash-engine/src/providers/llm/
  mod.rs              -- LlmProvider struct implementing CapabilityProvider
                        Multi-provider registry (HashMap<String, LlmConfig>)
                        Re-exports of submodules
  chat.rs             -- Chat completion logic using async-openai
                        chat() -> ChatResponse
                        chat_with_tools() -> ChatResponse (with tool definitions)
                        chat_stream() -> impl Stream<ChatChunk>
  config.rs           -- LlmConfig struct
                        LlmProvider::new(configs: HashMap<String, LlmConfig>)
                        Provider lookup by name
  embeddings.rs       -- Embedding logic using async-openai
                        embed() -> List<Embedding>
  models.rs           -- Model listing logic using async-openai
                        list_models(provider: String) -> List<String>
  stream_adapter.rs   -- Adapter: async-openai Stream -> Ash Stream<ChatChunk>
                        Handles chunk parsing, delta accumulation, done detection
  tool_dispatch.rs    -- Tool call parsing from response
                        Format tool results into provider-expected shape
```

## Tier Classification Summary

| Module | Tier | Construct | Rationale |
|--------|------|-----------|-----------|
| `llm/types.ash` | 1 (fn) | `fn` definitions | Pure data types and constructors |
| `llm/prompt.ash` | 1 (fn) | `fn` definitions | Pure message construction, formatting, rendering |
| `llm/loading.ash` | 2 (workflow) | `workflow` | IO: file reads, cache checks, env vars |
| `llm/dispatch.ash` | 3 (workflow) | `workflow` | Uses `act` on Llm capability |
| `llm/conversation.ash` | 3 (workflow) | `workflow` | Uses `act`, `receive` |
| `llm/tool_agent.ash` | 3 (workflow) | `workflow` | Uses `act`, tool dispatch |
| `llm/router.ash` | 3 (workflow) | `workflow` | Uses `act` for classification and routing |
| `llm/supervised.ash` | 3 (workflow) | `workflow` | Uses `act`, `spawn`, `kill`, `receive` |
| `providers/llm/*.rs` | N/A | Rust | CapabilityProvider implementation |

## Impact on Existing Specs

| Spec/Design | Impact | Component Areas |
|-------------|--------|-----------------|
| DESIGN-020 (Three-Vertex) | Validates the model; LLM stdlib is a worked example of fn/cap/workflow separation | Stdlib |
| DESIGN-015 (Unified Action) | New `LlmProvider` implements `CapabilityProvider` | Engine |
| PLAN-015 (Provider Migration) | Adds another provider to the unified system | Engine |
| SPEC-024 (Capability Role Reduction) | Llm capability follows the role reduction pattern | Language |
| SPEC-002 (Surface) | New capability declaration syntax in .ash files | Parser |
| SPEC-009 (Module System) | New module hierarchy: `llm/` with flat file layout | Stdlib |
| Cargo.toml | New dependency: `async-openai` crate | Build |

## Open Questions

1. **Should `with_llm_capabilities()` be a dedicated builder method or just use the existing
   `with_custom_provider()`?**
   - **Status:** RESOLVED — this phase treats `with_llm_capabilities()` as part of the contract.
   - The existing `with_custom_provider("llm", Arc::new(provider))` mechanism remains the general
     escape hatch, but the dedicated builder method is the documented phase-level convenience for
     registering the multi-provider `LlmProvider` from `HashMap<String, LlmConfig>`.

2. **Should streaming return `Stream<ChatChunk>` or a workflow that yields chunks?**
   - **Status:** OPEN
   - `Stream<ChatChunk>` is the simpler model and matches `async-openai`'s return type directly.
   A workflow that yields chunks via `send` would integrate better with Ash's actor model but
   adds complexity. Start with `Stream<ChatChunk>`, revisit if actor integration is needed.

3. **Where does tool execution happen in the tool-use loop?**
   - **Status:** RESOLVED — Phase 77 uses statically named dispatch helpers, not runtime lookup by string.
   - Ash's current action model lowers `act` targets to named `provider:action` pairs, and the
     typechecker validates explicit providers ahead of execution. In this phase, `tool_agent`
     therefore delegates tool execution to a companion workflow/helper that matches tool call names
     against a statically known set of tool branches and then invokes explicit `act` targets.
     Arbitrary runtime `act`-by-string dispatch is out of scope for Phase 77. If no branch matches,
     the workflow appends an error tool result and continues the loop. The `agent/` layer may later
     grow richer dispatch abstractions once the language/runtime support them directly.

4. **Should `ProviderConfig` be in `types.ash` or a separate config module?**
   - **Status:** RESOLVED -- `ProviderConfig` in `types.ash`
   - It's a pure data type used across providers. `types.ash` is the right home.

## Known Limitations

The following `LlmConfig` fields are defined but **not yet wired through** to the
`async-openai` client in Phase 77:

- `default_model` -- the model parameter must always be provided explicitly in dispatch
  workflows; the config-level default is not applied as a fallback.
- `timeout_ms` -- the request timeout uses `async-openai`'s default, not the configured value.
- `max_retries` -- retry on transient failures is not implemented; the provider fails immediately
  on 5xx or connection errors.

These will be addressed in a future phase when the provider wiring is enhanced.

## References

- DESIGN-020: Pure Functions and the Three-Vertex Model
- DESIGN-015: Unified Action System
- PLAN-015: Unified Action System (implementation plan)
- SPEC-024: Capability Role Reduction
- `crates/ash-engine/src/providers/mcp.rs` -- direct precedent for Rust-side provider
- `async-openai` crate -- OpenAI-compatible Rust client
