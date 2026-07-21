//! TASK-1878 regression coverage for structural record expressions in function-first Ash.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_source_executes_structural_record_expression_projection() {
    let source = r#"
        fn main() -> Int {
            do {
                person <- { name: "Ada", age: 41 };
                return person.age;
            }
        }
    "#;

    let engine = engine();
    let mut application = engine.parse(source).expect("source should parse");
    engine
        .check(&mut application)
        .expect("source should typecheck");

    let result = engine
        .run(source)
        .await
        .expect("record expression source should execute");
    assert_eq!(result, Value::Int(41));
}
