use ash_parser::surface::Type as SurfaceType;
use ash_typeck::TypeEnv;

fn name(name: &str) -> Box<str> {
    name.into()
}

#[test]
fn task_905_typeenv_rejects_constructor_variable_application_without_nominalizing() {
    let env = TypeEnv::new();
    let surface = SurfaceType::Constructor {
        name: name("M"),
        args: vec![SurfaceType::Name(name("Int"))],
    };

    let err = env
        .lower_surface_type_to_canonical(&surface)
        .expect_err("TASK-905 must fail closed until TASK-907 tracks constructor variables");
    let message = err.to_string();

    assert!(
        message.contains("constructor-variable application"),
        "expected explicit constructor-variable boundary, got: {message}"
    );
    assert!(
        message.contains("TASK-907"),
        "expected deferral to TASK-907 semantics, got: {message}"
    );
}
