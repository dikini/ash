//! TASK-960 typechecker coverage for fail-closed reserved callable arrows.

use ash_parser::surface::Definition;
use ash_typeck::types::Type;
use ash_typeck::{TypeEnv, fn_signature_type};

fn parse_error_text(source: &str) -> String {
    ash_parser::parse_surface_file(source)
        .expect_err("reserved callable syntax should be rejected before typechecking")
        .into_iter()
        .map(|err| err.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn function_signature(source: &str) -> Type {
    let module = ash_parser::parse_surface_file(source).expect("function should parse");
    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected function definition");
    };
    fn_signature_type(&TypeEnv::with_builtin_types(), function)
        .expect("function signature should typecheck")
}

#[test]
fn reserved_type_arrows_do_not_lower_to_type_fn_or_type_fun() {
    for (arrow, stratum) in [("-*>", "Act"), ("=>", "Proc"), ("=*>", "Workflow")] {
        let text = parse_error_text(&format!("type Handler = (Int) {arrow} Bool;"));

        assert!(
            text.contains(&format!("{stratum} callable syntax is reserved")),
            "reserved type arrow must fail before lowering to Type::Fn or Type::Fun, got:\n{text}"
        );
    }
}

#[test]
fn reserved_closure_arrows_do_not_typecheck_as_pure_or_effect_closures() {
    for (arrow, stratum) in [("-*>", "Act"), ("=>", "Proc"), ("=*>", "Workflow")] {
        for source in [
            format!("fn bad() -> Int {{ let f = |x: Int| {arrow} {{ x }}; 0 }}"),
            format!("fn bad() -> Int {{ apply(|x: Int| {arrow} {{ x }}) }}"),
            format!("fn bad() -> Int {{ |x: Int| {arrow} {{ x }} }}"),
        ] {
            let text = parse_error_text(&source);

            assert!(
                text.contains(&format!("{stratum} closures are reserved")),
                "reserved closure arrow must fail before Type::Fn/Type::Fun inference, got:\n{text}"
            );
        }
    }
}

#[test]
fn smart_constructor_returning_workflow_remains_pure_callable() {
    let signature = function_signature("fn build(spec: Int) -> Workflow<Int> { spec }");

    let Type::Fn(params, ret) = signature else {
        panic!("pure smart constructor must remain Type::Fn, got {signature:?}");
    };

    assert_eq!(params, vec![Type::Int]);
    assert!(
        matches!(
            ret.as_ref(),
            Type::Constructor { name, args, .. }
                if name.name == "Workflow" && args == &vec![Type::Int]
        ),
        "expected pure callable returning Workflow<Int>, got {ret:?}"
    );
}
