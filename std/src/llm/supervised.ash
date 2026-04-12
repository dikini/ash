-- Supervised Agent Workflow (SPEC-029 §8.4)
--
-- Tool-using agent with supervisor approval. Before executing any tools,
-- the supervisor model reviews and approves or rejects the tool calls.
--
-- This provides a safety layer for high-stakes operations where tool
-- execution should be validated before proceeding.
--
-- Example:
--   let tools = [email_send_def, database_write_def];
--   let history = [system("You are an assistant that can send emails.")];
--   let response = supervised_agent(
--       "openai", "gpt-4o", tools, "claude-sonnet-4",
--       history, "Send an email to the team about the meeting"
--   );

use types::{Message, ToolDef, ChatResponse, ToolCall};
use prompt::{user, assistant, assistant_with_tools, tool_result, append};
use dispatch::complete;
use dispatch::complete_with_tools;

-- Supervisor decision type
-- Approve: Tool calls are approved for execution
-- Reject: Tool calls are rejected with feedback
type SupervisorDecision = Approve | Reject { feedback: String };

-- Request supervisor approval for tool calls
--
-- Parameters:
--   provider: Provider name for the supervisor model
--   supervisor_model: Model to use for supervision
--   messages: Current conversation context
--   tool_calls: The tool calls awaiting approval
--
-- Returns: SupervisorDecision indicating approval or rejection
fn request_approval(
    provider: String,
    supervisor_model: String,
    messages: List<Message>,
    tool_calls: List<ToolCall>
) -> SupervisorDecision {
    -- Build approval request prompt
    let tool_desc = format_tool_calls_for_review(tool_calls);
    let approval_prompt = string::concat(
        "You are a supervisor reviewing tool calls for safety and appropriateness.\n\n"
        "Review the following tool calls and respond with either:\n"
        "- APPROVE (if the tool calls are safe and appropriate)\n"
        "- REJECT: <reason> (if the tool calls should not be executed)\n\n"
        "Tool calls to review:\n",
        tool_desc
    );

    let approval_msg = user(approval_prompt);
    let approval_history = append(messages, approval_msg);

    let response = complete(provider, supervisor_model, approval_history, None);

    match response.content {
        None => Reject { feedback: "No response from supervisor" },
        Some { value: text } => {
            let upper = string::trim(string::to_uppercase(text));
            if string::starts_with(upper, "APPROVE") then
                Approve
            else if string::starts_with(upper, "REJECT") then {
                -- Extract feedback after REJECT:
                let feedback = if string::contains(text, ":") then
                    string::trim(string::slice(text, string::find(text, ":") + 1, string::length(text)))
                else
                    "Tool calls rejected by supervisor";
                Reject { feedback: feedback }
            } else
                -- Default to reject if response is unclear
                Reject { feedback: "Unclear supervisor response" }
        }
    }
}

-- Format tool calls for supervisor review
--
-- Parameters:
--   calls: List of tool calls to format
--
-- Returns: Formatted string description
fn format_tool_calls_for_review(calls: List<ToolCall>) -> String {
    let format_single = fn(call: ToolCall) -> String {
        match call {
            ToolCall { id: i, name: n, arguments: a } => {
                string::concat("  - ", n, " (call_id: ", i, "): ", a)
            }
        }
    };
    string::join("\n", list::map(calls, format_single))
}

-- Execute a single tool call and return result message
--
-- Parameters:
--   call: The tool call to execute
--
-- Returns: Tool result message
fn execute_tool_call(call: ToolCall) -> Message {
    match call {
        ToolCall { id: call_id, name: tool_name, arguments: args } => {
            -- Tool execution would happen here
            let result = string::concat("Executed ", tool_name, " with args: ", args);
            tool_result(call_id, result)
        }
    }
}

-- Execute multiple tool calls
--
-- Parameters:
--   calls: List of tool calls to execute
--
-- Returns: List of tool result messages
fn execute_tool_calls(calls: List<ToolCall>) -> List<Message> {
    list::map(calls, execute_tool_call)
}

-- Supervised agent workflow
--
-- Like `tool_agent`, but requires supervisor approval before executing tools.
-- The supervisor model reviews each set of tool calls and can approve or
-- reject them. If rejected, the rejection feedback is returned to the user.
--
-- Parameters:
--   provider: Provider name
--   model: Primary model identifier
--   tools: List of available tool definitions
--   supervisor_model: Model to use for supervision
--   history: Conversation history
--   user_message: The user message to process
--
-- Returns: ChatResponse with final answer or rejection feedback
--
-- Example:
--   let tools = [email_def];
--   let response = supervised_agent(
--       "openai", "gpt-4o", tools, "claude-sonnet-4",
--       [], "Send confidential data to external@example.com"
--   );
--   -- If supervisor rejects, response.content contains rejection feedback
workflow supervised_agent(
    provider: String,
    model: String,
    tools: List<ToolDef>,
    supervisor_model: String,
    history: List<Message>,
    user_message: String
) -> ChatResponse {
    -- Add user message to history
    let msg = user(user_message);
    let messages = append(history, msg);

    -- Main agent loop with supervision
    loop {
        -- Get model response with tool support
        let response = complete_with_tools(provider, model, messages, tools, None);

        -- Check if the model wants to use tools
        match response.tool_calls {
            None -> {
                -- No tool calls - this is the final answer
                break response
            },
            Some { value: calls } => {
                -- Model requested tool calls - get supervisor approval
                let decision = request_approval(provider, supervisor_model, messages, calls);

                match decision {
                    Approve -> {
                        -- Approved - execute tools and continue
                        let content = match response.content {
                            None -> "",
                            Some { value: c } => c
                        };
                        let messages = append(messages, assistant_with_tools(content, calls));
                        let tool_messages = execute_tool_calls(calls);
                        let messages = list::append(messages, tool_messages);
                        continue
                    },
                    Reject { feedback: reason } -> {
                        -- Rejected - return error response with feedback
                        let error_response = ChatResponse {
                            content: Some { value: string::concat("Tool execution rejected by supervisor: ", reason) },
                            tool_calls: None,
                            finish_reason: Some { value: "supervisor_rejection" },
                            usage: None,
                            model: model,
                            id: "supervised_rejection"
                        };
                        break error_response
                    }
                }
            }
        }
    }
}
