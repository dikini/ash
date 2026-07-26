//! TASK-2013 regression contract for immutable implicit-thunk application facts.
//!
//! A successful `handle expr with handler` is source typing evidence.  It must
//! be retained for later inspection; validation alone is insufficient and must
//! never create a Core/runtime handler artifact.

use ash_parser::{
    surface::{Definition, Program, ProgramEntry},
    token::Span,
};
use ash_typeck::{checked_handler_application_facts_for_test, type_check_program, types::Type};

const DEVICE_PREFIX: &str = r#"
interface Device<T> {
    read(Int) -> Int
    write(Int) -> Int
}
type TestDevice = SystemDevice(Int);
impl Device<TestDevice> {
    read(value) = value
    write(value) = value
}
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

fn row_keys(row: &ash_typeck::NormalizedHandlerRow) -> Vec<String> {
    row.items.iter().map(|item| item.canonical_key()).collect()
}

fn last_span(source: &str, text: &str) -> Span {
    let start = source
        .rfind(text)
        .unwrap_or_else(|| panic!("fixture must contain `{text}`"));
    let prefix = &source[..start];
    Span::new(
        start,
        start + text.len(),
        prefix.lines().count(),
        prefix
            .rsplit_once('\n')
            .map_or(prefix.len() + 1, |(_, line)| line.len() + 1),
    )
}

const DERIVED_CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    derive handler clock;
}
"#;

#[test]
fn task_2013_derived_handler_application_instantiates_the_answer_and_empty_residual() {
    let source = format!(
        "{DERIVED_CLOCK_PREFIX}\
         fn main() -> Int {{ handle TestClock::sleep(0) with clock }}"
    );
    let expected_handler_span = last_span(&source, "clock");
    let expected_expression_span = last_span(&source, "TestClock::sleep(0)");
    let program = parse_program(&source);

    let checked = type_check_program(&program).expect(
        "a derived total identity handler must resolve through the normal source handler path",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(
        facts.len(),
        1,
        "one derived source handler application must publish exactly one immutable fact"
    );

    let fact = &facts[0];
    assert_eq!(fact.handler_name, "clock");
    assert_eq!(fact.handler_span, expected_handler_span);
    assert_eq!(fact.expression_span, expected_expression_span);
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        ["operation:TestClock::Clock::sleep"],
        "normal source resolution must retain the derived impl operation identity"
    );
    assert!(
        fact.input_row.tail.is_none(),
        "the one-operation control instantiates the derived residual r to the empty row"
    );
    assert!(
        fact.output_row.items.is_empty() && fact.output_row.tail.is_none(),
        "the empty residual must remain the application output row"
    );
}

#[test]
fn task_2013_derived_handler_application_binds_and_preserves_a_nonempty_residual() {
    let source = format!(
        "{DERIVED_CLOCK_PREFIX}\
         interface Audit<T> {{ record(Int) -> Int }}\
         type TestAudit = SystemAudit(Int);\
         impl Audit<TestAudit> {{ record(value) = value }}\
         fn main() -> Int {{\
           handle {{ TestClock::sleep(0); TestAudit::record(0) }} with clock\
         }}"
    );
    let expected_handler_span = last_span(&source, "clock");
    let expected_expression_span =
        last_span(&source, "{ TestClock::sleep(0); TestAudit::record(0) }");
    let program = parse_program(&source);

    let checked = type_check_program(&program).expect(
        "the residual row must bind to requirements beyond every operation peeled by derive handler",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1);

    let fact = &facts[0];
    assert_eq!(fact.handler_name, "clock");
    assert_eq!(fact.handler_span, expected_handler_span);
    assert_eq!(fact.expression_span, expected_expression_span);
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        [
            "operation:TestAudit::Audit::record",
            "operation:TestClock::Clock::sleep",
        ],
        "the published application input must retain NormalizedHandlerRow's canonical lexical order, even when derive handler peels an operation"
    );
    assert!(fact.input_row.tail.is_none());
    assert_eq!(
        row_keys(&fact.output_row),
        ["operation:TestAudit::Audit::record"],
        "the instantiated residual r must be preserved after the derived operation is peeled"
    );
    assert!(
        fact.output_row.tail.is_none(),
        "the concrete residual requirement must not be left as the derived fact's synthetic tail name"
    );
}

#[test]
fn task_2013_derived_handler_application_keeps_canonical_order_when_impl_declaration_order_differs()
{
    let program = parse_program(
        r#"
        interface Clock<T> {
            sleep(Int) -> Int
            wake(Int) -> Int
        }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            wake(milliseconds) = milliseconds
            sleep(milliseconds) = milliseconds
            derive handler clock;
        }
        fn main() -> Int {
            handle { TestClock::sleep(0); TestClock::wake(0) } with clock
        }
        "#,
    );

    let checked = type_check_program(&program).expect(
        "a total derived handler must publish the operand row independently of impl method order",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1);
    assert_eq!(
        row_keys(&facts[0].input_row),
        [
            "operation:TestClock::Clock::sleep",
            "operation:TestClock::Clock::wake",
        ],
        "a published application fact is normalized source evidence, not a declaration-order dispatch plan"
    );
}

#[test]
fn task_2013_derived_handler_marker_without_a_checked_fact_rejects_handle_admission() {
    let program = parse_program(
        r#"
        interface Clock<T> { sleep(Int) -> Int }
        impl Clock<Option<Int>> {
            sleep(milliseconds) = milliseconds
            derive handler clock;
        }
        fn main() -> Int { handle 0 with clock }
        "#,
    );

    let error = type_check_program(&program).expect_err(
        "a derived marker whose non-name impl target cannot materialize a checked fact must not admit a handle application",
    );
    assert!(
        error
            .to_string()
            .contains("handler 'clock' has no checked declaration"),
        "marker-only registration must fail before it can publish an application fact: {error}"
    );
}

#[test]
fn task_2013_derived_handler_application_preserves_an_open_annotated_residual() {
    let source = format!(
        "{DERIVED_CLOCK_PREFIX}\
         effect alias OpenClock = {{ TestClock::sleep | rest }};\
         fn main(computation: () -> {{ OpenClock }} Int) -> Int {{\
             handle computation() with clock\
         }}"
    );
    let expected_handler_span = last_span(&source, "clock");
    let expected_expression_span = last_span(&source, "computation()");
    let program = parse_program(&source);

    let checked = type_check_program(&program).expect(
        "a row-annotated computation parameter must provide immutable open-tail evidence to a derived handler application",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1);

    let fact = &facts[0];
    assert_eq!(fact.handler_name, "clock");
    assert_eq!(fact.handler_span, expected_handler_span);
    assert_eq!(fact.expression_span, expected_expression_span);
    assert_ne!(fact.handler_span, Span::default());
    assert_ne!(fact.expression_span, Span::default());
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        ["operation:TestClock::Clock::sleep"],
        "the annotated operand must retain the concrete declared operation identity"
    );
    assert_eq!(fact.input_row.tail.as_deref(), Some("rest"));
    assert!(
        !fact.input_row.tail_provenances().is_empty()
            && fact
                .input_row
                .tail_provenances()
                .iter()
                .all(|provenance| provenance.source_span() != Span::default()),
        "the annotated open tail must retain non-default source provenance"
    );
    assert!(
        fact.output_row.items.is_empty(),
        "the derived handler must peel its declared sleep operation exactly once"
    );
    assert_eq!(fact.output_row.tail.as_deref(), Some("rest"));
    assert_eq!(
        fact.output_row.tail_provenances(),
        fact.input_row.tail_provenances(),
        "the residual must preserve the operand tail's source provenance exactly"
    );
}

#[test]
fn task_2013_derived_handler_application_resolves_a_grouped_open_operand_row_without_lowering() {
    let source = format!(
        "{DERIVED_CLOCK_PREFIX}\
         effect group OpenClockGroup = {{ TestClock::sleep | rest }};\
         fn main(computation: () -> {{ OpenClockGroup }} Int) -> Int {{\
             handle computation() with clock\
         }}"
    );
    let expected_handler_span = last_span(&source, "clock");
    let expected_expression_span = last_span(&source, "computation()");
    let program = parse_program(&source);

    let checked = type_check_program(&program).expect(
        "a grouped row-annotated computation parameter must supply exact implicit-thunk evidence to a handler marker",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(
        facts.len(),
        1,
        "type checking a grouped-row handle application publishes one immutable source fact and does not invoke lowering"
    );

    let fact = &facts[0];
    assert_eq!(fact.handler_name, "clock");
    assert_eq!(fact.handler_span, expected_handler_span);
    assert_eq!(fact.expression_span, expected_expression_span);
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        ["operation:TestClock::Clock::sleep"],
        "the group must structurally normalize to the declared concrete operation identity"
    );
    assert_eq!(fact.input_row.tail.as_deref(), Some("rest"));
    assert!(
        fact.input_row
            .items
            .iter()
            .all(|item| !item.grants_authority() && !item.source_provenance().is_empty())
            && fact
                .input_row
                .tail_provenances()
                .iter()
                .all(|provenance| provenance.source_span() != Span::default()),
        "group expansion must retain authority-neutral item and open-tail source provenance"
    );
    assert!(
        fact.output_row.items.is_empty(),
        "the derived handler must peel only its declared operation from the grouped input"
    );
    assert_eq!(fact.output_row.tail.as_deref(), Some("rest"));
    assert_eq!(
        fact.output_row.tail_provenances(),
        fact.input_row.tail_provenances(),
        "the grouped open-tail provenance must survive exactly in the residual fact"
    );
}

#[test]
fn task_2013_derived_handler_application_rejects_a_computation_missing_a_declared_operation() {
    let program = parse_program(
        r#"
        interface Clock<T> {
            sleep(Int) -> Int
            wake(Int) -> Int
        }
        type TestClock = SystemClock(Int);
        impl Clock<TestClock> {
            sleep(milliseconds) = milliseconds
            wake(milliseconds) = milliseconds
            derive handler clock;
        }
        fn main() -> Int { handle TestClock::sleep(0) with clock }
        "#,
    );

    let error = type_check_program(&program).expect_err(
        "a total derived handler cannot accept a computation that omits one of its declared impl operations",
    );
    assert!(
        error
            .to_string()
            .contains("handler 'clock' input computation mismatch"),
        "the omission must reject at the derived handler input contract: {error}"
    );
}

#[test]
fn task_2013_handle_with_publishes_an_immutable_typed_application_fact() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => TestDevice::write(value),\
             done(value) => value,\
           }}\
         }}\
         fn main() -> Int {{ handle TestDevice::read(1) with h }}"
    ));

    let checked = type_check_program(&program)
        .expect("an exactly matched implicit thunk must typecheck without runtime lowering");
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(
        facts.len(),
        1,
        "one source handle expression publishes one immutable fact"
    );

    let fact = &facts[0];
    assert_eq!(fact.handler_name, "h");
    assert_ne!(
        fact.expression_span,
        Default::default(),
        "the operand anchor is retained"
    );
    assert_ne!(
        fact.handler_span,
        Default::default(),
        "the handler-name anchor is retained"
    );
    assert_eq!(fact.input_result_type, Type::Int);
    assert_eq!(
        row_keys(&fact.input_row),
        ["operation:TestDevice::Device::read"],
        "the application retains the exact normalized operand row"
    );
    assert_eq!(fact.answer_type, Type::Int);
    assert_eq!(
        row_keys(&fact.output_row),
        ["operation:TestDevice::Device::write"],
        "the application retains the handler answer/output row, not merely a validation result"
    );
    assert!(
        fact.input_row
            .items
            .iter()
            .all(|item| !item.source_provenance().is_empty())
            && fact
                .output_row
                .items
                .iter()
                .all(|item| !item.source_provenance().is_empty()),
        "application rows retain source provenance for later diagnostics"
    );
}

#[test]
fn task_2013_derived_handler_application_does_not_reuse_outer_row_for_shadowing_local_callable() {
    let program = parse_program(&format!(
        "{DERIVED_CLOCK_PREFIX}\
         effect alias OpenClock = {{ TestClock::sleep | rest }};\
         fn main(computation: () -> {{ OpenClock }} Int) -> Int {{\
             fn computation() -> Int {{ 0 }}\
             handle computation() with clock\
         }}"
    ));

    let error = type_check_program(&program).expect_err(
        "a local callable named computation must not inherit the outer parameter's OpenClock row",
    );
    let message = error.to_string();
    assert!(
        message.contains("handler 'clock' input computation mismatch")
            || message.contains("unsupported computation boundary"),
        "the local callable must reject at the handler input boundary instead of publishing an outer-row application fact: {message}"
    );
}
