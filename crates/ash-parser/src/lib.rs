//! Ash Parser
//!
//! This crate provides the lexer and parser for the Ash workflow language.

pub mod combinators;
pub mod desugar;
pub mod error;
pub mod error_recovery;
pub mod import_resolver;
pub mod input;
pub mod lexer;
pub mod lower;
pub mod module;
pub mod parse_expr;
pub mod parse_module;
pub mod parse_observe;
pub mod parse_pattern;
pub mod parse_policy;
pub mod parse_receive;
pub mod parse_send;
pub mod parse_set;
pub mod parse_type_def;
pub mod parse_use;
pub mod parse_utils;
pub mod parse_visibility;
pub mod parse_workflow;
pub mod resolver;
pub mod surface;
pub mod token;
pub mod use_tree;

pub use combinators::*;
pub use desugar::*;
pub use error::*;
pub use error_recovery::*;
pub use import_resolver::{Binding, BindingKind, ImportError, ImportResolver};
pub use input::*;
pub use lexer::*;
pub use lower::*;
pub use module::*;
pub use parse_expr::*;
pub use parse_module::*;
pub use parse_observe::*;
pub use parse_policy::*;
pub use parse_receive::*;
pub use parse_send::*;
pub use parse_set::*;
pub use parse_use::*;
// parse_utils is intentionally not exported - it's for internal use only
pub use parse_visibility::*;
pub use parse_workflow::*;
pub use resolver::{Fs, ModuleResolver, ResolveError};
pub use surface::*;
pub use token::*;
pub use use_tree::*;

#[cfg(test)]
mod lib_tests {
    // Integration tests for the parser modules

    use super::*;

    #[test]
    fn test_modules_are_public() {
        // Verify all modules are accessible
        let _ = new_input("test");
        let span = Span::new(0, 1, 1, 1);
        let _ = ParseError::new(span, "test error");
    }

    #[test]
    fn test_winnow_integration() {
        use winnow::prelude::*;
        use winnow::token::take_while;

        // Test that winnow parsers work with ParseInput
        let mut input = new_input("hello world");
        let result: ModalResult<&str> =
            take_while(1.., |c: char| c.is_ascii_alphabetic()).parse_next(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_end_to_end_basic() {
        // Basic end-to-end test demonstrating parser components working together
        let input_str = "test input";
        let input = new_input(input_str);

        // Verify input tracking
        assert_eq!(input.state.offset, 0);
        assert_eq!(input.state.line, 1);
        assert_eq!(input.state.column, 1);

        // Create a span
        let span = Span::new(0, 4, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 4);

        // Create an error
        let error = ParseError::new(span, "test message").with_expected("something else");
        assert_eq!(error.message, "test message");
        assert_eq!(error.expected.len(), 1);
    }

    #[test]
    fn test_module_decl_lowers_inline_module_roles_after_parse() {
        use ash_core::RoleObligationRef;
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide(); capability review: analyze(); role reviewer { capabilities: [approve, review], obligations: [check_tests] } }",
        );

        let decl = parse_module_decl.parse_next(&mut input).unwrap();
        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower role authority metadata");

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "reviewer");
        assert_eq!(roles[0].authority.len(), 2);
        assert!(matches!(
            &roles[0].obligations[..],
            [RoleObligationRef { name }] if name == "check_tests"
        ));
    }

    #[test]
    fn test_parse_module_decl_rejects_malformed_inline_module_role_definition() {
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { role reviewer { capabilities: [approve], obligations: [check_tests, } }",
        );

        let result = parse_module_decl.parse_next(&mut input);

        assert!(result.is_err());
    }

    #[test]
    fn test_module_decl_preserves_same_module_capability_metadata_for_role_authority() {
        use ash_core::{Capability, Constraint, Effect, RoleObligationRef};
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() where requires_mfa(); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        let decl = parse_module_decl.parse_next(&mut input).unwrap();
        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower role authority metadata");

        assert_eq!(roles.len(), 1);
        assert!(matches!(
            &roles[0].authority[..],
            [Capability {
                name,
                effect: Effect::Evaluative,
                constraints,
            }] if name == "approve"
                && matches!(
                    &constraints[..],
                    [Constraint {
                        predicate: ash_core::Predicate { name: predicate_name, arguments }
                    }] if predicate_name == "requires_mfa" && arguments.is_empty()
                )
        ));
        assert!(matches!(
            &roles[0].obligations[..],
            [RoleObligationRef { name }] if name == "check_tests"
        ));
    }

    #[test]
    fn test_module_decl_preserves_same_module_capability_constraint_arguments_for_role_authority() {
        use ash_core::{Capability, Constraint, Effect, Expr, RoleObligationRef, Value};
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() where requires_region(\"EU\"); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        let decl = parse_module_decl.parse_next(&mut input).unwrap();
        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower role authority metadata");

        assert_eq!(roles.len(), 1);
        assert!(matches!(
            &roles[0].authority[..],
            [Capability {
                name,
                effect: Effect::Evaluative,
                constraints,
            }] if name == "approve"
                && matches!(
                    &constraints[..],
                    [Constraint {
                        predicate: ash_core::Predicate { name: predicate_name, arguments }
                    }] if predicate_name == "requires_region"
                        && matches!(&arguments[..], [Expr::Literal(Value::String(region))] if region == "EU")
                )
        ));
        assert!(matches!(
            &roles[0].obligations[..],
            [RoleObligationRef { name }] if name == "check_tests"
        ));
    }

    #[test]
    fn test_module_decl_preserves_constraint_arguments_in_role_authority_metadata() {
        use ash_core::{Capability, Constraint, Effect};
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() returns Bool where requires_region(\"EU\"); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        let decl = parse_module_decl.parse_next(&mut input).unwrap();
        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower authority metadata");

        assert_eq!(roles.len(), 1);
        assert!(matches!(
            &roles[0].authority[..],
            [Capability {
                name,
                effect: Effect::Evaluative,
                constraints,
            }] if name == "approve"
                && matches!(
                    &constraints[..],
                    [Constraint {
                        predicate: ash_core::Predicate { name: predicate_name, arguments }
                    }] if predicate_name == "requires_region"
                        && matches!(
                            &arguments[..],
                            [ash_core::Expr::Literal(ash_core::Value::String(region))] if region == "EU"
                        )
                )
        ));
    }
}
