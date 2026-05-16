use ash_core::type_ir::CanonicalTypeExpr;
use ash_parser::surface::{Definition, Type as SurfaceType};
use ash_typeck::{Kind, TypeEnv, builtin_fn_signature_type, fn_signature_type};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn function(source: &str) -> ash_parser::surface::FnDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present")
}

fn builtin(source: &str) -> ash_parser::surface::BuiltinFnDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::BuiltinFn(builtin) => Some(builtin),
            _ => None,
        })
        .expect("builtin function should be present")
}

fn name(name: &str) -> Box<str> {
    name.into()
}

#[test]
fn fn_signature_accepts_constructor_variable_application_at_task_907_boundary() {
    let function = function(
        r#"
        fn keep<M : * -> *, A : *>(ma: M<A>) -> M<A> { ma }
        "#,
    );

    fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect("TASK-907 should accept M<A> when M : * -> * and A : *");
}

#[test]
fn builtin_signature_accepts_constructor_variable_application_at_task_907_boundary() {
    let builtin = builtin(
        r#"
        builtin fn pure<M : * -> *, A : *>(value: A) -> M<A>;
        "#,
    );

    builtin_fn_signature_type(&TypeEnv::with_builtin_types(), &builtin)
        .expect("TASK-907 should accept builtin signatures returning M<A>");
}

#[test]
fn applying_proper_type_variable_as_constructor_is_rejected_with_constructor_diagnostic() {
    let function = function(
        r#"
        fn bad<T : *, A : *>(value: T<A>) -> A { value }
        "#,
    );

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("T : * must not be usable as a constructor head");
    let message = err.to_string();

    assert!(
        message.contains("proper type variable") && message.contains("constructor"),
        "expected proper-type-variable-as-constructor diagnostic, got: {message}"
    );
}

#[test]
fn constructor_variable_application_rejects_missing_argument_spine() {
    let function = function(
        r#"
        fn bad<M : * -> *>(value: M) -> M { value }
        "#,
    );

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("bare M has kind * -> * and is not a proper parameter type");
    let message = err.to_string();

    assert!(
        message.contains("M")
            && (message.contains("wrong arity")
                || message.contains("kind")
                || message.contains("expected 1")),
        "expected wrong-arity/kind diagnostic for bare constructor variable, got: {message}"
    );
}

#[test]
fn constructor_variable_application_rejects_too_many_arguments() {
    let function = function(
        r#"
        fn bad<M : * -> *, A : *, B : *>(value: M<A, B>) -> A { value }
        "#,
    );

    let err = fn_signature_type(&TypeEnv::with_builtin_types(), &function)
        .expect_err("M : * -> * must reject a two-argument application");
    let message = err.to_string();

    assert!(
        message.contains("M")
            && (message.contains("wrong arity")
                || message.contains("kind")
                || message.contains("expected 1")
                || message.contains("found 2")),
        "expected wrong-arity/kind diagnostic for M<A, B>, got: {message}"
    );
}

#[test]
fn canonical_lowering_preserves_constructor_variable_app_instead_of_nominalizing() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type_parameter_kind("M", Kind::n_ary(1))
        .expect("test precondition: register constructor-kinded type parameter");
    env.register_type_parameter_kind("A", Kind::Type)
        .expect("test precondition: register proper type parameter");

    let surface = SurfaceType::Constructor {
        name: name("M"),
        args: vec![SurfaceType::Name(name("A"))],
    };

    let lowered = env
        .lower_surface_type_to_canonical(&surface)
        .expect("M<A> should lower after constructor variable kind registration");

    match lowered {
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            assert_eq!(app.constructor.name, "M");
            assert_eq!(app.constructor.kind, Kind::n_ary(1));
            assert_eq!(app.args, vec![CanonicalTypeExpr::Var("A".to_string())]);
            assert_eq!(app.kind, Kind::Type);
        }
        CanonicalTypeExpr::NominalApp { visible_name, .. } => {
            panic!("M<A> must not be lowered as nominal application {visible_name}<...>")
        }
        other => panic!("expected constructor-variable application, got {other:?}"),
    }
}
