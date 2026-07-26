//! TASK-2013 RED contract for implicit-thunk computation inference.
//!
//! This is intentionally a narrow, doc-hidden typechecker seam.  It proves
//! that handler computation evidence is derived structurally from parsed Ash,
//! retains normalized row provenance, and fails closed.  It does not create a
//! source thunk, Core term, provider frame, or runtime handler.

use ash_parser::{
    Spanned,
    surface::{
        Definition, Expr, HandlerClause, Pattern, Program, ProgramEntry, VariantPatternPayload,
    },
};
use ash_typeck::{
    CheckedComputation, infer_checked_computation_for_test,
    infer_checked_handler_computation_for_test, type_check_program, types::Type,
    union_checked_computations_for_test,
};

const CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = 0 }
"#;

const BRANCH_CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Int wake(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = 0 wake(milliseconds) = 0 }
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

fn function_body<'a>(program: &'a Program, name: &str) -> &'a Expr {
    program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => {
                Some(&function.body)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("fixture must define function {name}"))
}

fn infer_main(program: &Program) -> CheckedComputation {
    infer_checked_computation_for_test(program, "main")
        .expect("the selected main expression should have immutable computation evidence")
}

fn keys(computation: &CheckedComputation) -> Vec<String> {
    computation
        .normalized_row()
        .items
        .iter()
        .map(|item| item.canonical_key())
        .collect()
}

fn assert_inferable_branch_union(body: &str) {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX} fn main() -> Int {{ {body} }}"
    ));
    let checked = infer_main(&program);

    assert_eq!(checked.result_type(), &Type::Int);
    assert_eq!(
        keys(&checked),
        [
            "operation:TestClock::Clock::sleep",
            "operation:TestClock::Clock::wake",
        ],
        "every inferable branch child contributes its declared operation"
    );
    assert!(
        checked
            .normalized_row()
            .items
            .iter()
            .all(|item| item.source_provenance().len() == 1),
        "each branch operation retains its own source provenance"
    );
}

fn assert_pattern_bound_branch_inference(body: &str) {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX}\
         type Option<T> = Some {{ value: T }} | None;\
         fn main() -> Int {{ {body} }}"
    ));

    type_check_program(&program).expect(
        "ordinary expression checking must install each branch pattern binding before checking its operation argument",
    );

    let checked = infer_main(&program);
    assert_eq!(checked.result_type(), &Type::Int);
    assert_eq!(
        keys(&checked),
        [
            "operation:TestClock::Clock::sleep",
            "operation:TestClock::Clock::wake",
        ],
        "the pattern-bound branch operation and its fallback must both contribute their declared rows"
    );
    assert!(
        checked
            .normalized_row()
            .items
            .iter()
            .all(|item| item.source_provenance().len() == 1),
        "each operation inferred through a pattern-bound branch must retain its own provenance"
    );
}

#[test]
fn task_2013_declared_concrete_operation_call_has_singleton_row_result_and_anchors() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX} fn main() -> Int {{ TestClock::sleep(7) }}"
    ));
    let checked = infer_main(&program);

    assert_eq!(checked.result_type(), &Type::Int);
    assert_eq!(keys(&checked), ["operation:TestClock::Clock::sleep"]);
    assert_eq!(checked.normalized_row().tail, None);
    assert_eq!(
        checked.expression_anchor(),
        function_body(&program, "main").span()
    );

    let operation = &checked.normalized_row().items[0];
    assert_eq!(operation.source_provenance().len(), 1);
    assert!(
        operation.source_provenance()[0].source_span().start >= checked.expression_anchor().start,
        "the declared operation spelling must retain its own source anchor"
    );
}

#[test]
fn task_2013_audited_pure_composites_union_child_rows_without_assuming_unknown_forms_pure() {
    let fixtures = [
        ("binary", "TestClock::sleep(1) + TestClock::sleep(2)"),
        ("collection", "[TestClock::sleep(1), 2]"),
        ("record", "{ value: TestClock::sleep(1), stable: 2 }"),
        ("sequence", "{ TestClock::sleep(1); 2 }"),
    ];

    for (name, body) in fixtures {
        let program = parse_program(&format!("{CLOCK_PREFIX} fn main() {{ {body} }}"));
        let checked = infer_main(&program);
        assert_eq!(
            keys(&checked),
            ["operation:TestClock::Clock::sleep"],
            "{name} must structurally retain its inferable operation child"
        );
    }
}

#[test]
fn task_2013_infers_if_branches_and_unions_each_child_row() {
    assert_inferable_branch_union("if true then TestClock::sleep(1) else TestClock::wake(2)");
}

#[test]
fn task_2013_infers_match_scrutinee_and_arms_and_unions_each_child_row() {
    assert_inferable_branch_union(
        "match true { true => TestClock::sleep(1), false => TestClock::wake(2) }",
    );
}

#[test]
fn task_2013_infers_if_let_scrutinee_and_branches_and_unions_each_child_row() {
    assert_inferable_branch_union(
        "if let true = true then { TestClock::sleep(1) } else { TestClock::wake(2) }",
    );
}

#[test]
fn task_2013_infers_match_arm_operation_argument_bound_by_its_pattern() {
    assert_pattern_bound_branch_inference(
        "match Some { value: 7 } {\
           Some { value: milliseconds } => TestClock::sleep(milliseconds),\
           None => TestClock::wake(0),\
         }",
    );
}

#[test]
fn task_2013_infers_if_let_then_operation_argument_bound_by_its_pattern() {
    assert_pattern_bound_branch_inference(
        "if let Some { value: milliseconds } = Some { value: 7 }\
         then TestClock::sleep(milliseconds)\
         else TestClock::wake(0)",
    );
}

#[test]
fn task_2013_match_pattern_rejection_is_anchored_at_the_pattern_not_its_body() {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX}\
         type Option<T> = Some {{ value: T }} | None;\
         fn main() -> Int {{\
           match Some {{ value: 7 }} {{\
             Some {{ missing: milliseconds }} => TestClock::sleep(milliseconds),\
             None => TestClock::wake(0),\
           }}\
         }}"
    ));
    type_check_program(&program)
        .expect_err("ordinary match checking must reject a pattern field absent from Some");

    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = function_body(&program, "main")
    else {
        panic!("fixture main body must retain a match tail expression");
    };
    let Expr::Match { arms, .. } = tail_expr.as_ref() else {
        panic!("fixture must parse to a match expression");
    };
    let Pattern::Variant {
        payload: VariantPatternPayload::Record(fields),
        ..
    } = &arms[0].pattern
    else {
        panic!("first arm must retain its record variant pattern");
    };
    let Pattern::Variable {
        span: pattern_anchor,
        ..
    } = &fields[0].1
    else {
        panic!("invalid pattern field must retain its binding anchor");
    };

    let error = infer_checked_computation_for_test(&program, "main")
        .expect_err("computation inference must reject the invalid pattern");
    assert_eq!(
        error.source_anchor(),
        *pattern_anchor,
        "invalid match patterns must be diagnosed at their own source pattern, not at the arm body"
    );
    assert_ne!(
        error.source_anchor(),
        arms[0].body.span(),
        "invalid match patterns must not inherit the arm body anchor"
    );
}

#[test]
fn task_2013_if_let_pattern_rejection_is_anchored_at_the_pattern_not_its_then_branch() {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX}\
         type Option<T> = Some {{ value: T }} | None;\
         fn main() -> Int {{\
           if let Some {{ missing: milliseconds }} = Some {{ value: 7 }}\
           then TestClock::sleep(milliseconds)\
           else TestClock::wake(0)\
         }}"
    ));
    type_check_program(&program)
        .expect_err("ordinary if-let checking must reject a pattern field absent from Some");

    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = function_body(&program, "main")
    else {
        panic!("fixture main body must retain an if-let tail expression");
    };
    let Expr::IfLet {
        pattern,
        then_branch,
        ..
    } = tail_expr.as_ref()
    else {
        panic!("fixture must parse to an if-let expression");
    };
    let Pattern::Variant {
        payload: VariantPatternPayload::Record(fields),
        ..
    } = pattern
    else {
        panic!("if-let pattern must retain its record variant shape");
    };
    let Pattern::Variable {
        span: pattern_anchor,
        ..
    } = &fields[0].1
    else {
        panic!("invalid pattern field must retain its binding anchor");
    };

    let error = infer_checked_computation_for_test(&program, "main")
        .expect_err("computation inference must reject the invalid if-let pattern");
    assert_eq!(
        error.source_anchor(),
        *pattern_anchor,
        "invalid if-let patterns must be diagnosed at their own source pattern, not at the then branch"
    );
    assert_ne!(
        error.source_anchor(),
        then_branch.span(),
        "invalid if-let patterns must not inherit the then-branch anchor"
    );
}

#[test]
fn task_2013_match_nested_unsupported_body_child_keeps_its_own_anchor() {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX}\
         fn main() -> Int {{\
           match true {{\
             true => [TestClock::sleep(1), unknown!()],\
             false => TestClock::wake(0),\
           }}\
         }}"
    ));

    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = function_body(&program, "main")
    else {
        panic!("fixture main body must retain a match tail expression");
    };
    let Expr::Match { arms, .. } = tail_expr.as_ref() else {
        panic!("fixture must parse to a match expression");
    };
    let Expr::List { items, .. } = arms[0].body.as_ref() else {
        panic!("first match arm must retain its list body");
    };
    let child = &items[1];

    let error = infer_checked_computation_for_test(&program, "main")
        .expect_err("a generic nested branch child must remain fail-closed");
    assert!(
        error
            .to_string()
            .contains("unsupported-handler-computation-expression"),
        "the nested branch child must retain the stable fail-closed diagnostic: {error}"
    );
    assert_eq!(
        error.source_anchor(),
        child.span(),
        "an unsupported nested match body child must retain its own source anchor"
    );
    assert_ne!(
        error.source_anchor(),
        arms[0].body.span(),
        "an unsupported nested child must not collapse to its containing match arm body"
    );
}

#[test]
fn task_2013_unclassified_branch_child_fails_at_its_own_anchor() {
    let program = parse_program(&format!(
        "{BRANCH_CLOCK_PREFIX}\
         fn is_ready() -> Bool {{ true }}\
         fn main() -> Int {{ if is_ready() then TestClock::sleep(1) else TestClock::wake(2) }}"
    ));
    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = function_body(&program, "main")
    else {
        panic!("fixture main body must retain an if tail expression");
    };
    let Expr::If { condition, .. } = tail_expr.as_ref() else {
        panic!("fixture must parse to an if expression");
    };

    let error = infer_checked_computation_for_test(&program, "main")
        .expect_err("a generic branch condition must remain fail-closed");
    assert!(
        error
            .to_string()
            .contains("unsupported-handler-computation-expression"),
        "the branch-inference boundary must retain its stable diagnostic: {error}"
    );
    assert_eq!(
        error.source_anchor(),
        condition.span(),
        "an unclassified branch child must be diagnosed at its own anchor"
    );
}

#[test]
fn task_2013_handler_operand_annotation_normalizes_alias_group_tail_and_non_operations() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Base = {{ TestClock::sleep, resource read filesystem, role ops.admin, policy audit, fail NetworkError, evidence response.proved, | rest }};\
         effect group All = {{ Base, channel write audit_events, process spawn }};\
         handler h(comp: () -> {{ All }} Int) -> Int {{\
           on comp {{ TestClock::sleep(ms, resume) => ms, done(value) => value }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));

    let checked = infer_checked_handler_computation_for_test(&program, "h")
        .expect("a typed handler operand must normalize its declared computation row");
    assert_eq!(checked.result_type(), &Type::Int);
    assert_eq!(
        keys(&checked),
        [
            "operation:TestClock::Clock::sleep",
            "resource:read:filesystem",
            "role:ops.admin",
            "policy:audit",
            "channel:write:audit_events",
            "process:spawn",
            "fail:NetworkError",
            "evidence:response.proved",
        ]
    );
    assert_eq!(checked.normalized_row().tail.as_deref(), Some("rest"));
    assert!(
        checked
            .normalized_row()
            .items
            .iter()
            .all(|item| !item.grants_authority())
    );
}

#[test]
fn task_2013_structural_union_is_idempotent_associative_and_rejects_conflicting_tails() {
    let singleton = parse_program(&format!(
        "{CLOCK_PREFIX} fn one() -> Int {{ TestClock::sleep(1) }} fn main() -> Int {{ 0 }}"
    ));
    let left = infer_checked_computation_for_test(&singleton, "one").expect("left fact");
    let right = infer_checked_computation_for_test(&singleton, "one").expect("right fact");
    let idempotent = union_checked_computations_for_test(&[left.clone(), right.clone()])
        .expect("equal operation rows must merge idempotently");
    assert_eq!(keys(&idempotent), ["operation:TestClock::Clock::sleep"]);
    assert_eq!(
        idempotent, left,
        "unioning one normalized fact with itself must preserve its provenance exactly, not duplicate its anchor"
    );
    assert_eq!(
        union_checked_computations_for_test(&[
            union_checked_computations_for_test(&[left.clone(), right.clone()])
                .expect("left union"),
            left.clone(),
        ])
        .expect("associated union"),
        union_checked_computations_for_test(&[
            left.clone(),
            union_checked_computations_for_test(&[right, left.clone()]).expect("right union"),
        ])
        .expect("associated union"),
        "compatible normalized rows must union associatively"
    );

    let tails = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect alias Left = {{ TestClock::sleep | left }};\
         effect alias Right = {{ TestClock::sleep | right }};\
         handler left_handler(comp: () -> {{ Left }} Int) -> Int {{ on comp {{ TestClock::sleep(ms, resume) => ms, done(value) => value }} }}\
         handler right_handler(comp: () -> {{ Right }} Int) -> Int {{ on comp {{ TestClock::sleep(ms, resume) => ms, done(value) => value }} }}\
         fn main() -> Int {{ 0 }}"
    ));
    let left =
        infer_checked_handler_computation_for_test(&tails, "left_handler").expect("left tail");
    let right =
        infer_checked_handler_computation_for_test(&tails, "right_handler").expect("right tail");
    let error = union_checked_computations_for_test(&[left, right])
        .expect_err("distinct open tails must never be silently merged");
    assert!(
        error
            .to_string()
            .contains("conflicting handler-computation row tails")
    );
}

#[test]
fn task_2013_canonical_on_operand_is_inferred_not_manufactured_from_its_annotation() {
    let fixtures = [
        (
            "literal_cannot_inherit_declared_operation_row",
            "on 0 { TestClock::sleep(ms, resume) => ms, done(value) => value }",
        ),
        (
            "unrelated_pure_composite_cannot_inherit_declared_operation_row",
            "on 1 + 2 { TestClock::sleep(ms, resume) => ms, done(value) => value }",
        ),
    ];

    for (name, on_body) in fixtures {
        let program = parse_program(&format!(
            "{CLOCK_PREFIX}\
             handler h(comp: () -> {{ TestClock::sleep }} Int) -> Int {{ {on_body} }}\
             fn main() -> Int {{ 0 }}"
        ));
        let operand = program
            .definitions
            .iter()
            .find_map(|definition| match definition {
                Definition::Handler(handler) if handler.name.as_ref() == "h" => match &handler.body
                {
                    Expr::On { computation, .. } => Some(computation.as_ref()),
                    _ => None,
                },
                _ => None,
            })
            .expect("fixture must retain its on operand");
        let error = infer_checked_handler_computation_for_test(&program, "h").expect_err(
            "the declared operation row must not be manufactured when the canonical on operand has no operation",
        );
        assert_eq!(
            error.source_anchor(),
            operand.span(),
            "{name} must attribute the annotation-vs-inferred-row mismatch to the canonical operand"
        );
    }
}

#[test]
fn task_2013_unclassified_forms_fail_closed_at_their_own_source_anchor() {
    let fixtures = [
        (
            "generic_call",
            "fn id(value: Int) -> Int { value } fn main() -> Int { id(1) }",
        ),
        (
            "with_error",
            "fn main() -> Int { with_error { 1 } handle { _ => 2 } }",
        ),
        ("macro", "fn main() -> Int { unknown!() }"),
    ];

    for (name, definitions) in fixtures {
        let program = parse_program(&format!("{CLOCK_PREFIX} {definitions}"));
        let error = infer_checked_computation_for_test(&program, "main")
            .expect_err("{name} must not be treated as an implicitly pure handler computation");
        assert!(
            error
                .to_string()
                .contains("unsupported-handler-computation-expression"),
            "{name} must use the stable fail-closed diagnostic: {error}"
        );
        assert_eq!(
            error.source_anchor(),
            function_body(&program, "main").span(),
            "{name} must point to its source expression rather than a synthetic empty row"
        );
    }
}

#[test]
fn task_2013_unsupported_nested_child_forms_report_the_child_anchor_not_a_supported_parent() {
    let fixtures = [
        (
            "generic_call",
            "fn id(value: Int) -> Int { value } fn main() -> Int { TestClock::sleep(1) + id(1) }",
        ),
        (
            "with_error",
            "fn main() -> Int { [TestClock::sleep(1), with_error { 1 } handle { _ => 2 }] }",
        ),
        (
            "macro",
            "fn main() -> Int { { value: unknown!(), stable: TestClock::sleep(1) } }",
        ),
    ];

    for (name, definitions) in fixtures {
        let program = parse_program(&format!("{CLOCK_PREFIX} {definitions}"));
        let parent = match function_body(&program, "main") {
            Expr::Block {
                tail_expr: Some(tail),
                ..
            } => tail.as_ref(),
            body => body,
        };
        let child = match (name, parent) {
            ("generic_call", Expr::Binary { right, .. }) => right.as_ref(),
            ("with_error", Expr::List { items, .. }) => &items[1],
            ("macro", Expr::Record { fields, .. }) => &fields[0].1,
            _ => panic!("{name} fixture must retain the nested unsupported child: {parent:?}"),
        };
        let error = infer_checked_computation_for_test(&program, "main")
            .expect_err("a nested unsupported form must fail closed");
        assert_eq!(
            error.source_anchor(),
            child.span(),
            "{name} must preserve the offending child anchor"
        );
        assert_ne!(
            error.source_anchor(),
            parent.span(),
            "{name} must not collapse the diagnostic to the supported composite parent"
        );
    }
}

#[test]
fn task_2013_constructor_tuple_and_record_payloads_are_audited_pure_composites() {
    let fixtures = [
        (
            "tuple",
            "type Pair = Pair(Int, Int) | Empty; fn main() { Pair(TestClock::sleep(1), 2) }",
        ),
        (
            "record",
            "type Envelope = Envelope { value: Int, stable: Int } | Empty; fn main() { Envelope { value: TestClock::sleep(1), stable: 2 } }",
        ),
    ];

    for (name, definitions) in fixtures {
        let program = parse_program(&format!("{CLOCK_PREFIX} {definitions}"));
        let checked = infer_main(&program);
        assert_eq!(
            keys(&checked),
            ["operation:TestClock::Clock::sleep"],
            "{name} constructor payloads must recursively retain inferable operation children"
        );
        assert_eq!(
            checked.expression_anchor(),
            function_body(&program, "main").span(),
            "{name} keeps the constructor as the computation anchor"
        );
        assert_eq!(
            checked.normalized_row().items[0].source_provenance().len(),
            1,
            "{name} must preserve the single operation-child source anchor"
        );
    }
}

#[test]
fn task_2013_handler_computation_seam_reads_the_canonical_on_operand_not_clause_or_done_bodies() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler h(comp: () -> {{ TestClock::sleep }} Int) -> Int {{\
           on comp {{ TestClock::sleep(ms, resume) => ms, done(value) => value }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));
    let checked = infer_checked_handler_computation_for_test(&program, "h")
        .expect("canonical on operand has computation evidence");
    let operand = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Handler(handler) if handler.name.as_ref() == "h" => match &handler.body {
                Expr::On {
                    computation,
                    clauses,
                    ..
                } => {
                    assert!(matches!(
                        clauses.as_slice(),
                        [HandlerClause::Operation { .. }, HandlerClause::Done { .. }]
                    ));
                    Some(computation.as_ref())
                }
                _ => None,
            },
            _ => None,
        })
        .expect("fixture must retain canonical on operand");
    assert_eq!(checked.expression_anchor(), operand.span());
}

#[test]
fn task_2013_direct_on_operation_matches_annotation_semantically_and_retains_both_facts() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler h(comp: () -> {{ TestClock::sleep }} Int) -> Int {{\
           on TestClock::sleep(1) {{ TestClock::sleep(ms, resume) => ms, done(value) => value }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));
    let operand = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Handler(handler) if handler.name.as_ref() == "h" => match &handler.body {
                Expr::On { computation, .. } => Some(computation.as_ref()),
                _ => None,
            },
            _ => None,
        })
        .expect("fixture must retain the canonical direct operation operand");

    let checked = infer_checked_handler_computation_for_test(&program, "h").expect(
        "matching normalized operation identities must not be rejected merely because the annotation and inferred call have distinct provenance",
    );
    assert_eq!(checked.result_type(), &Type::Int);
    assert_eq!(keys(&checked), ["operation:TestClock::Clock::sleep"]);
    assert_eq!(checked.expression_anchor(), operand.span());
    let facts = checked.normalized_row().items[0].source_provenance();
    assert_eq!(
        facts.len(),
        2,
        "the checked row must retain the annotation and direct-call provenance independently"
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.source_span() == operand.span()),
        "the inferred direct call must retain its source anchor"
    );

    let mismatch = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler h(comp: () -> {{ resource read filesystem }} Int) -> Int {{\
           on TestClock::sleep(1) {{ TestClock::sleep(ms, resume) => ms, done(value) => value }}\
         }}\
         fn main() -> Int {{ 0 }}"
    ));
    let error = infer_checked_handler_computation_for_test(&mismatch, "h")
        .expect_err("a semantically distinct annotated row must remain rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported-handler-computation-expression")
    );
}
