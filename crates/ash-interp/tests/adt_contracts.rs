use ash_core::{Pattern, Value};
use ash_interp::pattern::match_pattern;

#[test]
fn variant_patterns_match_constructor_shaped_values() {
    let pattern = Pattern::Variant {
        name: "Some".into(),
        fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
    };
    let value = Value::variant("Some", vec![("value", Value::Int(42))]);

    let bindings = match_pattern(&pattern, &value).unwrap();

    assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
}

#[test]
fn variant_patterns_do_not_accept_tagged_record_encodings() {
    let pattern = Pattern::Variant {
        name: "Some".into(),
        fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
    };
    let value = Value::Record(Box::new(std::collections::HashMap::from([
        ("__variant".to_string(), Value::String("Some".to_string())),
        ("value".to_string(), Value::Int(42)),
    ])));

    assert!(match_pattern(&pattern, &value).is_err());
}
