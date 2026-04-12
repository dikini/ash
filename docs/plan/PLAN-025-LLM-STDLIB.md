# PLAN-025: LLM Standard Library

## Status: Draft

## Overview

Implement the LLM standard library for Ash as defined in DESIGN-025 and SPEC-029. This plan covers
a Rust-side `LlmProvider` backed by `async-openai`, an Ash-level pure type vocabulary and prompt
construction library, OpenAI-specific capability and dispatch workflows, agent orchestration
patterns, and integration tests. All constructs follow the three-vertex model: pure `fn` for types
and prompt construction, capabilities for effect contracts, and `workflow` for effectful loading,
LLM dispatch, and agent loops.

## Prerequisite

- PLAN-023 (Pure Functions Phase) must be complete so that `fn` definitions in `.ash` files are
  parseable, type-checkable, and executable via the pure runtime.
- The existing provider registration system (`with_custom_provider()` on `EngineBuilder`) must be
  stable, as `LlmProvider` follows the same `CapabilityProvider` trait pattern used by
  `McpProvider`, `FsProvider`, and `StdioProvider`.

## Design References

- [DESIGN-025: LLM Standard Library](../design/DESIGN-025-LLM-STDLIB.md)
- [SPEC-029: LLM Standard Library](../spec/SPEC-029-LLM-STDLIB.md)
- [DESIGN-020: Pure Functions and the Three-Vertex Model](../design/DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md)
- [DESIGN-015: Unified Action System](../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- `crates/ash-engine/src/providers/mcp.rs` -- Rust-side provider precedent
- `crates/ash-engine/src/lib.rs` -- Engine builder with `with_custom_provider()`

## Task Breakdown

### Track 1: Rust Provider Foundation

Rust-side `LlmProvider` that bridges `async-openai` into Ash's capability system, following the
`McpProvider` pattern in `crates/ash-engine/src/providers/mcp.rs`.

---

#### TASK-516: Add async-openai dependency to ash-engine/Cargo.toml

**Objective:** Add the `async-openai` crate as a dependency so that the LLM provider can use it
for OpenAI-compatible HTTP communication.

**Files to modify:**
- `crates/ash-engine/Cargo.toml`

**Changes:**
- Add `async-openai` dependency with appropriate version. This crate provides typed clients for
  the OpenAI chat completions, streaming, and embeddings endpoints.
- The dependency is added under `[dependencies]` alongside the existing `reqwest`, `serde`, and
  `serde_json` entries.

**Verification:**
```
cd crates/ash-engine && cargo check
```

---

#### TASK-517: Create LlmConfig struct

**Objective:** Define the `LlmConfig` struct that holds per-provider connection settings, with
validation and defaults.

**Files to create:**
- `crates/ash-engine/src/providers/llm/config.rs`

**Files to modify:**
- `crates/ash-engine/src/providers/llm/mod.rs` (re-export)

**Design (from DESIGN-025 D4, SPEC-029 §9.3):**

```rust
pub struct LlmConfig {
    pub api_base: String,       // e.g. "https://api.openai.com/v1"
    pub api_key: String,        // API key (or "dummy" for local providers)
    pub default_model: String,  // default model if none specified
    pub timeout_ms: u64,        // request timeout
    pub max_retries: u32,       // retry count for transient failures
}
```

**Implementation details:**
- `Default` impl: `api_base = "https://api.openai.com/v1"`, `api_key = ""`, `default_model = "gpt-4o"`,
  `timeout_ms = 30000`, `max_retries = 2`.
- Validation method `validate(&self) -> Result<(), String>`:
  - `api_base` must be a valid URL (parse with `url::Url` or basic scheme check).
  - `api_key` must not be empty (or explicitly allow empty for local providers with a flag).
- `Display` or `Debug` impl that redacts `api_key`.

**TDD steps:**
1. Write tests for `LlmConfig::default()` verifying all fields.
2. Write tests for `LlmConfig::validate()` accepting valid configs and rejecting invalid ones
   (empty api_base, invalid URL, empty api_key).
3. Write test for Debug redaction.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm::config
```

---

#### TASK-518: Create LlmProvider skeleton and list_models implementation

**Objective:** Create the `LlmProvider` struct implementing `CapabilityProvider`, with a
multi-provider registry, action dispatch, and a working `list_models` execute action.

**Files to create:**
- `crates/ash-engine/src/providers/llm/mod.rs`

**Files to modify:**
- `crates/ash-engine/src/providers/mod.rs` (add `pub mod llm;` and re-exports)
- `crates/ash-engine/src/lib.rs` (add `with_llm_capabilities` builder method)

**Design (from DESIGN-025 D4, SPEC-029 §9):**

```rust
pub struct LlmProvider {
    configs: HashMap<String, LlmConfig>,
    clients: tokio::sync::Mutex<HashMap<String, async_openai::Client<OpenAIConfig>>>,
}
```

- `LlmProvider::new(configs: HashMap<String, LlmConfig>) -> Result<Self, CapabilityError>`:
  validates each config, stores the registry.
- Lazy client creation: `get_client(&self, provider_name: &str)` looks up or creates an
  `async_openai::Client` per config on first use.

**CapabilityProvider implementation:**
- `name() -> "llm"` (engine registration key; the capability declared in Ash is `Llm`)
- Routing names such as `"openai"` or `"ollama"` stay inside the provider config map and are passed
  as action arguments, not as alternate registration keys.
- `effect() -> Effect::Operational` (read-only analysis, same as McpProvider)
- `observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>`:
  remains unused for the LLM action surface in Phase 77; returns `NotAvailable`.
- `execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError>`:
  dispatches on action names: `"chat"`, `"chat_with_tools"`, `"chat_stream"`, `"embed"`, `"list_models"`.
  The `list_models` action receives positional args `[provider]` and calls async-openai's model
  listing API to return available models.

**Engine builder method:**

```rust
impl EngineBuilder {
    pub fn with_llm_capabilities(
        mut self,
        configs: HashMap<String, LlmConfig>,
    ) -> Self {
        match LlmProvider::new(configs) {
            Ok(provider) => self.custom_providers.insert(
                "llm".to_string(),
                Arc::new(provider),
            ),
            Err(_) => { /* log warning, skip */ }
        };
        self
    }
}
```

**TDD steps:**
1. Test `LlmProvider::new()` with valid config map succeeds.
2. Test `LlmProvider::new()` with invalid config returns error.
3. Test `CapabilityProvider::name()` returns `"llm"`.
4. Test `CapabilityProvider::effect()` returns `Operational`.
5. Test `execute()` with unknown action returns `NotAvailable`.
6. Test `observe()` returns `NotAvailable` for the unused observe entry point.
7. Test engine builder: `Engine::new().with_llm_capabilities(configs).build()` succeeds.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm
cargo test -p ash-engine --lib tests::builder_llm
```

---

#### TASK-519: Implement chat completion

**Objective:** Implement the `"chat"` action that converts Ash `Value` arguments into
`async-openai` requests and converts responses back to Ash `Value`.

**Files to create:**
- `crates/ash-engine/src/providers/llm/chat.rs`

**Files to modify:**
- `crates/ash-engine/src/providers/llm/mod.rs` (wire `chat` action)

**Implementation details:**

1. **Message conversion:** `fn value_to_chat_message(value: &Value) -> Result<ChatCompletionRequestMessage, CapabilityError>`
   - Map Ash `Message { role, content, tool_calls, tool_call_id }` to `async-openai` types.
   - `Role::System` -> `ChatCompletionRequestMessage::System`, etc.
   - `Role::Tool` with `tool_call_id` -> `ChatCompletionRequestMessage::Tool`.

2. **Request construction:** Build `CreateChatCompletionRequest` from action args:
   - `provider` (String): lookup config, get/create client.
   - `model` (String): model identifier.
   - `messages` (List of Message values): convert each.
   - `params` (Option with CompletionParams fields): map temperature, top_p, max_tokens, stop, seed.

3. **Response conversion:** Convert `CreateChatCompletionResponse` to Ash `Value`:
   - Build a `ChatResponse`-shaped `Value::Record` with fields: `content`, `tool_calls`, `finish_reason`,
     `usage`, `model`, `id`.

4. **Error mapping** (from SPEC-029 §9.4):
   - 401/403 -> `PermissionDenied("auth_error".into())`
   - 404 -> `NotAvailable("model_not_found".into())`
   - 429 -> `ExecutionFailed("rate_limited".into())`
   - Connection errors -> `ExecutionFailed("network_error".into())`
   - 400 -> `InvalidArgument("invalid_request".into())`
   - 5xx -> `ExecutionFailed("server_error".into())`
   - Unknown provider name -> `NotAvailable(format!("provider:{name}"))` (from config lookup, before HTTP)

**TDD steps:**
1. Unit test: `value_to_chat_message` for each role variant.
2. Unit test: response-to-Value conversion produces correct field names and types.
3. Integration test with mock (using `wiremock` which is already in dev-dependencies):
   mock `/v1/chat/completions` endpoint, verify provider constructs correct request and returns
   correct Value.
4. Test error mapping: mock 401, 429, 404 responses and verify `CapabilityError` variants.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm::chat
```

---

#### TASK-520: Implement streaming adapter

**Objective:** Adapt `async-openai`'s `Stream<CreateChatCompletionStreamResponse>` into Ash's
stream model, with chunk parsing for delta content, tool call deltas, and finish reason.

**Files to create:**
- `crates/ash-engine/src/providers/llm/stream_adapter.rs`

**Files to modify:**
- `crates/ash-engine/src/providers/llm/mod.rs` (wire `chat_stream` action)

**Streaming contract (from SPEC-029 §9.5):**
- (SC1) Each incoming SSE event maps to exactly one `ChatChunk`.
- (SC2) Events with no delta content and no delta tool calls are dropped (keep-alive pings).
- (SC3) The final event (with `finish_reason`) produces a `ChatChunk` where `finish_reason` is set.
- (SC4) On stream error, the stream terminates with an error rather than silently stopping.
- (SC5) The stream does not buffer the entire response; chunks are yielded as they arrive.

**Implementation details:**
- Use `async-openai`'s streaming API which returns `Pin<Box<dyn Stream<Item = Result<...>>>>`.
- Convert each chunk to a `Value::Record` with fields: `delta_content`, `delta_tool_calls`,
  `finish_reason`.
- `delta_tool_calls` is a list of `ToolCallDelta` records with `index`, `id`, `name`, `arguments`.
- Return as `Value::Stream(...)` or the appropriate Ash stream representation.

**TDD steps:**
1. Unit test: single chunk with delta content converts correctly.
2. Unit test: chunk with tool call delta converts correctly.
3. Unit test: final chunk with finish_reason converts correctly.
4. Unit test: empty keep-alive chunk is filtered out.
5. Integration test with mock SSE stream: verify chunk order and content.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm::stream_adapter
```

---

#### TASK-521: Implement tool dispatch helpers

**Objective:** Parse tool calls from chat responses and format tool results for follow-up
requests, supporting the tool-use agent loop.

**Files to create:**
- `crates/ash-engine/src/providers/llm/tool_dispatch.rs`

**Implementation details:**

1. **Tool call parsing:** `fn extract_tool_calls(response: &Value) -> Result<Vec<ToolCallValue>, CapabilityError>`
   - Extract `tool_calls` field from `ChatResponse` Value.
   - Each tool call has: `id`, `name`, `arguments` (JSON string).

2. **Tool result formatting:** `fn format_tool_result_message(call_id: &str, content: &str) -> Value`
   - Build an Ash `Message` Value with `role: "tool"`, `tool_call_id: Some(call_id)`,
     `content: result_content`.

3. **Tool definitions to request:** `fn tool_defs_to_openai_tools(tools: &[Value]) -> Result<Vec<ChatCompletionTool>, CapabilityError>`
   - Convert Ash `ToolDef` values to `async-openai` `ChatCompletionTool` format.

**TDD steps:**
1. Test extracting tool calls from a response Value with `tool_calls` present.
2. Test extracting tool calls from a response Value with `tool_calls = None`.
3. Test formatting a tool result message produces correct Value shape.
4. Test converting `ToolDef` values to OpenAI tool format.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm::tool_dispatch
```

---

#### TASK-522: Implement embeddings

**Objective:** Implement the `"embed"` action for text embedding via `async-openai`.

**Files to create:**
- `crates/ash-engine/src/providers/llm/embeddings.rs`

**Files to modify:**
- `crates/ash-engine/src/providers/llm/mod.rs` (wire `embed` action)

**Implementation details:**
- Construct `CreateEmbeddingRequest` from action args: `provider`, `model`, `texts`.
- Convert `CreateEmbeddingResponse` to Ash `Value::List` of `Embedding` records with `index` and
  `embedding` (list of floats).
- Error mapping same as chat (SPEC-029 §9.4).
- Provider routing by name (lookup config, get/create client).

**Postconditions (from SPEC-029 §5.2.3):**
- (E1) `result.length == texts.length`
- (E2) `result[i].index == i` for all `i`

**TDD steps:**
1. Unit test: request construction from args.
2. Unit test: response conversion preserves index ordering.
3. Integration test with mock: verify end-to-end embedding returns correct structure.

**Verification:**
```
cargo test -p ash-engine --lib providers::llm::embeddings
```

---

#### TASK-523: Wire up engine builder method

**Objective:** Add `with_llm_capabilities()` to `EngineBuilder`, update module re-exports, and
add an integration test that the full engine lifecycle works with an LLM provider registered.

**Files to modify:**
- `crates/ash-engine/src/providers/mod.rs` -- add `pub mod llm;` and `pub use llm::{LlmConfig, LlmProvider};`
- `crates/ash-engine/src/lib.rs` -- add `with_llm_capabilities(HashMap<String, LlmConfig>) -> Self` method

**Implementation details:**
- `with_llm_capabilities` validates configs, creates `LlmProvider`, registers via
  `self.custom_providers.insert("llm".to_string(), Arc::new(provider))`.
- Returns `Self` for chaining, matching the `with_custom_provider` pattern.

**TDD steps:**
1. Test: `Engine::new().with_llm_capabilities(valid_configs).build()` succeeds.
2. Test: `Engine::new().with_llm_capabilities(invalid_configs).build()` handles gracefully.
3. Test: engine with LLM provider can parse and execute a workflow that uses `act llm:chat(...)`.
4. Test: multi-provider config (two entries) registers correctly.

**Verification:**
```
cargo test -p ash-engine --lib tests::builder_llm
cargo test -p ash-engine --lib tests::llm_integration
```

---

### Track 2: Ash Stdlib -- Pure Types and Functions

Depends on PLAN-023 (Pure Functions Phase) being complete so `fn` definitions are parseable and
executable. These are all Tier 1 (pure `fn`) modules -- no `act`, no `ret`, no workflow constructs.

---

#### TASK-524: Create std/src/llm/ module structure

**Objective:** Create the module root for the LLM stdlib with proper module declarations and
re-exports.

**Files to create:**
- `std/src/llm/mod.ash`

**Implementation details:**

Module declarations and re-exports:

```ash
-- Module root for the LLM standard library
-- Re-exports all public types and functions from submodules

module llm {
    use llm::types
    use llm::prompt
}
```

The module re-exports all public types and functions from `types.ash` and `prompt.ash` so that
`use llm::{Message, user, render_conversation}` is valid (SPEC-029 §2.3).

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_module
```

---

#### TASK-525: Create std/src/llm/types.ash

**Objective:** Define all LLM data types as pure type definitions, per SPEC-029 §3.

**Files to create:**
- `std/src/llm/types.ash`

**Types to define (from SPEC-029 §3.1--3.11):**

```
Role              -- System | User | Assistant | Tool
Message           -- { role, content, tool_calls, tool_call_id }
ChatResponse      -- { content, tool_calls, finish_reason, usage, model, id }
ToolCall          -- { id, name, arguments }
ToolDef           -- { name, description, parameters }
Usage             -- { prompt_tokens, completion_tokens, total_tokens }
ChatChunk         -- { delta_content, delta_tool_calls, finish_reason }
ToolCallDelta     -- { index, id, name, arguments }
Embedding         -- { index, embedding }
ProviderConfig    -- { name, api_base, api_key, default_model }
CompletionParams  -- { temperature, top_p, max_tokens, stop, seed }
```

All are pure type definitions with no effectful constructs. Invariants (I1--I12 from SPEC-029) are
documented as comments and can be enforced by constructors in `prompt.ash`.

**TDD steps:**
1. Verify the file parses without errors via the engine.
2. Verify each type is importable from `llm::types`.
3. Verify type fields have correct names and types.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_types
```

---

#### TASK-526: Create std/src/llm/prompt.ash -- Constructors

**Objective:** Implement the pure constructor functions that build `Message` values, per
SPEC-029 §4.1.

**Files to create:**
- `std/src/llm/prompt.ash`

**Functions (from SPEC-029 §4.1):**

```ash
fn system(content: String) -> Message
fn user(content: String) -> Message
fn assistant(content: String) -> Message
fn tool_result(call_id: String, content: String) -> Message
```

Each returns a `Message` with the appropriate `role`, `content`, and correct `None` values for
tool-related fields. These are pure `fn` definitions -- no `act`, no `ret`, no workflow constructs.

**Postconditions (from SPEC-029):**
- `system()`: `result.role == System`, `result.tool_calls == None`, `result.tool_call_id == None`
- `user()`: `result.role == User`, `result.tool_calls == None`, `result.tool_call_id == None`
- `assistant()`: `result.role == Assistant`, `result.tool_calls == None`, `result.tool_call_id == None`
- `tool_result()`: `result.role == Tool`, `result.tool_call_id == Some(call_id)`,
  `result.tool_calls == None`

**TDD steps:**
1. Test `system("hello")` produces `Message { role: System, content: "hello", ... }`.
2. Test `user("question")` produces correct role and content.
3. Test `assistant("reply")` produces correct role and content.
4. Test `tool_result("call_123", "result")` produces Tool role with tool_call_id set.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_prompt_constructors
```

---

#### TASK-527: Create std/src/llm/prompt.ash -- Inspectors

**Objective:** Implement the pure inspector functions that examine `ChatResponse` and `Message`
values, per SPEC-029 §4.2.

**Files to modify:**
- `std/src/llm/prompt.ash`

**Functions (from SPEC-029 §4.2):**

```ash
fn append_response(messages: List<Message>, response: ChatResponse) -> List<Message>
fn append_tool_result(messages: List<Message>, call_id: String, content: String) -> List<Message>
fn has_tool_calls(response: ChatResponse) -> Bool
fn is_final(response: ChatResponse) -> Bool
fn get_tool_calls(response: ChatResponse) -> List<ToolCall>
```

**Semantics:**
- `append_response`: appends an assistant `Message` constructed from the response content and
  tool_calls to the message list.
- `append_tool_result`: appends a tool-result message with the given call_id and content.
- `has_tool_calls`: `true` iff `response.tool_calls` is `Some`.
- `is_final`: `true` iff `finish_reason` is `"stop"` or `"length"`. A response with
  `finish_reason = "tool_calls"` is NOT final.
- `get_tool_calls`: returns the tool calls list if present, empty list otherwise.

**TDD steps:**
1. Test `has_tool_calls` with response containing tool calls returns `true`.
2. Test `has_tool_calls` with response having `tool_calls = None` returns `false`.
3. Test `is_final` with `"stop"` and `"length"` returns `true`; with `"tool_calls"` returns `false`.
4. Test `get_tool_calls` extracts calls correctly; empty list when None.
5. Test `append_response` appends correctly for text-only, tool-call-only, and mixed responses.
6. Test `append_tool_result` appends a tool message with correct call_id.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_prompt_inspectors
```

---

#### TASK-528: Create std/src/llm/prompt.ash -- Renderers

**Objective:** Implement the pure renderer functions that produce string representations of
conversations and templates, per SPEC-029 §4.3.

**Files to modify:**
- `std/src/llm/prompt.ash`

**Functions (from SPEC-029 §4.3):**

```ash
fn render_conversation(messages: List<Message>) -> String
fn render_template(template: String, vars: Map<String, String>) -> String
```

**Postconditions (from SPEC-029 §4.3):**
- `render_conversation`:
  - (R1) Each message contributes a non-empty text fragment.
  - (R2) Role prefixes are uppercase (`SYSTEM:`, `USER:`, `ASSISTANT:`, `TOOL:`).
  - (R3) Messages appear in input order.
  - (R4) Deterministic: equal inputs produce equal outputs.
- `render_template`: replaces `{{key}}` placeholders with `vars[key]`. Unresolved placeholders
  are left unreplaced. Deterministic.

**TDD steps:**
1. Test `render_conversation([system("a"), user("b")])` produces `"SYSTEM: a\nUSER: b\n"`.
2. Test `render_conversation([])` produces empty string.
3. Test determinism: same input produces same output twice.
4. Test `render_template("Hello {{name}}", {"name": "Ash"})` produces `"Hello Ash"`.
5. Test `render_template` with missing key leaves placeholder unreplaced.
6. Test `render_template` with multiple placeholders.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_prompt_renderers
```

---

### Track 3: Ash Stdlib -- Capability and Dispatch Workflows

These are `workflow` definitions (Tier 2 and Tier 3) that use `act` for LLM dispatch and IO for
prompt loading.

---

#### TASK-529: Create std/src/llm/openai.ash capability declaration

**Objective:** Create the OpenAI-specific capability declaration with the `Llm` capability and
the five actions defined in SPEC-029 §5.

**Files to create:**
- `std/src/llm/openai.ash`

**Capability declaration (from SPEC-029 §5.1):**

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

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_openai_capability
```

---

#### TASK-530: Create dispatch workflows

**Objective:** Implement the seven dispatch workflows as thin wrappers around `act` calls, per
SPEC-029 §6.

**Files to modify:**
- `std/src/llm/dispatch.ash`

**Workflows (from SPEC-029 §6.1--6.7):**

| Workflow | Delegates to | Signature |
|----------|-------------|-----------|
| `complete` | `act llm:chat(...)` | `(provider, model, messages, params) -> ChatResponse` |
| `complete_with_tools` | `act llm:chat_with_tools(...)` | `(provider, model, messages, tools, params) -> ChatResponse` |
| `complete_tuned` | `act llm:chat(...)` with non-optional params | `(provider, model, messages, params) -> ChatResponse` |
| `ask` | `act llm:chat(...)` with single user message | `(provider, model, question) -> ChatResponse` |
| `stream` | `act llm:chat_stream(...)` | `(provider, model, messages, params) -> Stream<ChatChunk>` |
| `embed` | `act llm:embed(...)` | `(provider, model, texts) -> List<Embedding>` |
| `list_models` | `act llm:list_models(...)` | `(provider) -> List<String>` |

All are Tier 3 workflows (use `act`). `ask` constructs `[user(question)]` from the question string
before dispatching.

**TDD steps:**
1. Test each workflow parses correctly.
2. Integration test: `ask` with mock provider returns ChatResponse.
3. Integration test: `complete` with mock provider.
4. Integration test: `list_models` with mock provider.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_openai_dispatch
```

---

#### TASK-531: Create loading workflows

**Objective:** Implement the two loading workflows for prompt sources, per SPEC-029 §7.

**Files to modify:**
- `std/src/llm/loading.ash`

**Workflows (from SPEC-029 §7.1--7.2):**

| Workflow | Purpose |
|----------|---------|
| `load_prompt(source)` | Load prompt from `file:path`, `env:VAR`, `cache:key`, or literal string |
| `load_system_prompt(name)` | Load named system prompt from configured directory |

These are Tier 2 workflows (perform IO via `fs:*` actions, but do not dispatch to LLM).

**`load_prompt` routing:**
- `file:path` -> read file via `act fs:read_to_string(path)`, return `system(content)`.
- `env:VAR` -> read environment variable, return `system(content)`.
- `cache:key` -> look up cached prompt.
- Other -> treat as literal string, return `system(source)`.

**Error conditions:** File not found, env var unset, cache miss produce runtime failure.

**TDD steps:**
1. Test `load_prompt("file:/tmp/test.txt")` reads file and returns system message.
2. Test `load_prompt("env:MY_VAR")` reads env var and returns system message.
3. Test `load_prompt("literal text")` returns system message with literal content.
4. Test `load_system_prompt("greeting")` reads from prompt directory.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_openai_loading
```

---

### Track 4: Agent Orchestration Workflows

Agent loops that use `act` (via dispatch workflows), `spawn`, `kill`, `check_health`, and
`receive`. All are Tier 3 workflows defined in separate files under `std/src/llm/`.

---

#### TASK-532: Create conversation workflow

**Objective:** Implement the multi-turn conversation workflow with mailbox receive, per
SPEC-029 §8.1.

**Files to create:**
- `std/src/llm/conversation.ash`

**Loop behavior (from SPEC-029 §8.1):**
1. Initialize `messages = [system(system_prompt)]`.
2. `receive` a user message; append to `messages`.
3. Call `complete(provider, model, messages, None)`.
4. Append response to `messages`.
5. If final or `turns >= max_turns`, return `messages`.
6. Otherwise go to step 2.

**Termination:** max_turns reached, final response, or user sends termination signal.

**Error handling:** on `act` failure, return messages accumulated so far.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_agent_conversation
```

---

#### TASK-533: Create tool_agent workflow

**Objective:** Implement the orient-decide-act tool-use loop, per SPEC-029 §8.2.

**Files to modify:**
- `std/src/llm/tool_agent.ash`

**Signature:**

```ash
workflow tool_agent(provider: String, model: String,
                    messages: List<Message>,
                    tools: List<ToolDef>,
                    max_rounds: Int) -> ChatResponse
    requires: max_rounds > 0
```

**Loop behavior (from SPEC-029 §8.2):**
1. `rounds = 0`.
2. Call `complete_with_tools(provider, model, messages, tools, None)`.
3. If `is_final(response)`: return response.
4. If `has_tool_calls(response)`: extract via `get_tool_calls(response)`.
5. Execute tool calls via a statically declared dispatcher workflow/helper that matches tool names
   and invokes explicit named `act` targets.
6. `messages = append_response(messages, response)`.
7. For each tool result: `messages = append_tool_result(messages, call_id, result)`.
8. `rounds += 1`. If `rounds >= max_rounds`: return response.
9. Go to step 2.

**Error handling:** on `act` failure, return last successful `ChatResponse`. On tool execution
failure within a matched static branch, append error tool result and continue. Unknown tool names
must not trigger runtime capability lookup; they append an error tool result and continue.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_agent_tool_agent
```

---

#### TASK-534: Create router workflow

**Objective:** Implement the task-complexity classification and model routing workflow, per
SPEC-029 §8.3.

**Files to modify:**
- `std/src/llm/router.ash`

**Signature:**

```ash
workflow router(provider: String, messages: List<Message>) -> ChatResponse
```

**Behavior (from SPEC-029 §8.3):**
1. Render a classification prompt from `messages`.
2. Use the Phase 77 fixed classifier model `"gpt-4o-mini"`.
3. Call `ask(provider, "gpt-4o-mini", classification_prompt)` so the classifier request uses the
   existing single-turn wrapper rather than passing a bare prompt where `complete` expects
   `List<Message>`.
4. Parse response to determine complexity: `Simple | Moderate | Complex`.
5. Map complexity to model identifier (e.g., `"gpt-4o-mini"` / `"gpt-4o"` / `"o1"`).
6. Call `complete(provider, selected_model, messages, None)`.
7. Return the response.

**Error handling:** if classification fails, default to moderate-complexity model.

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_agent_router
```

---

#### TASK-535: Create supervised_agent workflow

**Objective:** Implement the spawn/kill/restart supervised agent pattern, per SPEC-029 §8.4.

**Files to modify:**
- `std/src/llm/supervised.ash`

**Signature:**

```ash
type AgentConfig {
    provider: String,
    model: String,
    messages: List<Message>,
    tools: List<ToolDef>,
    max_rounds: Int,
    max_restarts: Int
}

workflow supervised_agent(config: AgentConfig) -> Result<ChatResponse, AgentError>
```

**Behavior (from SPEC-029 §8.4):**
1. `spawn tool_agent(config.provider, config.model, config.messages, config.tools, config.max_rounds)`.
2. Periodically `check_health(handle)`.
3. On `Healthy`: continue monitoring.
4. The child `tool_agent` sends its terminal `ChatResponse` to the supervisor mailbox before exit.
5. The supervisor `receive`s that payload and returns `Ok(received_response)`.
6. On `Failed`: `kill handle`; if `restarts < max_restarts`, respawn; otherwise return `Err(AgentError)`.

**Termination:** child completes normally (return `Ok` with the `ChatResponse` received from the child mailbox message), or max_restarts exceeded (return `Err(AgentError)`).

**Verification:**
```
cargo test -p ash-engine --lib tests::stdlib_llm_agent_supervised
```

---

### Track 5: Integration and Documentation

---

#### TASK-536: Integration tests

**Objective:** End-to-end integration tests verifying the Rust provider and Ash workflows work
together, with mock backends.

**Files to create/modify:**
- `crates/ash-engine/tests/llm_integration.rs` (or within `tests/` module in lib.rs)

**Tests:**
1. **End-to-end chat:** Engine with `with_llm_capabilities()` + mock OpenAI endpoint. Parse a
   workflow calling `act llm:chat(...)`, execute, verify ChatResponse Value.
2. **Multi-provider routing:** Register two providers ("local" pointing to ollama mock, "remote"
   pointing to OpenAI mock). Execute workflow that calls both, verify correct routing.
3. **Streaming test:** Mock SSE stream, verify chunks arrive in order with correct content.
4. **Embedding test:** Mock embedding endpoint, verify result structure and index ordering.
5. **Tool-use loop integration:** Execute `tool_agent` workflow with mock LLM that returns tool
   calls, verify the loop completes after tool execution.
6. **Error propagation:** Verify provider errors (auth, rate limit, not found) propagate
   correctly through `act` to the workflow.

**Verification:**
```
cargo test -p ash-engine --test llm_integration
```

---

#### TASK-537: Update CHANGELOG.md

**Objective:** Add Unreleased entries for all LLM stdlib changes.

**Files to modify:**
- `CHANGELOG.md`

**Entries to add under `## [Unreleased] / ### Added`:**
- LLM Standard Library implementation (DESIGN-025, SPEC-029, PLAN-025).
- Rust-side `LlmProvider` with `async-openai` backing, multi-provider routing.
- Ash stdlib `llm/` module with pure types, prompt constructors, inspectors, and renderers.
- OpenAI-specific capability declaration and dispatch workflows.
- Agent orchestration workflows: conversation, tool_agent, router, supervised_agent.
- Engine builder method `with_llm_capabilities(HashMap<String, LlmConfig>)`.
- Integration tests with mock backends.

**Verification:**
```
grep "PLAN-025\|LLM Standard Library\|LlmProvider" CHANGELOG.md
```

---

#### TASK-538: Documentation

**Objective:** Add module-level doc comments and a README for the LLM stdlib.

**Files to create:**
- `std/src/llm/README.md`

**Files to modify:**
- `crates/ash-engine/src/providers/llm/mod.rs` -- module-level doc comment
- `crates/ash-engine/src/providers/llm/config.rs` -- doc comments on `LlmConfig`
- `crates/ash-engine/src/providers/llm/chat.rs` -- doc comments
- `crates/ash-engine/src/providers/llm/stream_adapter.rs` -- doc comments
- `crates/ash-engine/src/providers/llm/embeddings.rs` -- doc comments
- `crates/ash-engine/src/providers/llm/tool_dispatch.rs` -- doc comments
- `std/src/llm/mod.ash` -- module-level comments
- `std/src/llm/types.ash` -- doc comments on each type
- `std/src/llm/prompt.ash` -- doc comments on each function
- `std/src/llm/openai.ash` -- capability declaration docs
- `std/src/llm/dispatch.ash` -- dispatch workflow docs
- `std/src/llm/loading.ash` -- loading workflow docs
- `std/src/llm/conversation.ash` -- conversation workflow docs
- `std/src/llm/tool_agent.ash` -- tool agent docs
- `std/src/llm/router.ash` -- router workflow docs
- `std/src/llm/supervised.ash` -- supervised agent docs

**README content:**
- Overview of the LLM stdlib architecture (three-tier model).
- How to register providers in the engine builder.
- How to use pure types and prompt functions.
- How to call dispatch workflows.
- How to build agent loops.
- Multi-provider routing examples.
- Namespace layout and future-proofing notes.

**Verification:**
```
cargo doc -p ash-engine --no-deps 2>&1 | head -20
```

---

## Dependency Graph

```
Track 1 (Rust Provider):
  TASK-516 (async-openai dep)
    -> TASK-517 (LlmConfig)
    -> TASK-518 (LlmProvider skeleton)
    -> TASK-519 (chat) -- depends on 517, 518
    -> TASK-520 (streaming) -- depends on 517, 518
    -> TASK-521 (tool dispatch) -- depends on 517
    -> TASK-522 (embeddings) -- depends on 517, 518
    -> TASK-523 (engine builder wiring) -- depends on 518, 519, 520, 521, 522

Track 2 (Ash Pure Types/Functions):
  TASK-524 (module structure) -- depends on PLAN-023 complete
    -> TASK-525 (types.ash) -- depends on 524
    -> TASK-526 (prompt constructors) -- depends on 525
    -> TASK-527 (prompt inspectors) -- depends on 525
    -> TASK-528 (prompt renderers) -- depends on 525

Track 3 (Capability/Dispatch):
  TASK-529 (capability declaration) -- depends on 525
    -> TASK-530 (dispatch workflows) -- depends on 529, Track 1
    -> TASK-531 (loading workflows) -- depends on 529

Track 4 (Agent Orchestration):
  TASK-532 (conversation) -- depends on 530
    -> TASK-533 (tool_agent) -- depends on 530, 521
    -> TASK-534 (router) -- depends on 530
    -> TASK-535 (supervised_agent) -- depends on 533

Track 5 (Integration/Docs):
  TASK-536 (integration tests) -- depends on Track 1, Track 3, Track 4
  TASK-537 (CHANGELOG) -- depends on all
  TASK-538 (documentation) -- depends on all
```

**Critical path:** TASK-516 -> 517 -> 518 -> 519 -> 523 -> 530 -> 533 -> 536
(where TASK-523 depends on TASK-519, TASK-520, TASK-521, TASK-522 all completing first)

---

## Milestone Definition

**Phase complete when:**

### Rust Provider (Track 1)
1. `async-openai` compiles as a dependency of `ash-engine`.
2. `LlmConfig` validates correctly: valid URLs accepted, invalid rejected.
3. `LlmProvider` implements `CapabilityProvider` with `name() = "llm"`, `effect() = Operational`.
4. **AC:** `chat` action sends correct request to mock endpoint and returns `ChatResponse` Value.
5. **AC:** `chat_stream` action returns chunks satisfying SC1--SC5 from SPEC-029 §9.5.
6. **AC:** `embed` action returns `List<Embedding>` satisfying E1--E2 from SPEC-029 §5.2.3.
7. **AC:** `list_models` execute action returns `List<String>`.
8. **AC:** Error mapping: 401/403 -> auth_error, 429 -> rate_limited, 404 -> model_not_found.
9. **AC:** `with_llm_capabilities(configs).build()` succeeds; engine has provider registered.
10. **AC:** Multi-provider routing: same provider dispatches to correct api_base by name.

### Ash Pure Types/Functions (Track 2)
11. `llm/types.ash` parses and all types are importable.
12. `llm/prompt.ash` constructors produce correct Message values for each role.
13. **AC:** `is_final` returns true for "stop"/"length", false for "tool_calls"/None.
14. **AC:** `has_tool_calls` returns true iff response.tool_calls is Some.
15. **AC:** `render_conversation` produces deterministic output with uppercase role prefixes.
16. **AC:** `render_template` replaces `{{key}}` placeholders, leaves unknowns unreplaced.

### Capability and Dispatch (Track 3)
17. `Llm` capability declaration parses with all five actions.
18. **AC:** `complete` workflow delegates to `act llm:chat(...)`.
19. **AC:** `ask` workflow constructs single user message and dispatches.
20. **AC:** `load_prompt("file:path")` reads file via io:fs capability.

### Agent Orchestration (Track 4)
21. `conversation` workflow parses and respects max_turns.
22. **AC:** `tool_agent` loop terminates on is_final or max_rounds.
23. **AC:** `router` classifies complexity and routes to appropriate model.
24. **AC:** `supervised_agent` spawns, monitors, and respawns on failure.

### Integration (Track 5)
25. All integration tests pass with mock backends.
26. CHANGELOG.md updated.
27. Module-level documentation complete.
28. `cargo test` passes, `cargo clippy` clean, `cargo fmt --check` clean.

---

## Normative Delta vs Existing Specs

### SPEC-009 (Module System)
- New module hierarchy: `llm/` with flat file layout under `std/src/`.

### SPEC-013 (Streams)
- `Stream<ChatChunk>` returned by `chat_stream` action.

### SPEC-024 (Capability Role Reduction)
- `Llm` capability follows the role reduction pattern with execute/observe roles.

### Cargo.toml (ash-engine)
- New dependency: `async-openai` crate.

---

## Estimated Total Effort

~87 hours across 23 tasks across 5 tracks.

| Track | Tasks | Est. Hours |
|-------|-------|------------|
| Track 1: Rust Provider | TASK-516 through TASK-523 (8 tasks) | 34 |
| Track 2: Pure Types/Functions | TASK-524 through TASK-528 (5 tasks) | 15 |
| Track 3: Capability/Dispatch | TASK-529 through TASK-531 (3 tasks) | 11 |
| Track 4: Agent Orchestration | TASK-532 through TASK-535 (4 tasks) | 17 |
| Track 5: Integration/Docs | TASK-536 through TASK-538 (3 tasks) | 10 |
