//! TASK-1884 regression coverage for ordinary expression statements in ambient do blocks.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn ambient_do_accepts_expression_statements_before_return() {
    let source = r"
        fn touch() -> Int {
            1
        }

        fn main() -> Int {
            do {
                touch();
                1 + 1;
                return 41;
            }
        }
    ";

    let engine = engine();
    let mut workflow = engine.parse(source).expect("source should parse");
    engine
        .check(&mut workflow)
        .expect("source should typecheck");

    let result = engine
        .run(source)
        .await
        .expect("ambient do expression statements should execute");
    assert_eq!(result, Value::Int(41));
}
