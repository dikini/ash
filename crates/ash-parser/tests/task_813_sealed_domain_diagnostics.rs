//! TASK-813: Sealed-domain diagnostics and non-interference tests for ash-parser.
//!
//! Negative cases (rejection of invalid constructs) and non-interference
//! with existing Phase 109/110 parsing and lowering behavior.

use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, StructuralFieldStatus, SummaryVersion,
};
use ash_parser::surface::{Definition, DomainSlot};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn test_module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(1),
        vec!["test".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-813".into(),
        },
    )
}

fn lower(source: &str) -> ash_parser::lower::LoweredTypeMetadata {
    let module = parse(source);
    ash_parser::lower::lower_module_type_metadata(&module, test_module_identity())
}

fn sealed_domains(
    module: &ash_parser::surface::ModuleFile,
) -> Vec<&ash_parser::surface::SealedDomainDef> {
    module
        .definitions
        .iter()
        .filter_map(|def| match def {
            Definition::SealedDomain(sd) => Some(sd),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Negative cases: rejection of invalid constructs
// ---------------------------------------------------------------------------

#[test]
fn parse_sealed_domain_rejects_generic_params() {
    // Generic parameters on sealed domains are unsupported.
    let source = "sealed type domain Foo[T] { X; }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "sealed domain with generic parameters should be rejected by the parser"
    );
}

#[test]
fn parse_sealed_domain_rejects_angle_generic_params() {
    // Angle generic parameters on sealed domains are unsupported.
    let source = "sealed type domain Foo<T> { X; }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "sealed domain with angle generic parameters should be rejected by the parser"
    );
}

#[test]
fn parse_sealed_domain_rejects_per_constructor_visibility() {
    // Per-constructor visibility is not supported in the first slice.
    let source = "sealed type domain Foo { pub Bar; }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "per-constructor visibility in sealed domains should be rejected"
    );
}

#[test]
fn parse_sealed_domain_in_inline_module_rejected() {
    // Sealed domains inside inline modules are explicitly unsupported.
    let source = r"mod inline { sealed type domain Foo { X; } }";
    let result = ash_parser::parse_surface_file(source);
    assert!(
        result.is_err(),
        "inline-module sealed domains should be rejected by the parser"
    );
}

#[test]
fn parse_sealed_domain_duplicate_constructors_rejected_or_preserved() {
    // The parser may or may not reject duplicate constructor names.
    // If it accepts them, the semantic validation in TypeEnv (task_812) will catch it.
    // We verify the parser doesn't crash and produces parseable output.
    let source = "sealed type domain Foo { X; X; }";
    let result = ash_parser::parse_surface_file(source);

    if let Ok(module) = result {
        // Parser accepted it; verify both constructors are present.
        let domains = sealed_domains(&module);
        assert_eq!(domains.len(), 1, "should have exactly one domain");
        assert_eq!(
            domains[0].constructors.len(),
            2,
            "both duplicate constructors should be present for semantic validation to reject"
        );
    }
    // If the parser rejects this, that's also correct.
}

// ---------------------------------------------------------------------------
// Lowering negative cases
// ---------------------------------------------------------------------------

#[test]
fn parse_sealed_domain_rejects_list_field_slot() {
    assert!(
        ash_parser::parse_surface_file("sealed type domain Bad { X<xs: List<Int>>; }").is_err()
    );
}

#[test]
fn parse_sealed_domain_rejects_tuple_or_unnamed_field_slot() {
    assert!(ash_parser::parse_surface_file("sealed type domain Bad { X<(Int, Int)>; }").is_err());
    assert!(ash_parser::parse_surface_file("sealed type domain Bad { X<Int>; }").is_err());
}

#[test]
fn parse_sealed_domain_rejects_path_like_field_slot() {
    assert!(ash_parser::parse_surface_file("sealed type domain Bad { X<field: A::B>; }").is_err());
}

#[test]
fn parse_sealed_domain_rejects_generic_or_applied_field_slot() {
    assert!(
        ash_parser::parse_surface_file("sealed type domain Bad { X<field: Foo<T>>; }").is_err()
    );
}

#[test]
fn lower_sealed_domain_rejects_unknown_field_domain_records_constraint() {
    // When a field references a non-existent domain, lowering records the
    // domain_constraint but does NOT validate the reference (lowering is
    // syntactic, not semantic).  The unknown domain appears as a cross-domain
    // NonStructural reference with a domain_constraint set.
    let source = "sealed type domain Foo { Bar<z: NonExistentDomain>; }";
    let lowered = lower(source);

    let domain = &lowered.summary.exported_sealed_domains[0];
    assert_eq!(domain.constructors.len(), 1);
    let ctor = &domain.constructors[0];
    assert_eq!(ctor.exported_name.as_str(), "Bar");
    assert_eq!(ctor.fields.len(), 1);

    let field = &ctor.fields[0];
    assert_eq!(field.name.as_str(), "z");
    // The domain_constraint is recorded but lowering doesn't validate it.
    assert!(
        field.domain_constraint.is_some(),
        "domain_constraint should be recorded for the unknown domain reference"
    );
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::NonStructural,
        "reference to a different (unknown) domain should be NonStructural"
    );
    let constraint = field.domain_constraint.as_ref().unwrap();
    assert_eq!(constraint.name.as_str(), "NonExistentDomain");
}

// ---------------------------------------------------------------------------
// Non-interference: ordinary parsing/lowering is unchanged
// ---------------------------------------------------------------------------

#[test]
fn ordinary_type_definition_parsing_unchanged_by_sealed_domain_support() {
    let source = r"type Option<T> = Some { value: T } | None;
fn id(x: Int) -> Int { x }";
    let module = parse(source);

    // Ordinary type still parsed correctly
    let types: Vec<_> = module
        .definitions
        .iter()
        .filter_map(|d| match d {
            Definition::Type(t) => Some(t.name.as_ref().to_string()),
            _ => None,
        })
        .collect();
    assert!(types.contains(&"Option".to_string()));

    // No sealed domains present
    assert!(sealed_domains(&module).is_empty());
}

#[test]
fn ordinary_type_lowering_version_unchanged_without_sealed_domains() {
    let lowered = lower("type Pair = MkPair { a: Int, b: Int };\n");
    assert_eq!(
        lowered.summary.version,
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        "V1 summary version should be unchanged when no sealed domains are present"
    );
    assert!(lowered.summary.exported_sealed_domains.is_empty());
}

#[test]
fn mixed_source_lowering_produces_correct_versions() {
    // Source with both ordinary types and sealed domains
    let source = r"type Status = Pending | Done;
sealed type domain Tag { A; B; }
";
    let lowered = lower(source);

    // V2 because sealed domains are present
    assert_eq!(
        lowered.summary.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );

    // Ordinary type is still lowered
    assert_eq!(lowered.type_defs.len(), 1);
    assert_eq!(lowered.type_defs[0].name, "Status");

    // Sealed domain is also present
    assert_eq!(lowered.summary.exported_sealed_domains.len(), 1);
    assert_eq!(
        lowered.summary.exported_sealed_domains[0]
            .exported_name
            .as_str(),
        "Tag"
    );
}

#[test]
fn sealed_domain_with_self_ref_and_cross_ref_lowered_correctly() {
    // Tree domain with both self-reference and cross-domain reference
    let source = r"sealed type domain Color { Red; Blue; }
sealed type domain Tree { Leaf; Branch<left: Tree, right: Tree, color: Color>; }
";
    let lowered = lower(source);

    assert_eq!(lowered.summary.exported_sealed_domains.len(), 2);

    let tree = &lowered.summary.exported_sealed_domains[1];
    assert_eq!(tree.exported_name.as_str(), "Tree");

    let branch = &tree.constructors[1];
    assert_eq!(branch.exported_name.as_str(), "Branch");
    assert_eq!(branch.fields.len(), 3);

    // left and right are self-referencing (structural)
    assert_eq!(
        branch.fields[0].structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert_eq!(
        branch.fields[1].structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    // color is a cross-domain reference (non-structural)
    assert_eq!(
        branch.fields[2].structural_status,
        StructuralFieldStatus::NonStructural
    );
    assert!(branch.fields[2].domain_constraint.is_some());
    assert_eq!(
        branch.fields[2]
            .domain_constraint
            .as_ref()
            .unwrap()
            .name
            .as_str(),
        "Color"
    );
}

#[test]
fn domain_slot_variants_correct() {
    // Verify that parsed DomainSlot variants are correct
    let source = "sealed type domain Mix { Mk<a: Type, b: Mix, c: Other>; }";
    let module = parse(source);
    let domains = sealed_domains(&module);
    let ctor = &domains[0].constructors[0];

    assert_eq!(ctor.fields[0].slot, DomainSlot::Type);
    assert_eq!(ctor.fields[1].slot, DomainSlot::DomainRef("Mix".into()));
    assert_eq!(ctor.fields[2].slot, DomainSlot::DomainRef("Other".into()));
}
