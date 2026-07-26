//! TASK-1888 regression coverage for postfix projection on ordinary primary expressions.

use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_postfix_projection_accepts_record_and_constructor_values_then_rejects_closed_admission()
 {
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
    let mut application = engine.parse(source).expect("source should parse");
    engine
        .check(&mut application)
        .expect("source should typecheck");

    let error = engine
        .run(source)
        .await
        .expect_err("postfix projection lacks validated typed lowering");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge accepts only atomic let values",
        "postfix projection must reject at the exact checked Core/CPS admission boundary"
    );
}
