use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam,
    CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".into())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".into())
}

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn cap_item(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: self::path(path),
        operation: operation.to_owned(),
    }
}

fn role_item(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Role {
        path: self::path(path),
    }
}

fn policy_item(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Policy {
        path: self::path(path),
    }
}

fn contract_item(contract: &str) -> CoreRowItem {
    CoreRowItem::Contract {
        contract: contract.to_owned(),
    }
}

fn resource_item(path: &[&str], mode: &str) -> CoreRowItem {
    CoreRowItem::Resource {
        path: self::path(path),
        mode: mode.to_owned(),
    }
}

fn evidence_item(path: &[&str]) -> CoreRowItem {
    CoreRowItem::Evidence {
        path: self::path(path),
    }
}

fn row(items: Vec<CoreRowItem>) -> CoreRow {
    CoreRow::closed(items)
}

fn kv_read_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: path(&["kv"]),
        operation: "read".into(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn audit_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: path(&["audit"]),
        operation: "emit".into(),
        arg_types: vec![string_ty()],
        result_type: unit_ty(),
    }
}

fn raise_audit() -> CoreExpr {
    CoreExpr::Raise {
        op: audit_op(),
        args: vec![CoreAtom::LitString("handled".into())],
    }
}

fn function_ty(params: Vec<CoreType>, result: CoreType, row: CoreRow) -> CoreType {
    CoreType::Function {
        params,
        result: Box::new(result),
        row,
    }
}

fn cont_ty(
    input: CoreType,
    answer: CoreType,
    row: CoreRow,
    multiplicity: CoreMultiplicity,
) -> CoreType {
    CoreType::Cont {
        input: Box::new(input),
        answer: Box::new(answer),
        row,
        multiplicity,
    }
}

fn param(name: &str, ty: CoreType) -> CoreParam {
    CoreParam {
        name: name.to_owned(),
        ty,
    }
}

fn resume_param(input: CoreType, row: CoreRow, multiplicity: CoreMultiplicity) -> CoreParam {
    param("resume", cont_ty(input, unit_ty(), row, multiplicity))
}

fn handler_clause(
    params: Vec<CoreParam>,
    resume: CoreParam,
    clause_body: CoreExpr,
    clause_row: CoreRow,
) -> CoreHandlerClause {
    CoreHandlerClause {
        op: kv_read_op(),
        params,
        resume,
        body: Box::new(clause_body),
        row: clause_row,
    }
}

fn resume_with(value: CoreAtom) -> CoreExpr {
    CoreExpr::Jump {
        cont: CoreContRef::Var("resume".into()),
        arg: value,
    }
}

fn raise_read() -> CoreExpr {
    CoreExpr::Raise {
        op: kv_read_op(),
        args: vec![CoreAtom::LitString("user:7".into())],
    }
}

fn handle_with(clause: CoreHandlerClause, body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        clause,
        body: Box::new(body),
    }
}

fn base_env() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(kv_read_op());
    env.operations_mut().insert(audit_op());
    env.values_mut().insert(
        "use_resume",
        function_ty(
            vec![cont_ty(
                string_ty(),
                unit_ty(),
                CoreRow::default(),
                CoreMultiplicity::Affine,
            )],
            unit_ty(),
            CoreRow::default(),
        ),
    );
    env.values_mut().insert(
        "needs_role",
        function_ty(
            vec![string_ty()],
            string_ty(),
            row(vec![role_item(&["ops"])]),
        ),
    );
    env.values_mut().insert(
        "needs_policy",
        function_ty(
            vec![string_ty()],
            string_ty(),
            row(vec![policy_item(&["tenant", "boundary"])]),
        ),
    );
    env.values_mut().insert(
        "needs_contract",
        function_ty(
            vec![string_ty()],
            string_ty(),
            row(vec![contract_item("nonempty-key")]),
        ),
    );
    env.values_mut().insert(
        "needs_resource",
        function_ty(
            vec![string_ty()],
            string_ty(),
            row(vec![resource_item(&["cache"], "read")]),
        ),
    );
    env.values_mut().insert(
        "needs_evidence",
        function_ty(
            vec![string_ty()],
            string_ty(),
            row(vec![evidence_item(&["proof", "tenant"])]),
        ),
    );
    env
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<ash_core::core_ash_typecheck::TypedCoreProgram, CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    type_check_core_program(valid, env)
}

fn assert_affine_validation_error(expr: CoreExpr, expected_detail: &str) {
    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("affine resume misuse should be rejected during validation");
    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. }),
        "unexpected validation error: {err:?}"
    );
    assert!(
        err.to_string().contains(expected_detail),
        "expected `{expected_detail}` in `{err}`"
    );
}

fn assert_type_mismatch(err: CoreTypeCheckError) {
    assert!(
        matches!(err, CoreTypeCheckError::TypeMismatch { .. }),
        "expected type mismatch, got {err:?}"
    );
}

#[test]
fn handle_accepts_handler_params_matching_operation_argument_types() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let typed = type_check(handle_with(clause, raise_read()), &base_env())
        .expect("matching handler parameter types should type-check");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn handle_rejects_handler_param_type_that_does_not_match_operation_arg_type() {
    let clause = handler_clause(
        vec![param("key", int_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        resume_with(CoreAtom::LitString("fallback".into())),
        CoreRow::default(),
    );

    let err = type_check(handle_with(clause, raise_read()), &base_env())
        .expect_err("handler parameter type must match operation argument type");

    assert_type_mismatch(err);
}

#[test]
fn handle_rejects_resume_input_that_does_not_match_operation_result_type() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(int_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        resume_with(CoreAtom::LitInt(7)),
        CoreRow::default(),
    );

    let err = type_check(handle_with(clause, raise_read()), &base_env())
        .expect_err("handler resume input must match operation result type");

    assert_type_mismatch(err);
}

#[test]
fn handle_rejects_clause_row_that_does_not_match_handler_body_local_row() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        raise_audit(),
        CoreRow::default(),
    );

    let err = type_check(handle_with(clause, raise_read()), &base_env())
        .expect_err("handler clause row must match handler body local row");

    assert!(matches!(err, CoreTypeCheckError::RowMismatch { .. }));
}

#[test]
fn handle_rejects_multishot_resume_before_typechecking() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::MultiShotPure,
        ),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    assert_affine_validation_error(
        handle_with(clause, raise_read()),
        "supports only affine handler resumes",
    );
}

#[test]
fn duplicate_resume_jump_is_rejected_by_affine_validation() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(resume_with(CoreAtom::LitString("cached".into()))),
            else_branch: Box::new(resume_with(CoreAtom::Var("key".into()))),
        },
        CoreRow::default(),
    );

    assert_affine_validation_error(
        handle_with(clause, raise_read()),
        "jumped to more than once",
    );
}

#[test]
fn passing_resume_as_ordinary_argument_is_rejected_by_affine_validation() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        CoreExpr::Call {
            func: CoreAtom::Var("use_resume".into()),
            args: vec![CoreAtom::Var("resume".into())],
        },
        CoreRow::default(),
    );

    assert_affine_validation_error(
        handle_with(clause, raise_read()),
        "passed as ordinary function argument",
    );
}

#[test]
fn storing_resume_in_record_is_rejected_by_affine_validation() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine),
        CoreExpr::LetVal {
            name: "saved".into(),
            ty: CoreType::Record(vec![(
                "resume".into(),
                resume_param(string_ty(), CoreRow::default(), CoreMultiplicity::Affine).ty,
            )]),
            value: CoreValue::Record {
                fields: vec![("resume".into(), CoreAtom::Var("resume".into()))],
            },
            body: Box::new(resume_with(CoreAtom::Var("key".into()))),
        },
        CoreRow::default(),
    );

    assert_affine_validation_error(handle_with(clause, raise_read()), "stored in record value");
}

#[test]
fn residual_local_row_preserves_resume_row() {
    let resume_row = row(vec![cap_item(&["audit"], "emit")]);
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), resume_row.clone(), CoreMultiplicity::Affine),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let typed = type_check(handle_with(clause, raise_read()), &base_env())
        .expect("resume row must be retained in Handle residual row");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &resume_row);
}

#[test]
fn handle_removes_operation_only_from_delimited_segment_and_preserves_ambient_rows() {
    let resume_row = row(vec![cap_item(&["kv"], "read")]);
    let clause_row = row(vec![cap_item(&["audit"], "emit")]);
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(string_ty(), resume_row, CoreMultiplicity::Affine),
        raise_audit(),
        clause_row,
    );
    let handled_body = CoreExpr::LetCall {
        name: "role_checked".into(),
        func: CoreAtom::Var("needs_role".into()),
        args: vec![CoreAtom::LitString("user:7".into())],
        body: Box::new(CoreExpr::LetCall {
            name: "policy_checked".into(),
            func: CoreAtom::Var("needs_policy".into()),
            args: vec![CoreAtom::Var("role_checked".into())],
            body: Box::new(CoreExpr::LetCall {
                name: "contract_checked".into(),
                func: CoreAtom::Var("needs_contract".into()),
                args: vec![CoreAtom::Var("policy_checked".into())],
                body: Box::new(CoreExpr::LetCall {
                    name: "resource_checked".into(),
                    func: CoreAtom::Var("needs_resource".into()),
                    args: vec![CoreAtom::Var("contract_checked".into())],
                    body: Box::new(CoreExpr::LetCall {
                        name: "evidence_checked".into(),
                        func: CoreAtom::Var("needs_evidence".into()),
                        args: vec![CoreAtom::Var("resource_checked".into())],
                        body: Box::new(raise_read()),
                    }),
                }),
            }),
        }),
    };
    let expected_row = row(vec![
        role_item(&["ops"]),
        policy_item(&["tenant", "boundary"]),
        contract_item("nonempty-key"),
        resource_item(&["cache"], "read"),
        evidence_item(&["proof", "tenant"]),
        cap_item(&["kv"], "read"),
        cap_item(&["audit"], "emit"),
    ]);

    let typed = type_check(handle_with(clause, handled_body), &base_env())
        .expect("Handle must preserve ambient rows and resume-row operation effects");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &expected_row);
}
