use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::parse_module::parse_module_decl;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::{Definition, Expr, Type};
use winnow::Parser;

#[test]
fn parses_interface_declaration_in_inline_module() {
    let mut input = new_input("mod interfaces { interface Explain<T> { explain(T) -> String } }");

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("interface declaration should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Interface(interface)] => {
            assert_eq!(interface.name.as_ref(), "Explain");
            assert_eq!(interface.type_params.len(), 1);
            assert_eq!(interface.type_params[0].as_ref(), "T");
            assert_eq!(interface.methods.len(), 1);
            assert_eq!(interface.methods[0].name.as_ref(), "explain");
            assert!(matches!(
                interface.methods[0].params.as_slice(),
                [Type::Name(param)] if param.as_ref() == "T"
            ));
            assert!(matches!(
                &interface.methods[0].return_type,
                Type::Name(name) if name.as_ref() == "String"
            ));
        }
        other => panic!("expected interface definition, got {other:?}"),
    }
}

#[test]
fn parses_impl_declaration_in_inline_module() {
    let mut input = new_input(
        "mod interfaces { impl Explain<PolicyDecision> { explain(decision) = \"policy\" } }",
    );

    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("impl declaration should parse");

    match decl.definitions().expect("inline module definitions") {
        [Definition::Impl(implementation)] => {
            assert_eq!(implementation.interface.as_ref(), "Explain");
            assert_eq!(implementation.type_args.len(), 1);
            assert!(matches!(
                implementation.type_args.as_slice(),
                [Type::Name(name)] if name.as_ref() == "PolicyDecision"
            ));
            assert_eq!(implementation.methods.len(), 1);
            assert_eq!(implementation.methods[0].name.as_ref(), "explain");
            assert_eq!(implementation.methods[0].params.len(), 1);
            assert_eq!(implementation.methods[0].params[0].as_ref(), "decision");
            assert!(matches!(
                implementation.methods[0].body,
                Expr::Literal(ash_parser::surface::Literal::String(ref value)) if value.as_ref() == "policy"
            ));
        }
        other => panic!("expected impl definition, got {other:?}"),
    }
}

#[test]
fn parses_visibility_qualified_interface_and_impl_declarations() {
    let mut interface_input =
        new_input("mod interfaces { pub interface Explain<T> { explain(T) -> String } }");
    let interface_decl = parse_module_decl
        .parse_next(&mut interface_input)
        .expect("public interface declaration should parse");

    match interface_decl
        .definitions()
        .expect("inline module definitions")
    {
        [Definition::Interface(interface)] => {
            assert!(matches!(
                interface.visibility,
                ash_parser::surface::Visibility::Public
            ));
            assert_eq!(interface.name.as_ref(), "Explain");
        }
        other => panic!("expected public interface definition, got {other:?}"),
    }

    let mut impl_input = new_input(
        "mod interfaces { pub impl Explain<PolicyDecision> { explain(decision) = \"policy\" } }",
    );
    let impl_decl = parse_module_decl
        .parse_next(&mut impl_input)
        .expect("public impl declaration should parse");

    match impl_decl.definitions().expect("inline module definitions") {
        [Definition::Impl(implementation)] => {
            assert!(matches!(
                implementation.visibility,
                ash_parser::surface::Visibility::Public
            ));
            assert_eq!(implementation.interface.as_ref(), "Explain");
        }
        other => panic!("expected public impl definition, got {other:?}"),
    }
}

#[test]
fn parses_zero_arity_interface_and_impl_declarations() {
    let mut interface_input =
        new_input("mod interfaces { interface Explain { explain(PolicyDecision) -> String } }");
    let interface_decl = parse_module_decl
        .parse_next(&mut interface_input)
        .expect("zero-arity interface declaration should parse");

    match interface_decl
        .definitions()
        .expect("inline module definitions")
    {
        [Definition::Interface(interface)] => {
            assert_eq!(interface.name.as_ref(), "Explain");
            assert!(interface.type_params.is_empty());
            assert_eq!(interface.methods.len(), 1);
        }
        other => panic!("expected zero-arity interface definition, got {other:?}"),
    }

    let mut impl_input =
        new_input("mod interfaces { impl Explain { explain(decision) = \"policy\" } }");
    let impl_decl = parse_module_decl
        .parse_next(&mut impl_input)
        .expect("zero-arity impl declaration should parse");

    match impl_decl.definitions().expect("inline module definitions") {
        [Definition::Impl(implementation)] => {
            assert_eq!(implementation.interface.as_ref(), "Explain");
            assert!(implementation.type_args.is_empty());
            assert_eq!(implementation.methods.len(), 1);
            assert_eq!(implementation.methods[0].name.as_ref(), "explain");
        }
        other => panic!("expected zero-arity impl definition, got {other:?}"),
    }
}

#[test]
fn parses_constrained_generic_params_on_workflow_definitions() {
    let mut input = new_input("workflow record_event<T: Explain>(value: T) { done }");

    let parsed = workflow_def(&mut input).expect("workflow with interface bound should parse");

    assert_eq!(parsed.type_params.len(), 1);
    assert_eq!(parsed.type_params[0].name.as_ref(), "T");
    assert_eq!(parsed.type_params[0].bounds.len(), 1);
    assert_eq!(
        parsed.type_params[0].bounds[0].interface.as_ref(),
        "Explain"
    );
    assert!(matches!(
        &parsed.params[0].ty,
        Type::Name(name) if name.as_ref() == "T"
    ));
}

#[test]
fn parses_explicit_interface_method_calls() {
    let mut input = new_input("Explain::explain(value)");

    let parsed = expr
        .parse_next(&mut input)
        .expect("interface method call should parse");

    match parsed {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("Explain"));
            assert_eq!(func.as_ref(), "explain");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], Expr::Variable(name) if name.as_ref() == "value"));
        }
        other => panic!("expected qualified call (module::func), got {other:?}"),
    }
}

#[test]
fn parses_interface_method_call_no_args() {
    // Explain::explain() is now a valid qualified fn call with zero args
    let mut input = new_input("Explain::explain()");
    let parsed = expr.parse_next(&mut input).unwrap();
    match parsed {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("Explain"));
            assert_eq!(func.as_ref(), "explain");
            assert!(args.is_empty());
        }
        other => panic!("expected qualified call (module::func), got {other:?}"),
    }
}

#[test]
fn rejects_malformed_interface_declarations() {
    let mut input = new_input("mod interfaces { interface Explain<T> { explain(T -> String } }");

    assert!(
        parse_module_decl.parse_next(&mut input).is_err(),
        "malformed interface method signatures must be rejected"
    );
}

#[test]
fn rejects_malformed_impl_declarations() {
    let mut input =
        new_input("mod interfaces { impl Explain<PolicyDecision> { explain(decision) } }");

    assert!(
        parse_module_decl.parse_next(&mut input).is_err(),
        "impl methods without a body must be rejected"
    );
}
