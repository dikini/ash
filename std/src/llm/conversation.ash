-- Conversation Workflow (SPEC-029 §8.1)
--
-- Simple multi-turn conversation workflow. Appends the user message to the
-- conversation history and returns the model's response.
--
-- This is the basic building block for chat interactions. For tool-using
-- agents, see the `tool_agent` workflow.
--
-- Example:
--   let history = [system("You are helpful")];
--   let response = conversation("openai", "gpt-4o", history, "Hello!");
--   -- response.content contains the assistant's reply

use types::{Message, ChatResponse};
use prompt::{user, append};
use dispatch::complete;

-- Conduct a multi-turn conversation
--
-- Parameters:
--   provider: Provider name (e.g., "openai", "ollama")
--   model: Model identifier (e.g., "gpt-4o", "llama3.1")
--   history: Conversation history as a list of messages
--   user_message: The new user message to add
--
-- Returns: ChatResponse containing the model's response
--
-- Example:
--   let history = [
--       system("You are a helpful assistant."),
--       user("What is 2 + 2?"),
--       assistant("The answer is 4.")
--   ];
--   let response = conversation("openai", "gpt-4o", history, "What about 3 + 3?");
--   -- response.content == Some("The answer is 6.")
workflow conversation(
    provider: String,
    model: String,
    history: List<Message>,
    user_message: String
) -> ChatResponse {
    let msg = user(user_message);
    let messages = append(history, msg);
    complete(provider, model, messages, None)
}
