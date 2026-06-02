#![allow(non_snake_case)]
//! TASK-1008 LSP diagnostic surface evidence.

use ash_lint::LintConfig;
use ash_lsp_core::diagnostics::compute_diagnostics;
use ash_parser::surface::{
    Expr as SurfaceExpr, Literal as SurfaceLiteral, Pattern as SurfacePattern,
    Workflow as SurfaceWorkflow,
};
use ash_parser::token::Span;
use ash_typeck::{TypeCheckError, type_check_workflow};

fn span() -> Span {
    Span::default()
}

#[test]
fn cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available() {
    let checked = type_check_workflow(
        &SurfaceWorkflow::Let {
            pattern: SurfacePattern::Literal(SurfaceLiteral::Int(0)),
            expr: SurfaceExpr::Literal(SurfaceLiteral::Int(1)),
            continuation: Some(Box::new(SurfaceWorkflow::Done { span: span() })),
            span: span(),
        },
        None,
    );
    let TypeCheckError::TypeError(typeck_message) =
        checked.expect_err("typeck should reject refutable workflow binders")
    else {
        panic!("expected typecheck error");
    };
    assert!(typeck_message.contains("workflow let"), "{typeck_message}");
    assert!(typeck_message.contains("irrefutable"), "{typeck_message}");

    let source = r"
        workflow main {
            let 0 = 1;
            done
        }
    ";
    let diagnostics = compute_diagnostics(source, &LintConfig::default());
    let joined = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !joined.contains("irrefutable"),
        "LSP core typecheck diagnostics are still deferred; current diagnostics were:\n{joined}"
    );
}
