//! TASK-1670: Verify thunk capture authority.

use ash_core::core_ash::{CoreAtom, CoreEvalMode, CoreExpr, CoreRow, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, TypedCoreProgram, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::{
    core_ash_lower::CoreLoweringContext,
    cps::{Atom, EffectItem, EffectItemKind, Env, HandlerChain, Term, ThunkMode, Value},
};

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_owned())
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow {
        items: vec![ash_core::core_ash::CoreRowItem::Operation {
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            operation: operation.to_owned(),
        }],
        tail: None,
    }
}

fn typed_fn(row: CoreRow) -> CoreType {
    CoreType::Function {
        params: vec![],
        result: Box::new(base("Int")),
        row,
    }
}

fn expected_cap_effect_item(path: &[&str], operation: &str) -> EffectItem {
    EffectItem {
        namespace: "cap".to_string(),
        name: path.join(".") + "." + operation,
        kind: EffectItemKind::Capability,
    }
}

fn mode_type(mode: CoreEvalMode, row: CoreRow) -> CoreType {
    CoreType::Mode {
        mode,
        inner: Box::new(base("Int")),
        latent_row: Some(row),
    }
}

fn lower_program(expr: CoreExpr, env: CoreTypeCheckEnv) -> (TypedCoreProgram, Term) {
    let program = RawCoreProgram::new(expr);
    let valid = validate_core_program(program).expect("test fixture should validate");
    let context = ash_core::cps::ContRef::Label("k0".to_string());
    let context = CoreLoweringContext::new(context, CoreRow::default());
    let checked = type_check_and_lower_core_program(valid, &env, context)
        .expect("test fixture should type-check and lower");
    checked.into_parts()
}

#[test]
fn lazy_letmode_lowering_uses_empty_capture_placeholders() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut()
        .insert("maker".to_string(), typed_fn(cap_row(&["jobs"], "read")));

    let expr = CoreExpr::LetMode {
        name: "thunk".to_string(),
        mode: CoreEvalMode::Lazy,
        ty: mode_type(CoreEvalMode::Lazy, cap_row(&["jobs"], "read")),
        expr: Box::new(CoreExpr::Call {
            func: CoreAtom::Var("maker".to_string()),
            args: vec![],
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("thunk".to_string()))),
    };

    let (_, lowered) = lower_program(expr, env);

    match lowered {
        Term::LetVal { value, .. } => match value {
            Value::ThunkClosure {
                mode,
                captured_env,
                captured_chain,
                ..
            } => {
                assert_eq!(mode, ThunkMode::Lazy);
                assert_eq!(captured_env, Env::default());
                assert_eq!(captured_chain, HandlerChain::default());
            }
            _ => panic!("expected thunk closure"),
        },
        _ => panic!("expected top-level let-binding for let-mode"),
    }
}

#[test]
fn memo_force_rows_still_carry_latent_row_in_thunk_placeholder() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut()
        .insert("maker".to_string(), typed_fn(cap_row(&["db"], "write")));

    let expr = CoreExpr::LetMode {
        name: "thunk".to_string(),
        mode: CoreEvalMode::Memo,
        ty: mode_type(CoreEvalMode::Memo, cap_row(&["db"], "write")),
        expr: Box::new(CoreExpr::Call {
            func: CoreAtom::Var("maker".to_string()),
            args: vec![],
        }),
        body: Box::new(CoreExpr::Force {
            name: "forced".to_string(),
            thunk: CoreAtom::Var("thunk".to_string()),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("forced".to_string()))),
        }),
    };

    let (_, lowered) = lower_program(expr, env);

    match lowered {
        Term::LetVal { value, body, .. } => {
            match value {
                Value::ThunkClosure { mode, row, .. } => {
                    assert_eq!(mode, ThunkMode::Memo);
                    assert_eq!(row.items, vec![expected_cap_effect_item(&["db"], "write")]);
                }
                _ => panic!("expected thunk closure"),
            }

            // Force must remain an explicit primitive lowering in the checked pipeline.
            if let Term::LetPrim { op, args, .. } = *body {
                assert_eq!(op, ash_core::cps::PrimOp::ForceThunk);
                assert_eq!(args, vec![Atom::Var("thunk".to_string())]);
            } else {
                panic!("expected force primitive lowering");
            }
        }
        _ => panic!("expected top-level let-binding for memo mode"),
    }
}
