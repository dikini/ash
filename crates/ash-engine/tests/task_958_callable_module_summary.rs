//! TASK-958 coverage for callable signatures crossing module import/export boundaries.

use ash_parser::surface::Type as SurfaceType;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
use std::path::Path;

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn imported_workflow(module_source: &str, import_name: &str) -> ash_engine::Workflow {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");

    write(&library, module_source);
    write(
        &caller,
        &format!("use library::{{{import_name}}}\nworkflow main {{ ret 0 }}\n"),
    );

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    engine
        .parse_file(&caller)
        .expect("caller with import should parse")
}

fn assert_surface_binary_callable(ty: &SurfaceType) {
    match ty {
        SurfaceType::Fn(params, ret) => {
            assert_eq!(params.len(), 2, "callable parameter arity must survive");
            assert!(matches!(&params[0], SurfaceType::Name(name) if name.as_ref() == "Int"));
            assert!(matches!(&params[1], SurfaceType::Name(name) if name.as_ref() == "String"));
            assert!(matches!(ret.as_ref(), SurfaceType::Name(name) if name.as_ref() == "Bool"));
        }
        other => {
            panic!("expected preferred callable syntax to import as SurfaceType::Fn, got {other:?}")
        }
    }
}

#[test]
fn imported_pub_fn_signature_preserves_n_ary_callable_parameter() {
    let workflow = imported_workflow(
        "pub fn accepts(predicate: (Int, String) -> Bool) -> Bool { true }\n",
        "accepts",
    );
    let signature = workflow
        .imported_fn_signatures
        .get("accepts")
        .expect("ordinary pub fn signature should be imported");

    assert_surface_binary_callable(&signature.params[0].ty);

    let typeck_signature = ash_typeck::fn_signature_type(&TypeEnv::with_builtin_types(), signature)
        .expect("imported pub fn signature should convert to typeck Type::Fn");
    assert_eq!(
        typeck_signature.to_string(),
        "((Int, String) -> Bool) -> Bool"
    );
}

#[test]
fn imported_builtin_signature_preserves_preferred_callable_syntax() {
    let workflow = imported_workflow(
        "pub builtin fn keep(predicate: (Int, String) -> Bool, left: Int, right: String) -> Bool;\n",
        "keep",
    );
    let signature = workflow
        .imported_builtin_signatures
        .get("keep")
        .expect("builtin signature should be imported");

    assert_surface_binary_callable(&signature.params[0].ty);

    let typeck_signature =
        ash_typeck::builtin_fn_signature_type(&TypeEnv::with_builtin_types(), signature)
            .expect("imported builtin signature should convert to typeck Type::Fn");
    assert_eq!(
        typeck_signature.to_string(),
        "((Int, String) -> Bool, Int, String) -> Bool"
    );
}

#[test]
fn workflow_returning_smart_constructor_remains_pure_callable() {
    let workflow = imported_workflow(
        "pub builtin fn build(spec: String) -> Workflow<Int>;\n",
        "build",
    );
    let signature = workflow
        .imported_builtin_signatures
        .get("build")
        .expect("workflow-returning builtin signature should be imported");

    let typeck_signature =
        ash_typeck::builtin_fn_signature_type(&TypeEnv::with_builtin_types(), signature)
            .expect("workflow-returning smart constructor signature should convert");

    match &typeck_signature {
        Type::Fn(params, ret) => {
            assert_eq!(params, &[Type::String]);
            assert!(
                matches!(
                    ret.as_ref(),
                    Type::Constructor { name, args, .. }
                        if name.name == "Workflow" && args == &[Type::Int]
                ),
                "return type should remain Workflow<Int>, got {ret:?}"
            );
        }
        other => panic!("smart constructor must remain a pure Type::Fn, got {other:?}"),
    }
    assert_eq!(typeck_signature.to_string(), "(String) -> Workflow<Int>");
}
