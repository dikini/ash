use std::path::{Path, PathBuf};

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreDischargeMode, CoreEffectOp, CoreExpr, CoreMultiplicity,
    CoreTrapReason, CoreType,
};
use ash_core::core_ash_text::{parse_core_expr, parse_core_file};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/ash-core/tests/fixtures/core")
        .join(name)
}

#[test]
fn parses_let_val_jump_fixture() {
    let expr = parse_core_file(fixture_path("let_val_jump.core")).unwrap();

    match expr {
        CoreExpr::LetVal {
            name, value, body, ..
        } => {
            assert_eq!(name, "x");
            assert!(matches!(
                value,
                ash_core::core_ash::CoreValue::Atom(CoreAtom::LitInt(1))
            ));
            assert!(matches!(
                *body,
                CoreExpr::Jump {
                    cont: CoreContRef::Label(_),
                    arg: CoreAtom::Var(_)
                }
            ));
        }
        other => panic!("expected let-val fixture, got {other:?}"),
    }
}

#[test]
fn parses_let_prim_if_fixture() {
    let expr = parse_core_file(fixture_path("let_prim_if.core")).unwrap();

    match expr {
        CoreExpr::LetPrim {
            name, args, body, ..
        } => {
            assert_eq!(name, "cond");
            assert_eq!(args, vec![CoreAtom::LitInt(1), CoreAtom::LitInt(10)]);
            assert!(matches!(*body, CoreExpr::If { .. }));
        }
        other => panic!("expected let-prim fixture, got {other:?}"),
    }
}

#[test]
fn parses_call_non_tail_fixture_with_lambda_value_body_expression() {
    let expr = parse_core_file(fixture_path("call_non_tail.core")).unwrap();

    match expr {
        CoreExpr::LetVal { value, body, .. } => {
            match value {
                ash_core::core_ash::CoreValue::Lam { body: lam_body, .. } => {
                    assert!(matches!(*lam_body, CoreExpr::LetPrim { .. }));
                }
                other => panic!("expected lambda value, got {other:?}"),
            }
            assert!(matches!(*body, CoreExpr::Call { .. }));
        }
        other => panic!("expected let-val call fixture, got {other:?}"),
    }
}

#[test]
fn parses_raise_handle_fixture_with_affine_resume() {
    let expr = parse_core_file(fixture_path("raise_handle.core")).unwrap();

    match expr {
        CoreExpr::Handle { clause, body } => {
            assert!(matches!(clause.op, CoreEffectOp::Capability { .. }));
            assert_eq!(clause.params.len(), 1);
            assert!(matches!(
                clause.resume.ty,
                CoreType::Cont {
                    multiplicity: CoreMultiplicity::Affine,
                    ..
                }
            ));
            assert!(matches!(*clause.body, CoreExpr::Jump { .. }));
            assert!(matches!(*body, CoreExpr::Raise { .. }));
        }
        other => panic!("expected handle fixture, got {other:?}"),
    }
}

#[test]
fn parses_contract_trap_fixture_without_contract_violation_effect() {
    let expr = parse_core_file(fixture_path("contract_trap.core")).unwrap();

    match expr {
        CoreExpr::RecordDischarge { discharge, body } => {
            assert_eq!(discharge.contract, "requires-positive");
            assert_eq!(discharge.mode, CoreDischargeMode::Dynamic);
            assert!(matches!(
                *body,
                CoreExpr::Trap {
                    reason: CoreTrapReason::ContractViolation(_)
                }
            ));
        }
        other => panic!("expected record-discharge fixture, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_and_surface_like_forms() {
    let unknown = parse_core_expr("(unknown-form x)").unwrap_err();
    assert!(
        unknown.to_string().contains("unsupported expression form"),
        "unexpected error: {unknown}"
    );

    let surface = parse_core_expr("(workflow demo { ret 1 })").unwrap_err();
    assert!(
        surface.to_string().contains("unsupported expression form"),
        "unexpected error: {surface}"
    );
}
