//! TASK-1884 regression coverage for ordinary expression statements in ambient do blocks.

use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn ambient_do_accepts_expression_statements_then_rejects_closed_admission() {
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
    let mut application = engine.parse(source).expect("source should parse");
    engine
        .check(&mut application)
        .expect("source should typecheck");

    let error = engine
        .run(source)
        .await
        .expect_err("ambient do statements lack validated typed lowering");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS pure ANF lowering accepts only typed atoms, approved integer binary primitives, and recursive Boolean Not",
        "ambient do statements must reject at the exact checked Core/CPS admission boundary"
    );
}
