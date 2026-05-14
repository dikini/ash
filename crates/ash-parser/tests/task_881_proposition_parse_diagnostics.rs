//! TASK-881: parser-owned proposition diagnostic helpers.

use ash_diagnostic::{AshLspError, Severity};
use ash_parser::error::ParseError;
use ash_parser::parse_surface_file;
use ash_parser::token::Span;

#[test]
fn task_881_parser_unsupported_proposition_surface_has_stable_code_and_help() {
    let span = Span::new(4, 12, 1, 5);
    let err = ParseError::unsupported_proposition_surface(
        span,
        "type alias",
        "move the proposition tail to an enabled type fn, fn, or builtin fn declaration",
    );

    assert_eq!(err.code().expect("stable parser code").0, "E168");
    assert_eq!(err.severity(), Severity::Error);
    assert_eq!(err.span(), Some(span.into()));
    assert!(err.message.contains("unsupported proposition syntax"));
    assert!(err.message.contains("type alias"));
    assert!(
        err.message
            .contains("expected enabled proposition tail site")
    );
    assert!(err.message.contains("next step"));
}

#[test]
fn task_881_parse_surface_file_routes_disabled_proposition_surface_to_e168() {
    let err = parse_surface_file("type Alias = Int where Int == Int")
        .expect_err("unsupported proposition tail on type alias must use E168");
    assert_eq!(err.len(), 1);
    let err = &err[0];

    assert_eq!(err.code().expect("stable parser code").0, "E168");
    assert!(err.message.contains("unsupported proposition syntax"));
    assert!(err.message.contains(
        "move the proposition tail to an enabled type fn, fn, or builtin fn declaration"
    ));
}

#[test]
fn task_881_parse_surface_file_does_not_mask_unrelated_errors_near_valid_tail() {
    let err = parse_surface_file(
        r#"type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    where Append<Nil, ys> == ys
{
    case Append<Nil, ys> = ys;
}
workflow main {
    let x = 1 == ;
    done
}"#,
    )
    .expect_err("malformed workflow equality should remain generic parser error");
    assert_eq!(err.len(), 1);
    let err = &err[0];

    assert_eq!(err.code().expect("stable parser code").0, "E001");
    assert!(!err.message.contains("unsupported proposition syntax"));
}

#[test]
fn task_881_parse_surface_file_does_not_mask_workflow_where_errors() {
    let err = parse_surface_file(
        r#"workflow main {
    where x
    done
}"#,
    )
    .expect_err("workflow-body where error should remain generic parser error");
    assert_eq!(err.len(), 1);
    let err = &err[0];

    assert_eq!(err.code().expect("stable parser code").0, "E001");
    assert!(!err.message.contains("unsupported proposition syntax"));
}

#[test]
fn task_881_parse_surface_file_does_not_mask_legacy_impl_where_errors() {
    let err = parse_surface_file("impl Explain<Int> where T: Debug { explain(value) = }")
        .expect_err("malformed impl body after legacy where bound should remain generic");
    assert_eq!(err.len(), 1);
    let err = &err[0];

    assert_eq!(err.code().expect("stable parser code").0, "E001");
    assert!(!err.message.contains("unsupported proposition syntax"));
}
