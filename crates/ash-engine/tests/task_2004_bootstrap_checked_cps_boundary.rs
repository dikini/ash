//! TASK-2004 RED contracts for canonical bootstrap through checked Core/CPS.

use ash_core::{Value, adt::tuple_field_name};
use ash_engine::{Engine, EntryBootstrapError};

const RUNTIME_ERROR_ENTRY: &str = r#"
use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> {
    Err { error: RuntimeError(42, "boom") }
}
"#;

#[tokio::test]
async fn bootstrap_admits_a_canonical_runtime_error_entry_through_checked_cps() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .bootstrap_entry_source_result(RUNTIME_ERROR_ENTRY)
        .await
        .expect("the canonical zero-input entry must execute through checked Core/CPS admission");

    assert_eq!(result.exit_code, 42);
    assert_eq!(
        result.terminal_value,
        Value::Variant {
            name: "Err".to_string(),
            fields: Box::new(vec![(
                "error".to_string(),
                Value::Variant {
                    name: "RuntimeError".to_string(),
                    fields: Box::new(vec![
                        (tuple_field_name(0), Value::Int(42)),
                        (tuple_field_name(1), Value::String("boom".to_string())),
                    ]),
                },
            )]),
        }
    );
}

#[tokio::test]
async fn bootstrap_rejects_unsupported_nested_lowering_before_direct_evaluation() {
    let engine = Engine::new().build().expect("engine builds");

    let error = engine
        .bootstrap_entry_source_result(
            r#"
            use result::Result
            use runtime::RuntimeError

            fn main() -> Result<(), RuntimeError> {
                Err { error: RuntimeError((1 + 2) + 3, "boom") }
            }
            "#,
        )
        .await
        .expect_err("unsupported nested bootstrap lowering must reject before direct evaluation");

    assert!(matches!(
        error,
        EntryBootstrapError::Execution(ref message) if message.contains("checked Core/CPS admission")
    ));
}
