use ash_parser::lower::{expand_and_lower_surface_module, lower_expanded_surface_module};
use ash_parser::surface::{ExpandedSurfaceModule, Expr, expand_surface_module};

#[test]
fn high_level_gate_runs_expansion_before_lowering() {
    let module = ash_parser::parse_surface_file("fn section() -> Int { (+) }")
        .expect("built-in section parses");
    expand_and_lower_surface_module(module)
        .expect("expanded built-in section reaches lowering gate");
}

#[test]
fn high_level_gate_rejects_unresolved_operator_before_core_lowering() {
    let module = ash_parser::parse_surface_file("fn bad(x: Int) -> Int { (x <??>) }")
        .expect("unknown symbolic section parses");
    let err = expand_and_lower_surface_module(module).expect_err("unresolved section rejected");
    assert!(err.to_string().contains("operator section `<??>`"));
}

#[test]
fn direct_expanded_gate_rejects_raw_sections_if_caller_constructs_invalid_carrier() {
    let module =
        ash_parser::parse_surface_file("fn bad(x: Int) -> Int { (+ x) }").expect("section parses");
    let raw = ExpandedSurfaceModule {
        module,
        diagnostics: Vec::new(),
    };
    let err = lower_expanded_surface_module(&raw)
        .expect_err("raw section in expanded carrier is rejected");
    assert!(
        err.to_string()
            .contains("operator section `+` must be resolved")
    );
}

#[test]
fn expanded_module_contains_no_raw_sections_after_successful_expansion() {
    let module = ash_parser::parse_surface_file("fn section() -> Int { (1 +) }")
        .expect("left section parses");
    let expanded = expand_surface_module(module).expect("built-in section elaborates");
    let mut raw_sections = 0usize;
    ash_parser::surface::visit_exprs_in_module(&expanded.module, &mut |expr| {
        if matches!(expr, Expr::OperatorSection { .. }) {
            raw_sections += 1;
        }
    });
    assert_eq!(raw_sections, 0);
}
