//! TASK-2013 RED parser contract for canonical source handlers.
//!
//! These tests deliberately require structural source carriers.  Parsing a
//! handler-looking block into a generic block/call would lose the concrete
//! operation identity and the continuation binder needed by later typing and
//! Core lowering.

use std::path::Path;

use ash_parser::surface::{Definition, Expr, HandlerClause, Literal, Pattern, Spanned};

fn parse_with_origin(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file_with_path(source, Some(Path::new("task-2013.ash")))
        .expect("canonical TASK-2013 handler surface should parse")
}

fn parse_error(source: &str) -> ash_parser::error::ParseError {
    let mut errors = ash_parser::parse_surface_file(source)
        .expect_err("invalid canonical `on` clause cardinality should be rejected by the parser");
    assert_eq!(
        errors.len(),
        1,
        "cardinality rejection should produce one deterministic parser error"
    );
    errors.remove(0)
}

fn on_tail(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let Definition::Function(main) = &module.definitions[0] else {
        panic!("expected the enclosing function");
    };
    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = &main.body
    else {
        panic!("expected the `on` expression as the function tail");
    };
    tail_expr
}

fn assert_span_text(source: &str, span: ash_parser::token::Span, expected: &str) {
    assert_eq!(
        &source[span.start..span.end],
        expected,
        "the AST span must cover the original source carrier"
    );
}

#[test]
fn task_2013_on_accepts_call_computation_with_exact_carrier_and_spans() {
    let source = "fn main() -> Unit { on run(req) { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation, span, ..
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert_span_text(
        source,
        *span,
        "on run(req) { TestClock::sleep(ms, resume) => null, done(value) => value, }",
    );
    let Expr::Call {
        func,
        module,
        args,
        span: computation_span,
    } = computation.as_ref()
    else {
        panic!("expected the `on` computation to retain its call carrier");
    };
    assert_eq!(func.as_ref(), "run");
    assert!(module.is_none());
    assert!(matches!(args.as_slice(), [Expr::Variable { name, .. }] if name.as_ref() == "req"));
    assert_span_text(source, *computation_span, "run(req)");
}

#[test]
fn task_2013_on_computation_span_excludes_same_line_slash_comment_before_clauses() {
    let source = "fn main() -> Unit { on run(req) // note\n { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation,
        clauses,
        ..
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert!(matches!(
        clauses.as_slice(),
        [HandlerClause::Operation { .. }, HandlerClause::Done { .. }]
    ));
    assert_span_text(source, computation.span(), "run(req)");
}

#[test]
fn task_2013_on_computation_span_excludes_same_line_dash_comment_before_clauses() {
    let source = "fn main() -> Unit { on run(req) -- note\n { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation,
        clauses,
        ..
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert!(matches!(
        clauses.as_slice(),
        [HandlerClause::Operation { .. }, HandlerClause::Done { .. }]
    ));
    assert_span_text(source, computation.span(), "run(req)");
}

#[test]
fn task_2013_on_computation_span_keeps_an_internal_line_comment() {
    let source = "fn main() -> Unit { on retries // note\n + 1 { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On { computation, .. } = on_tail(&module) else {
        panic!("expected canonical `on` expression");
    };

    assert!(matches!(computation.as_ref(), Expr::Binary { .. }));
    assert_span_text(source, computation.span(), "retries // note\n + 1");
}

#[test]
fn task_2013_on_computation_span_excludes_nested_block_comment_before_clauses() {
    let source = "fn main() -> Unit { on run(req) /* outer /* inner */ note */ { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On { computation, .. } = on_tail(&module) else {
        panic!("expected canonical `on` expression");
    };

    assert!(matches!(computation.as_ref(), Expr::Call { .. }));
    assert_span_text(source, computation.span(), "run(req)");
}

#[test]
fn task_2013_on_accepts_binary_computation_with_operand_spans() {
    let source = "fn main() -> Unit { on retries + 1 { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation, span, ..
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert_span_text(
        source,
        *span,
        "on retries + 1 { TestClock::sleep(ms, resume) => null, done(value) => value, }",
    );
    let Expr::Binary {
        left,
        right,
        span: computation_span,
        ..
    } = computation.as_ref()
    else {
        panic!("expected the `on` computation to retain its binary carrier");
    };
    assert!(matches!(left.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "retries"));
    assert!(matches!(right.as_ref(), Expr::Literal(Literal::Int(1))));
    assert_span_text(source, *computation_span, "retries + 1");
    assert_span_text(source, left.span(), "retries");
}

#[test]
fn task_2013_on_accepts_structural_record_computation_with_exact_span() {
    let source = "fn main() -> Unit { on { request: run(req) } { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation, span, ..
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert_span_text(
        source,
        *span,
        "on { request: run(req) } { TestClock::sleep(ms, resume) => null, done(value) => value, }",
    );
    let Expr::Record {
        fields,
        span: computation_span,
    } = computation.as_ref()
    else {
        panic!("expected the `on` computation to retain its structural record carrier");
    };
    assert!(
        matches!(fields.as_slice(), [(name, Expr::Call { func, .. })] if name.as_ref() == "request" && func.as_ref() == "run")
    );
    assert_span_text(source, *computation_span, "{ request: run(req) }");
}

#[test]
fn task_2013_on_keeps_named_record_as_computation_before_clause_delimiter() {
    let source = "fn main() -> Unit { on Result { value: run(req) } { TestClock::sleep(ms, resume) => null, done(value) => value, } }";
    let module = parse_with_origin(source);
    let Expr::On {
        computation,
        clauses,
        span,
    } = on_tail(&module)
    else {
        panic!("expected canonical `on` expression");
    };

    assert_span_text(
        source,
        *span,
        "on Result { value: run(req) } { TestClock::sleep(ms, resume) => null, done(value) => value, }",
    );
    let Expr::Constructor {
        name,
        fields,
        payload: ash_parser::surface::ConstructorPayload::Record(_),
        span: computation_span,
    } = computation.as_ref()
    else {
        panic!("expected the first brace to remain a named record computation");
    };
    assert_eq!(name.as_ref(), "Result");
    assert!(
        matches!(fields.as_slice(), [(field, Expr::Call { func, .. })] if field.as_ref() == "value" && func.as_ref() == "run")
    );
    assert_span_text(source, *computation_span, "Result { value: run(req) }");
    assert!(matches!(
        clauses.as_slice(),
        [HandlerClause::Operation { .. }, HandlerClause::Done { .. }]
    ));
}

#[test]
fn task_2013_preserves_on_clause_identity_binders_bodies_and_origins() {
    let module = parse_with_origin(
        "handler absorb_sleep(comp: Unit) -> Unit {\n\
             on comp {\n\
                 TestClock::sleep(ms, resume) => null,\n\
                 done(value) => value,\n\
             }\n\
         }",
    );

    assert_eq!(module.path.as_deref(), Some("task-2013.ash"));
    let Definition::Handler(handler) = &module.definitions[0] else {
        panic!("expected a handler-marked declaration");
    };
    assert!(handler.is_handler_marked);
    assert_eq!(handler.source.as_deref(), Some("task-2013.ash"));

    let Expr::On {
        computation,
        clauses,
        span,
    } = &handler.body
    else {
        panic!("expected canonical `on computation {{ ... }}` handler body");
    };
    assert!(matches!(
        computation.as_ref(),
        Expr::Variable { name, .. } if name.as_ref() == "comp"
    ));
    assert_eq!(clauses.len(), 2);
    assert_eq!(
        span.line, 2,
        "the `on` expression must retain its source span"
    );
    assert_eq!(handler.body.span(), *span);

    let HandlerClause::Operation {
        impl_type,
        operation,
        pattern,
        resume,
        body,
        span: operation_span,
    } = &clauses[0]
    else {
        panic!("expected the first clause to retain a concrete operation identity");
    };
    assert_eq!(impl_type.as_ref(), "TestClock");
    assert_eq!(operation.as_ref(), "sleep");
    assert!(matches!(
        pattern,
        Pattern::Variable { name, .. } if name.as_ref() == "ms"
    ));
    assert_eq!(resume.as_ref(), "resume");
    assert!(matches!(body.as_ref(), Expr::Literal(Literal::Null)));
    assert_eq!(operation_span.line, 3);

    let HandlerClause::Done {
        binding,
        body,
        span: done_span,
    } = &clauses[1]
    else {
        panic!("expected the final clause to retain the canonical `done` form");
    };
    assert_eq!(binding.as_ref(), "value");
    assert!(matches!(
        body.as_ref(),
        Expr::Variable { name, .. } if name.as_ref() == "value"
    ));
    assert_eq!(done_span.line, 4);
}

#[test]
fn task_2013_preserves_handle_with_handler_reference_and_span() {
    let module = parse_with_origin(
        "fn main() -> Unit {\n\
             handle TestClock::sleep(0) with absorb_sleep\n\
         }",
    );

    let Definition::Function(main) = &module.definitions[0] else {
        panic!("expected the enclosing ordinary function");
    };
    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = &main.body
    else {
        panic!("expected the function body to retain its tail expression");
    };
    let Expr::HandleWith {
        expression,
        handler,
        handler_span,
        span,
    } = tail_expr.as_ref()
    else {
        panic!("expected canonical `handle expression with handler_name` syntax");
    };

    assert_eq!(handler.as_ref(), "absorb_sleep");
    assert_eq!(
        handler_span.start + handler.as_ref().len(),
        handler_span.end
    );
    assert_eq!(span.line, 2);
    assert!(matches!(
        expression.as_ref(),
        Expr::Call {
            module: Some(module),
            func,
            args,
            ..
        } if module.as_ref() == "TestClock"
            && func.as_ref() == "sleep"
            && matches!(args.as_slice(), [Expr::Literal(Literal::Int(0))])
    ));
}

#[test]
fn task_2013_accepts_one_operation_and_one_done_clause() {
    let module = parse_with_origin(
        "fn main() -> Unit {\n\
             on comp {\n\
                 TestClock::sleep(ms, resume) => null,\n\
                 done(value) => value,\n\
             }\n\
         }",
    );

    let Definition::Function(main) = &module.definitions[0] else {
        panic!("expected the enclosing function");
    };
    let Expr::Block {
        tail_expr: Some(tail_expr),
        ..
    } = &main.body
    else {
        panic!("expected the `on` expression as the function tail");
    };
    let Expr::On { clauses, .. } = tail_expr.as_ref() else {
        panic!("expected canonical `on` expression");
    };
    assert!(matches!(
        clauses.as_slice(),
        [HandlerClause::Operation { .. }, HandlerClause::Done { .. }]
    ));
}

#[test]
fn task_2013_rejects_on_body_with_only_done_clause_at_its_closing_brace() {
    let error = parse_error(
        "fn main() -> Unit {\n\
             on comp {\n\
                 done(value) => value,\n\
             }\n\
         }",
    );

    assert_eq!(
        error.span.line, 4,
        "missing-operation error belongs to `on` closing brace"
    );
    assert_eq!(
        error.message, "parse error: missing concrete operation clause",
        "parser boundary must expose the stable missing-operation subject"
    );
}

#[test]
fn task_2013_rejects_on_body_without_done_clause_at_its_closing_brace() {
    let error = parse_error(
        "fn main() -> Unit {\n\
             on comp {\n\
                 TestClock::sleep(ms, resume) => null,\n\
             }\n\
         }",
    );

    assert_eq!(
        error.span.line, 4,
        "missing-done error belongs to `on` closing brace"
    );
    assert_eq!(
        error.message, "parse error: missing done clause",
        "parser boundary must expose the stable missing-done subject"
    );
}

#[test]
fn task_2013_rejects_second_done_clause_at_the_second_done_keyword() {
    let error = parse_error(
        "fn main() -> Unit {\n\
             on comp {\n\
                 TestClock::sleep(ms, resume) => null,\n\
                 done(first) => first,\n\
                 done(second) => second,\n\
             }\n\
         }",
    );

    assert_eq!(
        error.span.line, 5,
        "duplicate-done rejection must point at the second `done` clause"
    );
    assert_eq!(
        error.message, "parse error: duplicate done clause",
        "parser boundary must expose the stable duplicate-done subject"
    );
}
