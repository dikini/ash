use ash_parser::surface::Definition;
use ash_typeck::{TypeEnv, builtin_fn_signature_type, fn_signature_type};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

#[test]
fn fn_signature_type_no_longer_rejects_task_907_constructor_kinded_type_params() {
    let module = parse(
        r#"
        fn lift<F : * -> *>(value: Int) -> Int { value }
        "#,
    );
    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    fn_signature_type(&TypeEnv::with_builtin_types(), function)
        .expect("TASK-907 owns function-signature constructor-kinded binders");
}

#[test]
fn builtin_signature_type_no_longer_rejects_task_907_constructor_kinded_type_params() {
    let module = parse(
        r#"
        builtin fn pure<M : * -> *>(value: Int) -> Int;
        "#,
    );
    let builtin = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::BuiltinFn(builtin) => Some(builtin),
            _ => None,
        })
        .expect("builtin function should be present");

    builtin_fn_signature_type(&TypeEnv::with_builtin_types(), builtin)
        .expect("TASK-907 owns builtin-signature constructor-kinded binders");
}

#[test]
fn type_env_interface_registration_accepts_task_908_constructor_kinded_interface_binders() {
    let module = parse(
        r#"
        interface Functor<F : * -> *> {}
        "#,
    );
    let interface = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface should be present");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(interface).expect(
        "TypeEnv interface registration now accepts TASK-908 constructor-kinded interface binders",
    );
}

#[test]
fn type_env_impl_registration_rejects_constructor_kinded_type_params() {
    let module = parse(
        r#"
        interface Monad<M> {}
        impl <F : * -> *> Monad<F> {}
        "#,
    );
    let interface = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface should be present");
    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation),
            _ => None,
        })
        .expect("impl should be present");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(interface)
        .expect("test precondition: proper interface should register");
    let error = env
        .register_impl(implementation)
        .expect_err("type env must fail closed for constructor-kinded impl binders");

    assert!(error.to_string().contains("kinded binders"));
    assert!(error.to_string().contains("TASK-908"));
}

#[test]
fn proposition_registration_rejects_constructor_kinded_params() {
    let module = parse(
        r#"
        prop Maps<F : * -> *>;
        "#,
    );
    let proposition = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(proposition) => Some(proposition),
            _ => None,
        })
        .expect("proposition should be present");

    let mut env = TypeEnv::with_builtin_types();
    let error = env
        .register_proposition_predicate_decl(proposition)
        .expect_err("type env must fail closed for constructor-kinded proposition binders");

    assert!(error.to_string().contains("kinded binders"));
    assert!(error.to_string().contains("TASK-908"));
}

#[test]
fn explicit_proper_type_binders_still_typecheck_as_ordinary_params() {
    let module = parse(
        r#"
        fn identity<T : *>(value: T) -> T { value }
        "#,
    );
    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    fn_signature_type(&TypeEnv::with_builtin_types(), function)
        .expect("explicit proper type binders should remain ordinary type params");
}
