//! TASK-1880 regression coverage for ADT constructor expressions as match scrutinees.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_match_accepts_constructor_expression_scrutinee() {
    let source = r"
        type Option<T> = Some { value: T } | None;

        fn main() -> Int {
            match Some { value: 41 } {
                Some { value: value } => value,
                None => 0,
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
        .expect("constructor scrutinee match should execute");
    assert_eq!(result, Value::Int(41));
}
