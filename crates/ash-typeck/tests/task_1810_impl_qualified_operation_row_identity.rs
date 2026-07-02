//! TASK-1810 operation-row identity resolution for impl-qualified row items.

use ash_core::ast::{TypeBody, TypeDef, VariantDef, VariantPayload, Visibility};
use ash_typeck::type_env::TypeEnv;

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("fixture should parse before typechecking: {errors:?}"));
    ash_parser::surface::Program {
        definitions: module.definitions,
        helper_workflows: Vec::new(),
        workflow: module.workflow.expect("fixture should include workflow"),
    }
}

fn typecheck(source: &str) -> Result<ash_typeck::TypeCheckResult, ash_typeck::TypeCheckError> {
    let program = parse_program(source);
    let mut env = TypeEnv::with_builtin_types();
    register_nominal_type(&mut env, "PosixFs")?;
    register_nominal_type(&mut env, "MemFs")?;
    ash_typeck::type_check_program_in_env(&env, &program)
}

fn register_nominal_type(env: &mut TypeEnv, name: &str) -> Result<(), ash_typeck::TypeCheckError> {
    env.register_type(&TypeDef {
        name: name.to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: name.to_string(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }]),
        visibility: Visibility::Public,
        builtin: false,
    })?;
    Ok(())
}

fn assert_typechecks(source: &str) {
    typecheck(source).unwrap_or_else(|error| {
        panic!("fixture should typecheck, got:\n{error}");
    });
}

fn typecheck_error_text(source: &str) -> String {
    typecheck(source)
        .expect_err("fixture should fail operation-row identity resolution")
        .to_string()
}

fn assert_typecheck_err_contains(source: &str, expected: &[&str]) {
    let text = typecheck_error_text(source);
    for fragment in expected {
        assert!(
            text.contains(fragment),
            "expected error to contain {fragment:?}, got:\n{text}"
        );
    }
}

fn fs_fixture_with_row(row_item: &str) -> String {
    format!(
        r#"
        interface Fs<T> {{
            read(T) -> Int
        }}

        impl Fs<PosixFs> {{
            read(fs) = 0
        }}

        fn guarded_read() -> Int
        where
            row {{ {row_item} }}
        {{
            0
        }}

        workflow main {{ done }}
        "#
    )
}

#[test]
fn concrete_impl_qualified_operation_row_is_accepted() {
    assert_typechecks(&fs_fixture_with_row("PosixFs::read"));
}

#[test]
fn generic_impl_qualified_operation_row_is_accepted_when_bound_proves_interface() {
    assert_typechecks(
        r#"
        interface Fs<T> {
            read(T) -> Int
        }

        fn guarded_read<F>(fs: F) -> Int
        where
            F: Fs,
            row { F::read }
        {
            0
        }

        workflow main { done }
        "#,
    );
}

#[test]
fn interface_qualified_operation_row_is_rejected() {
    assert_typecheck_err_contains(
        r#"
        interface Fs<T> {
            read(T) -> Int
        }

        impl Fs<PosixFs> {
            read(fs) = 0
        }

        impl Fs<MemFs> {
            read(fs) = 0
        }

        fn guarded_read() -> Int
        where
            row { Fs::read }
        {
            0
        }

        workflow main { done }
        "#,
        &["interface-qualified operation row identity", "Fs::read"],
    );
}

#[test]
fn unknown_impl_type_in_operation_row_is_rejected() {
    assert_typecheck_err_contains(
        &fs_fixture_with_row("MissingFs::read"),
        &["unknown impl type", "MissingFs", "MissingFs::read"],
    );
}

#[test]
fn unknown_impl_operation_in_operation_row_is_rejected() {
    assert_typecheck_err_contains(
        &fs_fixture_with_row("PosixFs::write"),
        &["unknown operation", "PosixFs::write", "impl Fs<PosixFs>"],
    );
}

#[test]
fn dotted_uppercase_operation_row_remains_unresolved_metadata() {
    assert_typechecks(&fs_fixture_with_row("PosixFs.read"));
    assert_typechecks(&fs_fixture_with_row("PosixFs.write"));
}
