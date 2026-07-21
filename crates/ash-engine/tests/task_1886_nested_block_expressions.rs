//! TASK-1886 regression coverage for ordinary nested block expressions.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_nested_blocks_sequence_expression_statements() {
    let source = r"
        fn touch() -> Int {
            1
        }

        fn main() -> Int {
            {
                let base = 40;
                {
                    touch();
                    1 + 1;
                    base + 1
                }
            }
        }
    ";

    let engine = engine();
    let mut application = engine.parse(source).expect("source should parse");
    engine
        .check(&mut application)
        .expect("source should typecheck");

    let result = engine
        .run(source)
        .await
        .expect("nested block expression statements should execute");
    assert_eq!(result, Value::Int(41));
}
