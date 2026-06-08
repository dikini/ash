//! Go-to-definition support for Ash source files.
//!
//! MVP: resolves identifiers to their definition span within the same file.
//! Cross-file resolution is deferred to Phase 5 (multi-file workspace).

use ash_parser::module::ModuleDecl;
use ash_parser::surface::{Definition, ModuleFile};
use ash_parser::token::Span;
use lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use crate::position::{line_col_from_offset, offset_from_line_col, token_at_offset};

/// Convert a parser `Span` (1-indexed line/col, byte offsets) to an LSP `Range`
/// (0-indexed line/character) using the source text for end-position resolution.
fn span_to_range(source: &str, span: &Span) -> Option<Range> {
    // Parser spans are 1-indexed; LSP is 0-indexed.
    let start_line = u32::try_from(span.line).ok()?.saturating_sub(1);
    let start_char = u32::try_from(span.column).ok()?.saturating_sub(1);
    let (end_line, end_char) = line_col_from_offset(source, span.end)?;

    Some(Range {
        start: Position {
            line: start_line,
            character: start_char,
        },
        end: Position {
            line: end_line,
            character: end_char,
        },
    })
}

/// Search for a definition whose name matches `token` and return its span.
fn find_definition_span<'a>(token: &str, definitions: &'a [Definition]) -> Option<&'a Span> {
    for def in definitions {
        let name_matches = match def {
            Definition::Function(f) => f.name.as_ref() == token,
            Definition::Capability(c) => c.name.as_ref() == token,
            Definition::Policy(p) => p.name.as_ref() == token,
            Definition::Role(r) => r.name.as_ref() == token,
            Definition::Proxy(p) => p.name.as_ref() == token,
            Definition::Interface(i) => i.name.as_ref() == token,
            Definition::CapabilityInterface(i) => i.name.as_ref() == token,
            Definition::CapabilityImplementation(i) => i.name.as_ref() == token,
            Definition::ResourceType(r) => r.name.as_ref() == token,
            Definition::Type(t) => t.name.as_ref() == token,
            Definition::DataKind(d) => d.name.as_ref() == token,
            Definition::TypeFn(t) => t.name.as_ref() == token,
            Definition::PropositionPredicate(p) => p.name.as_ref() == token,
            Definition::Impl(i) => i.interface.as_ref() == token,
            Definition::BuiltinFn(b) => b.name.as_ref() == token,
            Definition::SealedDomain(d) => d.name.as_ref() == token,
            Definition::Law(_) => false,
        };
        if name_matches {
            return Some(match def {
                Definition::Function(f) => &f.span,
                Definition::Capability(c) => &c.span,
                Definition::Policy(p) => &p.span,
                Definition::Role(r) => &r.span,
                Definition::Proxy(p) => &p.span,
                Definition::Interface(i) => &i.span,
                Definition::CapabilityInterface(i) => &i.span,
                Definition::CapabilityImplementation(i) => &i.span,
                Definition::ResourceType(r) => &r.span,
                Definition::Type(t) => &t.span,
                Definition::DataKind(d) => &d.span,
                Definition::TypeFn(t) => &t.span,
                Definition::PropositionPredicate(p) => &p.span,
                Definition::Impl(i) => &i.span,
                Definition::BuiltinFn(b) => &b.span,
                Definition::SealedDomain(d) => &d.span,
                Definition::Law(l) => &l.span,
            });
        }
        // Check interface/impl method names.
        if let Some(span) = check_sub_def_names(token, def) {
            return Some(span);
        }
    }
    None
}

fn check_sub_def_names<'a>(token: &str, def: &'a Definition) -> Option<&'a Span> {
    match def {
        Definition::Interface(i) => i
            .methods
            .iter()
            .find(|m| m.name.as_ref() == token)
            .map(|m| &m.span),
        Definition::Impl(i) => i
            .methods
            .iter()
            .find(|m| m.name.as_ref() == token)
            .map(|m| &m.span),
        _ => None,
    }
}

#[allow(clippy::collapsible_if)]
fn find_module_decl_span<'a>(token: &str, decls: &'a [ModuleDecl]) -> Option<&'a Span> {
    for decl in decls {
        if decl.name.as_ref() == token {
            return Some(&decl.span);
        }
        if let Some(defs) = decl.definitions() {
            if let Some(span) = find_definition_span(token, defs) {
                return Some(span);
            }
        }
    }
    None
}

/// Perform go-to-definition at the given cursor position.
///
/// Returns `None` if the cursor is not on an identifier or no matching
/// definition is found.
#[must_use]
pub fn goto_definition(
    module: &ModuleFile,
    source: &str,
    uri: &lsp_types::Uri,
    line: u32,
    col: u32,
) -> Option<GotoDefinitionResponse> {
    let offset = offset_from_line_col(source, line, col)?;
    let token = token_at_offset(source, offset)?;

    // Check workflow entry.
    if let Some(ref wf) = module.workflow
        && wf.name.as_ref() == token
    {
        let range = span_to_range(source, &wf.span)?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range,
        }));
    }

    // Check module declarations and their inner definitions.
    if let Some(span) = find_module_decl_span(token, &module.module_decls) {
        let range = span_to_range(source, span)?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range,
        }));
    }

    // Check top-level definitions.
    if let Some(span) = find_definition_span(token, &module.definitions) {
        let range = span_to_range(source, span)?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range,
        }));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::parse_surface_file;
    use lsp_types::Uri;

    fn parse_uri() -> Uri {
        "file:///test.ash".parse().unwrap()
    }

    #[test]
    fn test_goto_workflow() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // "main" starts at col 9
        let result = goto_definition(&module, source, &parse_uri(), 0, 9);
        assert!(result.is_some(), "should find workflow definition");
    }

    #[test]
    fn test_goto_function() {
        let source = "fn helper(x: Int) -> Int { x }\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // "helper" starts at col 3 on line 0
        let result = goto_definition(&module, source, &parse_uri(), 0, 3);
        assert!(result.is_some(), "should find function definition");
    }

    #[test]
    fn test_goto_capability() {
        let source = "capability sensor: epistemic()\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // "sensor" starts at col 11
        let result = goto_definition(&module, source, &parse_uri(), 0, 11);
        assert!(result.is_some(), "should find capability definition");
    }

    #[test]
    fn test_goto_interface() {
        let source = "interface Store { get(String) -> String }\nworkflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // "Store" at col 11
        let result = goto_definition(&module, source, &parse_uri(), 0, 11);
        assert!(result.is_some(), "should find interface definition");
    }

    #[test]
    fn test_goto_nothing_on_whitespace() {
        let source = "workflow main { done }";
        let module = parse_surface_file(source).expect("parse ok");
        // space between "main" and "{"
        let result = goto_definition(&module, source, &parse_uri(), 0, 14);
        assert!(result.is_none(), "whitespace should not resolve");
    }

    #[test]
    fn test_goto_reference_to_function() {
        let source = "fn helper() -> Int { 1 }\nworkflow main { let x = helper() done }";
        let module = parse_surface_file(source).expect("parse ok");
        // "helper" on line 1 — find its column
        let line1_start = source.find('\n').unwrap() + 1;
        let helper_offset = source[line1_start..].find("helper").unwrap() + line1_start;
        let col = u32::try_from(helper_offset - line1_start).unwrap();
        let result = goto_definition(&module, source, &parse_uri(), 1, col);
        assert!(result.is_some(), "should resolve reference to function");
    }
}
