//! TASK-1890 regression coverage for tuple-payload ADTs in function-first Ash.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_match_accepts_tuple_constructor_expression_scrutinee() {
    let source = r#"
        type RuntimeError = RuntimeError(Int, String);

        fn main() -> Int {
            match RuntimeError(2, "missing config") {
                RuntimeError(code, message) => code,
            }
        }
    "#;

    let engine = engine();
    let mut workflow = engine.parse(source).expect("source should parse");
    engine
        .check(&mut workflow)
        .expect("source should typecheck");

    let result = engine
        .run(source)
        .await
        .expect("tuple constructor scrutinee match should execute");
    assert_eq!(result, Value::Int(2));
}
