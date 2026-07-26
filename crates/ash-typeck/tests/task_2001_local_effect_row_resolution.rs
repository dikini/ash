//! TASK-2001 RED: local effect-row declarations participate in row validation.

use ash_typeck::{TypeCheckError, error::TypeEnvError};

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
        .expect("source must define fn main");
    ash_parser::surface::Program {
        definitions: module.definitions,
        entry,
    }
}

fn assert_cycle_rejection_with_path(error: TypeCheckError, expected_path: &[&str]) {
    let TypeCheckError::TypeEnv(error) = error else {
        panic!("effect-row cycle must be rejected as a type-environment definition error");
    };
    let TypeEnvError::InvalidDefinition(message, _) = *error else {
        panic!("effect-row cycle must be rejected as an invalid definition");
    };
    assert!(
        message.contains("cyclic"),
        "cycle rejection must identify the semantic cycle: {message}"
    );

    let mut search_from = 0;
    for segment in expected_path {
        let Some(relative_index) = message[search_from..].find(segment) else {
            panic!(
                "cycle rejection must include path segment '{segment}' in order {expected_path:?}: {message}"
            );
        };
        search_from += relative_index + segment.len();
    }
}

fn assert_private_row_export_rejection(
    error: TypeCheckError,
    expected_public_callable: &str,
    expected_private_row: &str,
) {
    let TypeCheckError::TypeEnv(error) = error else {
        panic!("a private row in a public callable must fail as a type-environment export error");
    };
    let TypeEnvError::PrivateDependencyExportFailure {
        public_item,
        dependency,
        dependency_kind,
        ..
    } = *error
    else {
        panic!(
            "a private row in a public callable must use the structured private-dependency export error"
        );
    };
    assert_eq!(public_item, expected_public_callable);
    assert_eq!(dependency, expected_private_row);
    assert_eq!(dependency_kind, "effect row");
}

#[test]
fn task_2001_local_effect_alias_expands_before_row_validation() {
    let program = parse_program(
        r"
        effect alias Audit = { requires_proof };
        fn takes() -> Int where row { Audit, evidence audit_log } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a local alias must expose invalid row content to validation");
    assert!(
        error
            .to_string()
            .contains("unsupported row item family 'requires'"),
        "local alias must remain a requirement description: {error}"
    );
}

#[test]
fn task_2001_local_effect_group_expands_without_authority() {
    let program = parse_program(
        r"
        effect group Audit = { evidence audit_log };
        fn takes() -> Int where row { group Audit, evidence caller_log } { 0 }
        fn main() { 0 }
        ",
    );

    let result = ash_typeck::type_check_program(&program)
        .expect("a local group must validate as a non-granting requirement description");
    assert!(result.authority_provenance.resource_bindings.is_empty());
    assert!(result.authority_provenance.capability_bindings.is_empty());
}

#[test]
fn task_2001_local_effect_rows_allow_a_shared_acyclic_reference_after_unwinding() {
    let program = parse_program(
        r"
        effect alias Shared = { evidence audit_log };
        effect alias Audit = { Shared, group Workflow };
        effect group Workflow = { Shared };
        fn takes() -> Int where row { Audit } { 0 }
        fn main() { 0 }
        ",
    );

    let result = ash_typeck::type_check_program(&program).expect(
        "an acyclic alias/group graph may reuse a sibling row after its recursive expansion unwinds",
    );
    assert!(result.authority_provenance.resource_bindings.is_empty());
    assert!(result.authority_provenance.capability_bindings.is_empty());
}

#[test]
fn task_2001_rejects_a_direct_local_effect_alias_cycle_in_a_callable_row() {
    let program = parse_program(
        r"
        effect alias Audit = { Audit };
        fn takes() -> Int where row { Audit } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a callable row must reject a direct local alias expansion cycle");
    assert_cycle_rejection_with_path(error, &["Audit", "Audit"]);
}

#[test]
fn task_2001_rejects_a_direct_local_effect_group_cycle_in_a_callable_row() {
    let program = parse_program(
        r"
        effect group Audit = { group Audit };
        fn takes() -> Int where row { group Audit } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a callable row must reject a direct local group expansion cycle");
    assert_cycle_rejection_with_path(error, &["Audit", "Audit"]);
}

#[test]
fn task_2001_rejects_a_mutual_local_alias_group_cycle_in_a_callable_row() {
    let program = parse_program(
        r"
        effect alias Audit = { group Workflow };
        effect group Workflow = { Audit };
        fn takes() -> Int where row { Audit } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a callable row must reject a mutual local alias/group expansion cycle");
    assert_cycle_rejection_with_path(error, &["Audit", "Workflow", "Audit"]);
}

#[test]
fn task_2001_rejects_a_public_callable_row_that_names_a_private_local_alias() {
    let program = parse_program(
        r"
        effect alias HiddenAudit = { evidence audit_log };
        pub fn exposed() -> Int where row { HiddenAudit } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a public callable must not expose a private effect alias in its row");
    assert_private_row_export_rejection(error, "exposed", "HiddenAudit");
}

#[test]
fn task_2001_rejects_a_public_callable_row_that_names_a_private_local_group() {
    let program = parse_program(
        r"
        effect group HiddenAudit = { evidence audit_log };
        pub fn exposed() -> Int where row { group HiddenAudit } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program)
        .expect_err("a public callable must not expose a private effect group in its row");
    assert_private_row_export_rejection(error, "exposed", "HiddenAudit");
}

#[test]
fn task_2001_rejects_a_public_alias_group_chain_that_resolves_to_a_private_row() {
    let program = parse_program(
        r"
        effect alias HiddenAudit = { evidence audit_log };
        pub effect group PublishedGroup = { HiddenAudit };
        pub effect alias PublishedAlias = { group PublishedGroup };
        pub fn exposed() -> Int where row { PublishedAlias } { 0 }
        fn main() { 0 }
        ",
    );

    let error = ash_typeck::type_check_program(&program).expect_err(
        "a public alias/group chain must not hide a private local row behind public names",
    );
    assert_private_row_export_rejection(error, "exposed", "HiddenAudit");
}

#[test]
fn task_2001_accepts_a_public_only_non_granting_row_chain() {
    let program = parse_program(
        r"
        pub effect alias Audit = { evidence audit_log };
        pub effect group PublishedGroup = { Audit };
        pub effect alias PublishedAlias = { group PublishedGroup };
        pub fn exposed() -> Int where row { PublishedAlias } { 0 }
        fn main() { 0 }
        ",
    );

    let result = ash_typeck::type_check_program(&program)
        .expect("a public-only row chain remains a non-granting requirement description");
    assert!(result.authority_provenance.resource_bindings.is_empty());
    assert!(result.authority_provenance.capability_bindings.is_empty());
}

#[test]
fn task_2001_accepts_a_private_callable_using_a_private_row() {
    let program = parse_program(
        r"
        effect alias HiddenAudit = { evidence audit_log };
        fn internal() -> Int where row { HiddenAudit } { 0 }
        fn main() { 0 }
        ",
    );

    let result = ash_typeck::type_check_program(&program)
        .expect("a private callable may retain a private effect row");
    assert!(result.authority_provenance.resource_bindings.is_empty());
    assert!(result.authority_provenance.capability_bindings.is_empty());
}
