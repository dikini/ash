//! Module declaration parser for the Ash language.
//!
//! This module provides parsers for module declarations, supporting both
//! file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).

use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::stream::Stream;

use crate::combinators::keyword;
use crate::input::{ParseInput, update_position};
use crate::module::{ModuleDecl, ModuleSource};
use crate::parse_expr::expr;
use crate::parse_visibility::parse_visibility;
use crate::surface::{
    CapabilityDef, Constraint, Definition, EffectType, Expr, Param, Predicate, RoleDef, Type,
};

/// Parse a module declaration.
///
/// Supports both file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).
/// Visibility modifiers are optional.
///
/// # Examples
///
/// ```
/// use ash_parser::parse_module::parse_module_decl;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// // Parse file-based module
/// let mut input = new_input("mod foo;");
/// let result = parse_module_decl.parse_next(&mut input).unwrap();
/// assert!(result.is_file_based());
/// ```
pub fn parse_module_decl(input: &mut ParseInput) -> ModalResult<ModuleDecl> {
    // Parse optional visibility modifier
    skip_whitespace(input);
    let start_pos = input.state;
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    // Parse "mod" keyword
    let _ = keyword("mod").parse_next(input)?;
    skip_whitespace(input);

    // Parse module name
    let name = identifier(input)?;
    skip_whitespace(input);

    // Determine if this is file-based (`;`) or inline (`{ ... }`)
    let source = if literal_str(";").parse_next(input).is_ok() {
        ModuleSource::File
    } else {
        // Inline module: parse definitions inside `{ ... }`
        let definitions =
            delimited(literal_str("{"), parse_definitions, literal_str("}")).parse_next(input)?;
        ModuleSource::Inline(definitions)
    };

    let span = crate::input::span_from(&start_pos, &input.state);

    Ok(ModuleDecl {
        name: name.into(),
        visibility,
        source,
        span,
    })
}

/// Parse an identifier.
fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    crate::parse_expr::identifier(input)
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace(input);
        if input.input.starts_with(s) {
            // Update position state
            for c in s.chars() {
                input.state.advance(c);
            }
            // Advance the inner stream
            let _ = input.input.next_slice(s.len());
            Ok(s)
        } else {
            Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ))
        }
    }
}

/// Parse definitions inside an inline module.
fn parse_definitions(input: &mut ParseInput) -> ModalResult<Vec<Definition>> {
    let mut definitions = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        // Check for closing brace or EOF
        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        if starts_with_keyword(input, "role") {
            definitions.push(parse_role_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "capability") {
            definitions.push(parse_capability_definition(input)?);
            continue;
        }

        if starts_with_unsupported_inline_definition(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        skip_unknown_definition(input);

        if input.input.starts_with(";") {
            let _ = input.input.next_slice(1);
            input.state.advance(';');
        }
    }

    Ok(definitions)
}

fn parse_capability_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state;

    let _ = keyword("capability").parse_next(input)?;
    skip_whitespace(input);
    let name = identifier(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);
    let effect = parse_effect_type(input)?;
    skip_whitespace(input);
    let _ = literal_str("(").parse_next(input)?;
    let params = parse_parameter_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let return_type = parse_optional_return_type(input)?;
    skip_whitespace_and_comments(input);

    let constraints = if starts_with_keyword(input, "where") {
        parse_constraint_list(input)?
    } else {
        Vec::new()
    };

    Ok(Definition::Capability(CapabilityDef {
        name: name.into(),
        effect,
        params,
        return_type,
        constraints,
        span: crate::input::span_from(&start_pos, &input.state),
    }))
}

fn parse_role_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state;

    let _ = keyword("role").parse_next(input)?;
    skip_whitespace(input);
    let name = identifier(input)?;
    skip_whitespace(input);
    let _ = literal_str("{").parse_next(input)?;

    skip_whitespace_and_comments(input);
    let authority = parse_authority_clause(input)?;

    skip_whitespace_and_comments(input);
    consume_optional_comma(input);
    skip_whitespace_and_comments(input);
    let obligations = if starts_with_keyword(input, "obligations") {
        let obligations = parse_obligations_clause(input)?;
        skip_whitespace_and_comments(input);
        consume_optional_comma(input);
        obligations
    } else {
        Vec::new()
    };

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::Role(RoleDef {
        name: name.into(),
        authority,
        obligations,
        span: crate::input::span_from(&start_pos, &input.state),
    }))
}

fn parse_authority_clause(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let _ = keyword("authority").parse_next(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);
    parse_name_list(input)
}

fn parse_effect_type(input: &mut ParseInput) -> ModalResult<EffectType> {
    if keyword("observe").parse_next(input).is_ok() {
        Ok(EffectType::Observe)
    } else if keyword("read").parse_next(input).is_ok() {
        Ok(EffectType::Read)
    } else if keyword("analyze").parse_next(input).is_ok() {
        Ok(EffectType::Analyze)
    } else if keyword("decide").parse_next(input).is_ok() {
        Ok(EffectType::Decide)
    } else if keyword("act").parse_next(input).is_ok() {
        Ok(EffectType::Act)
    } else if keyword("write").parse_next(input).is_ok() {
        Ok(EffectType::Write)
    } else if keyword("external").parse_next(input).is_ok() {
        Ok(EffectType::External)
    } else if keyword("epistemic").parse_next(input).is_ok() {
        Ok(EffectType::Epistemic)
    } else if keyword("deliberative").parse_next(input).is_ok() {
        Ok(EffectType::Deliberative)
    } else if keyword("evaluative").parse_next(input).is_ok() {
        Ok(EffectType::Evaluative)
    } else if keyword("operational").parse_next(input).is_ok() {
        Ok(EffectType::Operational)
    } else {
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

fn parse_parameter_list(input: &mut ParseInput) -> ModalResult<Vec<Param>> {
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();

    if input.input.starts_with(")") {
        return Ok(params);
    }

    loop {
        let name = identifier(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ty = parse_surface_type(input)?;

        params.push(Param {
            name: name.into(),
            ty,
        });

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(params)
}

fn parse_constraint_list(input: &mut ParseInput) -> ModalResult<Vec<Constraint>> {
    let _ = keyword("where").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut constraints = Vec::new();

    loop {
        constraints.push(parse_constraint(input)?);

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(constraints)
}

fn parse_constraint(input: &mut ParseInput) -> ModalResult<Constraint> {
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let args = delimited(
        literal_str("("),
        parse_constraint_arguments,
        literal_str(")"),
    )
    .parse_next(input)?;

    Ok(Constraint {
        predicate: Predicate {
            name: name.into(),
            args,
        },
    })
}

fn parse_constraint_arguments(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let mut args = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with(")") {
            break;
        }

        args.push(expr(input)?);

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(args)
}

fn parse_optional_return_type(input: &mut ParseInput) -> ModalResult<Option<Type>> {
    if !starts_with_keyword(input, "returns") {
        return Ok(None);
    }

    let _ = keyword("returns").parse_next(input)?;
    skip_whitespace_and_comments(input);

    parse_surface_type(input).map(Some)
}

fn parse_surface_type(input: &mut ParseInput) -> ModalResult<Type> {
    skip_whitespace_and_comments(input);

    if starts_with_keyword(input, "capability") {
        let _ = keyword("capability").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name = identifier(input)?;
        return Ok(Type::Capability(name.into()));
    }

    if input.input.starts_with("[") {
        let _ = literal_str("[").parse_next(input)?;
        let inner = parse_surface_type(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("]").parse_next(input)?;
        return Ok(Type::List(Box::new(inner)));
    }

    if input.input.starts_with("{") {
        let _ = literal_str("{").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let mut fields = Vec::new();

        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            return Ok(Type::Record(fields));
        }

        loop {
            let field_name = identifier(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let field_type = parse_surface_type(input)?;
            fields.push((field_name.into(), field_type));

            if consume_comma_separator(input) {
                continue;
            }

            break;
        }

        let _ = literal_str("}").parse_next(input)?;
        return Ok(Type::Record(fields));
    }

    let name = identifier(input)?;
    Ok(Type::Name(name.into()))
}

fn parse_obligations_clause(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let _ = keyword("obligations").parse_next(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);

    parse_name_list(input)
}

fn parse_name_list(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let _ = literal_str("[").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut names = Vec::new();

    if input.input.starts_with("]") {
        let _ = literal_str("]").parse_next(input)?;
        return Ok(names);
    }

    loop {
        let name = identifier(input)?;
        names.push(name.into());

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    let _ = literal_str("]").parse_next(input)?;
    Ok(names)
}

fn starts_with_keyword(input: &ParseInput, word: &str) -> bool {
    if !input.input.starts_with(word) {
        return false;
    }

    let after = &input.input[word.len()..];
    after
        .chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
}

fn starts_with_unsupported_inline_definition(input: &ParseInput) -> bool {
    [
        "pub", "workflow", "policy", "type", "datatype", "memory", "mod",
    ]
    .into_iter()
    .any(|keyword| starts_with_keyword(input, keyword))
}

fn consume_optional_comma(input: &mut ParseInput) {
    if input.input.starts_with(",") {
        let _ = input.input.next_slice(1);
        input.state.advance(',');
    }
}

fn consume_comma_separator(input: &mut ParseInput) -> bool {
    skip_whitespace_and_comments(input);

    if !input.input.starts_with(",") {
        return false;
    }

    let _ = input.input.next_slice(1);
    input.state.advance(',');
    skip_whitespace_and_comments(input);
    true
}

fn skip_unknown_definition(input: &mut ParseInput) {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut consumed_any = false;

    while !input.input.is_empty() {
        if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && consumed_any {
            skip_whitespace_and_comments(input);
            if starts_with_keyword(input, "role")
                || starts_with_keyword(input, "capability")
                || starts_with_unsupported_inline_definition(input)
            {
                break;
            }
        }

        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && (input.input.starts_with(";") || input.input.starts_with("}"))
        {
            break;
        }

        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
        consumed_any = true;

        match c {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }
}

/// Skip whitespace (simple version for use in this module).
fn skip_whitespace(input: &mut ParseInput) {
    while input
        .input
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
    }
}

fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        skip_whitespace(input);

        // Check for line comment
        if input.input.starts_with("--") {
            while let Some(c) = input.input.next_token() {
                input.state.advance(c);
                if c == '\n' {
                    break;
                }
            }
            continue;
        }

        // Check for block comment
        if input.input.starts_with("/*") {
            let _ = input.input.next_slice(2);
            update_position(&mut input.state, "/*");
            let mut depth = 1;
            while depth > 0 && !input.input.is_empty() {
                if input.input.starts_with("/*") {
                    let _ = input.input.next_slice(2);
                    update_position(&mut input.state, "/*");
                    depth += 1;
                } else if input.input.starts_with("*/") {
                    let _ = input.input.next_slice(2);
                    update_position(&mut input.state, "*/");
                    depth -= 1;
                } else {
                    let Some(c) = input.input.next_token() else {
                        break;
                    };
                    input.state.advance(c);
                }
            }
            continue;
        }

        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;
    use crate::surface::{
        Constraint, Definition, EffectType, Expr, Literal, Predicate, Visibility,
    };

    /// Test helper to create a ParseInput for testing
    fn test_input(s: &str) -> ParseInput<'_> {
        new_input(s)
    }

    fn inline_module_with_unknown_item(body_after_unknown: &str) -> String {
        format!("mod governance {{ extension custom {{ enabled: true }} {body_after_unknown} }}")
    }

    fn assert_inline_module_rejects_after_unknown_item(
        body_after_unknown: &str,
        item_description: &str,
    ) {
        let source = inline_module_with_unknown_item(body_after_unknown);
        let mut input = test_input(&source);

        let result = parse_module_decl(&mut input);

        match result {
            Err(_) => {}
            Ok(decl) => panic!(
                "Expected parse to fail instead of silently skipping an unsupported {item_description} after unknown-item recovery, but parsed definitions: {:?}",
                decl.definitions()
            ),
        }
    }

    // ========================================================================
    // File-based Module Tests
    // ========================================================================

    #[test]
    fn test_parse_mod_foo_semicolon() {
        // Test: `mod foo;` → file-based module
        let mut input = test_input("mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Inherited);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
        assert!(matches!(decl.source, ModuleSource::File));
    }

    #[test]
    fn test_parse_pub_mod_foo_semicolon() {
        // Test: `pub mod foo;` → public file-based module
        let mut input = test_input("pub mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Public);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
    }

    #[test]
    fn test_parse_pub_crate_mod_foo_semicolon() {
        // Test: `pub(crate) mod foo;` → crate-visible file-based module
        let mut input = test_input("pub(crate) mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Crate);
        assert!(decl.is_file_based());
    }

    // ========================================================================
    // Inline Module Tests
    // ========================================================================

    #[test]
    fn test_parse_inline_module_empty() {
        // Test: `mod foo {}` → empty inline module
        let mut input = test_input("mod foo {}");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Inherited);
        assert!(!decl.is_file_based());
        assert!(decl.is_inline());

        let defs = decl
            .definitions()
            .expect("inline module should have definitions");
        assert!(defs.is_empty());
    }

    #[test]
    fn test_parse_inline_module_with_capability() {
        let mut input =
            test_input("mod foo { capability approve: decide() where requires_mfa(); }");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_inline());

        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.name.as_ref(), "approve");
        assert_eq!(capability.effect, EffectType::Decide);
        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_mfa" && args.is_empty()
        ));
    }

    #[test]
    fn test_parse_inline_module_with_capability_constraint_arguments() {
        let mut input =
            test_input("mod foo { capability approve: decide() where requires_region(\"EU\"); }");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_region"
                && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
        ));
    }

    #[test]
    fn test_parse_inline_module_preserves_capability_signature_metadata() {
        let mut input = test_input(
            "mod foo { capability approve: decide(user: User, scopes: [Scope]) returns Bool where requires_mfa(); }",
        );
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.params.len(), 2);
        assert!(matches!(
            &capability.params[..],
            [
                Param { name: user_name, ty: Type::Name(user_type) },
                Param { name: scopes_name, ty: Type::List(inner) }
            ] if user_name.as_ref() == "user"
                && user_type.as_ref() == "User"
                && scopes_name.as_ref() == "scopes"
                && matches!(inner.as_ref(), Type::Name(scope_type) if scope_type.as_ref() == "Scope")
        ));
        assert!(matches!(
            capability.return_type.as_ref(),
            Some(Type::Name(name)) if name.as_ref() == "Bool"
        ));
    }

    #[test]
    fn test_parse_inline_module_with_capability_returns_and_constraint_arguments() {
        let mut input = test_input(
            "mod foo { capability approve: decide() returns Bool where requires_region(\"EU\"); }",
        );
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_region"
                && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
        ));
    }

    #[test]
    fn test_parse_inline_module_rejects_invalid_constraint_predicate_identifier() {
        let mut input =
            test_input("mod foo { capability approve: decide() where 1requires_mfa(); }");

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail for a non-canonical predicate identifier"
        );
    }

    #[test]
    fn test_parse_inline_module_with_role_definition() {
        let mut input = test_input(
            "mod governance { role reviewer { authority: [approve, review], obligations: [check_tests, audit_log] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Role(role) = &definitions[0] else {
            panic!("expected first definition to be a role: {definitions:?}");
        };

        assert_eq!(role.name.as_ref(), "reviewer");
        assert_eq!(role.authority, vec!["approve".into(), "review".into()]);
        assert_eq!(role.obligations.len(), 2);
        assert_eq!(role.obligations[0].as_ref(), "check_tests");
        assert_eq!(role.obligations[1].as_ref(), "audit_log");
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_role() {
        let mut input = test_input(
            "mod governance { workflow main { done } role reviewer { authority: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail instead of silently skipping unsupported inline workflow items"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_capability_and_role() {
        let mut input = test_input(
            "mod governance { workflow main { done } capability approve: decide() where requires_mfa(); role reviewer { authority: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail instead of silently skipping unsupported inline workflow items"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "workflow main { done } role reviewer { authority: [approve] }",
            "workflow",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_policy_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "policy approval: when true then permit role reviewer { authority: [approve] }",
            "policy",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_datatype_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "datatype review_state = Pending | Approved; role reviewer { authority: [approve] }",
            "datatype",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_visibility_qualified_item_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "pub capability approve: decide() role reviewer { authority: [approve] }",
            "visibility-qualified item",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_canonical_datatype_definition() {
        let mut input = test_input(
            "mod governance { datatype review_state = Pending | Approved; role reviewer { authority: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected inline modules to reject unsupported canonical datatype definitions explicitly"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_visibility_qualified_items_until_supported() {
        let mut input = test_input("mod governance { pub capability approve: decide() }");

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected inline modules to reject visibility-qualified items explicitly until they are supported"
        );
    }

    #[test]
    fn test_parse_pub_inline_module() {
        // Test: `pub mod foo {}` → public inline module
        let mut input = test_input("pub mod foo {}");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Public);
        assert!(decl.is_inline());
    }

    // ========================================================================
    // Whitespace and Formatting Tests
    // ========================================================================

    #[test]
    fn test_parse_mod_with_whitespace() {
        // Test parsing with extra whitespace
        let mut input = test_input("  mod   foo   ;  ");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_file_based());
    }

    #[test]
    fn test_parse_inline_mod_with_whitespace() {
        // Test parsing inline module with extra whitespace
        let mut input = test_input("  mod   foo   {   }  ");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_inline());
    }

    #[test]
    fn test_parse_inline_module_definition_spans_track_comments_and_indentation() {
        let mut input = test_input(
            "mod foo {\n  -- comment before capability\n  capability approve: decide()\n}",
        );

        let decl = parse_module_decl(&mut input).expect("inline module should parse");
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.span.line, 3);
        assert_eq!(capability.span.column, 3);
    }

    // ========================================================================
    // Error Cases
    // ========================================================================

    #[test]
    fn test_parse_mod_missing_semicolon() {
        // Test: `mod foo` without semicolon should fail
        let mut input = test_input("mod foo");
        let result = parse_module_decl(&mut input);

        assert!(result.is_err(), "Expected parse to fail without semicolon");
    }

    #[test]
    fn test_parse_mod_missing_name() {
        // Test: `mod ;` should fail
        let mut input = test_input("mod ;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail without module name"
        );
    }

    #[test]
    fn test_parse_mod_unclosed_brace() {
        // Test: `mod foo {` with unclosed brace should fail
        let mut input = test_input("mod foo {");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail with unclosed brace"
        );
    }

    #[test]
    fn test_parse_inline_module_role_missing_authority_fails() {
        let mut input =
            test_input("mod governance { role reviewer { obligations: [check_tests] } }");

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail when a role definition is missing authority"
        );
    }

    #[test]
    fn test_parse_inline_module_role_with_malformed_obligations_fails() {
        let mut input = test_input(
            "mod governance { role reviewer { authority: [approve], obligations: [check_tests, } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail for malformed role obligations"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_canonical_policy_definition() {
        let mut input = test_input(
            "mod governance { policy approval: when true then permit role reviewer { authority: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected inline modules to reject unsupported canonical policy definitions explicitly"
        );
    }
}
