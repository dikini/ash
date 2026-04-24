//! Tests for builtin fn declaration parsing (TASK-615).

use ash_parser::input::new_input;
use ash_parser::parse_module::parse_builtin_fn_definition;
use ash_parser::surface::{Definition, Visibility};

// ---------------------------------------------------------------------------
// Helper: parse a builtin fn definition from source text
// ---------------------------------------------------------------------------
fn parse_builtin(input_str: &str) -> Definition {
    let mut input = new_input(input_str);
    parse_builtin_fn_definition(&mut input).expect("builtin fn definition should parse")
}

fn parse_builtin_err(input_str: &str) -> winnow::error::ErrMode<winnow::error::ContextError> {
    let mut input = new_input(input_str);
    parse_builtin_fn_definition(&mut input).expect_err("builtin fn should fail to parse")
}

// ---------------------------------------------------------------------------
// 1. Simple builtin fn
// ---------------------------------------------------------------------------
#[test]
fn parse_simple_builtin_fn() {
    let def = parse_builtin("builtin fn foo(x: Int) -> Int;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "foo");
    assert_eq!(f.params.len(), 1);
    assert_eq!(f.params[0].name.as_ref(), "x");
}

// ---------------------------------------------------------------------------
// 2. pub builtin fn
// ---------------------------------------------------------------------------
#[test]
fn parse_pub_builtin_fn() {
    let def = parse_builtin("pub builtin fn bar(s: String) -> Bool;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "bar");
    assert_eq!(f.params.len(), 1);
    assert!(
        !matches!(f.visibility, Visibility::Inherited),
        "expected pub visibility"
    );
}

// ---------------------------------------------------------------------------
// 3. builtin fn with type params
// ---------------------------------------------------------------------------
#[test]
fn parse_builtin_fn_with_type_params() {
    let def = parse_builtin("builtin fn with_type_params<T>(x: T) -> T;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "with_type_params");
    assert_eq!(f.type_params.len(), 1);
    assert_eq!(f.type_params[0].as_ref(), "T");
}

// ---------------------------------------------------------------------------
// 4. builtin fn with multiple params
// ---------------------------------------------------------------------------
#[test]
fn parse_builtin_fn_multiple_params() {
    let def = parse_builtin("builtin fn add(a: Int, b: Int) -> Int;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.as_ref(), "a");
    assert_eq!(f.params[1].name.as_ref(), "b");
}

// ---------------------------------------------------------------------------
// 5. builtin fn with no params
// ---------------------------------------------------------------------------
#[test]
fn parse_pub_builtin_fn_with_keyword_name_then() {
    let def = parse_builtin("pub builtin fn then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B>;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "then");
    assert_eq!(f.params.len(), 2);
}

#[test]
fn parse_pub_builtin_fn_with_keyword_name_guard() {
    let def = parse_builtin("pub builtin fn guard<A>(p: Policy, ma: Act<A>) -> Act<A>;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "guard");
    assert_eq!(f.params.len(), 2);
}

#[test]
fn parse_builtin_fn_no_params() {
    let def = parse_builtin("builtin fn magic() -> Int;");
    let Definition::BuiltinFn(f) = def else {
        panic!("expected BuiltinFn definition, got: {def:?}");
    };
    assert_eq!(f.name.as_ref(), "magic");
    assert_eq!(f.params.len(), 0);
}

// ---------------------------------------------------------------------------
// 6. Reject: builtin fn with body (braces) after return type
// ---------------------------------------------------------------------------
#[test]
fn reject_builtin_fn_with_body() {
    let err = parse_builtin_err("builtin fn foo(x: Int) -> Int { x }");
    assert!(
        matches!(err, winnow::error::ErrMode::Cut(_)),
        "expected Cut (unrecoverable) error for builtin fn with body, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// 6b. Reject: builtin fn with body (braces) in place of return type
// ---------------------------------------------------------------------------
#[test]
fn reject_builtin_fn_with_body_no_return_type() {
    let err = parse_builtin_err("builtin fn foo(x: Int) { x }");
    assert!(
        matches!(err, winnow::error::ErrMode::Cut(_)),
        "expected Cut (unrecoverable) error for builtin fn with body (no return type), got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// 7. Reject: builtin fn without return type
// ---------------------------------------------------------------------------
#[test]
fn reject_builtin_fn_without_return_type() {
    let err = parse_builtin_err("builtin fn foo(x: Int);");
    assert!(
        matches!(err, winnow::error::ErrMode::Cut(_)),
        "expected Cut (unrecoverable) error for builtin fn without return type, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// 8. Module-level integration: builtin fn parsed via module_file
// ---------------------------------------------------------------------------
#[test]
fn builtin_fn_in_module_file() {
    use ash_parser::parse_module::module_file;
    let mut input = new_input("builtin fn foo(x: Int) -> Int;");
    let result = module_file(&mut input).expect("module with builtin fn should parse");
    assert_eq!(result.definitions.len(), 1);
    assert!(
        matches!(result.definitions[0], Definition::BuiltinFn(ref f) if f.name.as_ref() == "foo"),
        "expected BuiltinFn(foo), got: {:?}",
        result.definitions[0]
    );
}

// ---------------------------------------------------------------------------
// 9. Module-level integration: pub builtin fn parsed via module_file
// ---------------------------------------------------------------------------
#[test]
fn pub_builtin_fn_in_module_file() {
    use ash_parser::parse_module::module_file;
    let mut input = new_input("pub builtin fn bar(s: String) -> Bool;");
    let result = module_file(&mut input).expect("module with pub builtin fn should parse");
    assert_eq!(result.definitions.len(), 1);
    assert!(
        matches!(result.definitions[0], Definition::BuiltinFn(ref f) if f.name.as_ref() == "bar"),
        "expected BuiltinFn(bar), got: {:?}",
        result.definitions[0]
    );
}

// ---------------------------------------------------------------------------
// 10. Inline module: builtin fn parsed inside mod { ... }
// ---------------------------------------------------------------------------
#[test]
fn builtin_fn_in_inline_module() {
    use ash_parser::module::ModuleSource;
    use ash_parser::parse_module::parse_module_decl;
    let mut input = new_input("mod mymod { builtin fn foo(x: Int) -> Int; }");
    let result = parse_module_decl(&mut input).expect("inline module with builtin fn should parse");
    let definitions = match result.source {
        ModuleSource::Inline(defs) => defs,
        ModuleSource::File => panic!("expected inline module"),
    };
    assert_eq!(definitions.len(), 1);
    assert!(matches!(definitions[0], Definition::BuiltinFn(ref f) if f.name.as_ref() == "foo"),);
}
