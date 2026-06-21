use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow, CoreType};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

#[test]
fn default_typecheck_environment_is_empty() {
    let env = CoreTypeCheckEnv::default();

    assert!(env.types().is_empty());
    assert!(env.values().is_empty());
    assert!(env.continuations().is_empty());
    assert!(env.rows().is_empty());
    assert!(env.operations().is_empty());
    assert!(env.discharges().is_empty());
}

#[test]
fn minimal_valid_atom_program_typechecks_through_public_api() {
    let valid = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(42))))
        .expect("literal atom program validates");

    let typed = type_check_core_program(valid, &CoreTypeCheckEnv::default())
        .expect("literal atom program type-checks");

    assert_eq!(typed.ty(), &CoreType::Base("Int".into()));
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn environment_bindings_can_typecheck_variable_atoms() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut()
        .insert("answer", CoreType::Base("Int".into()));
    let valid = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::Var(
        "answer".into(),
    ))))
    .expect("variable atom program validates");

    let typed = type_check_core_program(valid, &env).expect("bound variable type-checks");

    assert_eq!(typed.ty(), &CoreType::Base("Int".into()));
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn unknown_variable_reports_structured_typecheck_error() {
    let valid = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::Var(
        "missing".into(),
    ))))
    .expect("variable atom program validates");

    let err = type_check_core_program(valid, &CoreTypeCheckEnv::default())
        .expect_err("unbound variable must fail type-checking");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownValue {
            name: "missing".into()
        }
    );
}
