use ash_parser::surface::{
    ExpansionError, MacroHygienePolicy, MacroInputKind, MacroOutputKind, Visibility,
    collect_public_macro_summaries,
};

#[test]
fn public_macros_produce_syntax_phase_summaries() {
    let module = ash_parser::parse_surface_file(
        r"
macro private_inc(x) => add(x, 1);
pub macro inc(x) => add(x, 1);
",
    )
    .expect("module parses");

    let summaries = collect_public_macro_summaries(&module, "provider")
        .expect("public macro summaries collect");

    assert_eq!(summaries.len(), 1);
    let summary = &summaries[0];
    assert_eq!(summary.module_path.as_ref(), "provider");
    assert_eq!(summary.name.as_ref(), "inc");
    assert_eq!(summary.visibility, Visibility::Public);
    assert_eq!(summary.params, vec!["x".into()]);
    assert_eq!(summary.input_kind, MacroInputKind::ExprArgs);
    assert_eq!(summary.output_kind, MacroOutputKind::Expr);
    assert_eq!(
        summary.hygiene_policy,
        MacroHygienePolicy::BinderFreeExpression
    );
    assert!(summary.typed_signature.is_none());
    assert_eq!(summary.template_fingerprint.param_count, 1);
    assert!(
        summary.template_fingerprint.body_span.start < summary.template_fingerprint.body_span.end
    );
    assert!(summary.origin_span.start < summary.origin_span.end);
}

#[test]
fn malformed_public_macro_summary_rejects_before_export() {
    let module = ash_parser::parse_surface_file("pub macro bad(x) => y;").expect("module parses");

    let err = collect_public_macro_summaries(&module, "provider")
        .expect_err("public summary with free template variable rejects");

    assert!(matches!(
        err,
        ExpansionError::UnsupportedMacroTemplate { ref name, ref reason, .. }
            if name.as_ref() == "bad" && reason.as_ref() == "free variable"
    ));
}
