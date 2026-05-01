//! Ash Parser
//!
//! This crate provides the lexer and parser for the Ash workflow language.

use winnow::prelude::*;

pub mod capability_export;
pub mod capability_pipeline;
pub mod capability_resolver;
pub mod combinators;
pub mod desugar;
pub mod error;
pub mod error_recovery;
pub mod import_resolver;
pub mod input;
pub mod lexer;
pub mod lift;
pub mod lower;
pub mod module;
pub mod parse_crate_root;
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
pub mod workflow_contract_classifier;

pub use capability_resolver::{CapabilityResolver, CapabilityTarget};
pub use combinators::*;
pub use desugar::*;
pub use error::*;
pub use error_recovery::*;
pub use import_resolver::{Binding, BindingKind, ImportError, ImportResolver};
pub use input::*;
pub use lexer::*;
pub use lift::*;
pub use lower::*;
pub use module::*;
pub use parse_crate_root::*;
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

/// Parse a complete `.ash` source file, returning a `ModuleFile` with a
/// populated `CommentTable`.
pub fn parse_surface_file(source: &str) -> Result<surface::ModuleFile, Vec<error::ParseError>> {
    parse_surface_file_with_path(source, None)
}

/// Parse a complete `.ash` source file with an optional filesystem path.
pub fn parse_surface_file_with_path(
    source: &str,
    path: Option<&std::path::Path>,
) -> Result<surface::ModuleFile, Vec<error::ParseError>> {
    let mut input = input::new_input(source);
    match parse_module::module_file.parse_next(&mut input) {
        Ok(mut module) => {
            // Flush EOF comments as trailing on the last seen token
            if let Some(last) = input.state.comments.last_seen_token_span {
                input.state.comments.flush_pending_leading_to_trailing(last);
            }
            module.comments = input.state.comments;
            module.path = path.map(|p| p.to_string_lossy().into_owned().into());
            if let Some(source) = module.path.clone() {
                attach_type_definition_source(&mut module.definitions, &source);
                for module_decl in &mut module.module_decls {
                    if let module::ModuleSource::Inline(definitions) = &mut module_decl.source {
                        attach_type_definition_source(definitions, &source);
                    }
                }
            }
            Ok(module)
        }
        Err(e) => {
            let span = input::current_span(&input);
            Err(vec![error::ParseError::new(
                span,
                format!("parse error: {e}"),
            )])
        }
    }
}

fn attach_type_definition_source(definitions: &mut [surface::Definition], source: &str) {
    for definition in definitions {
        if let surface::Definition::Type(type_def) = definition {
            type_def.source = Some(source.into());
        }
    }
}

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
        assert_eq!(input.state.pos.offset, 0);
        assert_eq!(input.state.pos.line, 1);
        assert_eq!(input.state.pos.column, 1);

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

    #[test]
    fn test_parse_surface_file_populates_comment_table() {
        let source = r#"
            -- header comment
            capability sensor: epistemic();
            -- trailing comment
        "#;
        let result = parse_surface_file(source);
        assert!(result.is_ok(), "parse_surface_file failed: {:?}", result);
        let module = result.unwrap();
        assert!(
            module.comments.total_count() > 0,
            "expected non-empty CommentTable"
        );
    }

    #[test]
    fn test_parse_surface_file_backtracking_does_not_leak_comments() {
        // Verify that checkpoint/restore rolls back the CommentTable state.
        let mut input = new_input("-- comment\nx");
        let checkpoint = input.clone();
        crate::parse_utils::skip_whitespace_and_comments(&mut input);
        assert_eq!(input.state.comments.total_count(), 1);
        input = checkpoint;
        assert_eq!(input.state.comments.total_count(), 0);
    }

    #[test]
    fn test_variable_expr_span_accuracy() {
        let source = "  my_var  ";
        let mut input = new_input(source);
        let expr = crate::parse_expr::expr(&mut input).unwrap();
        match expr {
            crate::surface::Expr::Variable { name, span } => {
                assert_eq!(name.as_ref(), "my_var");
                assert_eq!(span.start, 2);
                assert_eq!(span.end, 8);
                assert_eq!(span.line, 1);
                assert_eq!(span.column, 3);
            }
            other => panic!("expected Expr::Variable, got {other:?}"),
        }
    }

    #[test]
    fn test_variable_pattern_span_accuracy() {
        let source = "  my_pat  ";
        let mut input = new_input(source);
        let pat = crate::parse_pattern::pattern(&mut input).unwrap();
        match pat {
            crate::surface::Pattern::Variable { name, span } => {
                assert_eq!(name.as_ref(), "my_pat");
                assert_eq!(span.start, 2);
                assert_eq!(span.end, 8);
                assert_eq!(span.line, 1);
                assert_eq!(span.column, 3);
            }
            other => panic!("expected Pattern::Variable, got {other:?}"),
        }
    }

    #[test]
    fn test_policy_var_span_accuracy() {
        let source = "  my_policy  ";
        let mut input = new_input(source);
        let pexpr = crate::parse_policy::policy_expr(&mut input).unwrap();
        match pexpr {
            crate::surface::PolicyExpr::Var { name, span } => {
                assert_eq!(name.as_ref(), "my_policy");
                assert_eq!(span.start, 2);
                assert_eq!(span.end, 11);
                assert_eq!(span.line, 1);
                assert_eq!(span.column, 3);
            }
            other => panic!("expected PolicyExpr::Var, got {other:?}"),
        }
    }
}
