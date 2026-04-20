//! Stream Storage for LLM Streaming
//!
//! Provides storage and retrieval of active streams for the `chat_stream` action.
//! This enables the runtime to pull chunks from a stream after it has been initiated.

use ash_core::Value;
use async_openai::types::CreateChatCompletionStreamResponse;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Type alias for the stream receiver
pub type StreamReceiver = mpsc::Receiver<StreamChunk>;

/// Storage for active LLM streams
///
/// Streams are stored with a unique ID and can be retrieved for chunk pulling.
#[derive(Debug)]
pub struct StreamStorage {
    /// Map of stream ID to receiver channel
    streams: Mutex<HashMap<String, Arc<Mutex<StreamReceiver>>>>,
}

impl StreamStorage {
    /// Create a new empty stream storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            streams: Mutex::new(HashMap::new()),
        }
    }

    /// Store a stream and return its ID
    ///
    /// # Arguments
    /// * `stream_id` - Unique identifier for this stream
    /// * `receiver` - Channel receiver for stream chunks
    ///
    pub fn store_stream(&self, stream_id: String, receiver: StreamReceiver) {
        let mut streams = self
            .streams
            .lock()
            .expect("stream_storage: store_stream lock poisoned");
        streams.insert(stream_id, Arc::new(Mutex::new(receiver)));
    }

    /// Pull the next chunk from a stored stream
    ///
    /// # Arguments
    /// * `stream_id` - ID of the stream to pull from
    ///
    /// # Returns
    /// * `Ok(Some(StreamChunk::Data(Value)))` - Next chunk as a `ChatChunk` value
    /// * `Ok(Some(StreamChunk::End))` - Stream has ended normally
    /// * `Ok(Some(StreamChunk::Error(msg)))` - Stream error occurred
    /// * `Ok(None)` - No chunk available yet, try again later
    /// * `Err(...)` - Stream not found or other error
    ///
    pub fn pull_chunk(&self, stream_id: &str) -> Result<Option<StreamChunk>, String> {
        let streams = self
            .streams
            .lock()
            .expect("stream_storage: pull_chunk streams lock poisoned");
        let receiver = streams
            .get(stream_id)
            .ok_or_else(|| format!("Stream '{stream_id}' not found"))?
            .clone();
        drop(streams); // Release lock before potentially blocking

        let mut receiver = receiver
            .lock()
            .expect("stream_storage: pull_chunk receiver lock poisoned");
        // Try to receive without blocking
        match receiver.try_recv() {
            Ok(chunk) => {
                // Remove stream if End or Error (terminal states)
                if matches!(chunk, StreamChunk::End | StreamChunk::Error(_)) {
                    drop(receiver);
                    let mut streams = self
                        .streams
                        .lock()
                        .expect("stream_storage: pull_chunk cleanup lock poisoned");
                    streams.remove(stream_id);
                }
                Ok(Some(chunk))
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // No chunk available yet - caller should try again later
                Ok(None)
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                // Channel disconnected - stream ended unexpectedly
                drop(receiver);
                self.streams
                    .lock()
                    .expect("stream_storage: pull_chunk disconnected cleanup lock poisoned")
                    .remove(stream_id);
                Ok(Some(StreamChunk::End))
            }
        }
    }

    /// Remove a stream from storage
    pub fn remove_stream(&self, stream_id: &str) {
        let mut streams = self
            .streams
            .lock()
            .expect("stream_storage: remove_stream lock poisoned");
        streams.remove(stream_id);
    }

    /// Check if a stream exists
    #[must_use]
    pub fn has_stream(&self, stream_id: &str) -> bool {
        let streams = self
            .streams
            .lock()
            .expect("stream_storage: has_stream lock poisoned");
        streams.contains_key(stream_id)
    }
}

impl Default for StreamStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Stream chunk result - distinguishes data, error, and end
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// A valid chunk with data
    Data(Value),
    /// Stream error occurred
    Error(String),
    /// Stream ended normally
    End,
}

/// Spawn a task to forward stream chunks to a channel
///
/// This bridges the async-openai stream to a channel that can be
/// consumed synchronously. Properly filters empty chunks and propagates errors.
pub fn spawn_stream_forwarder(
    mut stream: impl StreamExt<
        Item = Result<CreateChatCompletionStreamResponse, async_openai::error::OpenAIError>,
    > + Send
    + Unpin
    + 'static,
    sender: mpsc::Sender<StreamChunk>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    use crate::providers::llm::stream_adapter::stream_chunk_to_value;
                    if let Some(value) = stream_chunk_to_value(&chunk) {
                        // Valid chunk - send to consumer
                        if sender.send(StreamChunk::Data(value)).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                    // else: empty chunk filtered (keep-alive) - don't send anything
                }
                Err(e) => {
                    // Stream error - propagate to consumer
                    let _ = sender.send(StreamChunk::Error(e.to_string())).await;
                    break;
                }
            }
        }
        // Stream ended normally
        let _ = sender.send(StreamChunk::End).await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_storage_basic() {
        let storage = StreamStorage::new();
        let (tx, rx) = mpsc::channel(10);

        storage.store_stream("test-123".to_string(), rx);
        assert!(storage.has_stream("test-123"));

        // Send a data chunk
        let chunk = Value::String("test chunk".to_string());
        tx.try_send(StreamChunk::Data(chunk)).unwrap();

        // Pull the chunk
        let result = storage.pull_chunk("test-123");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Some(StreamChunk::Data(_))));

        // Pull again - should be None (no data available)
        let result = storage.pull_chunk("test-123");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_stream_storage_not_found() {
        let storage = StreamStorage::new();
        let result = storage.pull_chunk("non-existent");
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_storage_remove() {
        let storage = StreamStorage::new();
        let (_tx, rx) = mpsc::channel(10);

        storage.store_stream("test-456".to_string(), rx);
        assert!(storage.has_stream("test-456"));

        storage.remove_stream("test-456");
        assert!(!storage.has_stream("test-456"));
    }
}
