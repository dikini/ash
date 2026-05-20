use ash_parser::surface::{
    Definition, DoStmt, DoTarget, Expr, ImplDef, InterfaceDef, Literal, Type as SurfaceType,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, resolve_do_target_for_test};

fn span() -> Span {
    Span::default()
}

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn interface_named(module: &ash_parser::surface::ModuleFile, name: &str) -> InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => {
                Some(interface.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str) -> ImplDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl {name} should be present"))
}

fn env_with_monad_option_evidence() -> TypeEnv {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {
            return(Int) -> M<Int>
            bind(M<Int>, Int -> M<Int>) -> M<Int>
        }
        impl Monad<Option> {
            return(value) = Some { value: value }
            bind(value, f) = value
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register explicit evidence");
    env
}

fn env_with_monad_interface_only() -> TypeEnv {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env
}

fn target(name: &str, args: Vec<SurfaceType>) -> DoTarget {
    DoTarget {
        name: name.into(),
        args,
        span: span(),
    }
}

fn do_block(target_name: &str, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target(target_name, Vec::new()),
        stmts,
        span: span(),
    }
}

fn ret(value: Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: span(),
    }
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn unsupported_message(err: ConstructorError) -> String {
    match err {
        ConstructorError::UnsupportedExpression { kind, .. } => kind,
        other => other.to_string(),
    }
}

#[test]
fn do_option_resolves_through_explicit_monad_option_evidence() {
    let env = env_with_monad_option_evidence();

    let result = resolve_do_target_for_test(&env, &target("Option", Vec::new()));

    assert!(
        result.is_ok(),
        "do:Option should resolve through explicit Monad<Option> evidence: {result:?}"
    );
}

#[test]
fn do_option_return_typechecks_after_explicit_monad_evidence_resolution() {
    let env = env_with_monad_option_evidence();
    let expr = do_block("Option", vec![ret(int_lit(1))]);

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "do:Option return-only type boundary should type-check through Monad<Option> evidence: {result:?}"
    );
    assert_eq!(result.ty, computation_type("Option", Type::Int));
}

#[test]
fn do_list_without_registered_monad_evidence_reports_missing_monad_list() {
    let env = env_with_monad_interface_only();

    let err = resolve_do_target_for_test(&env, &target("List", Vec::new()))
        .expect_err("do:List has unary shape but no Monad<List> evidence");
    let message = unsupported_message(err);

    assert!(message.contains("missing Monad evidence"), "{message}");
    assert!(message.contains("Monad<List>"), "{message}");
    assert!(!message.contains("wrong target shape"), "{message}");
}

#[test]
fn wrong_shape_target_reports_shape_error_before_monad_evidence_lookup() {
    let env = env_with_monad_interface_only();

    let err = resolve_do_target_for_test(&env, &target("Int", Vec::new()))
        .expect_err("do:Int is a proper type, not a unary constructor");
    let message = unsupported_message(err);

    assert!(message.contains("do target Int has kind *"), "{message}");
    assert!(message.contains("expected * -> *"), "{message}");
    assert!(!message.contains("missing Monad evidence"), "{message}");
}
