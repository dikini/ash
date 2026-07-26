//! TASK-1886 regression coverage for ordinary nested block expressions.

use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_nested_blocks_sequence_expression_statements_then_rejects_closed_admission()
{
    let source = r"
        fn touch() -> Int {
            1
        }

        fn main() -> Int {
            {
                let base = 40;
                {
                    touch();
                    1 + 1;
                    base + 1
                }
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
        .expect_err("nested block expressions lack validated typed lowering");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge accepts only atomic let values",
        "nested block expressions must reject at the exact checked Core/CPS admission boundary"
    );
}
