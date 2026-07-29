//! TASK-1890 regression coverage for tuple-payload ADTs in function-first Ash.

use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_match_accepts_tuple_constructor_expression_scrutinee_then_rejects_closed_admission()
 {
    let source = r#"
        type RuntimeError = RuntimeError(Int, String);

        fn main() -> Int {
            match RuntimeError(2, "missing config") {
                RuntimeError(code, message) => code,
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
        .expect_err("tuple constructor scrutinee match lacks validated typed lowering");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS pure ANF lowering accepts only typed atoms, approved integer binary primitives, and recursive Boolean Not",
        "tuple constructor scrutinee match must reject at the exact checked Core/CPS admission boundary"
    );
}
