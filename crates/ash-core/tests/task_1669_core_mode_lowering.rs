use ash_core::core_ash::{CoreAtom, CoreEvalMode, CoreExpr, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, TypedCoreProgram, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::{
    core_ash_lower::CoreLoweringContext,
    cps::{
        self, EffectItem, EffectItemKind, Env, HandlerChain, PrimOp, Term, ThunkMode,
        Value as LoweredValue,
    },
};

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_owned())
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow {
        items: vec![CoreRowItem::Capability {
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            operation: operation.to_owned(),
        }],
        tail: None,
    }
}

fn typed_fn(param_types: Vec<CoreType>, result: CoreType, row: CoreRow) -> CoreType {
    CoreType::Function {
        params: param_types,
        result: Box::new(result),
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

fn mode_type(mode: CoreEvalMode, inner: CoreType, latent_row: CoreRow) -> CoreType {
    CoreType::Mode {
        mode,
        inner: Box::new(inner),
        latent_row: Some(latent_row),
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
fn lazy_letmode_lowers_initializer_as_thunk_closure() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "maker".to_string(),
        typed_fn(vec![], base("Int"), cap_row(&["jobs"], "read")),
    );

    let expr = CoreExpr::LetMode {
        name: "thunk".to_string(),
        mode: CoreEvalMode::Lazy,
        ty: mode_type(CoreEvalMode::Lazy, base("Int"), cap_row(&["jobs"], "read")),
        expr: Box::new(CoreExpr::Call {
            func: CoreAtom::Var("maker".to_string()),
            args: vec![],
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("thunk".to_string()))),
    };

    let (_, lowered) = lower_program(expr, env);

    match lowered {
        Term::LetVal { name, value, .. } => {
            assert_eq!(name, "thunk");
            match value {
                LoweredValue::ThunkClosure {
                    mode,
                    body,
                    captured_env,
                    captured_chain,
                    row,
                    memo_cell,
                    ..
                } => {
                    assert_eq!(mode, ThunkMode::Lazy);
                    assert_eq!(row.items, vec![expected_cap_effect_item(&["jobs"], "read")]);
                    assert_eq!(captured_env, Env::default());
                    assert_eq!(captured_chain, HandlerChain::default());
                    assert!(memo_cell.is_none());
                    match *body {
                        LoweredValue::Lam {
                            params, body, row, ..
                        } => {
                            assert!(params.is_empty());
                            assert_eq!(
                                row.items,
                                vec![expected_cap_effect_item(&["jobs"], "read")]
                            );
                            match *body {
                                Term::Call {
                                    func,
                                    args,
                                    cont,
                                    row,
                                    ..
                                } => {
                                    assert_eq!(func, cps::Atom::Var("maker".to_string()));
                                    assert!(args.is_empty());
                                    assert_eq!(cont, cps::ContRef::Var("__k0".to_string()));
                                    assert_eq!(
                                        row.items,
                                        vec![expected_cap_effect_item(&["jobs"], "read")]
                                    );
                                }
                                other => {
                                    panic!("expected thunk lambda body jump, got {other:?}")
                                }
                            }
                        }
                        other => panic!("expected thunk body lambda, got {other:?}"),
                    }
                }
                _ => panic!("lazy letmode should lower through ThunkClosure"),
            }
        }
        other => panic!("expected top-level LetVal for lazy letmode, got {other:?}"),
    }
}

#[test]
fn strict_letmode_lowering_remains_direct_binding() {
    let expr = CoreExpr::LetMode {
        name: "value".to_string(),
        mode: CoreEvalMode::Strict,
        ty: CoreType::Mode {
            mode: CoreEvalMode::Strict,
            inner: Box::new(base("Int")),
            latent_row: None,
        },
        expr: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("value".to_string()))),
    };

    let (_, lowered) = lower_program(expr, CoreTypeCheckEnv::default());

    match lowered {
        Term::LetCont {
            name,
            param,
            cont_body,
            body,
        } => {
            assert_eq!(name, "__k0");
            assert_eq!(param, "value");
            match *cont_body {
                Term::Jump {
                    arg: cps::Atom::Var(_),
                    row: cont_row,
                    ..
                } => {
                    assert_eq!(cont_row.items, vec![]);
                }
                other => panic!("expected jump continuation body, got {other:?}"),
            }
            match *body {
                Term::Jump {
                    arg: cps::Atom::Int(1),
                    row: body_row,
                    ..
                } => {
                    assert!(body_row.items.is_empty());
                }
                other => panic!("expected jump initializer body, got {other:?}"),
            }
        }
        other => panic!("expected top-level LetCont for strict letmode, got {other:?}"),
    }
}

#[test]
fn force_lowers_to_force_primitive_and_carries_latent_row() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "maker".to_string(),
        typed_fn(vec![], base("Int"), cap_row(&["db"], "write")),
    );

    let expr = CoreExpr::LetMode {
        name: "thunk".to_string(),
        mode: CoreEvalMode::Memo,
        ty: CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(base("Int")),
            latent_row: Some(cap_row(&["db"], "write")),
        },
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

    let (typed, lowered) = lower_program(expr, env);

    assert_eq!(typed.row(), &cap_row(&["db"], "write"));
    assert_eq!(typed.ty(), &base("Int"));

    match lowered {
        Term::LetVal { value, body, .. } => {
            assert!(matches!(
                value,
                LoweredValue::ThunkClosure {
                    mode: ThunkMode::Memo,
                    ..
                }
            ));
            match *body {
                Term::LetPrim {
                    name,
                    op,
                    args,
                    body,
                    ..
                } => {
                    assert_eq!(name, "forced");
                    assert_eq!(op, PrimOp::ForceThunk);
                    assert_eq!(args, vec![cps::Atom::Var("thunk".to_string())]);
                    match *body {
                        Term::Jump {
                            arg: cps::Atom::Var(_),
                            row,
                            ..
                        } => assert!(row.items.is_empty()),
                        other => panic!("expected forced body jump, got {other:?}"),
                    }
                }
                other => panic!("expected force prim term, got {other:?}"),
            }
        }
        other => panic!("expected top-level LetVal for memo letmode, got {other:?}"),
    }
}
