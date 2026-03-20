use ash_parser::input::new_input;
use ash_parser::parse_workflow::workflow;
use ash_parser::surface::{Pattern, ReceiveMode, StreamPattern, Workflow};

#[test]
fn main_workflow_parser_accepts_canonical_receive() {
    let mut input = new_input("receive { sensor:temp as reading => done }");
    let parsed = workflow(&mut input).expect("receive should parse through main workflow parser");

    match parsed {
        Workflow::Receive {
            mode,
            is_control,
            arms,
            ..
        } => {
            assert_eq!(mode, ReceiveMode::NonBlocking);
            assert!(!is_control);
            assert_eq!(arms.len(), 1);
            assert!(matches!(
                arms[0].pattern,
                StreamPattern::Binding {
                    ref capability,
                    ref channel,
                    pattern: Pattern::Variable(ref name),
                } if capability.as_ref() == "sensor"
                    && channel.as_ref() == "temp"
                    && name.as_ref() == "reading"
            ));
        }
        other => panic!("expected receive workflow, got {other:?}"),
    }
}

#[test]
fn main_workflow_parser_accepts_control_receive() {
    let mut input = new_input("receive control wait 30s { \"shutdown\" => done, _ => done }");
    let parsed =
        workflow(&mut input).expect("control receive should parse through main workflow parser");

    match parsed {
        Workflow::Receive {
            mode,
            is_control,
            arms,
            ..
        } => {
            assert_eq!(
                mode,
                ReceiveMode::Blocking(Some(std::time::Duration::from_secs(30)))
            );
            assert!(is_control);
            assert_eq!(arms.len(), 2);
            assert!(
                matches!(arms[0].pattern, StreamPattern::Literal(ref value) if value == "shutdown")
            );
            assert!(matches!(arms[1].pattern, StreamPattern::Wildcard));
        }
        other => panic!("expected receive workflow, got {other:?}"),
    }
}
