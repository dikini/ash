use ash_core::core_ash_text::parse_row_item;

#[test]
fn core_text_rejects_removed_role_and_policy_rows() {
    assert!(parse_row_item("role ops").is_err());
    assert!(parse_row_item("policy tenant.boundary").is_err());
}

#[test]
fn core_text_keeps_contract_rows() {
    let row = parse_row_item("contract payment").expect("contract rows remain supported");
    assert_eq!(
        ash_core::core_ash_text::format_row_item(&row),
        "contract payment"
    );
}
