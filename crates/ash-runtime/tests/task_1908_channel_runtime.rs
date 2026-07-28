use std::sync::Arc;

use ash_core::{EnvFrame, Expr, SendabilityRejection, Value};
use ash_runtime::{ChannelError, RuntimeState};
use ash_typeck::{Type, TypeVar};

#[tokio::test]
async fn typed_channel_send_receive_moves_owned_values_in_fifo_order() {
    let runtime_state = RuntimeState::new();
    let channel = runtime_state.create_channel(Type::Int, 2).await;

    runtime_state
        .send_channel(channel, Value::Int(1))
        .await
        .expect("first send succeeds");
    runtime_state
        .send_channel(channel, Value::Int(2))
        .await
        .expect("second send succeeds");

    assert_eq!(
        runtime_state
            .try_receive_channel(channel)
            .await
            .expect("first receive succeeds"),
        Value::Int(1)
    );
    assert_eq!(
        runtime_state
            .try_receive_channel(channel)
            .await
            .expect("second receive succeeds"),
        Value::Int(2)
    );
}

#[tokio::test]
async fn channel_send_rejects_type_mismatch_and_non_sendable_values() {
    let runtime_state = RuntimeState::new();
    let channel = runtime_state.create_channel(Type::Int, 1).await;

    assert_eq!(
        runtime_state
            .send_channel(channel, Value::String("not int".to_string()))
            .await,
        Err(ChannelError::TypeMismatch {
            channel_id: channel,
            expected: "Int".to_string(),
            actual: "String".to_string(),
        })
    );

    let closure = Value::Closure {
        params: vec![],
        body: Box::new(Expr::Literal(Value::Null)),
        env: Arc::new(EnvFrame::new()),
    };
    let closure_channel = runtime_state.create_channel(Type::Var(TypeVar(0)), 1).await;

    assert_eq!(
        runtime_state.send_channel(closure_channel, closure).await,
        Err(ChannelError::NonSendable {
            channel_id: closure_channel,
            reason: SendabilityRejection::Closure,
        })
    );
}

#[tokio::test]
async fn channel_receive_and_send_report_empty_full_and_closed_states() {
    let runtime_state = RuntimeState::new();
    let channel = runtime_state.create_channel(Type::String, 1).await;

    assert_eq!(
        runtime_state.try_receive_channel(channel).await,
        Err(ChannelError::Empty {
            channel_id: channel,
        })
    );

    runtime_state
        .send_channel(channel, Value::String("first".to_string()))
        .await
        .expect("first send fills channel");
    assert_eq!(
        runtime_state
            .send_channel(channel, Value::String("second".to_string()))
            .await,
        Err(ChannelError::Full {
            channel_id: channel,
            capacity: 1,
        })
    );

    runtime_state
        .close_channel(channel)
        .await
        .expect("close succeeds");

    assert_eq!(
        runtime_state
            .send_channel(channel, Value::String("after close".to_string()))
            .await,
        Err(ChannelError::Closed {
            channel_id: channel,
        })
    );
}

#[tokio::test]
async fn channel_select_reports_unsupported_multi_channel_cases() {
    let runtime_state = RuntimeState::new();
    let first = runtime_state.create_channel(Type::Int, 1).await;
    let second = runtime_state.create_channel(Type::Int, 1).await;

    assert_eq!(
        runtime_state.select_ready_channel(&[first, second]).await,
        Err(ChannelError::UnsupportedSelect {
            reason: "multi-channel select is not supported by the bounded channel runtime yet"
                .to_string(),
        })
    );
}
