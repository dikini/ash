//! Phase 201 engine gates for removed source forms.

use ash_core::Value;
use ash_engine::Engine;

fn removed_keyword(parts: &[&str]) -> String {
    parts.concat()
}

#[tokio::test]
async fn target_fn_main_still_checks_and_runs() {
    let engine = Engine::new().build().expect("engine builds");
    let source = "fn main() -> Int { 7 }\n";

    let mut application = engine.parse(source).expect("target fn main should parse");
    engine
        .check(&mut application)
        .expect("target fn main should check");
    let result = engine.run(source).await.expect("target fn main should run");

    assert_eq!(result, Value::Int(7));
}

#[test]
fn removed_entry_keyword_is_rejected_by_engine_parse() {
    let engine = Engine::new().build().expect("engine builds");
    let keyword = removed_keyword(&["work", "flow"]);
    let source = format!("{keyword} main {{ return 0 }}");

    let error = engine
        .parse(&source)
        .expect_err("removed entry keyword must not parse through engine");
    let text = error.to_string();

    assert!(text.contains("parse"), "{text}");
}

#[test]
fn removed_capability_interface_is_rejected_by_engine_parse() {
    let engine = Engine::new().build().expect("engine builds");
    let keyword = removed_keyword(&["cap", "ability"]);
    let source = format!("pub {keyword} interface Sensor:\n  observe read() returns Int;");

    let error = engine
        .parse(&source)
        .expect_err("removed capability interface syntax must not parse through engine");
    let text = error.to_string();

    assert!(text.contains("parse"), "{text}");
}

#[test]
fn removed_capability_implementation_is_rejected_by_engine_parse() {
    let engine = Engine::new().build().expect("engine builds");
    let keyword = removed_keyword(&["cap", "ability"]);
    let source = format!("{keyword} impl Noop for Sensor {{ observe read() returns Int {{ 0 }} }}");

    let error = engine
        .parse(&source)
        .expect_err("removed capability implementation syntax must not parse through engine");
    let text = error.to_string();

    assert!(text.contains("parse"), "{text}");
}
