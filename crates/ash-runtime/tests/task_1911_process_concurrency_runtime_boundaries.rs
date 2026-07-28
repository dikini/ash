//! TASK-1911 runtime process/channel boundary fixtures.

use ash_core::{ProcessHandle, ProcessId, SendabilityRejection, Value};
use ash_runtime::{ChannelError, RuntimeState};
use ash_typeck::{Type, TypeVar};

#[tokio::test]
async fn channel_can_transfer_unconsumed_process_handles_but_rejects_consumed_handles() {
    let runtime_state = RuntimeState::new();
    let channel = runtime_state.create_channel(Type::Var(TypeVar(0)), 2).await;
    let process_id = ProcessId::new();
    let sendable_handle = ProcessHandle::new(process_id, Some("Int".to_string()));

    runtime_state
        .send_channel(channel, Value::ProcessHandle(sendable_handle.clone()))
        .await
        .expect("unconsumed process handle is sendable");
    assert_eq!(
        runtime_state
            .try_receive_channel(channel)
            .await
            .expect("process handle receives"),
        Value::ProcessHandle(sendable_handle)
    );

    let consumed_handle = ProcessHandle::new(process_id, Some("Int".to_string()));
    assert!(
        consumed_handle.try_consume(),
        "test fixture consumes handle"
    );
    assert_eq!(
        runtime_state
            .send_channel(channel, Value::ProcessHandle(consumed_handle))
            .await,
        Err(ChannelError::NonSendable {
            channel_id: channel,
            reason: SendabilityRejection::ConsumedProcessHandle { process_id },
        })
    );
}
