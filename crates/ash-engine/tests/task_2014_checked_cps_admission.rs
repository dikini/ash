//! TASK-2014 RED contract for the sealed checked Core/CPS admission artifact.
//!
//! The artifact is an engine-owned in-memory validation boundary.  Its inputs
//! are a checked/lowered Core program and source facts produced by the
//! typechecker; effect rows are evidence only and never authorize a frame.

use ash_core::{
    core_ash::{
        CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity,
        CoreParam, CoreRow, CoreRowItem, CoreType,
    },
    core_ash_lower::CoreLoweringContext,
    core_ash_typecheck::{
        CheckedLoweredCoreProgram, CoreTypeCheckEnv, type_check_and_lower_core_program,
    },
    core_ash_validate::{RawCoreProgram, validate_core_program},
    cps::ContRef,
    semantic_summary::{SourceAnchor, SourceOrigin},
};
use ash_engine::checked_cps_admission::{
    CheckedCpsAdmissionError, CheckedCpsAdmissionV1, CheckedSourceFactsV1, CoreHandleLocatorV1,
    FrameInstallationInstructionV1, OperationIdentityV1, ProviderBindingV1,
};
use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{DeclaredConcreteOperation, TypeCheckResult, TypeEnv, type_check_program};

const HANDLER_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler absorb_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(milliseconds, resume) => resume(milliseconds),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with absorb_sleep }
";

fn parse_program(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2014 source fixture should parse: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must define fn main");
    Program {
        definitions: module.definitions,
        entry,
    }
}

fn checked_source() -> TypeCheckResult {
    type_check_program(&parse_program(HANDLER_SOURCE))
        .expect("fixture must produce checked source-handler facts")
}

const RESIDUAL_HANDLER_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
interface Audit<T> { record(Int) -> Int }
type TestClock = SystemClock(Int);
type TestAudit = SystemAudit(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
impl Audit<TestAudit> { record(value) = value }
handler absorb_sleep(comp: () -> { TestClock::sleep, TestAudit::record } Int) -> Int {
    on comp {
        TestClock::sleep(milliseconds, resume) => resume(milliseconds),
        done(value) => value,
    }
}
fn main() -> Int {
    handle { TestClock::sleep(0); TestAudit::record(0) } with absorb_sleep
}
";

const OPEN_TAIL_DERIVED_HANDLER_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    derive handler clock;
}
effect alias OpenClock = { TestClock::sleep | rest };
fn main(computation: () -> { OpenClock } Int) -> Int {
    handle computation() with clock
}
";

fn resolve_declared_operation(
    program: &Program,
    impl_type: &str,
    operation: &str,
) -> DeclaredConcreteOperation {
    let mut environment = TypeEnv::with_builtin_types();
    environment.set_current_module_identity(ash_typeck::standalone_program_module_identity());
    environment
        .register_surface_declarations(&program.definitions)
        .expect("fixture declarations register");
    for definition in &program.definitions {
        if let Definition::Interface(interface) = definition {
            environment
                .register_interface(interface)
                .expect("fixture interface registers");
        }
    }
    for definition in &program.definitions {
        if let Definition::Type(ty) = definition
            && !environment.has_type(ty.name.as_ref())
        {
            environment
                .register_type(&ash_parser::lower_surface_type_def(ty))
                .expect("fixture type registers");
        }
    }
    for definition in &program.definitions {
        if let Definition::Impl(implementation) = definition {
            environment
                .register_impl(implementation)
                .expect("fixture impl registers");
        }
    }
    environment
        .resolve_declared_concrete_operation(impl_type, operation)
        .expect("fixture concrete operation resolves")
}

fn checked_source_with_unhandled_residual() -> (TypeCheckResult, DeclaredConcreteOperation) {
    let program = parse_program(RESIDUAL_HANDLER_SOURCE);
    let audit_operation = resolve_declared_operation(&program, "TestAudit", "record");
    let checked = type_check_program(&program)
        .expect("fixture must retain a checked source-handler residual fact");
    (checked, audit_operation)
}

fn checked_operation(checked: &TypeCheckResult) -> DeclaredConcreteOperation {
    let clauses = &checked
        .checked_handlers
        .get("absorb_sleep")
        .expect("checked source must retain its handler declaration")
        .clauses;
    let [clause] = clauses.as_slice() else {
        panic!("fixture has exactly one concrete handler clause, found {clauses:?}");
    };
    clause.operation.clone()
}

fn operation_identity(checked: &TypeCheckResult) -> OperationIdentityV1 {
    OperationIdentityV1::from_declared(&checked_operation(checked))
}

fn source_anchor() -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-2014 checked handler fixture".to_string(),
        },
        None,
        "handler absorb_sleep",
    )
}

fn checked_source_facts(checked: &TypeCheckResult) -> CheckedSourceFactsV1 {
    CheckedSourceFactsV1::from_type_check(checked, "absorb_sleep", source_anchor())
        .expect("the selected checked handler/application facts must be admissible source input")
}

/// The source typechecker currently retains normalized residual row entries but
/// does not retain the resolved signature for every unhandled entry.  This
/// narrow admission projection supplies the resolver-produced concrete facts
/// and must verify them against the checked residual row before admission.
fn checked_source_facts_with_residual_operations(
    checked: &TypeCheckResult,
    residual_operations: Vec<DeclaredConcreteOperation>,
) -> CheckedSourceFactsV1 {
    CheckedSourceFactsV1::from_type_check_with_residual_operations(
        checked,
        "absorb_sleep",
        source_anchor(),
        residual_operations,
    )
    .expect("resolver-produced residual facts must match the checked source row")
}

fn core_operation() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec!["TestClock".to_string()],
        operation: "sleep".to_string(),
        arg_types: vec![CoreType::Base("Int".to_string())],
        result_type: CoreType::Base("Int".to_string()),
    }
}

fn checked_core_handle() -> CheckedLoweredCoreProgram {
    checked_core_handle_with_residual(CoreRow::default())
}

fn checked_core_handle_with_residual(residual: CoreRow) -> CheckedLoweredCoreProgram {
    let operation = core_operation();
    let mut environment = CoreTypeCheckEnv::default();
    assert!(environment.operations_mut().insert(operation.clone()));

    let raw = RawCoreProgram::new(CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: operation.clone(),
            params: vec![CoreParam {
                name: "milliseconds".to_string(),
                ty: CoreType::Base("Int".to_string()),
            }],
            resume: CoreParam {
                name: "resume".to_string(),
                ty: CoreType::Cont {
                    input: Box::new(CoreType::Base("Int".to_string())),
                    answer: Box::new(CoreType::Base("Int".to_string())),
                    row: residual,
                    multiplicity: CoreMultiplicity::Affine,
                },
            },
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Var("resume".to_string()),
                arg: CoreAtom::Var("milliseconds".to_string()),
            }),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Raise {
            op: operation,
            args: vec![CoreAtom::LitInt(0)],
        }),
    });
    let validated = validate_core_program(raw).expect("fixture Core must validate before typing");

    type_check_and_lower_core_program(
        validated,
        &environment,
        CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default()),
    )
    .expect("fixture Core must typecheck and lower before admission")
}

fn provider_installation(operation: OperationIdentityV1) -> FrameInstallationInstructionV1 {
    FrameInstallationInstructionV1::Provider {
        operation,
        provider_binding: ProviderBindingV1::new(
            operation_identity(&checked_source()),
            "clock-host",
            "sleep",
        ),
    }
}

fn handler_installation(operation: OperationIdentityV1) -> FrameInstallationInstructionV1 {
    FrameInstallationInstructionV1::SourceHandler {
        operation,
        handler_name: "absorb_sleep".to_string(),
        core_handle: CoreHandleLocatorV1::root(),
    }
}

fn valid_admission() -> CheckedCpsAdmissionV1 {
    let checked = checked_source();
    let operation = operation_identity(&checked);
    CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        checked_source_facts(&checked),
        vec![
            provider_installation(operation.clone()),
            handler_installation(operation),
        ],
    )
    .expect("fully checked Core/CPS and explicit matching instructions must admit")
}

#[test]
fn fully_handled_operation_admits_with_a_source_handler_instruction_and_no_provider() {
    let checked = checked_source();
    let operation = operation_identity(&checked);
    let admission = CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        checked_source_facts(&checked),
        vec![handler_installation(operation.clone())],
    )
    .expect("a checked source handler explicitly authorizes its fully handled operation");

    assert_eq!(
        admission.frame_installations(),
        &[handler_installation(operation)],
        "a row is not a provider authorization, and a fully handled operation needs no provider"
    );
}

#[test]
fn unhandled_residual_operation_without_an_explicit_provider_binding_rejects() {
    let (checked, audit_operation) = checked_source_with_unhandled_residual();
    let handled_operation = operation_identity(&checked);
    let residual = CoreRow::closed(vec![CoreRowItem::Operation {
        path: vec!["TestAudit".to_string()],
        operation: "record".to_string(),
    }]);
    let error = CheckedCpsAdmissionV1::validate(
        checked_core_handle_with_residual(residual),
        checked_source_facts_with_residual_operations(&checked, vec![audit_operation.clone()]),
        vec![handler_installation(handled_operation)],
    )
    .expect_err("an unhandled residual operation must not become a provider frame from its row");

    assert_eq!(
        error,
        CheckedCpsAdmissionError::MissingFrameInstallationAuthorization {
            operation: OperationIdentityV1::from_declared(&audit_operation),
        },
    );
}

#[test]
fn unexpanded_open_residual_tail_rejects_even_with_its_known_handler_instruction() {
    let checked = type_check_program(&parse_program(OPEN_TAIL_DERIVED_HANDLER_SOURCE))
        .expect("open-tail fixture must retain checked handler-application evidence");
    let handler = checked
        .checked_handlers
        .get("clock")
        .expect("derived handler must retain a checked declaration fact");
    let [clause] = handler.clauses.as_slice() else {
        panic!("derived fixture must retain one concrete clock clause");
    };
    let operation = OperationIdentityV1::from_declared(&clause.operation);
    let source_facts = CheckedSourceFactsV1::from_type_check(&checked, "clock", source_anchor())
        .expect("the unexpanded tail is checked source evidence, not an implicit provider grant");

    let error = CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        source_facts,
        vec![FrameInstallationInstructionV1::SourceHandler {
            operation,
            handler_name: "clock".to_string(),
            core_handle: CoreHandleLocatorV1::root(),
        }],
    )
    .expect_err(
        "an open residual tail must reject at admission until its concrete expansion is resolver-attested",
    );

    assert!(
        error.to_string().contains("open residual")
            || error.to_string().contains("resolver-attested"),
        "the rejection must identify the unresolved tail admission boundary: {error}",
    );
}

#[test]
fn valid_v1_admission_retains_exact_facts_and_explicit_instruction_order() {
    let checked = checked_source();
    let expected_operation = operation_identity(&checked);
    let admission = valid_admission();

    assert_eq!(
        admission.operation_identities(),
        std::slice::from_ref(&expected_operation)
    );
    assert_eq!(admission.source_anchors(), &[source_anchor()]);
    assert_eq!(admission.handler_clauses().len(), 1);
    assert_eq!(
        admission.handler_clauses()[0].operation(),
        &expected_operation
    );
    assert_eq!(admission.residual_rows().len(), 1);
    assert!(
        admission.residual_rows()[0].is_closed_empty(),
        "the checked source handler's peeled residual must remain an explicit descriptor"
    );
    assert_eq!(
        admission.frame_installations(),
        &[
            provider_installation(expected_operation.clone()),
            handler_installation(expected_operation),
        ],
        "explicit frame instructions retain input order and are not reconstructed from rows"
    );
}

#[test]
fn matching_row_without_explicit_frame_authorization_rejects_as_missing_admission() {
    let checked = checked_source();
    let operation = operation_identity(&checked);
    let error = CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        checked_source_facts(&checked),
        Vec::new(),
    )
    .expect_err("a matching checked row alone must never install a provider or handler frame");

    assert_eq!(
        error,
        CheckedCpsAdmissionError::MissingFrameInstallationAuthorization { operation },
    );
}

#[test]
fn provider_instruction_rejects_same_spelling_with_different_concrete_identity() {
    let checked = checked_source();
    let operation = operation_identity(&checked);
    let mismatched_provider_identity = OperationIdentityV1::new(
        "OtherClock",
        "OtherClockInterface",
        "sleep",
        ["String"],
        "String",
    );
    let instructions = vec![
        FrameInstallationInstructionV1::Provider {
            operation: operation.clone(),
            provider_binding: ProviderBindingV1::new(
                mismatched_provider_identity.clone(),
                "clock-host",
                "sleep",
            ),
        },
        handler_installation(operation.clone()),
    ];

    let error = CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        checked_source_facts(&checked),
        instructions,
    )
    .expect_err("provider authorization must compare exact impl/interface/signature identity");

    assert_eq!(
        error,
        CheckedCpsAdmissionError::ProviderIdentityMismatch {
            expected: operation,
            actual: mismatched_provider_identity,
        },
    );
}

#[test]
fn source_handler_instruction_rejects_a_missing_or_wrong_core_handle_locator() {
    let checked = checked_source();
    let operation = operation_identity(&checked);
    let instructions = vec![
        provider_installation(operation.clone()),
        FrameInstallationInstructionV1::SourceHandler {
            operation,
            handler_name: "absorb_sleep".to_string(),
            core_handle: CoreHandleLocatorV1::at_path([1]),
        },
    ];

    let error = CheckedCpsAdmissionV1::validate(
        checked_core_handle(),
        checked_source_facts(&checked),
        instructions,
    )
    .expect_err("a source handler instruction must name the exact checked Core Handle node");

    assert_eq!(
        error,
        CheckedCpsAdmissionError::CoreHandleLocatorMismatch {
            handler_name: "absorb_sleep".to_string(),
            locator: CoreHandleLocatorV1::at_path([1]),
        },
    );
}
