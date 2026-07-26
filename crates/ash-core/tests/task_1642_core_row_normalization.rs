use ash_core::core_ash::{CoreMultiplicity, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, core_row_included_in, core_types_equivalent,
    normalize_core_row,
};
use proptest::prelude::*;
use std::collections::HashSet;

fn operation(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Operation {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn role(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Role {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
    }
}

fn chan(path: &[&str], mode: &str, payload: CoreType) -> CoreRowItem {
    CoreRowItem::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: Box::new(payload),
    }
}

fn function_row_type(row: CoreRow) -> CoreType {
    CoreType::Function {
        params: vec![],
        result: Box::new(CoreType::Base("Unit".into())),
        row,
    }
}

fn cont_type(row: CoreRow) -> CoreType {
    CoreType::Cont {
        input: Box::new(CoreType::Base("String".into())),
        answer: Box::new(CoreType::Base("Unit".into())),
        row,
        multiplicity: CoreMultiplicity::Affine,
    }
}

#[test]
fn normalization_removes_exact_duplicate_items() {
    let read = operation(&["fs"], "read");
    let row = CoreRow::closed(vec![read.clone(), read.clone()]);

    let normalized = normalize_core_row(&row).expect("row normalizes");

    assert_eq!(normalized, CoreRow::closed(vec![read]));
}

#[test]
fn normalization_preserves_effect_kind_namespaces() {
    let read_operation = operation(&["fs"], "read");
    let read_role = role(&["fs", "read"]);
    let row = CoreRow::closed(vec![read_operation.clone(), read_role.clone()]);

    let normalized = normalize_core_row(&row).expect("row normalizes");

    assert_eq!(normalized, CoreRow::closed(vec![read_operation, read_role]));
}

#[test]
fn rows_compare_equal_when_items_are_reordered_for_function_types() {
    let left = function_row_type(CoreRow::closed(vec![
        role(&["tenant", "primary"]),
        operation(&["fs"], "write"),
    ]));
    let right = function_row_type(CoreRow::closed(vec![
        operation(&["fs"], "write"),
        role(&["tenant", "primary"]),
    ]));

    assert!(
        core_types_equivalent(&left, &right, &CoreTypeCheckEnv::default())
            .expect("comparison should not fail"),
        "function rows should compare equivalent regardless of item order"
    );
}

#[test]
fn rows_compare_equal_when_items_are_reordered_for_continuation_types() {
    let left = cont_type(CoreRow::closed(vec![
        operation(&["fs"], "write"),
        operation(&["audit"], "emit"),
    ]));
    let right = cont_type(CoreRow::closed(vec![
        operation(&["audit"], "emit"),
        operation(&["fs"], "write"),
    ]));

    assert!(
        core_types_equivalent(&left, &right, &CoreTypeCheckEnv::default())
            .expect("comparison should not fail"),
        "continuation rows should compare equivalent regardless of item order"
    );
}

#[test]
fn public_row_inclusion_uses_exact_row_item_matching() {
    let named_payload = CoreType::Named("Payload".into());
    let app_payload = CoreType::App {
        name: "Box".into(),
        args: vec![CoreType::Base("Int".into())],
    };
    let actual = CoreRow::closed(vec![
        chan(&["jobs"], "send", named_payload.clone()),
        chan(&["jobs"], "send", app_payload.clone()),
    ]);
    let expected = CoreRow::closed(vec![
        chan(&["jobs"], "send", named_payload),
        chan(&["jobs"], "send", app_payload),
    ]);

    let comparison =
        core_row_included_in(&actual, &expected).expect("public API should not require type env");

    assert!(comparison.is_included());
    assert!(comparison.missing_items().is_empty());
}

#[test]
fn closed_row_inclusion_succeeds_for_subset_and_fails_for_missing_item() {
    let read = operation(&["fs"], "read");
    let write_log = operation(&["log"], "write");
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
    let read = operation(&["fs"], "read");
    let write_log = operation(&["log"], "write");
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
    let read = operation(&["fs"], "read");
    let actual = CoreRow::open(vec![read.clone()], "r");
    let expected = CoreRow::open(vec![read], "s");

    let comparison = core_row_included_in(&actual, &expected).expect("open rows compare");

    assert!(!comparison.is_included());
    assert!(comparison.solutions().is_empty());
}

#[test]
fn role_items_are_not_expanded_into_capabilities() {
    let actual = CoreRow::closed(vec![role(&["admin"])]);
    let expected = CoreRow::closed(vec![operation(&["fs"], "read")]);

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

// The closed-row generator deliberately uses more than one item namespace.
// This makes the executable law suite reject a mutation that deduplicates or
// includes rows by textual path alone instead of the full `CoreRowItem` value.
fn closed_row_item_strategy() -> impl Strategy<Value = CoreRowItem> {
    prop_oneof![
        ("[a-z]{1,4}", "[a-z]{1,4}").prop_map(|(path, operation)| {
            CoreRowItem::Operation {
                path: vec![path],
                operation,
            }
        }),
        "[a-z]{1,4}".prop_map(|path| CoreRowItem::Role { path: vec![path] }),
        "[a-z]{1,4}".prop_map(|path| CoreRowItem::Policy { path: vec![path] }),
        "[a-z]{1,4}".prop_map(|contract| CoreRowItem::Contract { contract }),
    ]
}

fn expected_first_occurrences(items: &[CoreRowItem]) -> Vec<CoreRowItem> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|item| seen.insert((*item).clone()))
        .cloned()
        .collect()
}

proptest! {
    /// Normalization is a membership-preserving, duplicate-eliminating map on
    /// closed rows.  Both directions matter: an implementation must neither
    /// invent nor remove a distinct row item.
    #[test]
    fn closed_normalization_preserves_membership_and_eliminates_duplicates(
        items in prop::collection::vec(closed_row_item_strategy(), 0..32),
    ) {
        let input = CoreRow::closed(items.clone());
        let normalized = normalize_core_row(&input).expect("closed rows normalize");

        prop_assert!(normalized.items.len() <= input.items.len());
        prop_assert!(normalized.items.iter().all(|item| input.items.contains(item)));
        prop_assert!(input.items.iter().all(|item| normalized.items.contains(item)));

        let distinct: HashSet<_> = normalized.items.iter().collect();
        prop_assert_eq!(distinct.len(), normalized.items.len());
    }

    /// `normalize_core_row` is idempotent and preserves the first structural
    /// occurrence of each exact `CoreRowItem`.
    #[test]
    fn closed_normalization_is_idempotent_and_stable_first_occurrence(
        items in prop::collection::vec(closed_row_item_strategy(), 0..32),
    ) {
        let input = CoreRow::closed(items.clone());
        let normalized = normalize_core_row(&input).expect("closed rows normalize");
        let normalized_twice = normalize_core_row(&normalized).expect("normalized rows normalize");

        prop_assert_eq!(&normalized, &normalized_twice);
        prop_assert_eq!(normalized.items, expected_first_occurrences(&items));
        prop_assert_eq!(normalized.tail, None);
    }

    /// Closed inclusion is reflexive.  The comparison is a truth predicate
    /// over set-like membership, even though normalization retains a stable
    /// output order for diagnostics and reproducibility.
    #[test]
    fn closed_row_inclusion_is_reflexive(
        items in prop::collection::vec(closed_row_item_strategy(), 0..32),
    ) {
        let row = CoreRow::closed(items);
        let comparison = core_row_included_in(&row, &row).expect("closed rows compare");

        prop_assert!(comparison.is_included());
        prop_assert!(comparison.missing_items().is_empty());
        prop_assert!(comparison.solutions().is_empty());
    }

    /// Construct arbitrary generated closed-row chains A <= B <= C and
    /// exercise transitivity of the executable inclusion predicate.
    #[test]
    fn closed_row_inclusion_is_transitive_on_generated_chains(
        items in prop::collection::vec(closed_row_item_strategy(), 0..32),
    ) {
        let unique = expected_first_occurrences(&items);
        let a = CoreRow::closed(
            unique.iter().enumerate()
                .filter(|(index, _)| index % 3 == 0)
                .map(|(_, item)| item.clone())
                .collect(),
        );
        let b = CoreRow::closed(
            unique.iter().enumerate()
                .filter(|(index, _)| index % 3 != 2)
                .map(|(_, item)| item.clone())
                .collect(),
        );
        let c = CoreRow::closed(unique);

        let a_in_b = core_row_included_in(&a, &b).expect("closed rows compare");
        let b_in_c = core_row_included_in(&b, &c).expect("closed rows compare");
        let a_in_c = core_row_included_in(&a, &c).expect("closed rows compare");

        prop_assert!(a_in_b.is_included());
        prop_assert!(b_in_c.is_included());
        prop_assert!(a_in_c.is_included());
    }

    /// Reordering either side of a closed-row comparison changes neither the
    /// normalized membership set nor the inclusion truth value.
    #[test]
    fn closed_row_membership_and_inclusion_truth_are_permutation_invariant(
        actual_items in prop::collection::vec(closed_row_item_strategy(), 0..24),
        expected_items in prop::collection::vec(closed_row_item_strategy(), 0..24),
        actual_rotation in any::<usize>(),
        expected_rotation in any::<usize>(),
    ) {
        let mut reordered_actual = actual_items.clone();
        if !reordered_actual.is_empty() {
            let rotation = actual_rotation % reordered_actual.len();
            reordered_actual.rotate_left(rotation);
        }

        let mut reordered_expected = expected_items.clone();
        if !reordered_expected.is_empty() {
            let rotation = expected_rotation % reordered_expected.len();
            reordered_expected.rotate_left(rotation);
        }

        let actual = CoreRow::closed(actual_items);
        let expected = CoreRow::closed(expected_items);
        let permuted_actual = CoreRow::closed(reordered_actual);
        let permuted_expected = CoreRow::closed(reordered_expected);

        let baseline = core_row_included_in(&actual, &expected).expect("closed rows compare");
        let permuted = core_row_included_in(&permuted_actual, &permuted_expected)
            .expect("permuted closed rows compare");

        prop_assert_eq!(baseline.is_included(), permuted.is_included());
        let normalized_actual = normalize_core_row(&actual).expect("closed rows normalize");
        let normalized_permuted =
            normalize_core_row(&permuted_actual).expect("permuted closed rows normalize");
        prop_assert_eq!(
            normalized_actual.items.iter().collect::<HashSet<_>>(),
            normalized_permuted.items.iter().collect::<HashSet<_>>(),
        );
    }
}

#[test]
fn mutation_sentinel_normalization_keeps_the_first_not_last_duplicate_occurrence() {
    let read = operation(&["fs"], "read");
    let write = operation(&["fs"], "write");
    let row = CoreRow::closed(vec![write.clone(), read.clone(), write]);

    assert_eq!(
        normalize_core_row(&row).expect("closed rows normalize"),
        CoreRow::closed(vec![operation(&["fs"], "write"), read]),
        "catches a last-occurrence or unstable-order deduplication mutation"
    );
}

#[test]
fn mutation_sentinel_rows_are_requirements_not_authority_grants() {
    let role_requirement = CoreRow::closed(vec![role(&["ops"])]);
    let operation_requirement = CoreRow::closed(vec![operation(&["ops"], "deploy")]);

    let comparison = core_row_included_in(&role_requirement, &operation_requirement)
        .expect("closed rows compare");

    assert!(!comparison.is_included());
    assert_eq!(comparison.missing_items(), &[role(&["ops"])]);
}

#[test]
fn mutation_sentinel_ambiguous_groups_are_errors_on_both_inclusion_sides() {
    let group = CoreRow::closed(vec![CoreRowItem::EffectGroupRef {
        path: vec!["public".into(), "io".into()],
    }]);
    let closed = CoreRow::closed(vec![operation(&["fs"], "read")]);

    assert!(matches!(
        core_row_included_in(&group, &closed),
        Err(CoreTypeCheckError::AmbiguousRowReference { .. })
    ));
    assert!(matches!(
        core_row_included_in(&closed, &group),
        Err(CoreTypeCheckError::AmbiguousRowReference { .. })
    ));
}
