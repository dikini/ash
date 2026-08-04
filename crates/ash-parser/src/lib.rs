//! Ash Parser
//!
//! This crate provides the lexer and parser for the Ash workflow language.

use winnow::prelude::*;

mod canonical_expanded_module_graph;
pub mod canonical_module_graph;
pub mod capability_export;
pub mod capability_pipeline;
pub mod capability_resolver;
pub mod combinators;
pub mod contract_classifier;
pub mod error;
pub mod error_recovery;
pub mod import_resolver;
pub mod input;
pub mod lexer;
pub mod lower;
pub mod module;
pub mod parse_crate_root;
pub mod parse_expr;
pub mod parse_module;
pub mod parse_pattern;
pub mod parse_policy;
pub mod parse_type_def;
pub mod parse_use;
pub mod parse_utils;
pub mod parse_visibility;
pub mod resolver;
pub mod surface;
pub mod token;
pub mod use_tree;

pub use canonical_expanded_module_graph::{
    CanonicalExpandedModuleGraph, CanonicalExpandedModuleRef, CanonicalModuleExpansionError,
    CanonicalModuleExpansionFailure, CanonicalModuleExpansionInvariantFailure,
};
pub use canonical_module_graph::{
    CanonicalDiagnosticValue, CanonicalModuleGraph, CanonicalModuleGraphError,
    CanonicalModuleGraphResolver, CanonicalModuleState, CanonicalStructuralDiagnostic,
};
pub use capability_resolver::{CapabilityResolver, CapabilityTarget};
pub use combinators::*;
pub use error::*;
pub use import_resolver::{Binding, BindingKind, ImportError, ImportResolver};
pub use input::*;
pub use lexer::*;
pub use lower::*;
pub use module::*;
pub use parse_crate_root::*;
pub use parse_expr::*;
pub use parse_module::*;
pub use parse_policy::*;
pub use parse_use::*;
// parse_utils is intentionally not exported - it's for internal use only
pub use parse_visibility::*;
/// Compatibility-only resolver for callers that still require the legacy graph.
///
/// It cannot feed the canonical graph, interface, binding, lowering, or
/// admission routes. Use [`CanonicalModuleGraphResolver`] for parser-stage
/// structure.
pub use resolver::LegacyModuleResolver;
pub use resolver::{
    DiscoveredModuleDecl, DiscoveredModuleSource, Fs, ModuleUnitResolver, ResolveError,
    discover_module_declarations,
};
/// Deprecated compatibility name for the legacy graph resolver.
///
/// This alias cannot feed the canonical graph, interface, binding, lowering,
/// or admission routes. Use [`LegacyModuleResolver`] only for compatibility
/// callers, or [`CanonicalModuleGraphResolver`] for parser-stage structure.
#[deprecated(
    since = "0.1.0",
    note = "use LegacyModuleResolver for compatibility callers or CanonicalModuleGraphResolver for canonical parser structure"
)]
pub type ModuleResolver = LegacyModuleResolver;
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

    // Crate-root metadata is an outer-file preamble rather than a `ModuleFile`
    // item. Consume it with its grammar before parsing the authoritative module
    // carrier, while preserving the original input state (and therefore source
    // spans) for ordinary module files.
    let checkpoint = input.clone();
    let crate_metadata = match parse_crate_root::parse_crate_root_metadata(&mut input) {
        Ok(metadata) => Some(metadata),
        Err(_) => {
            input = checkpoint;
            None
        }
    };

    match parse_module::module_file.parse_next(&mut input) {
        Ok(mut module) => {
            // Flush EOF comments as trailing on the last seen token
            if let Some(last) = input.state.comments.last_seen_token_span {
                input.state.comments.flush_pending_leading_to_trailing(last);
            }
            module.comments = input.state.comments;
            module.crate_metadata = crate_metadata;
            module.path = path.map(|p| p.to_string_lossy().into_owned().into());
            if let Some(source) = module.path.clone() {
                attach_type_definition_source(&mut module.definitions, &source);
                for module_decl in &mut module.module_decls {
                    attach_inline_module_body_source(module_decl, &source);
                }
            }
            Ok(module)
        }
        Err(e) => {
            let span = input::current_span(&input);
            if let Some(message) = canonical_on_cardinality_diagnostic(&e) {
                return Err(vec![error::ParseError::new(
                    span,
                    format!("parse error: {message}"),
                )]);
            }
            if let Some(error) = reserved_callable_arrow_diagnostic(source) {
                return Err(vec![error]);
            }
            if let Some(form) = removed_declaration_at_span(source, span) {
                return Err(vec![error::ParseError::new(
                    span,
                    format!("`{form}` declarations are removed from target Ash"),
                )]);
            }
            if let Some((surface, help)) = unsupported_proposition_surface_at_span(source, span) {
                return Err(vec![error::ParseError::unsupported_proposition_surface(
                    span, surface, help,
                )]);
            }
            Err(vec![error::ParseError::new(
                span,
                format!("parse error: {e}"),
            )])
        }
    }
}

/// Parse a module body for the source-acquisition route.
///
/// Unlike [`parse_surface_file_with_path`], this retains the authoritative
/// ordered [`module::ModuleBody`] rather than projecting it into legacy
/// `ModuleFile` definition/module-declaration vectors. It remains syntax-only:
/// callers receive parsed `use` declarations but no import bindings.
///
/// This is a child-module route and intentionally does not parse a crate-root
/// metadata preamble. Canonical root acquisition uses
/// [`parse_root_module_body_with_path`] to consume that grammar before parsing
/// the same authoritative ordered body.
pub(crate) fn parse_module_body_with_path(
    source: &str,
    path: &std::path::Path,
) -> Result<(module::ModuleBody, parse_utils::CommentTable), ModuleBodyParseFailure> {
    let mut input = input::new_input(source);
    parse_module_body_from_input(source, path, &mut input)
}

/// Parse a crate root's optional outer metadata and its ordered module body.
///
/// The preamble and body intentionally share one parser input/state so their
/// spans and comment provenance remain owned by the parser. This is private to
/// source acquisition: it does not resolve dependency paths or bind imports.
pub(crate) fn parse_root_module_body_with_path(
    source: &str,
    path: &std::path::Path,
) -> Result<
    (
        module::ModuleBody,
        parse_utils::CommentTable,
        Option<surface::CrateRootMetadata>,
    ),
    ModuleBodyParseFailure,
> {
    let mut input = input::new_input(source);
    let checkpoint = input.clone();
    let crate_metadata = match parse_crate_root::parse_crate_root_metadata(&mut input) {
        Ok(metadata) => Some(metadata),
        Err(_) => {
            input = checkpoint;
            None
        }
    };

    let (body, comments) = parse_module_body_from_input(source, path, &mut input)?;
    Ok((body, comments, crate_metadata))
}

fn parse_module_body_from_input(
    source: &str,
    path: &std::path::Path,
    input: &mut input::ParseInput<'_>,
) -> Result<(module::ModuleBody, parse_utils::CommentTable), ModuleBodyParseFailure> {
    match parse_module::parse_module_body(input, false) {
        Ok(mut body) => {
            if let Some(last) = input.state.comments.last_seen_token_span {
                input.state.comments.flush_pending_leading_to_trailing(last);
            }
            let source_path: Box<str> = path.to_string_lossy().into_owned().into();
            attach_type_definition_source(body.definitions_mut(), &source_path);
            attach_inline_module_body_sources(body.module_decls_mut(), &source_path);
            body.rebuild_item_snapshot();
            Ok((body, std::mem::take(&mut input.state.comments)))
        }
        Err(e) => {
            let errors = module_parse_errors(source, input, e);
            let error_span = errors
                .first()
                .map_or_else(|| input::current_span(input), |error| error.span);
            let malformed_inline = input.state.innermost_inline_module_header().map(|header| {
                MalformedInlineModuleBody {
                    name: header.name.clone(),
                    header_span: header.span,
                    error_span,
                }
            });
            Err(ModuleBodyParseFailure {
                errors,
                malformed_inline,
            })
        }
    }
}

/// A parser-owned malformed inline-module body context.
///
/// This recovery carrier identifies an already consumed `mod <name> {` header
/// and the parser error that prevented a body from becoming a `ModuleDecl`.
/// It is crate-private so only canonical source acquisition can turn it into a
/// structural diagnostic; ordinary parser callers continue to receive their
/// normal [`error::ParseError`] values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MalformedInlineModuleBody {
    /// Parsed child spelling from the incomplete declaration header.
    pub(crate) name: Box<str>,
    /// Parser-owned span covering the completed `mod <name> {` header.
    pub(crate) header_span: token::Span,
    /// Parser-owned span of the body or closing-delimiter error.
    pub(crate) error_span: token::Span,
}

/// The structured failure emitted by the ordered module-body acquisition route.
///
/// Generic parser entry points deliberately flatten this to the same
/// [`error::ParseError`] collection they exposed before. Canonical module
/// acquisition alone consumes [`Self::malformed_inline`] to preserve a
/// declaration anchor without creating a synthetic AST node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleBodyParseFailure {
    errors: Vec<error::ParseError>,
    malformed_inline: Option<MalformedInlineModuleBody>,
}

impl ModuleBodyParseFailure {
    /// Returns the ordinary parse diagnostics for compatibility callers.
    pub(crate) fn errors(&self) -> &[error::ParseError] {
        &self.errors
    }

    /// Returns the innermost incomplete inline-module context, if present.
    pub(crate) fn malformed_inline(&self) -> Option<&MalformedInlineModuleBody> {
        self.malformed_inline.as_ref()
    }
}

fn module_parse_errors(
    source: &str,
    input: &input::ParseInput<'_>,
    error: winnow::error::ErrMode<winnow::error::ContextError>,
) -> Vec<error::ParseError> {
    let span = input::current_span(input);
    if let Some(message) = canonical_on_cardinality_diagnostic(&error) {
        return vec![error::ParseError::new(
            span,
            format!("parse error: {message}"),
        )];
    }
    if let Some(error) = reserved_callable_arrow_diagnostic(source) {
        return vec![error];
    }
    if let Some(form) = removed_declaration_at_span(source, span) {
        return vec![error::ParseError::new(
            span,
            format!("`{form}` declarations are removed from target Ash"),
        )];
    }
    if let Some((surface, help)) = unsupported_proposition_surface_at_span(source, span) {
        return vec![error::ParseError::unsupported_proposition_surface(
            span, surface, help,
        )];
    }
    vec![error::ParseError::new(
        span,
        format!("parse error: {error}"),
    )]
}

/// Convert the parser-internal cardinality markers for canonical `on` bodies
/// into the stable public diagnostic subjects promised by TASK-2013.
fn canonical_on_cardinality_diagnostic(
    error: &winnow::error::ErrMode<winnow::error::ContextError>,
) -> Option<&'static str> {
    let context = match error {
        winnow::error::ErrMode::Backtrack(context) | winnow::error::ErrMode::Cut(context) => {
            context
        }
        winnow::error::ErrMode::Incomplete(_) => return None,
    };
    let label = context.context().find_map(|context| match context {
        winnow::error::StrContext::Label(label) => Some(*label),
        _ => None,
    })?;
    match label {
        "missing concrete operation clause" | "missing done clause" | "duplicate done clause" => {
            Some(label)
        }
        _ => None,
    }
}

fn removed_declaration_at_span(source: &str, span: token::Span) -> Option<&'static str> {
    let line_index = span.line.saturating_sub(1);
    let line = source.lines().nth(line_index)?;
    let declaration = line.split("//").next().unwrap_or(line).trim_start();
    let declaration = if starts_with_declaration_keyword(declaration, "pub") {
        declaration["pub".len()..].trim_start()
    } else {
        declaration
    };

    ["capability", "proxy", "yield"]
        .into_iter()
        .find(|form| starts_with_declaration_keyword(declaration, form))
}

fn starts_with_declaration_keyword(source: &str, keyword: &str) -> bool {
    source.strip_prefix(keyword).is_some_and(|rest| {
        rest.chars().next().is_none_or(|character| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
    })
}

fn unsupported_proposition_surface_at_span(
    source: &str,
    span: token::Span,
) -> Option<(&'static str, &'static str)> {
    let line_index = span.line.saturating_sub(1);
    let line = source.lines().nth(line_index)?;
    let proposition_line = line.split("//").next().unwrap_or(line).trim_start();
    if proposition_line.starts_with("type ")
        && !proposition_line.starts_with("type fn ")
        && proposition_line.contains(" where ")
    {
        return Some((
            "type alias",
            "move the proposition tail to an enabled type fn, fn, or builtin fn declaration",
        ));
    }

    None
}

#[derive(Debug, Clone, Copy)]
enum ReservedCallableArrow {
    DashStar,
    Fat,
    EqualsStar,
}

impl ReservedCallableArrow {
    fn arrow(self) -> &'static str {
        match self {
            Self::DashStar => "-*>",
            Self::Fat => "=>",
            Self::EqualsStar => "=*>",
        }
    }
}

/// Return a targeted parse diagnostic when `source` contains a removed callable arrow in a type or
/// closure context.
///
/// This helper intentionally ignores `=>` in match-arm contexts plus arrows in
/// comments and string literals, so callers that perform pre-parsing source
/// staging can preserve SPEC-072's fail-closed diagnostics without stealing
/// unrelated syntax.
pub fn reserved_callable_arrow_diagnostic(source: &str) -> Option<error::ParseError> {
    find_reserved_callable_arrow(source).map(|(offset, arrow, context)| {
        let span = input::offset_to_span(source, offset, offset + arrow.arrow().len());
        match context {
            ReservedCallableArrowContext::Closure => {
                error::ParseError::new(span, reserved_closure_arrow_message(arrow))
                    .with_expected("pure closure arrow `->`")
            }
            ReservedCallableArrowContext::Type => {
                error::ParseError::new(span, reserved_type_arrow_message(arrow))
                    .with_expected("pure callable arrow `->`")
            }
        }
    })
}

#[derive(Debug, Clone, Copy)]
enum ReservedCallableArrowContext {
    Type,
    Closure,
}

fn find_reserved_callable_arrow(
    source: &str,
) -> Option<(usize, ReservedCallableArrow, ReservedCallableArrowContext)> {
    let mut offset = 0;

    while offset < source.len() {
        if let Some(next_offset) = skip_lexical_region(source, offset) {
            offset = next_offset;
            continue;
        }

        let rest = &source[offset..];
        let arrow = if rest.starts_with("=*>") {
            Some(ReservedCallableArrow::EqualsStar)
        } else if rest.starts_with("-*>") {
            Some(ReservedCallableArrow::DashStar)
        } else if rest.starts_with("=>") {
            Some(ReservedCallableArrow::Fat)
        } else {
            None
        };

        if let Some(arrow) = arrow {
            if is_reserved_closure_arrow_context(source, offset) {
                return Some((offset, arrow, ReservedCallableArrowContext::Closure));
            }
            if is_reserved_type_arrow_context(source, offset) {
                return Some((offset, arrow, ReservedCallableArrowContext::Type));
            }
            offset += arrow.arrow().len();
        } else {
            offset += source[offset..].chars().next().map_or(1, char::len_utf8);
        }
    }

    None
}

fn skip_lexical_region(source: &str, offset: usize) -> Option<usize> {
    let rest = &source[offset..];
    if rest.starts_with("//") {
        return Some(
            source[offset..]
                .find('\n')
                .map_or(source.len(), |newline| offset + newline + 1),
        );
    }
    if rest.starts_with("/*") {
        return Some(
            source[offset + 2..]
                .find("*/")
                .map_or(source.len(), |end| offset + 2 + end + 2),
        );
    }
    if rest.starts_with('"') {
        let mut escaped = false;
        for (relative, ch) in source[offset + 1..].char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                return Some(offset + 1 + relative + ch.len_utf8());
            }
        }
        return Some(source.len());
    }
    None
}

fn is_reserved_closure_arrow_context(source: &str, arrow_offset: usize) -> bool {
    previous_significant_char(source, arrow_offset).is_some_and(|(_, ch)| ch == '|')
}

fn is_reserved_type_arrow_context(source: &str, arrow_offset: usize) -> bool {
    if let Some((close_paren, ')')) = previous_significant_char(source, arrow_offset) {
        let Some(open_paren) = matching_open_paren_before(source, close_paren) else {
            return false;
        };

        return previous_significant_char(source, open_paren).is_some_and(|(idx, ch)| {
            matches!(ch, '=' | ':' | '(' | '[' | '>' | '<')
                || (ch == ',' && is_inside_generic_type_args(source, idx))
        });
    }

    let Some(domain_start) = bare_type_domain_start_before(source, arrow_offset) else {
        return false;
    };

    previous_significant_char(source, domain_start).is_some_and(|(idx, ch)| {
        matches!(ch, '=' | ':' | '(' | '[' | '>' | '<')
            || (ch == ',' && is_inside_generic_type_args(source, idx))
    })
}

fn is_inside_generic_type_args(source: &str, offset: usize) -> bool {
    let mut depth = 0usize;
    let mut idx = 0usize;

    while idx < offset.min(source.len()) {
        if let Some(next_offset) = skip_lexical_region(source, idx) {
            idx = next_offset;
            continue;
        }

        let Some(ch) = source[idx..].chars().next() else {
            break;
        };
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ => {}
        }
        idx += ch.len_utf8();
    }

    depth > 0
}

fn bare_type_domain_start_before(source: &str, arrow_offset: usize) -> Option<usize> {
    let (end, ch) = previous_significant_char(source, arrow_offset)?;
    if is_type_ident_char(ch) {
        return Some(scan_type_ident_start(source, end));
    }
    if ch == '>' {
        return generic_type_head_start_before(source, end);
    }
    None
}

fn generic_type_head_start_before(source: &str, close_angle: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in source[..=close_angle].char_indices().rev() {
        match ch {
            '>' => depth += 1,
            '<' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let (head_end, head_ch) = previous_significant_char(source, idx)?;
                    if !is_type_ident_char(head_ch) {
                        return None;
                    }
                    return Some(scan_type_ident_start(source, head_end));
                }
            }
            _ => {}
        }
    }
    None
}

fn scan_type_ident_start(source: &str, end: usize) -> usize {
    let mut start = end;
    for (idx, ch) in source[..end].char_indices().rev() {
        if is_type_ident_char(ch) {
            start = idx;
        } else {
            break;
        }
    }
    start
}

fn is_type_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':')
}

fn matching_open_paren_before(source: &str, close_paren: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in source[..=close_paren].char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn previous_significant_char(source: &str, offset: usize) -> Option<(usize, char)> {
    let mut end = offset.min(source.len());
    loop {
        while end > 0 {
            let (idx, ch) = source[..end].char_indices().next_back()?;
            if ch.is_whitespace() {
                end = idx;
                continue;
            }
            break;
        }

        if end == 0 {
            return None;
        }

        if source[..end].ends_with("*/")
            && let Some(start) = source[..end.saturating_sub(2)].rfind("/*")
        {
            end = start;
            continue;
        }

        let line_start = source[..end].rfind('\n').map_or(0, |idx| idx + 1);
        let line = &source[line_start..end];
        if let Some(comment) = line.rfind("//") {
            end = line_start + comment;
            continue;
        }

        return source[..end].char_indices().next_back();
    }
}

fn reserved_type_arrow_message(arrow: ReservedCallableArrow) -> String {
    format!(
        "removed callable arrow syntax is not accepted: `{}`; use the pure callable arrow `->`",
        arrow.arrow()
    )
}

fn reserved_closure_arrow_message(arrow: ReservedCallableArrow) -> String {
    format!(
        "removed callable arrow syntax is not accepted: `{}`; use the pure closure arrow `->`",
        arrow.arrow()
    )
}

fn attach_type_definition_source(definitions: &mut [surface::Definition], source: &str) {
    for definition in definitions {
        match definition {
            surface::Definition::Type(type_def) => type_def.source = Some(source.into()),
            surface::Definition::Newtype(newtype) => newtype.source = Some(source.into()),
            surface::Definition::EffectAlias(alias) => alias.source = Some(source.into()),
            surface::Definition::EffectGroup(group) => group.source = Some(source.into()),
            surface::Definition::Handler(handler) => handler.source = Some(source.into()),
            surface::Definition::Impl(implementation) => {
                for handler in &mut implementation.handlers {
                    handler.source = Some(source.into());
                }
            }
            _ => {}
        }
    }
}

fn attach_inline_module_body_sources(module_decls: &mut [module::ModuleDecl], source: &str) {
    for declaration in module_decls {
        attach_inline_module_body_source(declaration, source);
    }
}

fn attach_inline_module_body_source(declaration: &mut module::ModuleDecl, source: &str) {
    let module::ModuleSource::Inline(body) = &mut declaration.source else {
        return;
    };
    attach_type_definition_source(body.definitions_mut(), source);
    attach_inline_module_body_sources(body.module_decls_mut(), source);
    body.rebuild_item_snapshot();
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
            "mod governance { role reviewer { capabilities: [], obligations: [check_tests] } }",
        );

        let decl = parse_module_decl.parse_next(&mut input).unwrap();
        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower role authority metadata");

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "reviewer");
        assert!(roles[0].authority.is_empty());
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
    fn test_module_decl_rejects_removed_capability_metadata_for_role_authority() {
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() where requires_mfa(); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        assert!(parse_module_decl.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_module_decl_rejects_removed_capability_constraint_arguments_for_role_authority() {
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() where requires_region(\"EU\"); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        assert!(parse_module_decl.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_module_decl_rejects_removed_capability_returns_in_role_authority_metadata() {
        use winnow::prelude::*;

        let mut input = new_input(
            "mod governance { capability approve: decide() returns Bool where requires_region(\"EU\"); role reviewer { capabilities: [approve], obligations: [check_tests] } }",
        );

        assert!(parse_module_decl.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_parse_surface_file_populates_comment_table() {
        let source = r#"
            -- header comment
            fn sensor() -> Int { 1 }
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
