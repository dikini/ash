-- LLM Module
--
-- Provides types and functions for working with Large Language Models.
-- Includes chat completions, embeddings, and tool use.
--
-- All types are pure and immutable. Functions are side-effect free.
--
-- ## Example
--
-- ```
-- use std::llm::{system, user, assistant, render_markdown};
--
-- let conversation = [
--     system("You are a helpful assistant."),
--     user("What is 2 + 2?"),
--     assistant("The answer is 4.")
-- ];
--
-- let transcript = render_markdown(conversation);
-- ```

-- Type definitions (SPEC-029 §3)
pub mod types;

-- Prompt functions (SPEC-029 §4)
pub mod prompt;

-- OpenAI capability declaration (SPEC-029 §5)
pub mod openai;

-- Dispatch helpers (SPEC-029 §6)
pub mod dispatch;

-- Loading helpers (SPEC-029 §7)
pub mod loading;

-- Agent orchestration helpers (SPEC-029 §8)
pub mod conversation;
pub mod tool_agent;
pub mod supervised;

-- Re-export all types
pub use types::{
    Role,
    Message,
    ToolCall,
    ToolCallDelta,
    ToolDef,
    Usage,
    ChatResponse,
    Embedding,
    ChatChunk
};

-- Re-export prompt constructors
pub use prompt::{
    system,
    user,
    assistant,
    assistant_with_tools,
    tool_result,
    message
};

-- Re-export prompt inspectors
pub use prompt::{
    is_system,
    is_user,
    is_assistant,
    is_tool,
    sender,
    content,
    get_tool_calls,
    has_tool_calls,
    get_tool_call_id,
    append_response,
    append_tool_result,
    is_final
};

-- Re-export prompt renderers
pub use prompt::{
    render_plaintext,
    render_markdown,
    render_template
};

-- Re-export prompt utilities
pub use prompt::{
    count,
    last,
    filter_user,
    filter_assistant,
    append,
    prepend
};

-- Re-export dispatch helpers
pub use dispatch::{
    complete,
    complete_with_tools,
    complete_tuned,
    ask,
    stream,
    embed,
    list_models
};

-- Re-export loading helpers
pub use loading::{
    load_prompt,
    load_system_prompt
};

-- Re-export agent orchestration helpers
pub use conversation::conversation;
pub use tool_agent::tool_agent;
-- NOTE(TASK-792): supervised::supervised_agent remains available through its
-- public child module, but is intentionally not root re-exported until child
-- helper snippet warnings no longer mask unrelated imports. router.ash is
-- reference-only after Phase 201.
