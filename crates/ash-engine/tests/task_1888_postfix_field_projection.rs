//! TASK-1888 regression coverage for postfix projection on ordinary primary expressions.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_postfix_projection_accepts_record_and_constructor_values() {
    let source = r"
        type Box = Box { item: Int };

        fn from_record() -> Int {
            { item: 20 }.item
        }

        fn from_constructor() -> Int {
            (Box { item: 21 }).item
        }

        fn main() -> Int {
            from_record() + from_constructor()
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
        .expect("postfix projection should execute");
    assert_eq!(result, Value::Int(41));
}
