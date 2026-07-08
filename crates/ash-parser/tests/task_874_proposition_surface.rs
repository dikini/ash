use ash_parser::surface::{Definition, PropositionClause, PropositionClauseKind, Type, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn parse_err(source: &str) {
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "source should be rejected: {source}"
    );
}

fn assert_nonempty_span(span: ash_parser::token::Span) {
    assert!(
        span.end > span.start,
        "expected non-empty span, got {span:?}"
    );
}

fn assert_type_name(ty: &Type, expected: &str) {
    assert!(
        matches!(ty, Type::Name(name) if name.as_ref() == expected),
        "expected type name {expected}, got {ty:?}"
    );
}

#[test]
fn task_874_parses_type_fn_proposition_tail_with_all_clause_kinds_and_spans() {
    let module = parse(
        r#"type fn Append(xs: TypeList, ys: TypeList) -> TypeList
    decreases xs
    where Append<Nil, ys> == ys, Cons<A, T> != Nil, T: Iterator, NonEmpty<xs>
{
    case Append<Nil, ys> = ys;
}"#,
    );

    let Definition::TypeFn(type_fn) = &module.definitions[0] else {
        panic!("expected type fn definition");
    };
    let tail = type_fn
        .proposition_tail
        .as_ref()
        .expect("type fn proposition tail should be preserved");
    assert_nonempty_span(tail.where_span);
    assert_nonempty_span(tail.span);
    assert_eq!(tail.clauses.len(), 4);

    assert!(matches!(
        &tail.clauses[0],
        PropositionClause {
            kind: PropositionClauseKind::Equality { lhs, rhs, op_span },
            span,
        } if matches!(lhs, Type::Constructor { name, args } if name.as_ref() == "Append" && args.len() == 2)
            && matches!(rhs, Type::Name(name) if name.as_ref() == "ys")
            && op_span.end > op_span.start
            && span.end > span.start
    ));

    assert!(matches!(
        &tail.clauses[1],
        PropositionClause {
            kind: PropositionClauseKind::Disequality { lhs, rhs, op_span },
            span,
        } if matches!(lhs, Type::Constructor { name, args } if name.as_ref() == "Cons" && args.len() == 2)
            && matches!(rhs, Type::Name(name) if name.as_ref() == "Nil")
            && op_span.end > op_span.start
            && span.end > span.start
    ));

    assert!(matches!(
        &tail.clauses[2],
        PropositionClause {
            kind: PropositionClauseKind::InterfaceBound { subject, interface, colon_span },
            span,
        } if matches!(subject, Type::Name(name) if name.as_ref() == "T")
            && matches!(interface, Type::Name(name) if name.as_ref() == "Iterator")
            && colon_span.end > colon_span.start
            && span.end > span.start
    ));

    assert!(matches!(
        &tail.clauses[3],
        PropositionClause {
            kind: PropositionClauseKind::NamedPredicate { name, name_span, args },
            span,
        } if name.as_ref() == "NonEmpty"
            && name_span.end > name_span.start
            && args.len() == 1
            && matches!(&args[0], Type::Name(arg) if arg.as_ref() == "xs")
            && span.end > span.start
    ));
}

#[test]
fn task_874_parses_multi_argument_interface_bound_proposition_tail() {
    let module = parse(r#"fn checked<T>(x: T) -> T where T: Serializable<Json, Utf8> { x }"#);

    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected fn definition");
    };
    let tail = function
        .proposition_tail
        .as_ref()
        .expect("fn proposition tail should be preserved");
    assert_eq!(tail.clauses.len(), 1);
    let PropositionClauseKind::InterfaceBound {
        subject,
        interface,
        colon_span,
    } = &tail.clauses[0].kind
    else {
        panic!("expected interface-bound proposition");
    };
    assert_type_name(subject, "T");
    assert!(colon_span.end > colon_span.start);
    let Type::Constructor { name, args } = interface else {
        panic!("expected multi-argument interface application, got {interface:?}");
    };
    assert_eq!(name.as_ref(), "Serializable");
    assert_eq!(args.len(), 2);
    assert_type_name(&args[0], "Json");
    assert_type_name(&args[1], "Utf8");
}

#[test]
fn task_874_parses_fn_proposition_tail_before_runtime_contracts() {
    let module =
        parse(r#"fn checked_id<T>(x: T) -> T where T: Debug, Same<T, T> requires: x != 0 { x }"#);

    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected fn definition");
    };
    assert!(
        function.contract.is_some(),
        "runtime contract should remain separate"
    );
    let tail = function
        .proposition_tail
        .as_ref()
        .expect("fn proposition tail should be preserved");
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
            if name.as_ref() == "Same" && args.len() == 2
    ));
}

#[test]
fn task_874_parses_builtin_fn_proposition_tail() {
    let module = parse("builtin fn make_box<T>(value: T) -> Box<T> where Box<T> != Nil;");

    let Definition::BuiltinFn(function) = &module.definitions[0] else {
        panic!("expected builtin fn definition");
    };
    let tail = function
        .proposition_tail
        .as_ref()
        .expect("builtin fn proposition tail should be preserved");
    assert_eq!(tail.clauses.len(), 1);
    assert!(matches!(
        &tail.clauses[0].kind,
        PropositionClauseKind::Disequality { lhs, rhs, .. }
            if matches!(lhs, Type::Constructor { name, args } if name.as_ref() == "Box" && args.len() == 1)
                && matches!(rhs, Type::Name(name) if name.as_ref() == "Nil")
    ));
}

#[test]
fn task_874_parses_explicit_named_predicate_declarations() {
    let module = parse("pub prop NonEmpty<Xs: TypeList, Witness: Type>;\nprop Closed;");

    let Definition::PropositionPredicate(non_empty) = &module.definitions[0] else {
        panic!("expected proposition predicate declaration");
    };
    assert_eq!(non_empty.visibility, Visibility::Public);
    assert_eq!(non_empty.name.as_ref(), "NonEmpty");
    assert_eq!(non_empty.params.len(), 2);
    assert_eq!(non_empty.params[0].name.as_ref(), "Xs");
    assert_type_name(&non_empty.params[0].domain, "TypeList");
    assert_nonempty_span(non_empty.params[0].span);
    assert_eq!(non_empty.params[1].name.as_ref(), "Witness");
    assert_type_name(&non_empty.params[1].domain, "Type");
    assert_nonempty_span(non_empty.span);

    let Definition::PropositionPredicate(closed) = &module.definitions[1] else {
        panic!("expected zero-parameter proposition predicate declaration");
    };
    assert_eq!(closed.visibility, Visibility::Inherited);
    assert_eq!(closed.name.as_ref(), "Closed");
    assert!(closed.params.is_empty());
}

#[test]
fn task_874_preserves_impl_where_bounds_without_generalizing_them() {
    let module = parse("impl<T> Explain<T> where T: Debug { explain(value) = value }");

    let Definition::Impl(implementation) = &module.definitions[0] else {
        panic!("expected impl definition");
    };
    assert_eq!(implementation.where_bounds.len(), 1);
    assert_eq!(implementation.where_bounds[0].param.as_ref(), "T");
    assert_eq!(implementation.where_bounds[0].bound.as_ref(), "Debug");
    assert_nonempty_span(implementation.where_bounds[0].span);
}

#[test]
fn task_874_rejects_proposition_clauses_on_unsupported_or_malformed_surfaces() {
    parse_err("type Alias = Int where T == U;");
    parse_err("impl<T> Explain<T> where T == U { explain(value) = value }");
    parse_err("where T == U;");
    parse_err("T == U;");
    parse_err("T != U;");
    parse_err("T: Debug;");
    parse_err("NonEmpty<T>;");
    parse_err("Closed;");
    parse_err("mod nested { where T == U; }");
    parse_err("mod nested { T == U; }");
    parse_err("mod nested { T: Debug; }");
    parse_err("mod nested { NonEmpty<T>; }");
    parse_err("mod nested { Closed; }");
    parse_err("prop NonEmpty<Xs>;");
    parse_err("fn bad<T>(x: T) -> T where T == { x }");
    parse_err("builtin fn bad<T>(x: T) -> T where ;");
}
