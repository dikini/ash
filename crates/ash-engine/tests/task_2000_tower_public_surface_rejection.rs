//! TASK-2000 RED: deleted tower wrappers must fail closed at public entry points.

use std::path::Path;

use ash_engine::{Engine, EngineError};
use ash_typeck::TypeEnv;

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

fn check_source(source: &str) -> Result<(), EngineError> {
    let engine = engine();
    let mut entry = engine.parse_file_source(Path::new("tower-deletion.ash"), source)?;
    engine.check(&mut entry)
}

#[test]
fn task_2000_public_act_proc_types_and_bridge_builtins_reject() {
    let type_env = TypeEnv::with_builtin_types();
    let manifest = type_env.public_computation_manifest();
    for algebra in ["Act", "Proc"] {
        assert!(
            manifest.algebra(algebra).is_none(),
            "deleted public tower algebra {algebra} must not remain in the type manifest"
        );
        assert!(
            type_env.lookup_type_info(algebra).is_none(),
            "deleted public tower algebra {algebra} must not remain type-resolvable"
        );
    }
    for builtin in ["act::unit", "proc::unit", "proc::yield"] {
        assert!(
            manifest.operation(builtin).is_none() && type_env.lookup_variable(builtin).is_none(),
            "deleted public bridge builtin {builtin} must not remain nameable or typeable"
        );
    }

    for (surface, source) in [
        ("Act<T>", "fn main() -> Act<Int> { act::unit(1) }"),
        ("Proc<T>", "fn main() -> Proc<Int> { proc::unit(1) }"),
        ("act bridge builtin", "fn main() { act::unit(1) }"),
        ("proc bridge builtin", "fn main() { proc::yield() }"),
    ] {
        assert!(
            check_source(source).is_err(),
            "deleted public tower surface {surface} must reject rather than remain typeable"
        );
    }
}

#[test]
fn task_2000_canonical_ambient_do_control_remains_accepted() {
    check_source(
        "fn main() -> Int where row { evidence audit_log } { do { let value = 41; return value + 1 } }",
    )
    .expect("deleting Act/Proc wrappers must not reject canonical ambient do");
}
