//! Regression coverage for canonical ambient `do` bindings during handler prevalidation.

use ash_engine::{Engine, Entry};

#[test]
fn ambient_do_plain_and_record_binds_check_through_handler_prewalk() {
    let source = r"
        fn helper(value: Int) -> Int { value + 1 }

        fn main() -> Int {
            do {
                value <- helper(41);
                record <- { answer: value };
                return value
            }
        }
    ";

    let engine = Engine::new().build().expect("engine builds");
    let mut entry: Entry = engine.parse(source).expect("source should parse");

    // `Engine::check` runs the handler-application prewalk before ordinary
    // function-body checking.  Ambient binds are plain values, not monadic
    // target values, so both forms must be admitted by that production path.
    engine
        .check(&mut entry)
        .expect("canonical ambient do binds should typecheck");
}
