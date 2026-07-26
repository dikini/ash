//! TASK-2001 source-boundary regressions for stale declaration forms.
//!
//! The target declaration grammar must reject historical capability, proxy,
//! and yield declarations explicitly, including visibility-qualified forms.

use std::path::Path;

use ash_parser::surface::Definition;

fn assert_removed_declaration_rejects(source: &str, form: &str) {
    let errors = ash_parser::parse_surface_file_with_path(
        source,
        Some(Path::new("fixtures/task-2001-stale-declarations.ash")),
    )
    .expect_err("removed declaration forms must fail at the source boundary");

    assert_eq!(errors.len(), 1, "{source}: {errors:?}");
    assert_eq!(
        errors[0].message,
        format!("`{form}` declarations are removed from target Ash"),
        "{source}: {errors:?}"
    );
}

#[test]
fn task_2001_stale_top_level_declarations_reject_with_stable_removed_form_diagnostics() {
    for (form, source) in [
        ("capability", "capability audit {}"),
        ("proxy", "proxy assistant { return 0 }"),
        ("yield", "yield stream {}"),
    ] {
        assert_removed_declaration_rejects(source, form);
    }
}

#[test]
fn task_2001_visible_stale_declarations_reject_with_stable_removed_form_diagnostics() {
    for (form, source) in [
        ("capability", "pub capability audit {}"),
        ("proxy", "pub proxy assistant { return 0 }"),
        ("yield", "pub yield stream {}"),
    ] {
        assert_removed_declaration_rejects(source, form);
    }
}

#[test]
fn task_2001_active_effect_alias_and_handler_controls_remain_accepted() {
    let module = ash_parser::parse_surface_file_with_path(
        "pub effect alias Audit = {evidence audit_log};\npub handler identity(comp: Unit) -> Unit { comp }",
        Some(Path::new("fixtures/task-2001-active-declarations.ash")),
    )
    .expect("active target declarations must not be rejected by stale-form handling");

    assert_eq!(
        module.path.as_deref(),
        Some("fixtures/task-2001-active-declarations.ash")
    );
    assert!(matches!(module.definitions[0], Definition::EffectAlias(_)));
    assert!(matches!(module.definitions[1], Definition::Handler(_)));
}
