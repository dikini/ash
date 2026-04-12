-- LLM Types (SPEC-029 §3)
--
-- Pure type definitions for LLM chat, embeddings, and tool use.
-- All types are immutable and use ADT variants for options and roles.

-- Role ADT for message roles
-- System: System/instruction messages
-- User: User input messages
-- Assistant: Model response messages
-- Tool: Tool result messages with associated tool call ID
pub type Role = System | User | Assistant | Tool { tool_call_id: String };

-- ToolCall represents a tool invocation from the model
-- id: Unique identifier for this tool call
-- name: Name of the tool to invoke
-- arguments: JSON string of arguments
pub type ToolCall = ToolCall {
    id: String,
    name: String,
    arguments: String
};

-- ToolCallDelta represents a partial tool call in streaming responses
-- index: Position in the tool_calls array
-- id: Optional tool call ID (may be null in early chunks)
-- name: Optional function name (may be null in early chunks)
-- arguments: Optional partial JSON arguments
pub type ToolCallDelta = ToolCallDelta {
    index: Int,
    id: Option<String>,
    name: Option<String>,
    arguments: Option<String>
};

-- Message represents a chat message in a conversation
-- role: The role of the message sender
-- content: The text content (may be empty for tool calls)
-- tool_calls: Optional list of tool calls (for assistant messages)
-- tool_call_id: Optional tool call ID (for tool messages)
pub type Message = Message {
    role: Role,
    content: String,
    tool_calls: Option<List<ToolCall>>,
    tool_call_id: Option<String>
};

-- ToolDef defines a tool available to the model
-- name: Tool identifier
-- description: Human-readable description
-- parameters: JSON Schema string for parameters
pub type ToolDef = ToolDef {
    name: String,
    description: String,
    parameters: String
};

-- Usage tracks token consumption
-- prompt_tokens: Tokens in the prompt
-- completion_tokens: Tokens in the completion
-- total_tokens: Total tokens used
pub type Usage = Usage {
    prompt_tokens: Int,
    completion_tokens: Int,
    total_tokens: Int
};

-- ChatResponse represents a complete chat completion response
-- content: The response text (None if tool calls only)
-- tool_calls: Optional list of tool calls
-- finish_reason: Why the model stopped (e.g., "stop", "tool_calls")
-- usage: Token usage statistics
-- model: Model identifier string
-- id: Response ID
pub type ChatResponse = ChatResponse {
    content: Option<String>,
    tool_calls: Option<List<ToolCall>>,
    finish_reason: Option<String>,
    usage: Option<Usage>,
    model: String,
    id: String
};

-- Embedding represents a vector embedding
-- index: Position in the response array
-- embedding: The embedding vector (list of floats)
pub type Embedding = Embedding {
    index: Int,
    embedding: List<Float>
};

-- ChatChunk represents a streaming response chunk
-- delta_content: Incremental content (if present)
-- delta_tool_calls: Incremental tool call data (if present)
-- finish_reason: Final chunk reason (if stream ended)
pub type ChatChunk = ChatChunk {
    delta_content: Option<String>,
    delta_tool_calls: Option<List<ToolCallDelta>>,
    finish_reason: Option<String>
};

-- ChatOptions configures chat completion requests
-- model: Model to use
-- temperature: Sampling temperature (0.0 to 2.0)
-- max_tokens: Maximum tokens to generate
-- tools: Optional list of available tools
-- stream: Whether to stream the response
pub type ChatOptions = ChatOptions {
    model: String,
    temperature: Option<Float>,
    max_tokens: Option<Int>,
    tools: Option<List<ToolDef>>,
    stream: Bool
};

-- EmbeddingOptions configures embedding requests
-- model: Model to use
-- encoding_format: Format for embedding values
pub type EmbeddingOptions = EmbeddingOptions {
    model: String,
    encoding_format: Option<String>
};
