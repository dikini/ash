use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_text::{core_expr_to_string, parse_core_expr, parse_row, parse_row_item};
use ash_core::core_ash_typecheck::{
    CorePublicRowItemSummary, normalize_core_row, summarize_core_public_row,
};

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[test]
fn operation_row_text_aliases_parse_to_compatibility_variant() {
    let expected = CoreRowItem::Capability {
        path: path(&["console"]),
        operation: "read".to_owned(),
    };

    assert_eq!(
        parse_row_item("operation console.read").expect("operation row alias parses"),
        expected
    );
    assert_eq!(
        parse_row_item("op console.read").expect("short operation row alias parses"),
        expected
    );
    assert_eq!(
        parse_row_item("cap console.read").expect("legacy capability row parses"),
        expected
    );
}

#[test]
fn operation_constructor_documents_legacy_capability_storage() {
    let item = CoreRowItem::operation(path(&["console"]), "read");

    assert!(item.is_operation_requirement());
    assert_eq!(
        item,
        CoreRowItem::Capability {
            path: path(&["console"]),
            operation: "read".to_owned()
        }
    );
}

#[test]
fn target_row_families_parse_and_public_summary_preserves_family_identity() {
    let row = parse_row(
        "{operation console.read, resource fs read, role tenant.admin, policy pii.redact, \
         contract Signed, channel inbox recv String, proc spawn, fail String, evidence sig, tail r}",
    )
    .expect("target taxonomy row parses");

    let summary = summarize_core_public_row(&row).expect("row summary is public");

    assert_eq!(summary.tail(), Some("r"));
    assert_eq!(
        summary.items(),
        &[
            CorePublicRowItemSummary::Capability {
                path: path(&["console"]),
                operation: "read".to_owned(),
            },
            CorePublicRowItemSummary::Resource {
                path: path(&["fs"]),
                mode: "read".to_owned(),
            },
            CorePublicRowItemSummary::Role {
                path: path(&["tenant", "admin"]),
            },
            CorePublicRowItemSummary::Policy {
                path: path(&["pii", "redact"]),
            },
            CorePublicRowItemSummary::Contract {
                contract: "Signed".to_owned(),
            },
            CorePublicRowItemSummary::Channel {
                path: path(&["inbox"]),
                mode: "recv".to_owned(),
                payload_type: Box::new(CoreType::Base("String".to_owned())),
            },
            CorePublicRowItemSummary::Process {
                operation: "spawn".to_owned(),
            },
            CorePublicRowItemSummary::Failure {
                ty: Some(Box::new(CoreType::Base("String".to_owned()))),
            },
            CorePublicRowItemSummary::Evidence {
                path: path(&["sig"]),
            },
        ]
    );
}

#[test]
fn normalization_is_idempotent_and_preserves_target_family_boundaries() {
    let operation = CoreRowItem::operation(path(&["fs"]), "read");
    let role = CoreRowItem::Role {
        path: path(&["fs", "read"]),
    };
    let row = CoreRow::closed(vec![operation.clone(), role.clone(), operation.clone()]);

    let once = normalize_core_row(&row).expect("row normalizes");
    let twice = normalize_core_row(&once).expect("normalized row normalizes again");

    assert_eq!(once, twice);
    assert_eq!(once, CoreRow::closed(vec![operation, role]));
}

#[test]
fn operation_alias_round_trips_through_canonical_core_expression_text() {
    let source = "(let-val f : (fn () -> Unit {operation console.read}) \
                  (lam () : {operation console.read} (lit-unit)) \
                  (jump (label exit) f))";

    let expr = parse_core_expr(source).expect("operation row alias parses in expressions");
    let canonical = core_expr_to_string(&expr);

    assert!(canonical.contains("{cap console.read}"));
    assert_eq!(
        parse_core_expr(&canonical).expect("canonical Core expression reparses"),
        expr
    );
}
