//! TASK-1814 non-authority evidence for Phase 177 row validation.

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("source should parse: {errors:?}"));
    ash_parser::surface::Program {
        definitions: module.definitions,
        helper_workflows: Vec::new(),
        workflow: module.workflow.expect("fixture should include workflow"),
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

        workflow main { done }
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
