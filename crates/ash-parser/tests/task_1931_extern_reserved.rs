//! TASK-1931 extern decision-gate parser tests.

#[test]
fn extern_function_surface_remains_reserved_and_fails_closed() {
    let err = ash_parser::parse_surface_file(
        r#"
        extern fn host_read(path: String) -> String;
        "#,
    )
    .expect_err("extern fn must remain unavailable until a trusted-adapter phase owns it");

    let diagnostic = format!("{err:?}");
    assert!(
        diagnostic.contains("extern") || diagnostic.contains("expected"),
        "unexpected extern rejection diagnostic: {diagnostic}"
    );
}
