use ash_parser::surface::{Definition, PropositionClauseKind, Type, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("surface module should parse")
}

fn assert_nonempty_span(span: ash_parser::token::Span) {
    assert!(
        span.end > span.start,
        "expected non-empty span, got {span:?}"
    );
}

#[test]
fn task_878_parser_preserves_named_predicate_registration_metadata() {
    let module = parse("pub prop NonEmpty<Xs: TypeList, Witness: Type>;\nprop Stable;");

    let Definition::PropositionPredicate(non_empty) = &module.definitions[0] else {
        panic!("expected public proposition predicate declaration");
    };
    assert_eq!(non_empty.visibility, Visibility::Public);
    assert_eq!(non_empty.name.as_ref(), "NonEmpty");
    assert_nonempty_span(non_empty.span);
    assert_eq!(non_empty.params.len(), 2);
    assert_eq!(non_empty.params[0].name.as_ref(), "Xs");
    assert!(matches!(
        &non_empty.params[0].domain,
        Type::Name(name) if name.as_ref() == "TypeList"
    ));
    assert_nonempty_span(non_empty.params[0].span);
    assert_eq!(non_empty.params[1].name.as_ref(), "Witness");
    assert!(matches!(
        &non_empty.params[1].domain,
        Type::Name(name) if name.as_ref() == "Type"
    ));
    assert_nonempty_span(non_empty.params[1].span);

    let Definition::PropositionPredicate(stable) = &module.definitions[1] else {
        panic!("expected private zero-parameter proposition predicate declaration");
    };
    assert_eq!(stable.visibility, Visibility::Inherited);
    assert_eq!(stable.name.as_ref(), "Stable");
    assert!(stable.params.is_empty());
    assert_nonempty_span(stable.span);
}

#[test]
fn task_878_parser_preserves_named_predicate_uses_for_typeck_registration() {
    let module = parse(
        r#"prop NonEmpty<Xs: TypeList>;
fn require_non_empty<Xs>(xs: Xs) -> Xs where NonEmpty<Xs> { xs }
builtin fn trusted<T>(value: T) -> T where NonEmpty<T>;"#,
    );

    let Definition::Function(function) = &module.definitions[1] else {
        panic!("expected function with proposition tail");
    };
    let fn_tail = function
        .proposition_tail
        .as_ref()
        .expect("function proposition tail should parse");
    assert_eq!(fn_tail.clauses.len(), 1);
    assert!(matches!(
        &fn_tail.clauses[0].kind,
        PropositionClauseKind::NamedPredicate { name, name_span, args }
            if name.as_ref() == "NonEmpty"
                && name_span.end > name_span.start
                && args.len() == 1
                && matches!(&args[0], Type::Name(arg) if arg.as_ref() == "Xs")
    ));
    assert_nonempty_span(fn_tail.clauses[0].span);

    let Definition::BuiltinFn(function) = &module.definitions[2] else {
        panic!("expected builtin function with proposition tail");
    };
    let builtin_tail = function
        .proposition_tail
        .as_ref()
        .expect("builtin proposition tail should parse");
    assert_eq!(builtin_tail.clauses.len(), 1);
    assert!(matches!(
        &builtin_tail.clauses[0].kind,
        PropositionClauseKind::NamedPredicate { name, args, .. }
            if name.as_ref() == "NonEmpty" && args.len() == 1
    ));
}

#[test]
fn task_878_proposition_guard_does_not_reject_pub_use_paths() {
    parse(
        r#"mod error;
mod supervisor;
pub use error::RuntimeError;
pub use supervisor::{system_supervisor};
prop RuntimeReady<T: Type>;
"#,
    );
}
