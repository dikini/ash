//! TASK-882: SPEC-H parser surface acceptance and non-interference aggregation.

use ash_parser::parse_surface_file;
use ash_parser::surface::{Definition, PropositionClauseKind, Type};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    parse_surface_file(source).expect("surface module should parse")
}

fn parse_err(source: &str) {
    assert!(
        parse_surface_file(source).is_err(),
        "source should be rejected without broadening SPEC-H surfaces: {source}"
    );
}

#[test]
fn task_882_parser_h3_h7_surface_clauses_are_raw_and_runtime_contracts_stay_separate() {
    let module = parse(
        r#"
prop NonEmpty<Xs: TypeList>;
fn checked<T>(x: T) -> T where T: Debug, NonEmpty<T> requires: x != 0 { x }
"#,
    );

    assert!(matches!(
        &module.definitions[0],
        Definition::PropositionPredicate(predicate)
            if predicate.name.as_ref() == "NonEmpty" && predicate.params.len() == 1
    ));
    let Definition::Function(function) = &module.definitions[1] else {
        panic!("expected function definition with proposition tail");
    };
    assert!(
        function.contract.is_some(),
        "runtime requires/ensures contracts must remain separate from SPEC-H propositions"
    );
    let tail = function
        .proposition_tail
        .as_ref()
        .expect("function proposition tail should parse");
    assert_eq!(tail.clauses.len(), 2);
    assert!(matches!(
        &tail.clauses[0].kind,
        PropositionClauseKind::InterfaceBound { subject, interface, .. }
            if matches!(subject, Type::Name(name) if name.as_ref() == "T")
                && matches!(interface, Type::Name(name) if name.as_ref() == "Debug")
    ));
    assert!(matches!(
        &tail.clauses[1].kind,
        PropositionClauseKind::NamedPredicate { name, args, .. }
            if name.as_ref() == "NonEmpty" && args.len() == 1
    ));
}

#[test]
fn task_882_parser_h12_impl_where_bounds_are_not_generalized_to_propositions() {
    let module = parse("impl<T> Explain<T> where T: Debug { explain(value) = value }");
    let Definition::Impl(implementation) = &module.definitions[0] else {
        panic!("expected impl definition");
    };

    assert_eq!(implementation.where_bounds.len(), 1);
    assert_eq!(implementation.where_bounds[0].param.as_ref(), "T");
    assert_eq!(implementation.where_bounds[0].bound.as_ref(), "Debug");
    assert!(implementation.where_bounds[0].span.end > implementation.where_bounds[0].span.start);
    parse_err("impl<T> Explain<T> where T == U { explain(value) = value }");
}

#[test]
fn task_882_parser_h12_capability_and_workflow_where_syntax_do_not_enter_type_propositions() {
    parse_err("where T == U;");
    parse_err("fn main() { where x {}; }");
    parse_err("type Alias = Int where Int == Int");

    let module = parse(
        r#"
type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
    where Append<Nil, ys> == ys, Cons<A, T> != Nil
{
    case Append<Nil, ys> = ys;
}
"#,
    );
    let Definition::TypeFn(type_fn) = &module.definitions[0] else {
        panic!("expected enabled type fn proposition surface");
    };
    let tail = type_fn
        .proposition_tail
        .as_ref()
        .expect("type fn proposition tail should parse at enabled site only");
    assert_eq!(tail.clauses.len(), 2);
    assert!(matches!(
        &tail.clauses[0].kind,
        PropositionClauseKind::Equality { .. }
    ));
    assert!(matches!(
        &tail.clauses[1].kind,
        PropositionClauseKind::Disequality { .. }
    ));
}
