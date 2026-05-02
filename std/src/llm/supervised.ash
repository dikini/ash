-- Supervised Agent Reference Helpers (SPEC-029 §8.4)
--
-- This module keeps the supervised-tool-agent helper surface checkable while
-- the full executable tool loop remains deferred. The helper functions below
-- preserve approval prompts, supervisor decision parsing, and tool-call review
-- formatting. The public supervised_agent entry point currently returns an
-- explicit placeholder ChatResponse until mixed type/helper/workflow module
-- checking and the complete_with_tools bridge are available in one module path.
--
-- Reference behavior (deferred):
--   1. primary model proposes tool calls;
--   2. supervisor model approves or rejects them;
--   3. approved tools execute and feed results back into the conversation;
--   4. rejected calls return supervisor feedback.

use types::{Message, ToolDef, ChatResponse, ToolCall};
use prompt::{user, assistant, assistant_with_tools, tool_result, append};
use dispatch::complete;

-- Supervisor decision type
-- Approve: Tool calls are approved for execution
-- Reject: Tool calls are rejected with feedback
pub type SupervisorDecision = Approve | Reject { feedback: String };

fn concat3(a: String, b: String, c: String) -> String {
    string::concat(string::concat(a, b), c)
}

fn concat4(a: String, b: String, c: String, d: String) -> String {
    string::concat(concat3(a, b, c), d)
}

fn concat5(a: String, b: String, c: String, d: String, e: String) -> String {
    string::concat(concat4(a, b, c, d), e)
}

-- Build the supervisor approval request message (pure)
--
-- Parameters:
--   messages: Current conversation context
--   tool_calls: The tool calls awaiting approval
--
-- Returns: User Message containing the approval request prompt
pub fn build_approval_message(messages: List<Message>, tool_calls: List<ToolCall>) -> Message {
    let tool_desc = format_tool_calls_for_review(tool_calls);
    let approval_header = concat3(
        "You are a supervisor reviewing tool calls for safety and appropriateness.\n\n",
        "Review the following tool calls and respond with either:\n",
        "- APPROVE (if the tool calls are safe and appropriate)\n"
    );
    let approval_instructions = concat3(
        approval_header,
        "- REJECT: <reason> (if the tool calls should not be executed)\n\n",
        "Tool calls to review:\n"
    );
    let approval_prompt = string::concat(approval_instructions, tool_desc);

    user(approval_prompt)
}

-- Parse the supervisor model response into a SupervisorDecision (pure)
--
-- Parameters:
--   response: ChatResponse from the supervisor model
--
-- Returns: SupervisorDecision indicating approval or rejection
pub fn parse_supervisor_response(response: ChatResponse) -> SupervisorDecision {
    match response {
        ChatResponse { content: None, tool_calls: _, finish_reason: _, usage: _, model: _, id: _ } => Reject { feedback: "No response from supervisor" },
        ChatResponse { content: Some { value: text }, tool_calls: _, finish_reason: _, usage: _, model: _, id: _ } => {
            let upper = string::trim(string::to_upper(text));
            if string::starts_with(upper, "APPROVE") then
                Approve
            else
                Reject { feedback: text }
        }
    }
}

-- Format tool calls for supervisor review
--
-- Parameters:
--   calls: List of tool calls to format
--
-- Returns: Formatted string description
pub fn format_tool_calls_for_review(calls: List<ToolCall>) -> String {
    let first_call = list::head(calls);
    match first_call {
        None => "No tool calls awaiting approval",
        Some { value: first } => {
            match first {
                ToolCall { id: call_id, name: tool_name, arguments: args } => {
                    let prefix = concat5(
                        "Tool calls awaiting approval: ",
                        string::to_string(list::len(calls)),
                        "\nFirst call id: ",
                        call_id,
                        "\nFirst call name: "
                    );
                    concat4(prefix, tool_name, "\nFirst call arguments: ", args)
                }
            }
        }
    }
}

-- Build a placeholder tool result message for the deferred reference loop
--
-- Parameters:
--   call: The tool call to describe
--
-- Returns: Tool result message text documenting what would execute
pub fn execute_tool_call(call: ToolCall) -> Message {
    match call {
        ToolCall { id: call_id, name: tool_name, arguments: args } => {
            -- Tool execution would happen here
            let result = concat4("Executed ", tool_name, " with args: ", args);
            tool_result(call_id, result)
        }
    }
}

-- Build placeholder result messages for multiple deferred tool calls
--
-- Parameters:
--   calls: List of tool calls to describe
--
-- Returns: List of tool result messages documenting what would execute
pub fn execute_tool_calls(calls: List<ToolCall>) -> List<Message> {
    list::map(calls, execute_tool_call)
}

-- Supervised agent helper
--
-- Like `tool_agent`, but the full reference behavior requires supervisor
-- approval before executing tools. The executable workflow body remains deferred
-- until the module checker supports this file's mixed type/helper/workflow shape
-- and `complete_with_tools` bridge in one module path.
--
-- Parameters:
--   provider: Provider name
--   model: Primary model identifier
--   tools: List of available tool definitions
--   supervisor_model: Model to use for supervision
--   history: Conversation history
--   user_message: The user message to process
--
-- Returns: placeholder ChatResponse marking supervised_agent as deferred
--
-- Example shape:
--   let response = supervised_agent(
--       "openai", "gpt-4o", [email_def], "claude-sonnet-4",
--       [], "Send confidential data to external@example.com"
--   );
--   -- Current implementation returns finish_reason "supervised_reference_only".
--   -- Full supervisor approval/rejection execution is reference behavior above.
pub fn supervised_agent(
    provider: String,
    model: String,
    tools: List<ToolDef>,
    supervisor_model: String,
    history: List<Message>,
    user_message: String
) -> ChatResponse {
    -- Phase 109 keeps this module checkable while the full supervised tool loop
    -- remains reference behavior above. The helper functions preserve the
    -- approval/rejection formatting contract; the public entry point returns an
    -- honest placeholder response until mixed ordinary-type declarations plus
    -- workflow definitions and `complete_with_tools` calls share one checkable
    -- module path.
    ChatResponse { content: Some { value: "supervised_agent deferred" }, tool_calls: None, finish_reason: Some { value: "supervised_reference_only" }, usage: None, model: model, id: "supervised_reference_only" }
}
