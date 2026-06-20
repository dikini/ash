use ash_core::core_ash::{CoreRow, CoreRowItem};
use ash_core::core_ash_typecheck::{CoreTypeCheckError, core_row_included_in, normalize_core_row};

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn role(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Role {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
    }
}

#[test]
fn normalization_removes_exact_duplicate_items() {
    let read = cap(&["fs"], "read");
    let row = CoreRow::closed(vec![read.clone(), read.clone()]);

    let normalized = normalize_core_row(&row).expect("row normalizes");

    assert_eq!(normalized, CoreRow::closed(vec![read]));
}

#[test]
fn normalization_preserves_effect_kind_namespaces() {
    let read_capability = cap(&["fs"], "read");
    let read_role = role(&["fs", "read"]);
    let row = CoreRow::closed(vec![read_capability.clone(), read_role.clone()]);

    let normalized = normalize_core_row(&row).expect("row normalizes");

    assert_eq!(
        normalized,
        CoreRow::closed(vec![read_capability, read_role])
    );
}

#[test]
fn closed_row_inclusion_succeeds_for_subset_and_fails_for_missing_item() {
    let read = cap(&["fs"], "read");
    let write_log = cap(&["log"], "write");
    let actual = CoreRow::closed(vec![read.clone()]);
    let expected = CoreRow::closed(vec![read.clone(), write_log.clone()]);

    let success = core_row_included_in(&actual, &expected).expect("closed rows compare");
    assert!(success.is_included());
    assert!(success.solutions().is_empty());

    let failure = core_row_included_in(&expected, &actual).expect("closed rows compare");
    assert!(!failure.is_included());
    assert_eq!(failure.missing_items(), &[write_log]);
}

#[test]
fn open_row_inclusion_solves_structural_remainder() {
    let read = cap(&["fs"], "read");
    let write_log = cap(&["log"], "write");
    let actual = CoreRow::open(vec![read.clone()], "r");
    let expected = CoreRow::closed(vec![read, write_log.clone()]);

    let comparison = core_row_included_in(&actual, &expected).expect("open row compares");

    assert!(comparison.is_included());
    assert_eq!(comparison.solutions().len(), 1);
    assert_eq!(comparison.solutions()[0].variable(), "r");
    assert_eq!(
        comparison.solutions()[0].row(),
        &CoreRow::closed(vec![write_log])
    );
}

#[test]
fn different_open_row_tails_are_not_solved_implicitly() {
    let read = cap(&["fs"], "read");
    let actual = CoreRow::open(vec![read.clone()], "r");
    let expected = CoreRow::open(vec![read], "s");

    let comparison = core_row_included_in(&actual, &expected).expect("open rows compare");

    assert!(!comparison.is_included());
    assert!(comparison.solutions().is_empty());
}

#[test]
fn role_items_are_not_expanded_into_capabilities() {
    let actual = CoreRow::closed(vec![role(&["admin"])]);
    let expected = CoreRow::closed(vec![cap(&["fs"], "read")]);

    let comparison = core_row_included_in(&actual, &expected).expect("rows compare");

    assert!(!comparison.is_included());
    assert_eq!(comparison.missing_items(), &[role(&["admin"])]);
}

#[test]
fn ambiguous_group_references_are_rejected_before_solving() {
    let row = CoreRow::closed(vec![CoreRowItem::EffectGroupRef {
        path: vec!["public".into(), "io".into()],
    }]);

    let err = normalize_core_row(&row).expect_err("group reference must be expanded first");

    assert_eq!(
        err,
        CoreTypeCheckError::AmbiguousRowReference {
            detail: "effect group public.io must be expanded before row comparison".into()
        }
    );
}
