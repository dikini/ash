use std::path::{Path, PathBuf};

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr,
    CoreHandlerClause, CoreMultiplicity, CoreParam, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_lower::{CoreLoweringContext, CoreLoweringError};
use ash_core::core_ash_text::{CoreTextError, parse_core_file};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, TypedCoreProgram, type_check_and_lower_core_program,
    type_check_core_program,
};
use ash_core::core_ash_validate::{
    CoreValidationError, RawCoreProgram, ValidCoreProgram, validate_core_program,
};
use ash_core::cps::{ContRef, EffectItem, EffectItemKind, EffectRow, Term};

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

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".into())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".into())
}

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![cap(path, operation)])
}

fn chan_row(path: &[&str], mode: &str, payload: CoreType) -> CoreRow {
    CoreRow::closed(vec![CoreRowItem::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: Box::new(payload),
    }])
}

fn contract_row(contract: &str) -> CoreRow {
    CoreRow::closed(vec![CoreRowItem::Contract {
        contract: contract.to_owned(),
    }])
}

fn function_ty(params: Vec<CoreType>, result: CoreType, row: CoreRow) -> CoreType {
    CoreType::Function {
        params,
        result: Box::new(result),
        row,
    }
}

fn cont_ty(input: CoreType, answer: CoreType, row: CoreRow) -> CoreType {
    CoreType::Cont {
        input: Box::new(input),
        answer: Box::new(answer),
        row,
        multiplicity: CoreMultiplicity::Affine,
    }
}

fn console_read_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["console".into()],
        operation: "read".into(),
        arg_types: vec![string_ty()],
        result_type: unit_ty(),
    }
}

fn audit_emit_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["audit".into()],
        operation: "emit".into(),
        arg_types: vec![string_ty()],
        result_type: unit_ty(),
    }
}

fn handle_resume_param(input: CoreType, answer: CoreType, row: CoreRow) -> CoreParam {
    CoreParam {
        name: "resume".into(),
        ty: CoreType::Cont {
            input: Box::new(input),
            answer: Box::new(answer),
            row,
            multiplicity: CoreMultiplicity::Affine,
        },
    }
}

#[derive(Debug)]
enum PipelineError {
    Parse(CoreTextError),
    Validate(CoreValidationError),
    TypeCheck(CoreTypeCheckError),
    Lower(CoreLoweringError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "parse failed: {error}"),
            Self::Validate(error) => write!(f, "validation failed: {error}"),
            Self::TypeCheck(error) => write!(f, "type checking failed: {error}"),
            Self::Lower(error) => write!(f, "lowering failed: {error}"),
        }
    }
}

fn parse_validate_and_typecheck_fixture(
    name: &str,
    env: &CoreTypeCheckEnv,
) -> Result<(ValidCoreProgram, TypedCoreProgram), PipelineError> {
    let core = parse_core_file(fixture_path(name)).map_err(PipelineError::Parse)?;
    let valid =
        validate_core_program(RawCoreProgram::new(core)).map_err(PipelineError::Validate)?;
    let typed = type_check_core_program(valid.clone(), env).map_err(PipelineError::TypeCheck)?;
    Ok((valid, typed))
}

fn check_then_lower_fixture(
    name: &str,
    env: &CoreTypeCheckEnv,
    context: CoreLoweringContext,
) -> Result<(TypedCoreProgram, Term), PipelineError> {
    let (valid, _typed) = parse_validate_and_typecheck_fixture(name, env)?;
    let checked =
        type_check_and_lower_core_program(valid, env, context).map_err(|error| match error {
            ash_core::core_ash_typecheck::CoreCheckedLoweringError::TypeCheck(error) => {
                PipelineError::TypeCheck(error)
            }
            ash_core::core_ash_typecheck::CoreCheckedLoweringError::Lower(error) => {
                PipelineError::Lower(error)
            }
        })?;
    let (typed, lowered) = checked.into_parts();
    Ok((typed, lowered))
}

fn expect_typecheck_error(name: &str, env: &CoreTypeCheckEnv) -> CoreTypeCheckError {
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());
    match check_then_lower_fixture(name, env, context) {
        Err(PipelineError::TypeCheck(error)) => error,
        Err(other) => panic!("expected {name} to fail during type checking, got {other}"),
        Ok(_) => panic!("expected {name} to fail before lowering"),
    }
}

fn first_jump_row(term: &Term) -> Option<&EffectRow> {
    match term {
        Term::LetVal { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. } => first_jump_row(body),
        Term::LetPrim { body, .. } => first_jump_row(body),
        Term::LetCont {
            cont_body, body, ..
        } => first_jump_row(cont_body).or_else(|| first_jump_row(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_jump_row(then_branch).or_else(|| first_jump_row(else_branch)),
        Term::Handle { body, .. } => first_jump_row(body),
        Term::Jump { row, .. } => Some(row),
        Term::Call { .. }
        | Term::Raise { .. }
        | Term::Match { .. }
        | Term::Return { .. }
        | Term::LetContCall { .. }
        | Term::Trap { .. } => None,
    }
}

fn first_call_row(term: &Term) -> Option<&EffectRow> {
    match term {
        Term::LetVal { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. } => first_call_row(body),
        Term::LetPrim { body, .. } => first_call_row(body),
        Term::LetCont {
            cont_body, body, ..
        } => first_call_row(cont_body).or_else(|| first_call_row(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_call_row(then_branch).or_else(|| first_call_row(else_branch)),
        Term::Handle { body, .. } => first_call_row(body),
        Term::Call { row, .. } => Some(row),
        Term::Jump { .. }
        | Term::Raise { .. }
        | Term::Match { .. }
        | Term::Return { .. }
        | Term::LetContCall { .. }
        | Term::Trap { .. } => None,
    }
}

fn first_handle_row(term: &Term) -> Option<&EffectRow> {
    match term {
        Term::LetVal { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. } => first_handle_row(body),
        Term::LetPrim { body, .. } => first_handle_row(body),
        Term::LetCont {
            cont_body, body, ..
        } => first_handle_row(cont_body).or_else(|| first_handle_row(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_handle_row(then_branch).or_else(|| first_handle_row(else_branch)),
        Term::Handle { row, .. } => Some(row),
        Term::Call { .. }
        | Term::Jump { .. }
        | Term::Raise { .. }
        | Term::Match { .. }
        | Term::Return { .. }
        | Term::LetContCall { .. }
        | Term::Trap { .. } => None,
    }
}

fn let_cont_body_call_row(term: &Term) -> Option<&EffectRow> {
    let Term::LetCont { body, .. } = term else {
        return None;
    };
    let Term::Call { row, .. } = body.as_ref() else {
        return None;
    };
    Some(row)
}

#[test]
fn valid_jump_fixture_typechecks_before_lowering_and_preserves_target_continuation_row() {
    let exit_row = cap_row(&["console"], "write");
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut()
        .insert("exit", cont_ty(int_ty(), unit_ty(), exit_row.clone()));
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default())
        .with_cont_row("exit", exit_row.clone());

    let (typed, _lowered) = check_then_lower_fixture("let_val_jump.core", &env, context)
        .expect("fixture checks and lowers");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed
            .facts()
            .jump_continuation_rows()
            .get(&CoreContRef::Label("exit".into())),
        Some(&exit_row),
        "type checking must preserve the target continuation row before lowering"
    );
}

#[test]
fn typed_jump_continuation_row_facts_are_handed_to_lowering() {
    let exit_row = cap_row(&["console"], "write");
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut()
        .insert("exit", cont_ty(int_ty(), unit_ty(), exit_row.clone()));

    let (_valid, typed) = parse_validate_and_typecheck_fixture("let_val_jump.core", &env)
        .expect("fixture should type-check before lowering");
    assert_eq!(
        typed
            .facts()
            .jump_continuation_rows()
            .get(&CoreContRef::Label("exit".into())),
        Some(&exit_row)
    );

    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());
    let checked = type_check_and_lower_core_program(
        parse_validate_and_typecheck_fixture("let_val_jump.core", &env)
            .expect("fixture should parse, validate, and type-check")
            .0,
        &env,
        context,
    )
    .expect("checked lowering should consume typed facts");
    let lowered = checked.lowered();
    let jump_row = first_jump_row(lowered).expect("fixture lowers to a jump");

    assert_eq!(
        jump_row,
        &EffectRow {
            items: vec![EffectItem {
                namespace: "cap".into(),
                name: "console.write".into(),
                kind: EffectItemKind::Capability,
            }],
        },
        "lowering must consume the checked continuation-row fact instead of recomputing an empty row"
    );
}

#[test]
fn valid_call_fixture_typechecks_before_lowering_and_preserves_function_local_row() {
    let read_row = cap_row(&["db"], "read");
    let exit_row = cap_row(&["console"], "write");
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "read_user",
        function_ty(vec![int_ty()], string_ty(), read_row.clone()),
    );
    env.continuations_mut()
        .insert("exit", cont_ty(string_ty(), unit_ty(), exit_row.clone()));
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default())
        .with_function_row("read_user", read_row.clone())
        .with_cont_row("exit", exit_row.clone());

    let (typed, _lowered) = check_then_lower_fixture("let_call.core", &env, context)
        .expect("fixture checks and lowers");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(
        typed.row(),
        &read_row,
        "LetCall must report the callee local row before lowering"
    );
    assert_eq!(
        typed
            .facts()
            .jump_continuation_rows()
            .get(&CoreContRef::Label("exit".into())),
        Some(&exit_row)
    );
}

#[test]
fn checked_lowering_uses_typechecked_external_function_rows() {
    let read_row = cap_row(&["db"], "read");
    let exit_row = cap_row(&["console"], "write");
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "read_user",
        function_ty(vec![int_ty()], string_ty(), read_row.clone()),
    );
    env.continuations_mut()
        .insert("exit", cont_ty(string_ty(), unit_ty(), exit_row));
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(
        parse_validate_and_typecheck_fixture("let_call.core", &env)
            .expect("fixture should parse, validate, and type-check")
            .0,
        &env,
        context,
    )
    .expect("checked lowering should consume checked function rows");
    let call_row = first_call_row(checked.lowered()).expect("fixture lowers to a call");

    assert!(
        call_row.items.iter().any(|item| item.namespace == "cap"
            && item.name == "db.read"
            && item.kind == EffectItemKind::Capability),
        "checked lowering must use the typechecked external function row"
    );
    assert!(
        call_row.items.iter().any(|item| item.namespace == "cap"
            && item.name == "console.write"
            && item.kind == EffectItemKind::Capability),
        "checked lowering must also preserve the checked continuation row"
    );
}

#[test]
fn checked_lowering_uses_local_function_row_from_letcall_binding() {
    let reader_row = cap_row(&["db"], "read");
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "make_reader",
        function_ty(
            Vec::new(),
            function_ty(Vec::new(), unit_ty(), reader_row.clone()),
            CoreRow::default(),
        ),
    );

    let program = validate_core_program(RawCoreProgram::new(CoreExpr::LetCall {
        name: "reader".into(),
        func: CoreAtom::Var("make_reader".into()),
        args: Vec::new(),
        body: Box::new(CoreExpr::Call {
            func: CoreAtom::Var("reader".into()),
            args: Vec::new(),
        }),
    }))
    .expect("program should validate");

    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());
    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should use local function row discovered for let-call binding");

    assert_eq!(
        checked.typed().row(),
        &reader_row,
        "LetCall should preserve callee result row from local function binding"
    );
    let continuation_call_row =
        let_cont_body_call_row(checked.lowered()).expect("LetCall lowers to LetCont with a Call");
    assert!(
        continuation_call_row
            .items
            .iter()
            .any(|item| item.namespace == "cap"
                && item.name == "db.read"
                && item.kind == EffectItemKind::Capability),
        "continuation call should carry function row from the let-call result binding"
    );
}

#[test]
fn checked_lowering_handles_sibling_branch_local_letcall_bindings_with_same_name() {
    let db_reader_row = cap_row(&["db"], "read");
    let console_reader_row = cap_row(&["console"], "write");

    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "make_db_reader",
        function_ty(
            Vec::new(),
            function_ty(Vec::new(), unit_ty(), db_reader_row.clone()),
            CoreRow::default(),
        ),
    );
    env.values_mut().insert(
        "make_console_reader",
        function_ty(
            Vec::new(),
            function_ty(Vec::new(), unit_ty(), console_reader_row.clone()),
            CoreRow::default(),
        ),
    );

    let program = validate_core_program(RawCoreProgram::new(CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(CoreExpr::LetCall {
            name: "reader".into(),
            func: CoreAtom::Var("make_db_reader".into()),
            args: Vec::new(),
            body: Box::new(CoreExpr::Call {
                func: CoreAtom::Var("reader".into()),
                args: Vec::new(),
            }),
        }),
        else_branch: Box::new(CoreExpr::LetCall {
            name: "reader".into(),
            func: CoreAtom::Var("make_console_reader".into()),
            args: Vec::new(),
            body: Box::new(CoreExpr::Call {
                func: CoreAtom::Var("reader".into()),
                args: Vec::new(),
            }),
        }),
    }))
    .expect("program should validate");

    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());
    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve branch-local LetCall function rows");
    let db_capability = CoreRowItem::Capability {
        path: vec!["db".to_owned()],
        operation: "read".to_owned(),
    };
    let console_capability = CoreRowItem::Capability {
        path: vec!["console".to_owned()],
        operation: "write".to_owned(),
    };

    assert!(
        checked
            .typed()
            .row()
            .items
            .iter()
            .all(|item| matches!(item, CoreRowItem::Capability { .. }))
    );
    assert!(
        checked
            .typed()
            .row()
            .items
            .iter()
            .any(|item| item == &db_capability),
        "typed program row should include db.read from then branch"
    );
    assert!(
        checked
            .typed()
            .row()
            .items
            .iter()
            .any(|item| item == &console_capability),
        "typed program row should include console.write from else branch"
    );

    let lowered = checked.lowered();
    let Term::If {
        then_branch,
        else_branch,
        ..
    } = lowered
    else {
        panic!("checked lowering of top-level if should produce an if-term");
    };

    let then_call_row = let_cont_body_call_row(then_branch)
        .expect("then branch should lower reader call through a let continuation");
    let else_call_row = let_cont_body_call_row(else_branch)
        .expect("else branch should lower reader call through a let continuation");

    assert!(
        then_call_row
            .items
            .iter()
            .any(|item| item.namespace == "cap"
                && item.name == "db.read"
                && item.kind == EffectItemKind::Capability),
        "then-branch continuation call should use db reader effect"
    );
    assert!(
        !then_call_row
            .items
            .iter()
            .any(|item| item.namespace == "cap"
                && item.name == "console.write"
                && item.kind == EffectItemKind::Capability),
        "then-branch continuation call should not capture else branch console effect"
    );

    assert!(
        else_call_row
            .items
            .iter()
            .any(|item| item.namespace == "cap"
                && item.name == "console.write"
                && item.kind == EffectItemKind::Capability),
        "else-branch continuation call should use console reader effect"
    );
    assert!(
        !else_call_row
            .items
            .iter()
            .any(|item| item.namespace == "cap"
                && item.name == "db.read"
                && item.kind == EffectItemKind::Capability),
        "else-branch continuation call should not capture then branch db effect"
    );
}

#[test]
fn checked_lowering_uses_typechecked_handle_residual_row() {
    let resume_row = cap_row(&["audit"], "emit");
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(console_read_op());
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: console_read_op(),
            params: vec![CoreParam {
                name: "prompt".into(),
                ty: string_ty(),
            }],
            resume: handle_resume_param(unit_ty(), unit_ty(), resume_row.clone()),
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Var("resume".into()),
                arg: CoreAtom::LitUnit,
            }),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Raise {
            op: console_read_op(),
            args: vec![CoreAtom::LitString("ready".into())],
        }),
    }))
    .expect("Core handle expression validates");
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve checked handle rows");

    assert_eq!(checked.typed().row(), &resume_row);
    let handle_row = first_handle_row(checked.lowered()).expect("expression lowers to a handle");
    assert!(
        handle_row.items.iter().any(|item| item.namespace == "cap"
            && item.name == "audit.emit"
            && item.kind == EffectItemKind::Capability),
        "lowered Handle.row must use the checked residual row, not the empty clause row"
    );
}

#[test]
fn checked_lowering_uses_local_function_row_from_handle_body() {
    let local_row = cap_row(&["db"], "read");
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(console_read_op());
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: console_read_op(),
            params: vec![CoreParam {
                name: "prompt".into(),
                ty: string_ty(),
            }],
            resume: handle_resume_param(unit_ty(), unit_ty(), CoreRow::default()),
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Var("resume".into()),
                arg: CoreAtom::LitUnit,
            }),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::LetVal {
            name: "read_row".into(),
            ty: function_ty(Vec::new(), unit_ty(), local_row.clone()),
            value: CoreValue::Lam {
                params: Vec::new(),
                body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
                row: local_row.clone(),
            },
            body: Box::new(CoreExpr::LetCall {
                name: "value".into(),
                func: CoreAtom::Var("read_row".into()),
                args: Vec::new(),
                body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            }),
        }),
    }))
    .expect("handle with local function body should validate");
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve local function row from handle body");
    let handle_row = first_handle_row(checked.lowered()).expect("expression lowers to a handle");

    assert!(
        handle_row.items.iter().any(|item| item.namespace == "cap"
            && item.name == "db.read"
            && item.kind == EffectItemKind::Capability),
        "Handle.row should include the local function's latent row"
    );
}

#[test]
fn checked_lowering_uses_structural_handle_residual_subtraction_for_row_type_equivalence() {
    let body_payload = CoreType::Record(vec![("a".into(), int_ty()), ("b".into(), string_ty())]);
    let op_payload = CoreType::Record(vec![("b".into(), string_ty()), ("a".into(), int_ty())]);
    let local_row = chan_row(&["jobs"], "send", body_payload.clone());
    let local_function_ty = function_ty(Vec::new(), unit_ty(), local_row.clone());
    let clause_op = CoreEffectOp::Channel {
        path: vec!["jobs".into()],
        mode: "send".into(),
        payload_type: op_payload.clone(),
        result_type: unit_ty(),
    };
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(clause_op.clone());

    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: clause_op,
            params: vec![CoreParam {
                name: "payload".into(),
                ty: op_payload,
            }],
            resume: handle_resume_param(unit_ty(), unit_ty(), CoreRow::default()),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::LetRec {
            name: "reader".into(),
            ty: local_function_ty,
            value: CoreValue::Lam {
                params: Vec::new(),
                body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
                row: local_row.clone(),
            },
            body: Box::new(CoreExpr::LetCall {
                name: "ignored".into(),
                func: CoreAtom::Var("reader".into()),
                args: Vec::new(),
                body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            }),
        }),
    }))
    .expect("handle with typed local function should validate");

    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve structural residual subtraction");

    let handle_row = first_handle_row(checked.lowered()).expect("expression lowers to a handle");
    assert!(
        handle_row.items.is_empty(),
        "lowered handle row should drop handled channel when row item is structurally equal"
    );
}

#[test]
fn checked_lowering_preserves_discharged_rows_inside_handle_body() {
    let contract = "requires-clean";
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(console_read_op());
    env.values_mut().insert(
        "contract_checked_unit",
        function_ty(Vec::new(), unit_ty(), contract_row(contract)),
    );
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: console_read_op(),
            params: vec![CoreParam {
                name: "prompt".into(),
                ty: string_ty(),
            }],
            resume: handle_resume_param(unit_ty(), unit_ty(), CoreRow::default()),
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Var("resume".into()),
                arg: CoreAtom::LitUnit,
            }),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::RecordDischarge {
            discharge: CoreContractDischarge {
                contract: contract.into(),
                mode: CoreDischargeMode::Dynamic,
                evidence: None,
                source_span: None,
            },
            body: Box::new(CoreExpr::Call {
                func: CoreAtom::Var("contract_checked_unit".into()),
                args: Vec::new(),
            }),
        }),
    }))
    .expect("Core handle expression validates");
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve discharged rows inside handles");

    assert_eq!(checked.typed().row(), &CoreRow::default());
    let handle_row = first_handle_row(checked.lowered()).expect("expression lowers to a handle");
    assert!(
        handle_row
            .items
            .iter()
            .all(|item| item.namespace != "contract" || item.name != contract),
        "lowered Handle.row must not reintroduce a discharged contract requirement"
    );
}

#[test]
fn checked_lowering_preserves_discharged_rows_in_letcall_continuation_rows() {
    let contract = "requires-clean";
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "start",
        function_ty(Vec::new(), unit_ty(), CoreRow::default()),
    );
    env.values_mut().insert(
        "contract_checked_unit",
        function_ty(Vec::new(), unit_ty(), contract_row(contract)),
    );
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::LetCall {
        name: "ignored".into(),
        func: CoreAtom::Var("start".into()),
        args: Vec::new(),
        body: Box::new(CoreExpr::RecordDischarge {
            discharge: CoreContractDischarge {
                contract: contract.into(),
                mode: CoreDischargeMode::Dynamic,
                evidence: None,
                source_span: None,
            },
            body: Box::new(CoreExpr::Call {
                func: CoreAtom::Var("contract_checked_unit".into()),
                args: Vec::new(),
            }),
        }),
    }))
    .expect("Core let-call expression validates");
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let checked = type_check_and_lower_core_program(program, &env, context)
        .expect("checked lowering should preserve discharged rows in continuation rows");

    assert_eq!(checked.typed().row(), &CoreRow::default());
    let outer_call_row =
        let_cont_body_call_row(checked.lowered()).expect("LetCall lowers to LetCont with a Call");
    assert!(
        outer_call_row
            .items
            .iter()
            .all(|item| item.namespace != "contract" || item.name != contract),
        "lowered LetCall call row must not reintroduce a discharged continuation-body contract"
    );
}

#[test]
fn valid_handle_fixture_typechecks_before_lowering_and_preserves_resume_row_fact() {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(console_read_op());
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());

    let (typed, _lowered) = check_then_lower_fixture("raise_handle.core", &env, context)
        .expect("fixture checks and lowers");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    assert_eq!(
        typed
            .facts()
            .jump_continuation_rows()
            .get(&CoreContRef::Var("k".into())),
        Some(&CoreRow::default()),
        "handler resume continuation rows must be available to lowering"
    );
}

#[test]
fn invalid_type_mismatch_fixture_fails_typechecking_before_lowering() {
    let error = expect_typecheck_error("invalid_type_mismatch.core", &CoreTypeCheckEnv::default());

    assert!(
        matches!(error, CoreTypeCheckError::TypeMismatch { .. }),
        "expected type mismatch, got {error:?}"
    );
}

#[test]
fn invalid_row_mismatch_fixture_fails_typechecking_before_lowering() {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(audit_emit_op());

    let error = expect_typecheck_error("invalid_row_mismatch.core", &env);

    assert!(
        matches!(error, CoreTypeCheckError::RowMismatch { .. }),
        "expected row mismatch, got {error:?}"
    );
}

#[test]
fn invalid_operation_arity_fixture_fails_typechecking_before_lowering() {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(console_read_op());

    let error = expect_typecheck_error("invalid_operation_arity_mismatch.core", &env);

    assert_eq!(
        error,
        CoreTypeCheckError::ArgumentCountMismatch {
            expected: 1,
            actual: 0,
        }
    );
}

#[test]
fn invalid_affine_resume_fixture_fails_validation_before_typechecking_or_lowering() {
    let context = CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default());
    let error = match check_then_lower_fixture(
        "invalid_affine_resume_misuse.core",
        &CoreTypeCheckEnv::default(),
        context,
    ) {
        Err(PipelineError::Validate(error)) => error,
        Err(other) => panic!("expected affine misuse to fail during validation, got {other}"),
        Ok(_) => panic!("expected affine misuse to fail before lowering"),
    };

    assert!(
        matches!(error, CoreValidationError::AffineResumeViolation { .. }),
        "expected affine resume validation error, got {error:?}"
    );
}
