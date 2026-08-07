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

#[test]
fn predicate_like_row_families_fail_closed_before_lowering() {
    for family in [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ] {
        let program = parse_program(&format!(
            r#"
            fn guarded() -> Int
            where row {{
                process spawn,
                channel jobs,
                {family}_fact
            }} {{
                0
            }}

            fn main() -> Int {{ 0 }}
            "#,
        ));

        let error = ash_typeck::type_check_program(&program)
            .expect_err("{family} predicate-style row family must be rejected");
        let message = error.to_string();
        assert!(
            message.contains("unsupported row item family") && message.contains(family),
            "family={family} error={message}"
        );
    }
}

#[test]
fn ordinary_process_channel_rows_remain_valid_requirements() {
    let program = parse_program(
        r#"
        fn guarded() -> Int
        where row {
            process spawn,
            channel jobs
        } {
            0
        }

        fn main() -> Int { 0 }
        "#,
    );

    ash_typeck::type_check_program(&program)
        .expect("ordinary process/channel row requirements remain valid");
}
