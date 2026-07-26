//! TASK-1878 regression coverage for structural record expressions in function-first Ash.

use ash_engine::Engine;

const CLOSED_ADMISSION_ATOMIC_LET_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge accepts only atomic let values";

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_source_checks_record_projection_then_rejects_closed_admission() {
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

    let error = engine
        .run(source)
        .await
        .expect_err("record expression source lacks validated typed Core/CPS lowering");
    assert_eq!(
        error.to_string(),
        CLOSED_ADMISSION_ATOMIC_LET_ERROR,
        "record expression source must reject at the exact checked Core/CPS admission boundary"
    );
}
