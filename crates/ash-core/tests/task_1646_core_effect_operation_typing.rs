use ash_core::core_ash::{
    CoreAtom, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr, CoreRow,
    CoreRowItem, CoreTrapReason, CoreType,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".into())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".into())
}

fn positive_int_ty() -> CoreType {
    CoreType::Refinement {
        base: Box::new(int_ty()),
        predicate: "result > 0".into(),
    }
}

fn never_ty() -> CoreType {
    CoreType::Named("Never".into())
}

fn named_ty(name: &str) -> CoreType {
    CoreType::Named(name.into())
}

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn operation_item(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Operation {
        path: self::path(path),
        operation: operation.to_owned(),
    }
}

fn channel_item(path: &[&str], mode: &str, payload_type: CoreType) -> CoreRowItem {
    CoreRowItem::Channel {
        path: self::path(path),
        mode: mode.to_owned(),
        payload_type: Box::new(payload_type),
    }
}

fn process_item(operation: &str) -> CoreRowItem {
    CoreRowItem::Process {
        operation: operation.to_owned(),
    }
}

fn failure_item(ty: CoreType) -> CoreRowItem {
    CoreRowItem::Failure {
        ty: Some(Box::new(ty)),
    }
}

fn operation_read_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: path(&["kv"]),
        operation: "read".into(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn channel_send_op() -> CoreEffectOp {
    CoreEffectOp::Channel {
        path: path(&["jobs"]),
        mode: "send".into(),
        payload_type: named_ty("Job"),
        result_type: unit_ty(),
    }
}

fn channel_send_record_op(payload_type: CoreType) -> CoreEffectOp {
    CoreEffectOp::Channel {
        path: path(&["jobs"]),
        mode: "send".into(),
        payload_type,
        result_type: unit_ty(),
    }
}

fn process_spawn_op() -> CoreEffectOp {
    CoreEffectOp::Process {
        operation: "spawn".into(),
        arg_types: vec![string_ty()],
        result_type: named_ty("ProcessHandle"),
    }
}

fn typed_failure_op() -> CoreEffectOp {
    CoreEffectOp::Failure {
        ty: Some(named_ty("ConfigError")),
    }
}

fn env_with_op(op: CoreEffectOp) -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.types_mut().insert_name("ConfigError");
    env.types_mut().insert_name("Job");
    env.types_mut().insert_name("Never");
    env.types_mut().insert_name("ProcessHandle");
    env.values_mut().insert("err", named_ty("ConfigError"));
    env.values_mut().insert("job", named_ty("Job"));
    env.operations_mut().insert(op);
    env
}

fn cap_int_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: path(&["counter"]),
        operation: "set".into(),
        arg_types: vec![int_ty()],
        result_type: unit_ty(),
    }
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<ash_core::core_ash_typecheck::TypedCoreProgram, CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    type_check_core_program(valid, env)
}

#[test]
fn operation_raise_checks_signature_and_reports_operation_only_local_row() {
    let op = operation_read_op();
    let env = env_with_op(op.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::LitString("user:7".into())],
        },
        &env,
    )
    .expect("known capability raise with matching argument type should type-check");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![operation_item(&["kv"], "read")])
    );
}

#[test]
fn operation_raise_rejects_unknown_operation_identity() {
    let known = operation_read_op();
    let mut unknown = known.clone();
    if let CoreEffectOp::Operation { operation, .. } = &mut unknown {
        *operation = "write".into();
    }
    let env = env_with_op(known);

    let err = type_check(
        CoreExpr::Raise {
            op: unknown,
            args: vec![CoreAtom::LitString("user:7".into())],
        },
        &env,
    )
    .expect_err("capability operation path/name/signature must be known");

    assert!(matches!(err, CoreTypeCheckError::UnknownOperation { .. }));
}

#[test]
fn operation_raise_arity_mismatch_fails() {
    let op = operation_read_op();
    let env = env_with_op(op.clone());

    let err = type_check(
        CoreExpr::Raise {
            op,
            args: Vec::new(),
        },
        &env,
    )
    .expect_err("capability raise must reject missing arguments");

    assert_eq!(
        err,
        CoreTypeCheckError::ArgumentCountMismatch {
            expected: 1,
            actual: 0
        }
    );
}

#[test]
fn operation_raise_argument_type_mismatch_fails() {
    let op = operation_read_op();
    let env = env_with_op(op.clone());

    let err = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::LitInt(7)],
        },
        &env,
    )
    .expect_err("capability raise must reject argument type mismatch");

    assert!(matches!(err, CoreTypeCheckError::TypeMismatch { .. }));
}

#[test]
fn operation_raise_accepts_refinement_argument_where_base_is_expected() {
    let op = cap_int_op();
    let mut env = env_with_op(op.clone());
    env.discharges_mut()
        .insert_refinement_predicate("result > 0");
    env.values_mut().insert("positive", positive_int_ty());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::Var("positive".into())],
        },
        &env,
    )
    .expect("operation argument checks should allow refinement-to-base compatibility");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![operation_item(&["counter"], "set")])
    );
}

#[test]
fn failure_raise_with_payload_type_checks() {
    let op = typed_failure_op();
    let env = env_with_op(op.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::Var("err".into())],
        },
        &env,
    )
    .expect("typed failure raise should check its payload");

    assert_eq!(typed.ty(), &never_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![failure_item(named_ty("ConfigError"))])
    );
}

#[test]
fn failure_raise_without_payload_checks_as_zero_arg_never() {
    let op = CoreEffectOp::Failure { ty: None };
    let env = env_with_op(op.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: Vec::new(),
        },
        &env,
    )
    .expect("untyped failure raise should be a zero-argument fail operation");

    assert_eq!(typed.ty(), &never_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![CoreRowItem::Failure { ty: None }])
    );
}

#[test]
fn failure_raise_payload_type_mismatch_fails() {
    let op = typed_failure_op();
    let env = env_with_op(op.clone());

    let err = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::LitString("wrong payload".into())],
        },
        &env,
    )
    .expect_err("typed failure raise must reject mismatched payload");

    assert!(matches!(err, CoreTypeCheckError::TypeMismatch { .. }));
}

#[test]
fn channel_raise_checks_payload_signature_and_operation_row() {
    let op = channel_send_op();
    let env = env_with_op(op.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::Var("job".into())],
        },
        &env,
    )
    .expect("channel send raise should type-check its payload");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![channel_item(&["jobs"], "send", named_ty("Job"))])
    );
}

#[test]
fn channel_raise_accepts_structurally_equivalent_payload_record_type() {
    let registered_payload =
        CoreType::Record(vec![("b".into(), string_ty()), ("a".into(), int_ty())]);
    let requested_payload =
        CoreType::Record(vec![("a".into(), int_ty()), ("b".into(), string_ty())]);

    let op = channel_send_record_op(registered_payload.clone());
    let mut env = env_with_op(op.clone());
    env.values_mut()
        .insert("job_payload", requested_payload.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op: channel_send_record_op(requested_payload),
            args: vec![CoreAtom::Var("job_payload".into())],
        },
        &env,
    )
    .expect("channel raise should type-check using structural channel payload comparison");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed.row(),
        &CoreRow::closed(vec![channel_item(
            &["jobs"],
            "send",
            registered_payload.clone()
        )])
    );
}

#[test]
fn process_raise_checks_argument_signature_and_operation_row() {
    let op = process_spawn_op();
    let env = env_with_op(op.clone());

    let typed = type_check(
        CoreExpr::Raise {
            op,
            args: vec![CoreAtom::LitString("worker".into())],
        },
        &env,
    )
    .expect("process operation raise should type-check its arguments");

    assert_eq!(typed.ty(), &named_ty("ProcessHandle"));
    assert_eq!(typed.row(), &CoreRow::closed(vec![process_item("spawn")]));
}

#[test]
fn contract_violation_remains_trap_metadata_not_a_raised_operation() {
    let discharge = CoreContractDischarge {
        contract: "requires-positive".into(),
        mode: CoreDischargeMode::Dynamic,
        evidence: None,
        source_span: None,
    };

    let typed = type_check(
        CoreExpr::If {
            cond: CoreAtom::LitBool(false),
            then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
            else_branch: Box::new(CoreExpr::Trap {
                reason: CoreTrapReason::ContractViolation(discharge.contract.clone()),
            }),
        },
        &CoreTypeCheckEnv::default(),
    )
    .expect("contract violation should type-check only as trap metadata");

    assert_eq!(typed.ty(), &int_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    assert!(!matches!(
        CoreRow::closed(vec![CoreRowItem::Contract {
            contract: discharge.contract
        }])
        .items
        .as_slice(),
        [CoreRowItem::Failure { .. }]
    ));
}
