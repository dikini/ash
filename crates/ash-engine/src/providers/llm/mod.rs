//! LLM Capability Provider
//!
//! Provides OpenAI-compatible LLM communication through the `async-openai` crate.
//! Supports multi-provider routing, chat completions, streaming, embeddings, and tool use.

pub mod chat;
pub mod config;
pub mod embeddings;
pub mod error;
pub mod models;
pub mod provider;
pub mod stream_adapter;
pub mod stream_storage;
pub mod tool_dispatch;

pub use config::LlmConfig;
pub use provider::LlmProvider;
pub use stream_adapter::{extract_chat_stream_args, stream_chunk_to_value, ChatStream};
pub use stream_storage::StreamStorage;
pub use tool_dispatch::{extract_tool_calls, format_tool_result_message, tool_defs_to_openai_tools, ToolCallValue};
