//! TASK-1880 regression coverage for ADT constructor expressions as match scrutinees.

use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_match_accepts_constructor_expression_scrutinee_then_rejects_closed_admission()
 {
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

    let error = engine
        .run(source)
        .await
        .expect_err("constructor scrutinee match lacks validated typed lowering");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS pure ANF lowering accepts only typed atoms, approved integer binary primitives, and recursive Boolean Not",
        "constructor scrutinee matches must reject at the exact checked Core/CPS admission boundary"
    );
}
