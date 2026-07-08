use ash_core::type_ir::TypeConstructorExpr;
use ash_parser::surface::{Definition, DoTarget, ImplDef, InterfaceDef, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::{ConstructorError, TypeEnvError};
use ash_typeck::type_env::InterfaceEvidenceArg;
use ash_typeck::{
    Kind, QualifiedName, Type, TypeEnv, fn_signature_type, resolve_do_target_for_test,
};

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

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str, index: usize) -> ImplDef {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .nth(index)
        .unwrap_or_else(|| panic!("impl {name} at index {index} should be present"))
}

fn target(name: &str, args: Vec<SurfaceType>) -> DoTarget {
    DoTarget {
        name: name.into(),
        args,
        span: Span::default(),
    }
}

fn env_with_monad_interface_only() -> TypeEnv {
    let module = parse("interface Monad<M : * -> *> {}");
    let interface = interface_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env
}

fn env_with_monad_option_evidence() -> TypeEnv {
    let module = parse(
        r#"
interface Monad<M : * -> *> {
            return(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }
        impl Monad<Option> {
            return(value) = Some { value: value }
            bind(value, f) = value
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register");
    env
}

#[test]
fn hkt2_interface_method_signature_accepts_constructor_application() {
    let module = parse(
        r#"
        interface Functor<F : * -> *> {
            map(F<Int>) -> F<Int>
        }
        "#,
    );
    let interface = interface_named(&module, "Functor");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("F<A>-shaped method signatures should typecheck for F : * -> *");
}

#[test]
fn hkt3_impl_monad_option_registers_empty_method_mvp_evidence() {
    let env = env_with_monad_option_evidence();

    let evidence = env
        .resolve_interface_evidence("Monad", &[SurfaceType::Name("Option".into())])
        .expect("explicit Monad<Option> evidence lookup should select the registered impl");

    assert_eq!(evidence.interface, "Monad");
    assert!(matches!(
        evidence.head_args.first(),
        Some(InterfaceEvidenceArg::Constructor(_))
    ));
}

#[test]
fn hkt4_result_partial_impl_head_is_registered_only_as_shape_evidence() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl <E : *> Monad<Result<_, E>> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register");
    env.register_impl(&implementation)
        .expect("SPEC-066 partial Result<_, E> shape should be accepted as an impl head");

    let scheme = env
        .impl_schemes()
        .first()
        .expect("partial Result Monad impl scheme should be registered");
    assert!(matches!(
        scheme.head_args.as_slice(),
        [InterfaceEvidenceArg::Constructor(expr)]
            if matches!(expr.as_ref(), TypeConstructorExpr::PartialApplication(_))
    ));
    assert!(
        scheme.methods.is_empty(),
        "TASK-910 must not overclaim generalized runtime method lowering for partial Result evidence"
    );
}

#[test]
fn hkt5_bare_constructor_variable_in_proper_type_position_is_wrong_kind() {
    let module = parse(
        r#"
        fn bad<M : * -> *>(value: M) -> M { value }
        "#,
    );
    let function = module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("bare M has kind * -> * and is not a proper type");
    let message = err.to_string();

    assert!(
        message.contains("M")
            && (message.contains("kind")
                || message.contains("wrong arity")
                || message.contains("expected 1")),
        "expected wrong-kind diagnostic for bare constructor variable, got: {message}"
    );
}

#[test]
fn hkt6_duplicate_monad_option_impls_are_rejected_as_overlap() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Option> {}
        impl Monad<Option> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let first = impl_named(&module, "Monad", 0);
    let duplicate = impl_named(&module, "Monad", 1);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&first)
        .expect("first Monad<Option> impl should register");
    let err = env
        .register_impl(&duplicate)
        .expect_err("duplicate Monad<Option> impl should be rejected");
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && message.contains("Option")
            && (message.contains("duplicate") || message.contains("overlap")),
        "expected overlapping higher-kinded evidence diagnostic, got: {message}"
    );
}

#[test]
fn hkt7_do_option_uses_registered_monad_evidence_at_type_boundary() {
    let env = env_with_monad_option_evidence();
    let expr = ash_parser::surface::Expr::DoBlock {
        target: target("Option", Vec::new()),
        stmts: vec![ash_parser::surface::DoStmt::Return {
            value: Box::new(ash_parser::surface::Expr::Literal(
                ash_parser::surface::Literal::Int(1),
            )),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    resolve_do_target_for_test(&env, &target("Option", Vec::new()))
        .expect("do:Option should resolve through explicit Monad<Option> evidence");
    let result = check_expr(&env, &expr);

    assert!(result.is_ok(), "do:Option should typecheck: {result:?}");
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: Kind::Type,
        }
    );
}

#[test]
fn hkt8_do_list_without_monad_evidence_reports_missing_evidence() {
    let env = env_with_monad_interface_only();

    let err = resolve_do_target_for_test(&env, &target("List", Vec::new()))
        .expect_err("do:List should fail without Monad<List> evidence");
    let ConstructorError::UnsupportedExpression { kind, .. } = err else {
        panic!("expected unsupported-expression diagnostic, got {err:?}");
    };

    assert!(kind.contains("missing Monad evidence"), "{kind}");
    assert!(kind.contains("Monad<List>"), "{kind}");
}

#[test]
fn applying_proper_type_variable_as_constructor_is_rejected() {
    let module = parse(
        r#"
        fn bad<T : *, A : *>(value: T<A>) -> A { value }
        "#,
    );
    let function = module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("T : * must not be usable as a constructor head");
    let message = err.to_string();

    assert!(
        message.contains("proper type variable") && message.contains("constructor"),
        "expected proper-type-variable-as-constructor diagnostic, got: {message}"
    );
}

#[test]
fn constructor_variable_wrong_argument_count_is_rejected_before_evidence_lookup() {
    let module = parse(
        r#"
        fn bad<M : * -> *, A : *, B : *>(value: M<A, B>) -> A { value }
        "#,
    );
    let function = module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("M : * -> * must reject a two-argument application");
    let message = err.to_string();

    assert!(
        message.contains("M")
            && (message.contains("wrong arity")
                || message.contains("expected 1")
                || message.contains("found 2")),
        "expected wrong-arity diagnostic for M<A, B>, got: {message}"
    );
}

#[test]
fn impl_head_wrong_kind_for_interface_parameter_is_rejected() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Int> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register");
    let err = env
        .register_impl(&implementation)
        .expect_err("impl Monad<Int> must reject Int because Monad expects * -> *");
    let TypeEnvError::InvalidDefinition(message, _) = err else {
        panic!("expected invalid-definition diagnostic, got {err:?}");
    };

    assert!(message.contains("Monad"), "{message}");
    assert!(message.contains("Int"), "{message}");
    assert!(message.contains("expected * -> *"), "{message}");
}
