//! TASK-2001 RED fixtures for the remaining canonical module declarations.
//!
//! These tests intentionally name the surface carriers required by
//! SPEC-095b.  They must stay structural: a successful parse alone is not
//! enough unless the declaration kind, marker-bearing handler signature, and
//! newtype representation survive for later lowering.

use std::path::Path;

use ash_parser::surface::{Definition, Expr, HandlerClause, Type, Visibility};

fn parse_with_origin(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file_with_path(source, Some(Path::new("task-2001.ash")))
        .expect("canonical TASK-2001 declaration should parse")
}

#[test]
fn task_2001_parses_module_level_handler_declaration_with_handler_marker() {
    let module = parse_with_origin("handler identity(comp: Unit) -> Unit { comp }");

    assert_eq!(module.definitions.len(), 1);
    let Definition::Handler(handler) = &module.definitions[0] else {
        panic!("expected a distinct module-level handler declaration");
    };
    assert_eq!(handler.visibility, Visibility::Inherited);
    assert_eq!(handler.name.as_ref(), "identity");
    assert_eq!(handler.params.len(), 1);
    assert!(handler.is_handler_marked);
    assert!(handler.span.end > handler.span.start);
    assert_eq!(handler.source.as_deref(), Some("task-2001.ash"));
}

#[test]
fn task_2001_parses_nominal_newtype_with_constructor_and_representation() {
    let module = parse_with_origin("pub newtype OrderId = OrderId(Int);");

    assert_eq!(module.definitions.len(), 1);
    let Definition::Newtype(newtype) = &module.definitions[0] else {
        panic!("expected a distinct nominal newtype declaration");
    };
    assert_eq!(newtype.visibility, Visibility::Public);
    assert_eq!(newtype.name.as_ref(), "OrderId");
    assert_eq!(newtype.constructor.as_ref(), "OrderId");
    assert!(matches!(
        newtype.representation,
        Type::Name(ref name) if name.as_ref() == "Int"
    ));
    assert!(newtype.span.end > newtype.span.start);
    assert_eq!(newtype.source.as_deref(), Some("task-2001.ash"));
}

#[test]
fn task_2001_rejects_historical_proxy_alongside_canonical_declarations() {
    let error = ash_parser::parse_surface_file("proxy assistant { return 0 }")
        .expect_err("historical proxy definitions must remain rejected");

    assert_eq!(
        error[0].message,
        "`proxy` declarations are removed from target Ash"
    );
}

#[test]
fn task_2001_parses_canonical_co_located_handler_and_derive_intents() {
    let module = parse_with_origin(
        "impl Fs for PosixFs {\
           handler logging_fs(comp: Unit) -> Unit {\
             on comp {\
               PosixFs::read(path, resume) => path,\
               done(value) => value\
             }\
           }\
           derive handler posix_fs;\
         }",
    );

    assert_eq!(module.definitions.len(), 1);
    let Definition::Impl(implementation) = &module.definitions[0] else {
        panic!("canonical co-located members must remain owned by their impl");
    };
    assert_eq!(implementation.handlers.len(), 1);
    assert_eq!(implementation.derived_handlers.len(), 1);
    assert!(implementation.methods.is_empty());

    let handler = &implementation.handlers[0];
    assert!(handler.is_handler_marked);
    assert_eq!(handler.name.as_ref(), "logging_fs");
    assert_eq!(handler.source.as_deref(), Some("task-2001.ash"));
    assert!(matches!(
        &handler.body,
        Expr::On { clauses, .. }
            if matches!(
                clauses.as_slice(),
                [
                    HandlerClause::Operation {
                        impl_type,
                        operation,
                        resume,
                        ..
                    },
                    HandlerClause::Done { binding, .. },
                ]
                    if impl_type.as_ref() == "PosixFs"
                        && operation.as_ref() == "read"
                        && resume.as_ref() == "resume"
                        && binding.as_ref() == "value"
            )
    ));
    assert!(handler.span.end > handler.span.start);
    assert_eq!(implementation.derived_handlers[0].name.as_ref(), "posix_fs");
    assert!(
        implementation.derived_handlers[0].span.end > implementation.derived_handlers[0].span.start
    );
}
