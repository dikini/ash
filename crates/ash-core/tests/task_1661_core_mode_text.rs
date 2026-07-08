use std::path::{Path, PathBuf};

use ash_core::core_ash::CoreThunkMode;
use ash_core::core_ash::{
    CoreAtom, CoreCaptureSet, CoreEvalMode, CoreExpr, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_text::{
    core_expr_to_string, parse_core_expr, parse_core_file, parse_type, parse_value,
};

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
fn parses_mode_types_and_round_trips() {
    assert_eq!(
        parse_type("(strict Int)").unwrap(),
        CoreType::Mode {
            mode: CoreEvalMode::Strict,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: None,
        }
    );

    assert_eq!(
        parse_type("(memo (record-type (a : Int) (b : String)) {operation db.read})").unwrap(),
        CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Record(vec![
                ("a".to_string(), CoreType::Base("Int".to_string())),
                ("b".to_string(), CoreType::Base("String".to_string())),
            ])),
            latent_row: Some(CoreRow::closed(vec![CoreRowItem::Operation {
                path: vec!["db".to_string()],
                operation: "read".to_string(),
            }])),
        }
    );

    assert_eq!(
        parse_type("(lazy String {})").unwrap(),
        CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            inner: Box::new(CoreType::Base("String".into())),
            latent_row: Some(CoreRow::default()),
        }
    );
}

#[test]
fn parses_and_serializes_thunk_mode_bindings_and_force() {
    let expr =
        parse_core_file(fixture_path("mode_forms.core")).expect("valid mode fixture should parse");
    assert_eq!(
        core_expr_to_string(&expr),
        "(let-mode t lazy : (lazy Int {}) (lit-int 1) (force v t t))"
    );

    assert!(matches!(
        expr,
        CoreExpr::LetMode {
            name: _,
            mode: CoreEvalMode::Lazy,
            ty: CoreType::Mode {
                mode: CoreEvalMode::Lazy,
                ..
            },
            ..
        }
    ));
}

#[test]
fn parses_mode_thunk_value_and_defaults_captures_empty() {
    let value = parse_value("(thunk lazy Int {} (lit-int 42))").unwrap();
    match value {
        CoreValue::Thunk {
            mode,
            result_ty,
            row,
            captures,
            ..
        } => {
            assert_eq!(mode, CoreThunkMode::Lazy);
            assert_eq!(result_ty, CoreType::Base("Int".into()));
            assert_eq!(row, CoreRow::default());
            assert_eq!(captures, CoreCaptureSet { values: vec![] });
        }
        _ => panic!("expected thunk value"),
    }
}

#[test]
fn force_and_let_mode_forms_are_accepted_from_text_roundtrip() {
    let expr = parse_core_expr(
        "(force t x (let-mode y memo : (memo Int {operation db.read}) (lit-int 1) (jump (label exit) y)))",
    )
    .unwrap();
    let round_trip = core_expr_to_string(&expr);
    assert_eq!(
        round_trip,
        "(force t x (let-mode y memo : (memo Int {operation db.read}) (lit-int 1) (jump (label exit) y)))"
    );
    assert_eq!(parse_core_expr(&round_trip).unwrap(), expr);
}

#[test]
fn accepts_invalid_mode_shape_syntax_for_later_validation() {
    let expr = parse_core_file(fixture_path("mode_invalid_type_mismatch.core"))
        .expect("invalid mode mismatch fixture should still parse");
    match expr {
        CoreExpr::LetMode { ref name, mode, .. } => {
            assert_eq!(name, "x");
            assert_eq!(mode, CoreEvalMode::Lazy);
            assert!(matches!(
                &expr,
                CoreExpr::LetMode {
                    mode: CoreEvalMode::Lazy,
                    ..
                }
            ));
        }
        _ => panic!("expected let-mode fixture"),
    }
}

#[test]
fn serializes_thunk_metadata_without_capture_payload() {
    let value = CoreValue::Thunk {
        mode: CoreThunkMode::Memo,
        result_ty: CoreType::Base("Unit".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        row: CoreRow::default(),
        captures: CoreCaptureSet {
            values: vec!["x".to_string(), "y".to_string()],
        },
    };
    let expr = CoreExpr::LetVal {
        name: "c".to_string(),
        ty: CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Base("Unit".into())),
            latent_row: Some(CoreRow::default()),
        },
        value,
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("c".to_string()))),
    };
    let text = core_expr_to_string(&expr);
    assert!(!text.contains("captures"));
}
