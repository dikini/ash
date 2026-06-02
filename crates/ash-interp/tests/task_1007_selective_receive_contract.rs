use std::sync::Arc;

use ash_core::stream::{MailboxEntry, Receive, ReceiveArm, ReceiveMode};
use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::execute_stream::execute_receive;
use ash_interp::mailbox::{Mailbox, SharedMailbox};
use ash_interp::policy::PolicyEvaluator;
use ash_interp::stream::StreamContext;

fn shared_mailbox(entries: Vec<MailboxEntry>) -> SharedMailbox {
    let mut mailbox = Mailbox::new();
    for entry in entries {
        mailbox.push(entry).expect("mailbox push should succeed");
    }
    Arc::new(tokio::sync::Mutex::new(mailbox))
}

fn literal_string(value: &str) -> Pattern {
    Pattern::Literal(Value::String(value.to_string()))
}

fn ret_int(value: i64) -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(value)),
    }
}

#[tokio::test]
async fn selective_receive_guard_order_no_match_behavior_preserved() {
    let ctx = Context::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let stream_ctx = StreamContext::new();
    let mailbox = shared_mailbox(vec![MailboxEntry::new(
        "sensor",
        "temp",
        Value::String("target".to_string()),
    )]);

    let guarded_receive = Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![
            ReceiveArm {
                pattern: literal_string("target"),
                guard: Some(Expr::Literal(Value::Bool(false))),
                body: ret_int(1),
            },
            ReceiveArm {
                pattern: literal_string("target"),
                guard: None,
                body: ret_int(2),
            },
            ReceiveArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: ret_int(3),
            },
        ],
        control_arms: None,
    };

    let result = execute_receive(
        &guarded_receive,
        ctx.clone(),
        mailbox.clone(),
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
    )
    .await
    .expect("receive should execute matching unguarded arm after guarded arm fails");
    assert_eq!(result, Value::Int(2));
    assert!(
        mailbox.lock().await.is_empty(),
        "matched message is consumed"
    );

    let no_match_mailbox = shared_mailbox(vec![MailboxEntry::new(
        "sensor",
        "temp",
        Value::String("other".to_string()),
    )]);
    let no_match_receive = Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![ReceiveArm {
            pattern: literal_string("target"),
            guard: None,
            body: ret_int(9),
        }],
        control_arms: None,
    };

    let no_match = execute_receive(
        &no_match_receive,
        ctx,
        no_match_mailbox.clone(),
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
    )
    .await
    .expect("selective receive with no matching arm should not be an exhaustiveness error");
    assert_eq!(no_match, Value::Null);
    assert_eq!(
        no_match_mailbox.lock().await.len(),
        1,
        "non-matching message remains buffered for a later selective receive"
    );
}
