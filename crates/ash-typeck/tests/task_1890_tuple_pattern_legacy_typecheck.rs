//! TASK-1890 regression: source tuple-ADT patterns retain positional payload types.
//!
//! The legacy pattern-checking entry point is still used where the expected
//! scrutinee type is not yet concrete.  Parsed tuple variants deliberately
//! leave their named-field list empty, so that path must read the positional
//! types from `VariantPayload::Tuple` rather than treating the constructor as
//! arity zero.

use ash_core::ast::{TypeBody, VariantPayload};
use ash_parser::surface::{Definition, Expr, Pattern};
use ash_typeck::check_pattern::{TypeEnv, check_pattern};
use ash_typeck::{Type, TypeVar};

const SOURCE: &str = r#"
    type RuntimeError = RuntimeError(Int, String);

    fn main(error: RuntimeError) -> Int {
        match error {
            RuntimeError(code, message) => code,
        }
    }
"#;

fn source_tuple_type_and_pattern() -> (ash_core::ast::TypeDef, Pattern) {
    let module = ash_parser::parse_surface_file(SOURCE)
        .unwrap_or_else(|errors| panic!("TASK-1890 source should parse: {errors:?}"));

    let type_def = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Type(type_def) if type_def.name.as_ref() == "RuntimeError" => {
                Some(ash_parser::lower_surface_type_def(type_def))
            }
            _ => None,
        })
        .expect("fixture must declare RuntimeError");

    let pattern = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                let Expr::Block {
                    tail_expr: Some(tail_expr),
                    ..
                } = &function.body
                else {
                    panic!("main must have a tail expression");
                };
                let Expr::Match { arms, .. } = tail_expr.as_ref() else {
                    panic!("main tail must be a match");
                };
                Some(arms[0].pattern.clone())
            }
            _ => None,
        })
        .expect("fixture must contain the tuple pattern");

    (type_def, pattern)
}

#[test]
fn task_1890_source_tuple_pattern_checks_through_legacy_variant_lookup() {
    let (type_def, pattern) = source_tuple_type_and_pattern();
    let TypeBody::Enum(variants) = &type_def.body else {
        panic!("RuntimeError must lower to an enum");
    };
    assert!(
        variants[0].fields.is_empty(),
        "tuple variants have no named fields"
    );
    assert!(matches!(variants[0].payload, VariantPayload::Tuple(ref items) if items.len() == 2));

    let mut env = TypeEnv::new();
    env.add_type_def("RuntimeError".to_string(), type_def);

    let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh()))
        .expect("RuntimeError(code, message) must use its tuple payload types");

    assert_eq!(bindings.get("code"), Some(&Type::Int));
    assert_eq!(bindings.get("message"), Some(&Type::String));
}
