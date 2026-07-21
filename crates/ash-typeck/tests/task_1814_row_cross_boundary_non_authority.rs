//! TASK-1814 non-authority evidence for Phase 177 row validation.

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("source should parse: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name.as_ref() == "main" =>
            {
                Some(ash_parser::surface::ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("source should define fn main");
    ash_parser::surface::Program {
        definitions: module.definitions,
        entry,
    }
}

#[test]
fn supported_row_mentions_do_not_grant_runtime_authority_provenance() {
    let program = parse_program(
        r#"
        fn guarded() -> Int
        where
            row {
                fs.read,
                resource File read,
                role Reader,
                policy AllowRead,
                fail ReadFailure,
                evidence read_allowed,
                group FsReads
            }
        {
            0
        }

        fn main() -> Int { 0 }
        "#,
    );

    let result = ash_typeck::type_check_program(&program)
        .expect("supported rows validate without authority semantics");

    assert!(
        result.authority_provenance.resource_bindings.is_empty(),
        "resource row requirements must not create resource authority bindings"
    );
    assert!(
        result.authority_provenance.capability_bindings.is_empty(),
        "operation row requirements must not create capability authority bindings"
    );
}
