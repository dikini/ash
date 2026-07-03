//! TASK-1882 regression coverage for ordinary expressions as match scrutinees.

use ash_core::Value;
use ash_engine::Engine;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

#[tokio::test]
async fn function_first_match_accepts_call_field_and_binary_scrutinees() {
    let source = r"
        type Option<T> = Some { value: T } | None;
        type Box = Box { item: Int };

        fn make() -> Option<Int> {
            Some { value: 41 }
        }

        fn from_call() -> Int {
            match make() {
                Some { value: value } => value,
                None => 0,
            }
        }

        fn from_field() -> Int {
            let holder = { inner: Box { item: 41 } };
            match holder.inner {
                Box { item: item } => item,
            }
        }

        fn from_binary() -> Int {
            match 40 + 1 {
                41 => 1,
                _ => 0,
            }
        }

        fn main() -> Int {
            from_call() + from_field() + from_binary()
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
        .expect("ordinary scrutinee matches should execute");
    assert_eq!(result, Value::Int(83));
}
