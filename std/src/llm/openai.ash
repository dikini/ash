-- OpenAI Provider Capability (SPEC-029 §5)
--
-- Declares the Llm capability for OpenAI-compatible providers.
-- This capability provides execute actions for chat completion, streaming,
-- embeddings, and model discovery.
--
-- All actions are effectful (execute) and require the Llm provider to be
-- registered in the engine with appropriate configuration.

use types::{
    Message,
    ToolDef,
    ChatResponse,
    ChatChunk,
    Embedding,
    CompletionParams
};

-- Llm capability for OpenAI-compatible providers
--
-- This capability provides:
-- - chat: Single non-streaming chat completion
-- - chat_with_tools: Chat completion with tool definitions
-- - chat_stream: Streaming chat completion
-- - embed: Text embedding generation
-- - list_models: List available models from the provider
pub capability Llm: execute chat(
        provider: String,
        model: String,
        messages: List<Message>,
        params: Option<CompletionParams>
    ) -> ChatResponse
  | execute chat_with_tools(
        provider: String,
        model: String,
        messages: List<Message>,
        tools: List<ToolDef>,
        params: Option<CompletionParams>
    ) -> ChatResponse
  | execute chat_stream(
        provider: String,
        model: String,
        messages: List<Message>,
        params: Option<CompletionParams>
    ) -> Stream<ChatChunk>
  | execute embed(
        provider: String,
        model: String,
        texts: List<String>
    ) -> List<Embedding>
  | execute list_models(
        provider: String
    ) -> List<String>;
