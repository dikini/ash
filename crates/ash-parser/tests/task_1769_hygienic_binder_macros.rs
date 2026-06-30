use ash_parser::surface::{Definition, Expr, IdentifierHygieneContext, expand_surface_module};

fn function_body(expanded: &ash_parser::surface::ExpandedSurfaceModule, name: &str) -> Expr {
    expanded
        .module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => {
                Some(function.body.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("function `{name}` exists"))
}

fn block_tail(expr: &Expr) -> &Expr {
    match expr {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }
}

#[test]
fn binder_macro_renames_generated_function_parameters_and_uses() {
    let module = ash_parser::parse_surface_file(
        r#"
macro identity_fn(v) => fn(v: Int) -> Int { v };
fn use_macro() -> Int { identity_fn!(0) }
"#,
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("binder macro expands");
    let body_expr = function_body(&expanded, "use_macro");
    let Expr::FnDef { params, body, .. } = block_tail(&body_expr) else {
        panic!("expected macro to expand to anonymous fn expression")
    };

    assert_eq!(params.len(), 1);
    let generated_param = params[0].0.as_ref();
    assert!(
        generated_param.starts_with("$ash_generated_macro_1_v_"),
        "expected generated binder, got {generated_param}"
    );
    let Expr::Variable { name, .. } = block_tail(body) else {
        panic!("expected generated binder use in fn body")
    };
    assert_eq!(name.as_ref(), generated_param);
    assert!(expanded.hygiene.iter().any(|item| {
        item.name.as_ref() == generated_param
            && item.context == IdentifierHygieneContext::Generated
            && item.expansion_id.is_some()
    }));
}

#[test]
fn generated_binders_do_not_capture_call_site_arguments() {
    let module = ash_parser::parse_surface_file(
        r#"
macro const_fn(y) => fn(x: Int) -> Int { y };
fn use_macro(x: Int) -> Int { const_fn!(x) }
"#,
    )
    .expect("module parses");

    let expanded = expand_surface_module(module).expect("binder macro expands");
    let body_expr = function_body(&expanded, "use_macro");
    let Expr::FnDef { params, body, .. } = block_tail(&body_expr) else {
        panic!("expected macro to expand to anonymous fn expression")
    };
    assert!(
        params[0]
            .0
            .as_ref()
            .starts_with("$ash_generated_macro_1_x_")
    );
    let Expr::Variable { name, .. } = block_tail(body) else {
        panic!("expected call-site argument variable in generated body")
    };
    assert_eq!(
        name.as_ref(),
        "x",
        "macro parameter substitution must preserve call-site variable identity"
    );
}

#[test]
fn unsupported_body_binders_remain_fail_closed() {
    let module = ash_parser::parse_surface_file(
        r#"
macro with_block(x) => fn(y: Int) -> Int { let z = x; z };
fn use_macro(n: Int) -> Int { with_block!(n) }
"#,
    )
    .expect("module parses");

    let err = expand_surface_module(module).expect_err("block binders remain outside TASK-1769");
    assert!(
        err.to_string()
            .contains("macro `with_block` uses unsupported template syntax: block"),
        "unexpected error: {err}"
    );
}
