use ash_parser::{input::new_input, workflow_def};
use ash_typeck::{Type, TypeVar, type_check_workflow};

#[test]
fn runtime_args_surface_parses_and_typechecks() {
    let source = r#"
        workflow main(args: cap Args) {
            observe Args 0;
            done;
        }
    "#;

    let mut input = new_input(source);
    let def = workflow_def(&mut input).expect("Args entry workflow should parse");

    assert_eq!(def.params.len(), 1);
    let bindings = vec![(def.params[0].name.to_string(), Type::Var(TypeVar::fresh()))];

    let result = type_check_workflow(&def.body, Some(&bindings))
        .expect("Args entry workflow should typecheck without pipeline errors");

    assert!(
        result.is_ok(),
        "Args entry workflow should typecheck successfully: {result:?}"
    );
}
