# SPEC-029: LLM Standard Library

## Status: Draft
## Date: 2026-04-12
## Version: 0.1

---

## 1. Overview

This specification defines the normative behavior of the Ash LLM standard library: a
provider-agnostic vocabulary of pure types and prompt functions, an OpenAI-specific
capability contract and dispatch layer, loading workflows for prompt sources, and agent
orchestration patterns. All constructs follow the three-vertex model (DESIGN-020, SPEC-027):
pure `fn` for data types and prompt construction, capabilities for effect contracts, and
`workflow` for effectful loading, LLM dispatch, and agent loops.

This spec defines **what is correct**, not how to implement it.

### 1.1 References

| Document | Relationship |
|----------|-------------|
| DESIGN-025 | Architectural context for this spec |
| DESIGN-020 | Three-vertex model (fn / cap / workflow) |
| DESIGN-015 | Unified action system |
| SPEC-027 | Pure functions (`fn`) |
| SPEC-028 | Function constraint system |
| SPEC-024 | Capability role reduction |
| SPEC-020 | Algebraic data types |
| SPEC-009 | Module system |
| SPEC-013 | Streams |

### 1.2 Tier Classification

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
| Rust `providers/llm/*.rs` | N/A | Rust | CapabilityProvider implementation |

### 1.3 Composition Rules

Per DESIGN-020, the following composition constraints hold:

- `fn` -> `fn` (freely) -- prompt construction chains pure functions
- `workflow` -> `fn` (freely) -- dispatch workflows call prompt.ash fns
- `workflow` -> `cap` (freely) -- dispatch workflows use `act` on Llm
- `fn` -X-> `workflow` (forbidden) -- functions never invoke workflows
- `fn` -X-> `cap` (forbidden) -- functions never use capabilities

---

## 2. Module Structure

### 2.1 Namespace Layout

```
std/src/
  llm/                  -- protocol module: LLM shared vocabulary
    mod.ash               -- module root, re-exports from types and prompt
    types.ash             -- pure fn: data type definitions
    prompt.ash            -- pure fn: constructors, inspectors, renderers
    openai.ash            -- capability declaration (Llm)
    dispatch.ash          -- dispatch workflows (complete, stream, embed, etc.)
    loading.ash           -- loading workflows (load_prompt, load_system_prompt)
    conversation.ash      -- orchestration: multi-turn conversation
    tool_agent.ash        -- orchestration: tool-use agent loop
    router.ash            -- orchestration: multi-model routing
    supervised.ash        -- orchestration: supervised agent
```

### 2.2 Module Grammar

```
llm-module       ::= llm-root | types-module | prompt-module
llm-root         ::= "mod" "{" use-stmt* re-export* "}"
types-module      ::= fn-def*
prompt-module     ::= fn-def*
openai-module     ::= capability-def
dispatch-module   ::= workflow-def*
loading-module    ::= workflow-def*
conversation-module ::= workflow-def*
tool-agent-module ::= workflow-def*
router-module     ::= workflow-def*
supervised-module ::= workflow-def*
```

### 2.3 Re-exports

`llm/mod.ash` re-exports all public types and functions from `types.ash` and `prompt.ash` so that
`use llm::{Message, user, render_conversation}` is valid.

### 2.4 Namespace Future-Proofing

The namespace is designed to accommodate future growth without restructuring:

```
std/src/llm/              -- shared vocab usable by any LLM provider
std/src/llm/openai.ash    -- OpenAI-specific capability declaration
std/src/llm/dispatch.ash  -- dispatch workflows wrapping act calls
std/src/llm/loading.ash   -- loading workflows for prompt sources
std/src/llm/conversation.ash -- orchestration: multi-turn conversation
std/src/llm/tool_agent.ash   -- orchestration: tool-use agent loop
std/src/llm/router.ash       -- orchestration: multi-model routing
std/src/llm/supervised.ash   -- orchestration: supervised agent
std/src/mcp/               -- peer: MCP protocol
std/src/a2a/               -- peer: A2A protocol
std/src/agent/             -- composes across protocols
```

Invariants:

- `llm/` types and prompt functions are provider-agnostic.
- `llm/openai.ash` declares the `Llm` capability. Future provider-specific files (e.g.,
  `llm/anthropic.ash`) would declare their own capabilities.
- Agent orchestration patterns are each in their own file for modularity.
- `agent/` sits above all protocols and composes across them.

---

## 3. Types

All types in this section are defined in `llm/types.ash` as pure `fn` definitions (Tier 1).
They are provider-agnostic and contain no effectful constructs.

### 3.1 Role

```ash
type Role = System | User | Assistant | Tool
```

Semantics: identifies the speaker of a message in a conversation.

| Variant | Meaning |
|---------|---------|
| `System` | Provider-level instruction |
| `User` | End-user input |
| `Assistant` | Model-generated response |
| `Tool` | Result of a tool execution |

### 3.2 Message

```ash
type Message {
    role: Role,
    content: String,
    tool_calls: Option<List<ToolCall>>,
    tool_call_id: Option<String>
}
```

Semantics: a single conversational turn.

| Field | Type | Constraints |
|-------|------|-------------|
| `role` | `Role` | Must be a valid `Role` variant |
| `content` | `String` | May be empty for tool-call-only assistant messages |
| `tool_calls` | `Option<List<ToolCall>>` | `Some` only when `role = Assistant`; `None` otherwise |
| `tool_call_id` | `Option<String>` | `Some` only when `role = Tool`; `None` otherwise |

Invariants:

- (I1) If `role = Assistant` and `tool_calls = Some(_)`, then `content` may be empty.
- (I2) If `role = Tool`, then `tool_call_id` must be `Some(_)`.
- (I3) If `role = System | User`, then `tool_calls = None` and `tool_call_id = None`.

### 3.3 ChatResponse

```ash
type ChatResponse {
    content: Option<String>,
    tool_calls: Option<List<ToolCall>>,
    finish_reason: Option<String>,
    usage: Option<Usage>,
    model: String,
    id: String
}
```

Semantics: the result of a chat completion request.

| Field | Type | Meaning |
|-------|------|---------|
| `content` | `Option<String>` | Generated text; `None` when model produces only tool calls |
| `tool_calls` | `Option<List<ToolCall>>` | Tool invocations requested by the model |
| `finish_reason` | `Option<String>` | Why generation stopped (`"stop"`, `"tool_calls"`, `"length"`, etc.) |
| `usage` | `Option<Usage>` | Token usage statistics |
| `model` | `String` | Model identifier that produced the response |
| `id` | `String` | Unique response identifier assigned by the provider |

### 3.4 ToolCall

```ash
type ToolCall {
    id: String,
    name: String,
    arguments: String
}
```

Semantics: a single tool invocation requested by the model.

| Field | Type | Meaning |
|-------|------|---------|
| `id` | `String` | Unique identifier for this tool call; used to match tool results |
| `name` | `String` | Name of the tool to invoke; must match a `ToolDef.name` |
| `arguments` | `String` | JSON-encoded arguments for the tool invocation |

Invariants:

- (I4) `id` must be non-empty.
- (I5) `name` must be non-empty.
- (I6) `arguments` must be valid JSON if non-empty.

### 3.5 ToolDef

```ash
type ToolDef {
    name: String,
    description: String,
    parameters: String
}
```

Semantics: a tool definition provided to the model for function calling.

| Field | Type | Meaning |
|-------|------|---------|
| `name` | `String` | Unique tool identifier |
| `description` | `String` | Human-readable description of the tool |
| `parameters` | `String` | JSON Schema describing the tool's parameters |

Invariants:

- (I7) `name` must be non-empty.
- (I8) `parameters` must be a valid JSON Schema object.

### 3.6 Usage

```ash
type Usage {
    prompt_tokens: Int,
    completion_tokens: Int,
    total_tokens: Int
}
```

Semantics: token consumption statistics.

Invariant:

- (I9) `total_tokens = prompt_tokens + completion_tokens`

### 3.7 ChatChunk

```ash
type ChatChunk {
    delta_content: Option<String>,
    delta_tool_calls: Option<List<ToolCallDelta>>,
    finish_reason: Option<String>
}
```

Semantics: a single incremental chunk from a streaming response.

| Field | Type | Meaning |
|-------|------|---------|
| `delta_content` | `Option<String>` | Incremental text fragment |
| `delta_tool_calls` | `Option<List<ToolCallDelta>>` | Incremental tool call fragments |
| `finish_reason` | `Option<String>` | Set only on the final chunk |

### 3.8 ToolCallDelta

```ash
type ToolCallDelta {
    index: Int,
    id: Option<String>,
    name: Option<String>,
    arguments: Option<String>
}
```

Semantics: an incremental fragment of a tool call within a streaming response.

| Field | Type | Meaning |
|-------|------|---------|
| `index` | `Int` | Index of the tool call being built; >= 0 |
| `id` | `Option<String>` | Tool call ID; present only on first chunk for this index |
| `name` | `Option<String>` | Tool name; present only on first chunk for this index |
| `arguments` | `Option<String>` | Incremental argument fragment |

Invariant:

- (I10) `index >= 0`

### 3.9 Embedding

```ash
type Embedding {
    index: Int,
    embedding: List<Float>
}
```

Semantics: a vector embedding for a single input text.

| Field | Type | Meaning |
|-------|------|---------|
| `index` | `Int` | Position of the corresponding input text in the request; >= 0 |
| `embedding` | `List<Float>` | The embedding vector; length determined by the model |

### 3.10 ProviderConfig

```ash
type ProviderConfig {
    name: String,
    api_base: String,
    api_key: String,
    default_model: String
}
```

Semantics: configuration for a named LLM provider endpoint.

| Field | Type | Meaning |
|-------|------|---------|
| `name` | `String` | Provider name used as the routing key in dispatch workflows |
| `api_base` | `String` | Base URL for the OpenAI-compatible API |
| `api_key` | `String` | Authentication key; may be a placeholder for local providers |
| `default_model` | `String` | Model used when none specified |

Invariant:

- (I11) `name` must be non-empty.
- (I12) `api_base` must be a valid URL.

### 3.11 CompletionParams

```ash
type CompletionParams {
    temperature: Option<Float>,
    top_p: Option<Float>,
    max_tokens: Option<Int>,
    stop: Option<List<String>>,
    seed: Option<Int>
}
```

Semantics: optional tuning parameters for a completion request.

| Field | Type | Constraints |
|-------|------|-------------|
| `temperature` | `Option<Float>` | If `Some`, must be >= 0.0 |
| `top_p` | `Option<Float>` | If `Some`, must be in (0.0, 1.0] |
| `max_tokens` | `Option<Int>` | If `Some`, must be > 0 |
| `stop` | `Option<List<String>>` | If `Some`, must contain 1..=4 strings |
| `seed` | `Option<Int>` | Deterministic sampling seed |

---

## 4. Pure Functions (prompt.ash)

All functions in this section are pure (`fn`, Tier 1). They contain no `act`, no `ret`, no
workflow constructs. They compose freely with each other and with any other pure `fn`.

### 4.1 Constructors

#### 4.1.1 system

```ash
fn system(content: String) -> Message
    requires: string::length(content) >= 0
    ensures: result.role == System
             && result.tool_calls == None
             && result.tool_call_id == None
{
    Message { role: System, content: content, tool_calls: None, tool_call_id: None }
}
```

Precondition: none (content may be empty).
Postcondition: returns a `Message` with `role = System`, `content` set, no tool fields.

#### 4.1.2 user

```ash
fn user(content: String) -> Message
    ensures: result.role == User
             && result.tool_calls == None
             && result.tool_call_id == None
{
    Message { role: User, content: content, tool_calls: None, tool_call_id: None }
}
```

Precondition: none.
Postcondition: returns a `Message` with `role = User`, `content` set, no tool fields.

#### 4.1.3 assistant

```ash
fn assistant(content: String) -> Message
    ensures: result.role == Assistant
             && result.tool_calls == None
             && result.tool_call_id == None
{
    Message { role: Assistant, content: content, tool_calls: None, tool_call_id: None }
}
```

Precondition: none.
Postcondition: returns a `Message` with `role = Assistant`, `content` set. The returned
message represents a text-only assistant turn with no tool calls.

#### 4.1.4 tool_result

```ash
fn tool_result(call_id: String, content: String) -> Message
    requires: string::length(call_id) > 0
    ensures: result.role == Tool
             && result.tool_call_id == Some(call_id)
             && result.tool_calls == None
{
    Message { role: Tool, content: content, tool_calls: None, tool_call_id: Some(call_id) }
}
```

Precondition: `call_id` must be non-empty.
Postcondition: returns a `Message` with `role = Tool`, `content` set, `tool_call_id = Some(call_id)`.

### 4.2 Inspectors

#### 4.2.1 append_response

```ash
fn append_response(messages: List<Message>, response: ChatResponse) -> List<Message>
    ensures: result == messages ++ [assistant_msg]
```

where `assistant_msg` is constructed from the response:

- If `response.content = Some(text)` and `response.tool_calls = Some(calls)`: the appended
  message has `role = Assistant`, `content = text`, `tool_calls = Some(calls)`.
- If `response.content = Some(text)` and `response.tool_calls = None`: the appended message
  has `role = Assistant`, `content = text`, `tool_calls = None`.
- If `response.content = None` and `response.tool_calls = Some(calls)`: the appended message
  has `role = Assistant`, `content = ""`, `tool_calls = Some(calls)`.

Precondition: `response` is a valid `ChatResponse`.
Postcondition: returns `messages` with one additional `Message` appended.

#### 4.2.2 append_tool_result

```ash
fn append_tool_result(messages: List<Message>, call_id: String, content: String) -> List<Message
    requires: string::length(call_id) > 0
    ensures: result == messages ++ [tool_result(call_id, content)]
{
    messages ++ [tool_result(call_id, content)]
}
```

Precondition: `call_id` must be non-empty.
Postcondition: returns `messages` with a single tool-result message appended.

#### 4.2.3 has_tool_calls

```ash
fn has_tool_calls(response: ChatResponse) -> Bool
    ensures: result == (response.tool_calls != None)
{
    match response.tool_calls {
        Some(_) => true,
        None => false
    }
}
```

Precondition: none.
Postcondition: `true` if and only if `response.tool_calls` is `Some`.

#### 4.2.4 is_final

```ash
fn is_final(response: ChatResponse) -> Bool
    ensures: result == (response.finish_reason == Some("stop")
                        || response.finish_reason == Some("length"))
{
    match response.finish_reason {
        Some(reason) => reason == "stop" || reason == "length",
        None => false
    }
}
```

Precondition: none.
Postcondition: `true` if and only if `finish_reason` is `"stop"` or `"length"`. A response
with `finish_reason = "tool_calls"` is NOT final -- the agent loop must continue executing
tools and feeding results back.

#### 4.2.5 get_tool_calls

```ash
fn get_tool_calls(response: ChatResponse) -> List<ToolCall>
    ensures: has_tool_calls(response) == true IMPLIES result != []
{
    match response.tool_calls {
        Some(calls) => calls,
        None => []
    }
}
```

Precondition: none.
Postcondition: returns the list of `ToolCall` values if present, empty list otherwise.

### 4.3 Renderers

#### 4.3.1 render_conversation

```ash
fn render_conversation(messages: List<Message>) -> String
```

Precondition: none.
Postcondition: returns a human-readable string representation of the conversation. Each
message is rendered on a separate line (or block) with the role as a prefix. The exact
formatting is unspecified but must satisfy:

- (R1) Each message in `messages` contributes a non-empty text fragment.
- (R2) Role prefixes are uppercase (`SYSTEM:`, `USER:`, `ASSISTANT:`, `TOOL:`).
- (R3) Messages appear in the same order as the input list.
- (R4) The concatenation is deterministic: equal inputs produce equal outputs.

#### 4.3.2 render_template

```ash
fn render_template(template: String, vars: Map<String, String>) -> String
```

Precondition: `template` is a valid template string.
Postcondition: returns `template` with all occurrences of `{{key}}` replaced by `vars[key]`.
If a key in the template is not present in `vars`, the placeholder is left unreplaced.
Replacement is deterministic.

---

## 5. Capability Contract

### 5.1 Declaration

```ash
pub capability Llm: execute
    chat(provider: String, model: String, messages: List<Message>,
         params: Option<CompletionParams>) -> ChatResponse
  | execute
    chat_with_tools(provider: String, model: String, messages: List<Message>,
                    tools: List<ToolDef>,
                    params: Option<CompletionParams>) -> ChatResponse
  | execute
    chat_stream(provider: String, model: String, messages: List<Message>,
                params: Option<CompletionParams>) -> Stream<ChatChunk>
  | execute
    embed(provider: String, model: String, texts: List<String>) -> List<Embedding>
  | execute
    list_models(provider: String) -> List<String>;
```

**Internal action:** In addition to the public capability actions above, the Rust-side
`LlmProvider` exposes an internal `pull_stream_chunk` action used by the streaming
adapter to consume individual SSE events from the `async-openai` stream. This action
is not part of the public `Llm` capability surface; it is an implementation detail of
the `stream_adapter` module. The `chat_stream` execute action returns a `Stream<ChatChunk>`
that internally delegates to `pull_stream_chunk`.

**Final-chunk rule:** Per section 3.7, the `stream_adapter` drops `delta_content` and
`delta_tool_calls` on the final chunk (the one with `finish_reason` set). The final
chunk carries only the `finish_reason` field. This ensures consumers do not double-count
the terminal delta, which may be empty or a duplicate of the penultimate chunk in some
provider implementations.

### 5.2 Actions

#### 5.2.1 chat

| Property | Value |
|----------|-------|
| Role | `execute` |
| Effect level | Operational |
| Parameters | `provider: String`, `model: String`, `messages: List<Message>`, `params: Option<CompletionParams>` |
| Return | `ChatResponse` |

Semantics: send a chat completion request to the named provider using the specified model.
Returns a single `ChatResponse`.

Error conditions:
- Provider not found in the LLM provider's internal routing table: `CapabilityError::NotAvailable(format!("provider:{provider}"))`
- Model not available: `CapabilityError::NotAvailable("model_not_found".into())`
- Authentication failure: `CapabilityError::PermissionDenied("auth_error".into())`
- Rate limit exceeded: `CapabilityError::ExecutionFailed("rate_limited".into())`
- Network failure: `CapabilityError::ExecutionFailed("network_error".into())`
- Invalid request (e.g., empty messages): `CapabilityError::InvalidArgument("invalid_request".into())`

#### 5.2.2 chat_with_tools

| Property | Value |
|----------|-------|
| Role | `execute` |
| Effect level | Operational |
| Parameters | `provider: String`, `model: String`, `messages: List<Message>`, `tools: List<ToolDef>`, `params: Option<CompletionParams>` |
| Return | `ChatResponse` |

Semantics: send a chat completion request with tool definitions. Identical to `chat` except that
the `tools` list is converted to `async-openai`'s `ChatCompletionTool` format and included in
the request. The model may produce tool calls in the response.

Error conditions: same as `chat`.

Postcondition: if the response contains tool calls, `has_tool_calls(response) == true` and
`finish_reason == Some("tool_calls")`.

#### 5.2.3 chat_stream

| Property | Value |
|----------|-------|
| Role | `execute` |
| Effect level | Operational |
| Parameters | `provider: String`, `model: String`, `messages: List<Message>`, `params: Option<CompletionParams>` |
| Return | `Stream<ChatChunk>` |

Semantics: send a streaming chat completion request. Returns a `Stream<ChatChunk>` (SPEC-013)
that yields incremental chunks. The final chunk has `finish_reason = Some(reason)`.

Error conditions: same as `chat`, but may also fail mid-stream with a stream error.

Streaming contract:
- (S1) Chunks are yielded in order.
- (S2) Exactly one final chunk has `finish_reason = Some(_)`.
- (S3) `delta_content` and `delta_tool_calls` are `None` on the final chunk.
- (S4) Concatenating all `delta_content` values in order produces the same text as a
  non-streaming `chat` call with the same parameters.

#### 5.2.4 embed

| Property | Value |
|----------|-------|
| Role | `execute` |
| Effect level | Operational |
| Parameters | `provider: String`, `model: String`, `texts: List<String>` |
| Return | `List<Embedding>` |

Semantics: compute vector embeddings for the input texts.

Error conditions: same as `chat`, plus:
- Empty input list: `CapabilityError::InvalidArgument("invalid_request".into())`

Postcondition:
- (E1) `result.length == texts.length`
- (E2) `result[i].index == i` for all `i` in `0..texts.length`

#### 5.2.5 list_models

| Property | Value |
|----------|-------|
| Role | `execute` |
| Effect level | Operational |
| Parameters | `provider: String` |
| Return | `List<String>` |

Semantics: list model identifiers available on the named provider.

Error conditions:
- Provider not found in the LLM provider's internal routing table: `CapabilityError::NotAvailable(format!("provider:{provider}"))`
- Network failure: `CapabilityError::ExecutionFailed("network_error".into())`

---

## 6. Dispatch Workflows

All dispatch workflows are defined in `llm/dispatch.ash` as `workflow` definitions (Tier 3).
Each is a thin wrapper around a statically named `act llm:action(...)` target. Phase 77 does not
require or assume runtime capability/action lookup from arbitrary strings.

### 6.1 complete

```ash
workflow complete(provider: String, model: String,
                  messages: List<Message>,
                  params: Option<CompletionParams>) -> ChatResponse {
    act llm:chat(provider, model, messages, params)
}
```

Semantics: single non-streaming chat completion. Delegates directly to `llm:chat`.

### 6.2 complete_with_tools

```ash
workflow complete_with_tools(provider: String, model: String,
                             messages: List<Message>,
                             tools: List<ToolDef>,
                             params: Option<CompletionParams>) -> ChatResponse {
    act llm:chat_with_tools(provider, model, messages, tools, params)
}
```

Semantics: chat completion with tool definitions. The `tools` list is passed as a separate
parameter to the `chat_with_tools` capability action. The Rust provider converts `ToolDef`
values into `async-openai`'s `ChatCompletionTool` wire format. This keeps `CompletionParams`
as purely tuning parameters.

Postcondition: if the response contains tool calls, `has_tool_calls(response) == true`
and `finish_reason == Some("tool_calls")`.

### 6.3 complete_tuned

```ash
workflow complete_tuned(provider: String, model: String,
                        messages: List<Message>,
                        params: CompletionParams) -> ChatResponse {
    act llm:chat(provider, model, messages, Some(params))
}
```

Semantics: chat completion with explicit tuning parameters (temperature, top_p, max_tokens,
etc.). Unlike `complete`, which takes `Option<CompletionParams>`, this workflow requires
a non-optional `CompletionParams`, ensuring the caller explicitly sets tuning.

Precondition: `params` must satisfy the constraints in section 3.11.

### 6.4 ask

```ash
workflow ask(provider: String, model: String, question: String) -> ChatResponse {
    let messages = [user(question)];
    act llm:chat(provider, model, messages, None)
}
```

Semantics: convenience single-turn completion. Constructs a single-element message list
from the question string and dispatches to `llm:chat`.

### 6.5 stream

```ash
workflow stream(provider: String, model: String,
                messages: List<Message>,
                params: Option<CompletionParams>) -> Stream<ChatChunk> {
    act llm:chat_stream(provider, model, messages, params)
}
```

Semantics: streaming chat completion. Delegates directly to `llm:chat_stream`.

### 6.6 embed

```ash
workflow embed(provider: String, model: String, texts: List<String>) -> List<Embedding> {
    act llm:embed(provider, model, texts)
}
```

Semantics: text embedding. Delegates directly to `llm:embed`.

### 6.7 list_models

```ash
workflow list_models(provider: String) -> List<String> {
    act llm:list_models(provider)
}
```

Semantics: list available models. Delegates directly to `llm:list_models`.

---

## 7. Loading Workflows

Loading workflows are defined in `llm/loading.ash` as `workflow` definitions (Tier 2).
They perform IO (file reads, cache checks, environment variable lookups) but do not dispatch
to the LLM capability.

**Note:** The `env:VAR` and `cache:key` source forms require stdlib modules (`std::env`,
`std::cache`) that are not yet available. In Phase 77, loading.ash supports file-based and
literal-string loading only. Environment variable and cache integration is deferred to a
future phase. See `std/src/llm/loading.ash` for the current implementation.

### 7.1 load_prompt

```ash
workflow load_prompt(source: String) -> Message
```

Semantics: load a prompt from a source identifier. The `source` string determines the
loading strategy:

| Source form | Strategy |
|-------------|----------|
| `file:path` | Read the file at `path`, return `system(content)` |
| `env:VAR` | Read environment variable `VAR`, return `system(content)` |
| `cache:key` | Look up a previously cached prompt by key |
| Other | Treat as a literal string, return `system(source)` |

Error conditions:
- File not found: runtime failure
- Environment variable unset: runtime failure
- Cache miss: runtime failure

### 7.2 load_system_prompt

```ash
workflow load_system_prompt(name: String) -> Message
```

Semantics: load a named system prompt from the configured prompt directory. The prompt
directory is resolved from the engine configuration. The `name` parameter maps to a file
name within that directory.

Postcondition: `result.role == System`.

Error conditions:
- Named prompt not found: runtime failure
- Prompt directory not configured: runtime failure
- IO error reading the file: runtime failure

---

## 8. Agent Workflows

Agent workflows are defined in separate files under `llm/` as `workflow` definitions (Tier 3).
They use `act` (via dispatch workflows), `spawn`, `kill`, `check_health`, and `receive`.

| Workflow | File |
|----------|------|
| `conversation` | `llm/conversation.ash` |
| `tool_agent` | `llm/tool_agent.ash` |
| `router` | `llm/router.ash` |
| `supervised_agent` | `llm/supervised.ash` |

### 8.1 conversation

```ash
workflow conversation(provider: String, model: String,
                      system_prompt: String,
                      max_turns: Int) -> List<Message>
    requires: max_turns > 0
```

Semantics: maintain a multi-turn conversation with the model. The loop proceeds as follows:

1. Initialize `messages = [system(system_prompt)]`.
2. `receive` a user message; append it to `messages`.
3. Call `complete(provider, model, messages, None)`.
4. Append the response to `messages`.
5. If the response is final or `turns >= max_turns`, return `messages`.
6. Otherwise, go to step 2.

Termination conditions:
- `max_turns` reached: return the accumulated messages.
- Final response received: return the accumulated messages.
- User sends a termination signal: return the accumulated messages.

Error handling: on any `act` failure, the workflow returns the messages accumulated so far.

### 8.2 tool_agent

```ash
workflow tool_agent(provider: String, model: String,
                    messages: List<Message>,
                    tools: List<ToolDef>,
                    max_rounds: Int) -> ChatResponse
    requires: max_rounds > 0
```

Semantics: orient-decide-act tool-use loop. The loop proceeds as follows:

1. Set `rounds = 0`.
2. Call `complete_with_tools(provider, model, messages, tools, None)`.
3. If `is_final(response)`: return `response`.
4. If `has_tool_calls(response)`: extract tool calls via `get_tool_calls(response)`.
5. Execute tool calls through a statically declared dispatcher workflow/helper.
6. `messages = append_response(messages, response)`.
7. For each tool call result: `messages = append_tool_result(messages, call_id, result)`.
8. `rounds = rounds + 1`.
9. If `rounds >= max_rounds`: return `response`.
10. Go to step 2.

Tool dispatch contract for Phase 77:
- The dispatcher helper MUST branch on statically known tool names and lower each supported branch
  to an explicit named `act` target.
- The dispatcher helper MUST NOT attempt runtime `act`-by-string lookup from `ToolCall.name`.
- If a tool call name has no matching branch, the workflow appends an error tool result for that
  call and continues the loop.

Termination conditions:
- `is_final(response) == true`: model stopped without requesting tools.
- `max_rounds` reached: return the last response even if it contains unexecuted tool calls.
- LLM dispatch failure: return the last response.

Error handling: on `complete_with_tools` / LLM `act` failure, return the last successfully obtained
`ChatResponse`. On tool execution failure within a matched static branch, append a tool result with
an error message and continue the loop.

### 8.3 router

```ash
workflow router(provider: String, messages: List<Message>) -> ChatResponse
```

Semantics: classify task complexity from the conversation and route to an appropriate model.
The classification is itself performed via an LLM call.

Behavior:
1. Render a classification prompt from `messages`.
2. Use the Phase 77 fixed classifier model `"gpt-4o-mini"`.
3. Call `ask(provider, "gpt-4o-mini", classification_prompt)` so the single prompt is wrapped as
   the required `List<Message>` for the underlying `chat`/`complete` dispatch.
4. Parse the response to determine complexity: `Simple | Moderate | Complex`.
5. Map complexity to a model identifier.
6. Call `complete(provider, selected_model, messages, None)`.
7. Return the response.

Error handling: if classification fails, default to a moderate-complexity model.

### 8.4 supervised_agent

```ash
workflow supervised_agent(config: AgentConfig) -> Result<ChatResponse, AgentError>
```

Where:

```ash
type AgentConfig {
    provider: String,
    model: String,
    messages: List<Message>,
    tools: List<ToolDef>,
    max_rounds: Int,
    max_restarts: Int
}

type AgentError {
    max_restarts_exceeded: Int,
    last_response: Option<ChatResponse>
}
```

Semantics: spawn a `tool_agent` as a supervised child process with health monitoring and
restart semantics.

Behavior:
1. `spawn tool_agent(config.provider, config.model, config.messages, config.tools, config.max_rounds)`.
2. Periodically `check_health(handle)`.
3. On `Healthy`: continue monitoring.
4. The child `tool_agent` sends its terminal `ChatResponse` to the supervisor's mailbox before it exits.
5. The supervisor `receive`s that mailbox payload and returns `Ok(received_response)`.
6. On `Failed`: `kill handle`; if `restarts < max_restarts`, respawn; otherwise, return error.

Termination conditions:
- Child completes normally: return `Ok` with the `ChatResponse` payload received from the child's mailbox message.
- `max_restarts` exceeded: return `Err(AgentError { max_restarts_exceeded: restarts, last_response })`.

---

## 9. Rust Provider Contract

This section defines what the Rust-side `LlmProvider` must satisfy to be a conforming
implementation of the `Llm` capability declared in section 5.

### 9.1 Trait Implementation

The `LlmProvider` must implement `ash_core::capability::CapabilityProvider`:

```rust
impl CapabilityProvider for LlmProvider {
    fn name(&self) -> &str;
    fn effect(&self) -> Effect;
    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>;
    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError>;
}
```

### 9.2 Actions

The provider must handle these dispatch entry points.

For `execute(action_name, args)`, `args` is the already-evaluated positional argument slice produced
by the `act` AST/workflow call site. The provider does not receive a record/object payload such as
`{ provider, model, ... }`; instead it receives the values in the same semantic order used by the
workflow signature.

| Provider Method | Name | Positional input shape | Return |
|-----------------|------|------------------------|--------|
| `execute` | `"chat"` | `args = [provider, model, messages, params]` | `ChatResponse` as Value |
| `execute` | `"chat_with_tools"` | `args = [provider, model, messages, tools, params]` | `ChatResponse` as Value |
| `execute` | `"chat_stream"` | `args = [provider, model, messages, params]` | `Stream<ChatChunk>` as Value |
| `execute` | `"embed"` | `args = [provider, model, texts]` | `List<Embedding>` as Value |
| `execute` | `"list_models"` | `args = [provider]` | `List<String>` as Value |

Semantic argument order:

- `provider`: routing key into the provider's internal `HashMap<String, LlmConfig>`.
- `model`: model identifier string for the selected backend.
- `messages`: `List<Message>` conversation transcript.
- `tools`: `List<ToolDef>` when tool calling is enabled.
- `params`: optional completion-tuning value matching `CompletionParams`.
- `texts`: `List<String>` input batch for embeddings.

### 9.3 Provider Routing by Name

The provider holds an internal registry mapping provider names to `LlmConfig`:

```rust
pub struct LlmConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}
```

When an action is received, the first semantic provider selector from the positional input is used
to look up the corresponding `LlmConfig`. If the name is not found, the provider returns
`CapabilityError::NotAvailable(format!("provider:{provider}"))`.

### 9.4 Error Mapping

Errors from the underlying HTTP client (`async-openai`) and provider-side routing checks must be
mapped to `CapabilityError`:

| Source Error | Mapped To |
|-------------|-----------|
| Provider name not in registry | `CapabilityError::NotAvailable(format!("provider:{name}"))` |
| Authentication failure (401/403) | `CapabilityError::PermissionDenied("auth_error".into())` |
| Model not found (404) | `CapabilityError::NotAvailable("model_not_found".into())` |
| Rate limited (429) | `CapabilityError::ExecutionFailed("rate_limited".into())` |
| Connection refused / timeout | `CapabilityError::ExecutionFailed("network_error".into())` |
| Invalid request (400) | `CapabilityError::InvalidArgument("invalid_request".into())` |
| Server error (5xx) | `CapabilityError::ExecutionFailed("server_error".into())` |

### 9.5 Streaming Contract

The `stream_adapter` module adapts `async-openai`'s streaming response into Ash's
`Stream<ChatChunk>`. It must satisfy:

- (SC1) Each incoming SSE event is mapped to exactly one `ChatChunk`.
- (SC2) Events with no delta content and no delta tool calls are dropped (keep-alive pings).
- (SC3) The final event (with `finish_reason`) produces a `ChatChunk` where `finish_reason` is set.
- (SC4) On stream error, the stream terminates with an error rather than silently stopping.
- (SC5) The stream does not buffer the entire response; chunks are yielded as they arrive.

### 9.6 Engine Registration

```rust
engine.with_custom_provider("llm", Arc::new(LlmProvider::new(configs)?))
```

The engine registration key for the capability provider is `"llm"`. Names such as `"openai"`,
`"ollama"`, or other endpoint labels are routing keys inside `configs` and are passed as the first
argument to the LLM actions; they are not alternate engine registration keys.

Or via a dedicated builder method:

```rust
engine.with_llm_capabilities(provider_configs)
```

where `provider_configs: HashMap<String, LlmConfig>` maps provider names to their
configurations.

### 9.7 Statelessness

The `LlmProvider` is stateless with respect to conversations. It receives a full message
list on each call and returns a response. Conversation state management belongs to Ash
workflows, not the provider.

---

## 10. Known Limitations

### 10.1 Config Fields Not Yet Wired

The following `LlmConfig` fields are defined in section 9.3 but **not yet wired through** to the
`async-openai` client in Phase 77:

- `default_model` -- the model parameter must always be provided explicitly in dispatch
  workflows; the config-level default is not applied as a fallback.
- `timeout_ms` -- the request timeout uses `async-openai`'s default, not the configured value.
- `max_retries` -- retry on transient failures is not implemented; the provider fails immediately
  on 5xx or connection errors.

### 10.2 Loading Workflow Dependencies

The `env:VAR` and `cache:key` source forms in `load_prompt` (section 7.1) and the
`config::get_string` call in `load_system_prompt` (section 7.2) depend on stdlib modules
(`std::env`, `std::cache`, `std::config`) that are not yet available. Phase 77 loading.ash
supports file-based and literal-string loading only. Environment variable and cache integration
is deferred to a future phase.

---

## 11. Conformance

### 11.1 Conforming Implementation

A conforming implementation of the LLM standard library must:

1. **Types (Section 3):** Provide all types defined in `llm/types.ash` with the exact field
   names, types, and invariants specified. Each type invariant (I1--I12) must hold for all
   values produced by constructors and returned from functions.

2. **Pure Functions (Section 4):** Implement all functions in `llm/prompt.ash` with the exact
   signatures, preconditions, and postconditions specified. Functions must be pure: no side
   effects, no capability use, deterministic for equal inputs.

3. **Capability Contract (Section 5):** Declare the `Llm` capability with all five actions
   (`chat`, `chat_with_tools`, `chat_stream`, `embed`, `list_models`) and their specified roles, effect levels,
   parameter types, and return types.

4. **Dispatch Workflows (Section 6):** Implement all seven dispatch workflows as `workflow`
   definitions that delegate to `act llm:action(...)`.

5. **Loading Workflows (Section 7):** Implement `load_prompt` and `load_system_prompt` as
   `workflow` definitions performing IO, not as `fn` definitions.

6. **Agent Workflows (Section 8):** Implement `conversation`, `tool_agent`, `router`, and
   `supervised_agent` with the specified loop semantics, termination conditions, and error
   handling behavior.

7. **Rust Provider (Section 9):** Provide an `LlmProvider` implementing
   `CapabilityProvider` that satisfies the action contract, provider routing, error mapping,
   and streaming contract.

### 11.2 Composition Conformance

A conforming implementation must respect the three-vertex composition rules:

- No `fn` definition in `types.ash` or `prompt.ash` contains `act`, `ret`, `spawn`, `kill`,
  `receive`, `send`, `check_health`, or any workflow construct.
- All `workflow` definitions in `llm/` that use `act` are classified as Tier 3.
- No `fn` invokes a `workflow` or uses a capability.

### 11.3 Optional Extensions

A conforming implementation may additionally provide:

- Additional provider-specific parameters beyond those in `CompletionParams`.
- Additional actions on the `Llm` capability (e.g., fine-tuning, batch).
- Additional agent orchestration patterns.
- Caching layers for loading workflows.

Extensions must not alter the semantics of any normative construct defined in this spec.
