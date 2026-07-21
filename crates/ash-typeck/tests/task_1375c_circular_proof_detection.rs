use ash_parser::surface::{
    Definition, Expr, FnDef, Literal, ProgramEntry, ProofBody, ProofDef, Visibility,
};
use ash_parser::token::Span;

fn name(value: &str) -> ash_parser::surface::Name {
    value.into()
}

fn parse_module(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn proof_expr(name_str: &str, body_expr: Expr) -> ProofDef {
    ProofDef {
        name: name(name_str),
        params: vec![],
        constraints: vec![],
        body: ProofBody::Expr(body_expr),
        span: Span::default(),
    }
}

fn call_expr(func: &str) -> Expr {
    Expr::Call {
        func: name(func),
        module: None,
        args: vec![],
        span: Span::default(),
    }
}

fn qualified_call_expr(module: &str, func: &str) -> Expr {
    Expr::Call {
        func: name(func),
        module: Some(name(module)),
        args: vec![],
        span: Span::default(),
    }
}

fn call_expr_with_arg(func: &str, arg: Expr) -> Expr {
    Expr::Call {
        func: name(func),
        module: None,
        args: vec![arg],
        span: Span::default(),
    }
}

fn literal_true() -> Expr {
    Expr::Literal(Literal::Bool(true))
}

// ============================================================
// Circular dependency tests (should fail)
// ============================================================

#[test]
fn two_proofs_calling_each_other_is_rejected() {
    let proofs = vec![
        proof_expr("proof_a", call_expr("proof_b")),
        proof_expr("proof_b", call_expr("proof_a")),
    ];

    let err = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect_err("mutually recursive proofs should be rejected");

    let msg = err.to_string();
    assert!(
        msg.contains("circular proof dependency"),
        "expected circular proof dependency error, got: {msg}"
    );
}

#[test]
fn three_proofs_in_a_cycle_is_rejected() {
    let proofs = vec![
        proof_expr("proof_a", call_expr("proof_b")),
        proof_expr("proof_b", call_expr("proof_c")),
        proof_expr("proof_c", call_expr("proof_a")),
    ];

    let err = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect_err("three-way cyclic proofs should be rejected");

    let msg = err.to_string();
    assert!(
        msg.contains("circular proof dependency"),
        "expected circular proof dependency error, got: {msg}"
    );
}

#[test]
fn proof_calling_itself_is_rejected() {
    let proofs = vec![proof_expr("proof_a", call_expr("proof_a"))];

    let err = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect_err("self-recursive proof should be rejected");

    let msg = err.to_string();
    assert!(
        msg.contains("circular proof dependency"),
        "expected circular proof dependency error, got: {msg}"
    );
}

// ============================================================
// Acyclic tests (should pass)
// ============================================================

#[test]
fn acyclic_chain_passes() {
    let proofs = vec![
        proof_expr("proof_a", call_expr("proof_b")),
        proof_expr("proof_b", call_expr("proof_c")),
        proof_expr("proof_c", literal_true()),
    ];

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect("acyclic proof chain should pass");
}

#[test]
fn proof_with_no_calls_passes() {
    let proofs = vec![proof_expr("proof_a", literal_true())];

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect("proof with no calls should pass");
}

#[test]
fn proof_calling_non_proof_function_passes() {
    let proofs = vec![proof_expr("proof_a", call_expr("some_regular_fn"))];

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect("proof calling non-proof function should pass");
}

#[test]
fn qualified_call_with_matching_local_proof_name_is_not_local_cycle() {
    let proofs = vec![proof_expr(
        "proof_a",
        qualified_call_expr("other", "proof_a"),
    )];

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect("module-qualified calls should not be treated as local proof calls by basename");
}

#[test]
fn duplicate_proof_names_are_rejected_before_cycle_detection() {
    let proofs = vec![
        proof_expr("proof_a", call_expr("proof_b")),
        proof_expr("proof_b", call_expr("proof_a")),
        proof_expr("proof_a", literal_true()),
    ];

    let err = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect_err("duplicate proof names should not overwrite call graph entries");

    let msg = err.to_string();
    assert!(
        msg.contains("duplicate proof declaration"),
        "expected duplicate proof diagnostic, got: {msg}"
    );
}

#[test]
fn mixed_acyclic_and_non_proof_calls_passes() {
    let proofs = vec![
        proof_expr(
            "proof_a",
            call_expr_with_arg("some_fn", call_expr("proof_b")),
        ),
        proof_expr("proof_b", literal_true()),
    ];

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_cycles(&proofs)
        .expect("mixed acyclic proof calls should pass");
}

// ============================================================
// Integration: cycle detection via typecheck_program
// ============================================================

fn program_from_module(module: ash_parser::surface::ModuleFile) -> ash_parser::surface::Program {
    let mut definitions = module.definitions;
    definitions.push(Definition::Function(FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Null),
        span: Span::default(),
    }));
    ash_parser::surface::Program {
        definitions,
        entry: ProgramEntry {
            function: "main".into(),
            span: Span::default(),
        },
    }
}

#[test]
fn module_with_circular_proofs_is_rejected_by_typecheck() {
    let module = parse_module(
        r#"
        law reflexive(x: Int): x == x
        law symmetric(x: Int, y: Int): x == y
        proof reflexive(x: Int) {
            symmetric(x, x)
        }
        proof symmetric(x: Int, y: Int) {
            reflexive(x)
        }
        "#,
    );

    let err = ash_typeck::type_check_program(&program_from_module(module))
        .expect_err("module with circular proofs should be rejected");

    let msg = err.to_string();
    assert!(
        msg.contains("circular proof dependency"),
        "expected circular proof dependency error from typecheck, got: {msg}"
    );
}

#[test]
fn module_with_acyclic_proofs_passes_typecheck() {
    let module = parse_module(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            true
        }
        "#,
    );

    ash_typeck::type_check_program(&program_from_module(module))
        .expect("module with acyclic proofs should pass typecheck");
}
