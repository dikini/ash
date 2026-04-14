-- Prompt Functions (SPEC-029 §4)
--
-- Pure functions for constructing, inspecting, and rendering chat messages.
-- All functions are side-effect free data transformations.

use types::{Role, Message, ToolCall, ChatResponse};

-- ============================================================================
-- Constructors
-- ============================================================================

-- Create a system message with the given content
--
-- Example:
--   let msg = system("You are a helpful assistant");
pub fn system(content: String) -> Message {
    Message {
        role: System,
        content: content,
        tool_calls: None,
        tool_call_id: None
    }
}

-- Create a user message with the given content
--
-- Example:
--   let msg = user("What is 2 + 2?");
pub fn user(content: String) -> Message {
    Message {
        role: User,
        content: content,
        tool_calls: None,
        tool_call_id: None
    }
}

-- Create an assistant message with the given content
--
-- Example:
--   let msg = assistant("The answer is 4.");
pub fn assistant(content: String) -> Message {
    Message {
        role: Assistant,
        content: content,
        tool_calls: None,
        tool_call_id: None
    }
}

-- Create an assistant message with tool calls
--
-- Example:
--   let tool_call = ToolCall { id: "call_1", name: "add", arguments: "{\"a\": 2, \"b\": 2}" };
--   let msg = assistant_with_tools("", [tool_call]);
pub fn assistant_with_tools(content: String, tool_calls: List<ToolCall>) -> Message {
    Message {
        role: Assistant,
        content: content,
        tool_calls: Some { value: tool_calls },
        tool_call_id: None
    }
}

-- Create a tool result message
--
-- Example:
--   let msg = tool_result("call_1", "4");
pub fn tool_result(call_id: String, content: String) -> Message {
    Message {
        role: Tool,
        content: content,
        tool_calls: None,
        tool_call_id: Some { value: call_id }
    }
}

-- Create a message with explicit role
--
-- Example:
--   let msg = message(User, "Hello");
pub fn message(role: Role, content: String) -> Message {
    Message {
        role: role,
        content: content,
        tool_calls: None,
        tool_call_id: None
    }
}

-- ============================================================================
-- Inspectors
-- ============================================================================

-- Check if message is from the system
pub fn is_system(msg: Message) -> Bool {
    match msg {
        Message { role: System, content: _, tool_calls: _, tool_call_id: _ } => true,
        _ => false
    }
}

-- Check if message is from the user
pub fn is_user(msg: Message) -> Bool {
    match msg {
        Message { role: User, content: _, tool_calls: _, tool_call_id: _ } => true,
        _ => false
    }
}

-- Check if message is from the assistant
pub fn is_assistant(msg: Message) -> Bool {
    match msg {
        Message { role: Assistant, content: _, tool_calls: _, tool_call_id: _ } => true,
        _ => false
    }
}

-- Check if message is a tool result
pub fn is_tool(msg: Message) -> Bool {
    match msg {
        Message { role: Tool, content: _, tool_calls: _, tool_call_id: _ } => true,
        _ => false
    }
}

-- Get the role of a message
pub fn role(msg: Message) -> Role {
    match msg {
        Message { role: r, content: _, tool_calls: _, tool_call_id: _ } => r
    }
}

-- Get the content of a message
pub fn content(msg: Message) -> String {
    match msg {
        Message { role: _, content: c, tool_calls: _, tool_call_id: _ } => c
    }
}

-- Get tool calls from an assistant message (returns empty list if none)
pub fn get_tool_calls(msg: Message) -> List<ToolCall> {
    match msg {
        Message { role: _, content: _, tool_calls: Some { value: calls }, tool_call_id: _ } => calls,
        _ => []
    }
}

-- Check if a chat response has tool calls
pub fn has_tool_calls(response: ChatResponse) -> Bool {
    match response {
        ChatResponse { content: _, tool_calls: Some { value: _ }, finish_reason: _, usage: _, model: _, id: _ } => true,
        _ => false
    }
}

-- Get the tool call ID from a tool message
pub fn get_tool_call_id(msg: Message) -> Option<String> {
    match msg {
        Message { role: _, content: _, tool_calls: _, tool_call_id: id } => id
    }
}

-- Append assistant message from a ChatResponse to conversation history
pub fn append_response(messages: List<Message>, response: ChatResponse) -> List<Message> {
    match response {
        ChatResponse { content: Some { value: text }, tool_calls: calls, finish_reason: _, usage: _, model: _, id: _ } => {
            let msg = Message {
                role: Assistant,
                content: text,
                tool_calls: calls,
                tool_call_id: None
            };
            list::append(messages, [msg])
        },
        ChatResponse { content: None, tool_calls: Some { value: tool_calls }, finish_reason: _, usage: _, model: _, id: _ } => {
            let msg = Message {
                role: Assistant,
                content: "",
                tool_calls: Some { value: tool_calls },
                tool_call_id: None
            };
            list::append(messages, [msg])
        }
    }
}

-- Append a tool result message to conversation history
pub fn append_tool_result(messages: List<Message>, call_id: String, content: String) -> List<Message> {
    list::append(messages, [tool_result(call_id, content)])
}

-- Check if a chat response indicates the conversation is finished
-- (finish_reason is "stop" or "length", not "tool_calls")
pub fn is_final(response: ChatResponse) -> Bool {
    match response {
        ChatResponse { content: _, tool_calls: _, finish_reason: Some { value: reason }, usage: _, model: _, id: _ } => {
            reason == "stop" || reason == "length"
        },
        _ => false
    }
}

-- ============================================================================
-- Renderers
-- ============================================================================

-- Render a template by substituting {{key}} placeholders with values
-- vars: key-value pairs as Map<String, String> (alias for List<(String, String)>)
-- NOTE: Full template substitution requires runtime string::replace support.
-- This stub returns template unchanged; signature matches SPEC-029 §4.3.2.
pub fn render_template(template: String, vars: Map<String, String>) -> String {
    template
}

-- Helper: Get role name as string
fn role_name(role: Role) -> String {
    match role {
        System => "system",
        User => "user",
        Assistant => "assistant",
        Tool => "tool"
    }
}

-- Helper: Format a single message as plaintext
fn format_message_plain(msg: Message) -> String {
    match msg {
        Message { role: r, content: c, tool_calls: _, tool_call_id: _ } => {
            let prefix = string::concat(role_name(r), ": ");
            string::concat(prefix, c)
        }
    }
}

-- Render messages as simple plaintext
-- Each message on its own line with role prefix
--
-- Example:
--   system: You are helpful
--   user: Hello
--   assistant: Hi there
pub fn render_plaintext(messages: List<Message>) -> String {
    let formatted = list::map(messages, format_message_plain);
    string::join("\n", formatted)
}

-- Helper: Format a single message as markdown
fn format_message_md(msg: Message) -> String {
    match msg {
        Message { role: r, content: c, tool_calls: calls, tool_call_id: _ } => {
            let header = match r {
                System => "**System**",
                User => "**User**",
                Assistant => "**Assistant**",
                Tool => string::concat("**Tool** (", id, ")")
            };
            let body = match calls {
                Some { value: tool_calls } => {
                    let tool_text = render_tool_calls(tool_calls);
                    string::concat(c, tool_text)
                },
                None => c
            };
            string::concat(header, ":\n", body)
        }
    }
}

-- Helper: Render tool calls as markdown
fn render_tool_calls(calls: List<ToolCall>) -> String {
    if list::is_empty(calls) then
        ""
    else {
        let format_call = fn(call: ToolCall) -> String {
            match call {
                ToolCall { id: i, name: n, arguments: a } => {
                    let header = string::concat("\n\n- **", n, "** (`", i, "`)");
                    let args = string::concat("\n  ```json\n  ", a, "\n  ```");
                    string::concat(header, args)
                }
            }
        };
        string::concat("\n\n**Tool Calls:**", string::concat_list(list::map(calls, format_call)))
    }
}

-- Render messages as markdown
-- Formats each message with a bold header and preserves structure
--
-- Example:
--   **System**:
--   You are helpful
--
--   **User**:
--   Hello
--
--   **Assistant**:
--   Hi there
pub fn render_markdown(messages: List<Message>) -> String {
    let formatted = list::map(messages, format_message_md);
    string::join("\n\n", formatted)
}

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Count messages in a conversation
pub fn count(messages: List<Message>) -> Int {
    list::len(messages)
}

-- Get the last message in the conversation (if any)
pub fn last(messages: List<Message>) -> Option<Message> {
    list::last(messages)
}

-- Get only user messages
pub fn filter_user(messages: List<Message>) -> List<Message> {
    list::filter(messages, is_user)
}

-- Get only assistant messages
pub fn filter_assistant(messages: List<Message>) -> List<Message> {
    list::filter(messages, is_assistant)
}

-- Append a message to the conversation
pub fn append(messages: List<Message>, msg: Message) -> List<Message> {
    list::append(messages, [msg])
}

-- Prepend a message to the conversation
pub fn prepend(messages: List<Message>, msg: Message) -> List<Message> {
    list::prepend(messages, msg)
}
