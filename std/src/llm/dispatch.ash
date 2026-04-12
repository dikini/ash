-- LLM Dispatch Workflows (SPEC-029 §6)
--
-- High-level workflows for LLM operations. All workflows in this module
-- are Tier 3 (effectful) and use `act` to invoke the Llm capability.
--
-- These workflows provide a convenient interface over the raw capability
-- actions, handling message construction, option wrapping, and error cases.

use types::{
    Message,
    ToolDef,
    ChatResponse,
    ChatChunk,
    Embedding,
    CompletionParams
};
use prompt::user;

-- Complete a chat request with the LLM
--
-- Sends a non-streaming chat completion request to the specified provider.
--
-- Parameters:
--   provider: Provider name (e.g., "openai", "ollama")
--   model: Model identifier (e.g., "gpt-4o", "llama3.1")
--   messages: Conversation history as a list of messages
--   params: Optional tuning parameters (temperature, max_tokens, etc.)
--
-- Returns: ChatResponse containing the model's response
--
-- Example:
--   let response = complete("openai", "gpt-4o", messages, None);
workflow complete(
    provider: String,
    model: String,
    messages: List<Message>,
    params: Option<CompletionParams>
) -> ChatResponse {
    act execute Llm.chat with
        provider: provider,
        model: model,
        messages: messages,
        params: params
}

-- Complete a chat request with tool definitions
--
-- Similar to `complete`, but provides tool definitions that the model
-- can use to make function calls. The model may return tool calls
-- in its response.
--
-- Parameters:
--   provider: Provider name
--   model: Model identifier
--   messages: Conversation history
--   tools: List of available tool definitions
--   params: Optional tuning parameters
--
-- Returns: ChatResponse which may include tool_calls
--
-- Example:
--   let tools = [calculator_def, search_def];
--   let response = complete_with_tools("openai", "gpt-4o", messages, tools, None);
workflow complete_with_tools(
    provider: String,
    model: String,
    messages: List<Message>,
    tools: List<ToolDef>,
    params: Option<CompletionParams>
) -> ChatResponse {
    act execute Llm.chat_with_tools with
        provider: provider,
        model: model,
        messages: messages,
        tools: tools,
        params: params
}

-- Complete a chat request with explicit tuning parameters
--
-- Like `complete`, but requires non-optional CompletionParams.
-- This ensures the caller explicitly sets tuning parameters.
--
-- Parameters:
--   provider: Provider name
--   model: Model identifier
--   messages: Conversation history
--   params: Required tuning parameters (temperature, max_tokens, etc.)
--
-- Returns: ChatResponse
--
-- Example:
--   let params = CompletionParams {
--       temperature: Some(0.7),
--       top_p: None,
--       max_tokens: Some(1000),
--       stop: None,
--       seed: None
--   };
--   let response = complete_tuned("openai", "gpt-4o", messages, params);
workflow complete_tuned(
    provider: String,
    model: String,
    messages: List<Message>,
    params: CompletionParams
) -> ChatResponse {
    act execute Llm.chat with
        provider: provider,
        model: model,
        messages: messages,
        params: Some(params)
}

-- Ask a single question (convenience wrapper)
--
-- Simple single-turn completion. Constructs a single user message
-- from the question string and dispatches to the LLM.
--
-- Parameters:
--   provider: Provider name
--   model: Model identifier
--   question: The question text
--
-- Returns: ChatResponse
--
-- Example:
--   let response = ask("openai", "gpt-4o", "What is the capital of France?");
workflow ask(
    provider: String,
    model: String,
    question: String
) -> ChatResponse {
    let messages = [user(question)];
    act execute Llm.chat with
        provider: provider,
        model: model,
        messages: messages,
        params: None
}

-- Stream a chat completion
--
-- Initiates a streaming chat completion request. Returns a Stream
-- that yields incremental chunks as they arrive from the provider.
--
-- Parameters:
--   provider: Provider name
--   model: Model identifier
--   messages: Conversation history
--   params: Optional tuning parameters
--
-- Returns: Stream<ChatChunk> yielding incremental response chunks
--
-- Example:
--   let stream = stream("openai", "gpt-4o", messages, None);
--   -- Process chunks as they arrive
workflow stream(
    provider: String,
    model: String,
    messages: List<Message>,
    params: Option<CompletionParams>
) -> Stream<ChatChunk> {
    act execute Llm.chat_stream with
        provider: provider,
        model: model,
        messages: messages,
        params: params
}

-- Generate embeddings for text
--
-- Computes vector embeddings for a list of input texts using
-- the specified model.
--
-- Parameters:
--   provider: Provider name
--   model: Embedding model identifier (e.g., "text-embedding-3-small")
--   texts: List of input texts to embed
--
-- Returns: List<Embedding> with the same length as input texts
--
-- Postconditions:
--   - result.length == texts.length
--   - result[i].index == i for all i
--
-- Example:
--   let texts = ["Hello world", "Goodbye world"];
--   let embeddings = embed("openai", "text-embedding-3-small", texts);
workflow embed(
    provider: String,
    model: String,
    texts: List<String>
) -> List<Embedding> {
    act execute Llm.embed with
        provider: provider,
        model: model,
        texts: texts
}

-- List available models from a provider
--
-- Queries the provider for available model identifiers.
--
-- Parameters:
--   provider: Provider name
--
-- Returns: List<String> of model identifiers
--
-- Example:
--   let models = list_models("openai");
--   -- models might contain ["gpt-4o", "gpt-4o-mini", "gpt-3.5-turbo"]
workflow list_models(
    provider: String
) -> List<String> {
    act execute Llm.list_models with provider: provider
}
