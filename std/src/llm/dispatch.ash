-- LLM dispatch provider functions.
--
-- Runtime providers own the effectful boundary. The stdlib exposes target Ash
-- function declarations and ordinary convenience wrappers only.

use types::{
    Message,
    ToolDef,
    ChatResponse,
    ChatChunk,
    Embedding,
    CompletionParams
};
use prompt::user;

pub builtin fn complete(
    provider: String,
    model: String,
    messages: List<Message>,
    params: Option<CompletionParams>
) -> ChatResponse;

pub builtin fn complete_with_tools(
    provider: String,
    model: String,
    messages: List<Message>,
    tools: List<ToolDef>,
    params: Option<CompletionParams>
) -> ChatResponse;

pub fn complete_tuned(
    provider: String,
    model: String,
    messages: List<Message>,
    params: CompletionParams
) -> ChatResponse {
    complete(provider, model, messages, Some(params))
}

pub fn ask(provider: String, model: String, question: String) -> ChatResponse {
    let messages = [user(question)];
    complete(provider, model, messages, None)
}

pub builtin fn stream(
    provider: String,
    model: String,
    messages: List<Message>,
    params: Option<CompletionParams>
) -> Stream<ChatChunk>;

pub builtin fn embed(
    provider: String,
    model: String,
    texts: List<String>
) -> List<Embedding>;

pub builtin fn list_models(provider: String) -> List<String>;
