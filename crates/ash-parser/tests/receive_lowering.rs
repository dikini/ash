use ash_core::{
    Pattern as CorePattern, ReceiveMode as CoreReceiveMode, ReceivePattern,
    Workflow as CoreWorkflow,
};
use ash_parser::input::new_input;
use ash_parser::lower::lower_workflow;
use ash_parser::parse_workflow::workflow_def;

#[test]
fn receive_lowering_preserves_mode_and_arm_structure() {
    let mut input = new_input(
        "workflow main { receive wait 30s { sensor:temp as reading if ready => done, _ => done } }",
    );
    let surface = workflow_def(&mut input).expect("workflow should parse");
    let lowered = lower_workflow(&surface);

    match lowered {
        CoreWorkflow::Receive {
            mode,
            arms,
            control,
        } => {
            assert_eq!(
                mode,
                CoreReceiveMode::Blocking(Some(std::time::Duration::from_secs(30)))
            );
            assert!(!control);
            assert_eq!(arms.len(), 2);
            assert!(matches!(
                arms[0].pattern,
                ReceivePattern::Stream {
                    ref capability,
                    ref channel,
                    pattern: CorePattern::Variable(ref name),
                } if capability == "sensor" && channel == "temp" && name == "reading"
            ));
            assert!(arms[0].guard.is_some());
            assert!(matches!(arms[1].pattern, ReceivePattern::Wildcard));
        }
        other => panic!("expected lowered receive workflow, got {other:?}"),
    }
}

#[test]
fn control_receive_lowering_preserves_literal_patterns() {
    let mut input = new_input("workflow main { receive control { \"shutdown\" => done } }");
    let surface = workflow_def(&mut input).expect("workflow should parse");
    let lowered = lower_workflow(&surface);

    match lowered {
        CoreWorkflow::Receive {
            mode,
            arms,
            control,
        } => {
            assert_eq!(mode, CoreReceiveMode::NonBlocking);
            assert!(control);
            assert_eq!(arms.len(), 1);
            assert!(matches!(
                arms[0].pattern,
                ReceivePattern::Literal(ash_core::Value::String(ref value)) if value == "shutdown"
            ));
        }
        other => panic!("expected lowered receive workflow, got {other:?}"),
    }
}
