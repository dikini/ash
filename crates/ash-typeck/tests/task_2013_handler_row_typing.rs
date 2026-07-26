//! TASK-2013 RED contract for row-aware source-handler declaration typing.
//!
//! These tests exercise parsed Ash through the public typechecker.  They do
//! not lower to Core or construct an engine/runtime handler.  The eventual
//! checked declaration fact is required to retain the exact residual row and
//! continuation multiplicity that these source programs demand.

use ash_parser::surface::{Definition, Expr, HandlerClause, Program, ProgramEntry};
use ash_typeck::{
    ContinuationMultiplicity, NormalizedHandlerRow, checked_handler_row_fact_for_test,
    type_check_program, types::Type,
};

const DEVICE_PREFIX: &str = r#"
interface Device<T> { read(Int) -> Int write(Int) -> Int }
type TestDevice = SystemDevice(Int);
impl Device<TestDevice> { read(value) = value write(value) = value }
"#;

fn parse_program(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2013 source should parse: {errors:?}"));
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

fn handler_clauses_mut(program: &mut Program) -> &mut Vec<HandlerClause> {
    let handler = program
        .definitions
        .iter_mut()
        .find_map(|definition| match definition {
            Definition::Handler(handler) if handler.name.as_ref() == "h" => Some(handler),
            _ => None,
        })
        .expect("fixture must define handler h");
    let Expr::On { clauses, .. } = &mut handler.body else {
        panic!("fixture handler must have a canonical on body");
    };
    clauses
}

fn row_keys(row: &NormalizedHandlerRow) -> Vec<String> {
    row.items.iter().map(|item| item.canonical_key()).collect()
}

#[test]
fn task_2013_handler_rejects_a_declared_clause_absent_from_the_inferred_operand_row() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, read_resume) => read_resume(value),\
             TestDevice::write(value, write_resume) => write_resume(value),\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let error = type_check_program(&program).expect_err(
        "a handler may peel only operations actually present in its inferred operand row",
    );
    assert!(
        error.to_string().contains("TestDevice::Device::write")
            || error.to_string().contains("absent")
            || error.to_string().contains("residual"),
        "the absence diagnostic must identify the rejected declared operation without a runtime fallback: {error}"
    );
}

#[test]
fn task_2013_handler_rejects_a_second_source_clause_for_the_same_canonical_operation() {
    let mut program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{ TestDevice::read(value, resume) => resume(value), done(result) => result }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));
    let clauses = handler_clauses_mut(&mut program);
    let operation = clauses
        .iter()
        .find(|clause| matches!(clause, HandlerClause::Operation { .. }))
        .expect("fixture must have an operation clause")
        .clone();
    clauses.insert(1, operation);

    let error = type_check_program(&program)
        .expect_err("the same canonical operation may be peeled exactly once");
    assert!(
        error.to_string().contains("duplicate") && error.to_string().contains("operation"),
        "duplicate operation identity must reject before handler facts publish: {error}"
    );
}

#[test]
fn task_2013_done_binder_has_the_operand_result_not_the_handler_answer() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> String {{\
           on comp {{\
             TestDevice::read(value, resume) => resume(value),\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let error = type_check_program(&program).expect_err(
        "done(result) must bind the handled operand's Int result, so it cannot return String unchanged",
    );
    assert!(
        error.to_string().contains("done")
            && (error.to_string().contains("String") || error.to_string().contains("Int")),
        "done-result/answer disagreement must remain a typed source diagnostic: {error}"
    );
}

#[test]
fn task_2013_every_operation_and_done_body_share_one_answer_type() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read, TestDevice::write }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, read_resume) => read_resume(value),\
             TestDevice::write(value, write_resume) => \"wrong answer\",\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let error = type_check_program(&program)
        .expect_err("every operation branch and done body must agree on one answer type");
    assert!(
        error.to_string().contains("handler operation body") || error.to_string().contains("Int"),
        "branch answer mismatch must reject in declaration checking: {error}"
    );
}

#[test]
fn task_2013_closed_empty_residual_allows_repeated_resume() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => {{ resume(value); resume(value) }},\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let result = type_check_program(&program).expect(
        "peeling the only closed operation leaves {}; its continuation is MultiShotPure and may resume twice",
    );
    let fact = checked_handler_row_fact_for_test(&result, "h")
        .expect("the successful declaration must publish a typed handler fact");
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        ["operation:TestDevice::Device::read"],
        "the implicitly thunked operand retains its declared operation requirement"
    );
    assert!(fact.residual_row.items.is_empty());
    assert_eq!(fact.residual_row.tail, None);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(fact.done_binding, "result");
    assert_eq!(fact.done_binding_type, Type::Int);
    assert_eq!(fact.clauses.len(), 1);
    assert_eq!(
        fact.clauses[0].continuation_row, fact.residual_row,
        "a continuation receives the declaration's exact residual row"
    );
    assert_eq!(
        fact.clauses[0].continuation_multiplicity,
        ContinuationMultiplicity::MultiShotPure,
        "only a normalized closed-empty residual is multi-shot"
    );
}

#[test]
fn task_2013_each_distinct_declared_clause_peels_once_and_shares_the_closed_residual() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read, TestDevice::write }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, read_resume) => read_resume(value),\
             TestDevice::write(value, write_resume) => write_resume(value),\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let result = type_check_program(&program)
        .expect("each distinct declared operation present in the input row may be peeled once");
    let fact = checked_handler_row_fact_for_test(&result, "h").expect("checked handler fact");
    assert_eq!(
        row_keys(&fact.input_row),
        [
            "operation:TestDevice::Device::read",
            "operation:TestDevice::Device::write",
        ]
    );
    assert!(fact.residual_row.items.is_empty());
    assert_eq!(fact.residual_row.tail, None);
    assert_eq!(
        fact.clauses
            .iter()
            .map(|clause| clause.operation.operation.as_str())
            .collect::<Vec<_>>(),
        ["read", "write"]
    );
    assert!(fact.clauses.iter().all(|clause| {
        clause.continuation_row == fact.residual_row
            && clause.continuation_multiplicity == ContinuationMultiplicity::MultiShotPure
    }));
}

#[test]
fn task_2013_clause_body_operations_extend_the_handler_output_not_the_residual_continuation_row() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => TestDevice::write(value),\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let result = type_check_program(&program).expect(
        "a declared clause-body operation is a typed requirement, not runtime dispatch authority",
    );
    let fact = checked_handler_row_fact_for_test(&result, "h").expect("checked handler fact");
    assert_eq!(
        row_keys(&fact.residual_row),
        Vec::<String>::new(),
        "R-H remains the exact residual; clause-body requirements belong to output_row"
    );
    assert_eq!(
        row_keys(&fact.output_row),
        ["operation:TestDevice::Device::write"],
        "handler output must union R-H with the declared clause-body operation"
    );
    assert_eq!(
        row_keys(&fact.clauses[0].continuation_row),
        Vec::<String>::new(),
        "the continuation carries only R-H; clause-body requirements belong exclusively to the handler output"
    );
    assert_eq!(
        fact.clauses[0].continuation_multiplicity,
        ContinuationMultiplicity::MultiShotPure,
        "continuation multiplicity derives from R-H, not from the handler body's output effects"
    );
}

#[test]
fn task_2013_done_body_operations_extend_the_handler_output_not_the_residual_continuation_row() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => resume(value),\
             done(result) => TestDevice::write(result),\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let result = type_check_program(&program).expect(
        "a declared done-body operation is a typed output requirement, never runtime dispatch authority",
    );
    let fact = checked_handler_row_fact_for_test(&result, "h").expect("checked handler fact");
    assert_eq!(
        row_keys(&fact.residual_row),
        Vec::<String>::new(),
        "the handled input operation alone is peeled from R"
    );
    assert_eq!(
        row_keys(&fact.output_row),
        ["operation:TestDevice::Device::write"],
        "handler output must union R-H with the done-body operation requirement"
    );
    assert_eq!(
        row_keys(&fact.clauses[0].continuation_row),
        Vec::<String>::new(),
        "continuations carry the residual R-H, never done-body output requirements"
    );
    assert_eq!(
        fact.clauses[0].continuation_multiplicity,
        ContinuationMultiplicity::MultiShotPure,
        "a done-body effect cannot downgrade the independently residual-based continuation multiplicity"
    );
}

#[test]
fn task_2013_nested_or_mixed_resume_never_falls_through_to_an_ordinary_unbound_variable() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => {{ resume(value); value }},\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    match type_check_program(&program) {
        Ok(result) => {
            let fact = checked_handler_row_fact_for_test(&result, "h")
                .expect("a supported nested resume must remain a typed handler fact");
            assert_eq!(fact.clauses.len(), 1);
            assert!(
                fact.clauses[0].continuation_row.items.is_empty()
                    && fact.clauses[0].continuation_row.tail.is_none()
            );
            assert_eq!(
                fact.clauses[0].continuation_multiplicity,
                ContinuationMultiplicity::MultiShotPure,
                "supported nested continuation use must preserve its closed-empty multiplicity"
            );
        }
        Err(error) => assert!(
            error
                .to_string()
                .contains("unsupported-handler-continuation-use"),
            "nested/mixed resume must fail through the stable handler-continuation boundary, not ordinary expression lookup: {error}"
        ),
    }
}

#[test]
fn task_2013_operation_clause_pattern_must_match_the_declared_payload_type() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(\"not an Int\", resume) => resume(0),\
             done(result) => result,\
           }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let error = type_check_program(&program).expect_err(
        "an operation clause pattern must be checked against the declared Int payload before a handler fact publishes",
    );
    assert!(
        error.to_string().contains("handler operation pattern")
            && error.to_string().contains("Int"),
        "payload-pattern mismatch must retain its handler-specific type boundary: {error}"
    );
}

#[test]
fn task_2013_every_nonempty_or_open_residual_keeps_resume_affine() {
    let residuals = [
        "resource read filesystem",
        "role ops.admin",
        "policy audit",
        "channel write audit_events",
        "process spawn",
        "fail NetworkError",
        "evidence response.proved",
        "TestDevice::write",
        "Unmanaged",
        "| rest",
    ];

    for residual in residuals {
        let prelude = if residual == "Unmanaged" {
            "effect alias Unmanaged = { resource read filesystem };"
        } else {
            ""
        };
        let comma = if residual.starts_with('|') { "" } else { "," };
        let row = format!("{{ TestDevice::read {comma} {residual} }}");
        let program = parse_program(&format!(
            "{DEVICE_PREFIX}{prelude}\
             handler h(comp: () -> {row} Int) -> Int {{\
               on comp {{\
                 TestDevice::read(value, resume) => {{ resume(value); resume(value) }},\
                 done(result) => result,\
               }}\
             }}\
             fn main() -> Int {{ 0 }}"
        ));

        let error = type_check_program(&program).expect_err(
            "a nonempty or open residual makes the continuation affine, so a second resume must reject",
        );
        assert!(
            error.to_string().contains("affine") || error.to_string().contains("resume"),
            "{residual} must not be erased into a multi-shot residual: {error}"
        );
    }
}

#[test]
fn task_2013_alias_group_and_all_nonoperation_families_are_not_peeled_by_operation_clauses() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         effect alias Base = {{ resource read filesystem, role ops.admin, policy audit, fail NetworkError, evidence response.proved | rest }};\
         effect group Remaining = {{ Base, channel write audit_events, process spawn, TestDevice::write }};\
         handler h(comp: () -> {{ TestDevice::read, group Remaining }} Int) -> Int {{\
           on comp {{ TestDevice::read(value, resume) => resume(value), done(result) => result }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let result = type_check_program(&program).expect(
        "one concrete clause may peel only TestDevice::read while every alias/group-expanded non-operation remains residual",
    );
    let fact = checked_handler_row_fact_for_test(&result, "h")
        .expect("successful row typing publishes a declaration fact only");
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(fact.done_binding_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        [
            "operation:TestDevice::Device::read",
            "operation:TestDevice::Device::write",
            "resource:read:filesystem",
            "role:ops.admin",
            "policy:audit",
            "channel:write:audit_events",
            "process:spawn",
            "fail:NetworkError",
            "evidence:response.proved",
        ],
        "input normalization must retain every parseable non-operation family and the unhandled operation"
    );
    assert_eq!(
        row_keys(&fact.residual_row),
        [
            "operation:TestDevice::Device::write",
            "resource:read:filesystem",
            "role:ops.admin",
            "policy:audit",
            "channel:write:audit_events",
            "process:spawn",
            "fail:NetworkError",
            "evidence:response.proved",
        ],
        "an operation clause peels only its one canonical operation; it cannot erase alias/group-expanded requirements"
    );
    assert_eq!(fact.residual_row.tail.as_deref(), Some("rest"));
    assert_eq!(fact.clauses.len(), 1);
    assert_eq!(fact.clauses[0].continuation_row, fact.residual_row);
    assert_eq!(
        fact.clauses[0].continuation_multiplicity,
        ContinuationMultiplicity::Affine,
        "non-operation requirements and an open tail keep the continuation affine"
    );
}
