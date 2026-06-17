use ash_parser::parse_pattern::pattern;
use ash_parser::surface::Pattern;
use ash_parser::test_input;

#[test]
fn test_parse_variant_record_pattern_direct() {
    let mut input = test_input("Cons { head: h, tail: rest }");
    let result = pattern(&mut input);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "variant record pattern should parse: {:?}", result.err());
}

#[test]
fn test_parse_list_pattern_direct() {
    let mut input = test_input("[h, ..rest]");
    let result = pattern(&mut input);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "list pattern should parse: {:?}", result.err());
}

#[test]
fn test_parse_empty_list_pattern_direct() {
    let mut input = test_input("[]");
    let result = pattern(&mut input);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "empty list pattern should parse: {:?}", result.err());
}
