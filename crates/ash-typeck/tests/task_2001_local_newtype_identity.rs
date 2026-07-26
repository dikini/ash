//! TASK-2001: local nominal-newtype identities retain their defining module.
//!
//! The module-aware type-checking entry point establishes the current module
//! before declaration registration.  The registration assertion below inspects
//! that same declaration boundary directly because `TypeCheckResult` does not
//! expose its transient `TypeEnv`.

use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin, TypeDeclId};
use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{TypeCheckConfig, TypeEnv, type_check_program_in_env_for_module_with_config};

const SOURCE: &str = r#"
    type Payload = Payload(Int);
    newtype OrderId = OrderId(Payload);
    fn main() -> Int { 0 }
"#;

fn parse_program() -> Program {
    let module = ash_parser::parse_surface_file(SOURCE)
        .unwrap_or_else(|errors| panic!("TASK-2001 source should parse: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must define main");
    Program {
        definitions: module.definitions,
        entry,
    }
}

fn defining_module() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(2001)),
        ModuleId(20_010),
        vec!["task_2001".to_string(), "local_orders".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2001 local newtype identity regression".to_string(),
        },
    )
}

#[test]
fn module_aware_typecheck_registers_local_newtype_with_defining_module_identity() {
    let program = parse_program();
    let module = defining_module();

    type_check_program_in_env_for_module_with_config(
        &TypeEnv::with_builtin_types(),
        &program,
        module.clone(),
        &TypeCheckConfig::default(),
    )
    .expect("module-aware local newtype fixture should typecheck");

    // This is the declaration-registration transition performed by the
    // module-aware entry point immediately after installing `module`.
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module.clone());
    env.register_surface_declarations(&program.definitions)
        .expect("module-aware declaration registration should succeed");

    assert_eq!(
        env.nominal_newtype("OrderId")
            .expect("local newtype should remain registered")
            .identity(),
        TypeDeclId::ordinary(module, "OrderId"),
        "a module-aware local newtype must retain its real defining-module identity"
    );
}

#[test]
fn standalone_declaration_registration_keeps_the_documented_fallback_identity() {
    let program = parse_program();
    let mut env = TypeEnv::with_builtin_types();
    env.register_surface_declarations(&program.definitions)
        .expect("standalone declaration registration should succeed");

    let expected_fallback = TypeDeclId::ordinary(
        ModuleIdentity::new(
            Some(CrateId(usize::MAX)),
            ModuleId(usize::MAX),
            vec!["typeenv".to_string(), "defeq_fallback".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "TASK-826 guarded TypeEnv defeq fallback identity".to_string(),
            },
        ),
        "OrderId",
    );
    assert_eq!(
        env.nominal_newtype("OrderId")
            .expect("standalone local newtype should remain registered")
            .identity(),
        expected_fallback,
        "without a module context the existing documented fallback remains explicit"
    );
}
