use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, TypeVar};

fn constructor_var_app(name: &str, args: Vec<Type>) -> Type {
    Type::ConstructorVariableApp {
        constructor: name.into(),
        args,
        kind: Kind::Type,
    }
}

fn option(arg: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![arg],
        kind: Kind::Type,
    }
}

#[test]
fn same_constructor_variable_head_unifies_compatible_proper_arguments() {
    let env = TypeEnv::with_builtin_types();
    let arg = TypeVar::fresh();
    let left = constructor_var_app("M", vec![Type::Var(arg)]);
    let right = constructor_var_app("M", vec![Type::Int]);

    let substitution = env
        .unify_types(&left, &right)
        .expect("same constructor variable head should structurally unify proper args");

    assert_eq!(substitution.get(arg), Some(&Type::Int));
}

#[test]
fn same_constructor_variable_head_fails_closed_for_incompatible_closed_arguments() {
    let env = TypeEnv::with_builtin_types();
    let left = constructor_var_app("M", vec![Type::Int]);
    let right = constructor_var_app("M", vec![Type::String]);

    let err = env
        .unify_types(&left, &right)
        .expect_err("M<Int> and M<String> must fail closed");
    let message = err.to_string();

    assert!(
        message.contains("Int") || message.contains("String") || message.contains("Cannot unify"),
        "expected incompatible argument-spine diagnostic, got: {message}"
    );
}

#[test]
fn different_constructor_variable_heads_do_not_unify() {
    let env = TypeEnv::with_builtin_types();
    let left = constructor_var_app("M", vec![Type::Int]);
    let right = constructor_var_app("N", vec![Type::Int]);

    let err = env
        .unify_types(&left, &right)
        .expect_err("constructor-variable heads are rigid and must match by name");
    let message = err.to_string();

    assert!(
        message.contains("M") && message.contains("N"),
        "expected constructor-head mismatch diagnostic, got: {message}"
    );
}

#[test]
fn constructor_variable_application_does_not_invert_against_nominal_option() {
    let env = TypeEnv::with_builtin_types();
    let arg = TypeVar::fresh();
    let open_constructor = constructor_var_app("M", vec![Type::Var(arg)]);
    let nominal_option = option(Type::Var(arg));

    let err = env
        .unify_types(&open_constructor, &nominal_option)
        .expect_err("M<A> must not invert against nominal Option<A>");
    let message = err.to_string();

    assert!(
        message.contains("does not invert")
            || (message.contains("constructor") && message.contains("nominal"))
            || message.contains("Cannot unify"),
        "expected non-inverting constructor-variable diagnostic, got: {message}"
    );
}

#[test]
fn constructor_variable_head_is_not_solved_from_expected_nominal_output() {
    let env = TypeEnv::with_builtin_types();
    let arg = TypeVar::fresh();
    let left = constructor_var_app("M", vec![Type::Var(arg)]);
    let expected = option(Type::Int);

    let err = env
        .unify_types(&left, &expected)
        .expect_err("unification must not solve M := Option from expected output Option<Int>");
    let message = err.to_string();

    assert!(
        message.contains("does not invert")
            || message.contains("constructor-variable")
            || message.contains("Cannot unify"),
        "expected fail-closed non-inversion diagnostic, got: {message}"
    );
}
