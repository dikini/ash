-- Checkable LLM loading helper placeholders (SPEC-029 §7)
--
-- Phase 107 keeps this module in the std corpus by preserving the public
-- helper names and return types while deferring rich file/env/cache loading
-- semantics until the relevant parser/runtime surfaces are canonicalized.
-- The current implementation treats inputs as literal prompt text and wraps
-- them as system messages.
--
-- The imported IO names below intentionally pin the supported std import
-- surfaces for future loading work, even though the placeholder bodies do not
-- force filesystem effects yet.

use types::{Message, Role, ToolCall};
use prompt::system;
use io::{PathBuf, from_string, join, read_to_string};

-- Load a prompt from literal prompt text.
--
-- Parameters:
--   source: Literal prompt text or future source identifier.
--
-- Returns: Message with role=System containing the provided text.
--
-- Deferred: interpreting prefixes such as `file:`, `env:`, or `cache:` as
-- effectful loading strategies requires follow-on parser/runtime work.
--
-- Example:
--   let prompt = load_prompt("You are a helpful assistant.");
pub fn load_prompt(source: String) -> Message {
    system(source)
}

-- Load a named system prompt placeholder.
--
-- Parameters:
--   name: Name or text for the system prompt.
--
-- Returns: Message with role=System containing the provided name/text.
--
-- Deferred: configured prompt-directory lookup and IO failure reporting remain
-- future work; this corpus-safe placeholder wraps the provided text directly.
--
-- Example:
--   let sys_prompt = load_system_prompt("code_review");
pub fn load_system_prompt(name: String) -> Message {
    system(name)
}
