-- Tool Agent Helper (SPEC-029 §8.2)
--
-- Agent helper that can use tools to accomplish tasks. Maintains a loop
-- where the model may request tool executions, which are performed and
-- the results fed back to the model.
--
-- The loop continues until the model responds without tool calls (final
-- answer) or an error occurs.
--
-- Example:
--   let tools = [calculator_def, search_def];
--   let history = [system("You are a helpful assistant with tools.")];
--   let response = tool_agent("openai", "gpt-4o", tools, history, "What is 15 * 23?");

use types::{Message, ToolDef, ChatResponse, ToolCall};
use prompt::{user, assistant_with_tools, tool_result, append};
use dispatch::complete_with_tools;

-- Execute a tool call and return the result message
--
-- This is a placeholder for the actual tool execution mechanism.
-- In a real implementation, this would dispatch to registered tools.
--
-- Parameters:
--   call: The tool call to execute
--
-- Returns: Tool result message
pub fn execute_tool_call(call: ToolCall) -> Message {
    match call {
        ToolCall { id: call_id, name: tool_name, arguments: args } => {
            -- Tool execution would happen here
            -- For now, return a placeholder result
            let result = string::concat("Executed ", tool_name, " with args: ", args);
            tool_result(call_id, result)
        }
    }
}

-- Execute multiple tool calls and return result messages
--
-- Parameters:
--   calls: List of tool calls to execute
--
-- Returns: List of tool result messages
pub fn execute_tool_calls(calls: List<ToolCall>) -> List<Message> {
    list::map(calls, execute_tool_call)
}

-- Tool-using agent helper
--
-- Maintains a conversation loop where the model can request tool executions.
-- Tool results are fed back to the model, which may request additional tools
-- or provide a final answer.
--
-- Parameters:
--   provider: Provider name (e.g., "openai", "ollama")
--   model: Model identifier (e.g., "gpt-4o")
--   tools: List of available tool definitions
--   history: Conversation history
--   user_message: The user message to process
--
-- Returns: ChatResponse with the final model response
--
-- Example:
--   let calc_tool = ToolDef {
--       name: "add",
--       description: "Add two numbers",
--       parameters: "{\"type\": \"object\", \"properties\": {\"a\": {\"type\": \"number\"}, \"b\": {\"type\": \"number\"}}}"
--   };
--   let response = tool_agent("openai", "gpt-4o", [calc_tool], [], "What is 5 + 3?");
pub fn tool_agent(
    provider: String,
    model: String,
    tools: List<ToolDef>,
    history: List<Message>,
    user_message: String
) -> ChatResponse {
    -- Add user message to history
    let msg = user(user_message);
    let messages = append(history, msg);

    complete_with_tools(provider, model, messages, tools, None)
}
