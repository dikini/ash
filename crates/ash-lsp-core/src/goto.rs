//! Go-to-definition support for Ash source files.
//!
//! MVP: resolves identifiers to their definition span within the same file.
//! Cross-file resolution is deferred to Phase 5 (multi-file workspace).

use ash_parser::module::ModuleDecl;
use ash_parser::surface::{Definition, ModuleFile};
use ash_parser::token::Span;
use lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use crate::position::{is_ident_char, line_col_from_offset, offset_from_line_col, token_at_offset};

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
            Definition::Notation(n) => n.pattern.raw.as_ref() == token,
            Definition::Macro(_) | Definition::Law(_) => false,
            Definition::Function(f) => f.name.as_ref() == token,
            Definition::Capability(c) => c.name.as_ref() == token,
            Definition::Policy(p) => p.name.as_ref() == token,
            Definition::Role(r) => r.name.as_ref() == token,
            Definition::Interface(i) => i.name.as_ref() == token,
            Definition::ResourceType(r) => r.name.as_ref() == token,
            Definition::Type(t) => t.name.as_ref() == token,
            Definition::DataKind(d) => d.name.as_ref() == token,
            Definition::TypeFn(t) => t.name.as_ref() == token,
            Definition::PropositionPredicate(p) => p.name.as_ref() == token,
            Definition::Impl(i) => i.interface.as_ref() == token,
            Definition::BuiltinFn(b) => b.name.as_ref() == token,
            Definition::SealedDomain(d) => d.name.as_ref() == token,

            Definition::Proof(p) => p.name.as_ref() == token,
        };
        if name_matches {
            return Some(match def {
                Definition::Notation(n) => &n.span,
                Definition::Macro(m) => &m.span,
                Definition::Function(f) => &f.span,
                Definition::Capability(c) => &c.span,
                Definition::Policy(p) => &p.span,
                Definition::Role(r) => &r.span,
                Definition::Interface(i) => &i.span,
                Definition::ResourceType(r) => &r.span,
                Definition::Type(t) => &t.span,
                Definition::DataKind(d) => &d.span,
                Definition::TypeFn(t) => &t.span,
                Definition::PropositionPredicate(p) => &p.span,
                Definition::Impl(i) => &i.span,
                Definition::BuiltinFn(b) => &b.span,
                Definition::SealedDomain(d) => &d.span,
                Definition::Law(l) => &l.span,
                Definition::Proof(p) => &p.span,
            });
        }
        // Check interface/impl method names.
        if let Some(span) = check_sub_def_names(token, def) {
            return Some(span);
        }
    }
    None
}

fn find_macro_definition_span<'a>(token: &str, definitions: &'a [Definition]) -> Option<&'a Span> {
    definitions.iter().find_map(|def| match def {
        Definition::Macro(m) if m.name.as_ref() == token => Some(&m.span),
        _ => None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SameFileIdentity {
    Macro {
        name: String,
        line: usize,
        column: usize,
    },
    Callable {
        name: String,
        line: usize,
        column: usize,
        kind: CallableIdentityKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableIdentityKind {
    Function,
    BuiltinFn,
}

fn collect_callable_definition_spans<'a>(
    token: &str,
    definitions: &'a [Definition],
    matches: &mut Vec<(&'a Span, CallableIdentityKind)>,
) {
    for def in definitions {
        match def {
            Definition::Function(f) if f.name.as_ref() == token => {
                matches.push((&f.span, CallableIdentityKind::Function));
            }
            Definition::BuiltinFn(b) if b.name.as_ref() == token => {
                matches.push((&b.span, CallableIdentityKind::BuiltinFn));
            }
            _ => {}
        }
    }
}

fn macro_identity(module: &ModuleFile, token: &str) -> Option<SameFileIdentity> {
    find_macro_decl_span(token, &module.module_decls)
        .or_else(|| find_macro_definition_span(token, &module.definitions))
        .map(|span| SameFileIdentity::Macro {
            name: token.to_string(),
            line: span.line,
            column: span.column,
        })
}

fn collect_callable_matches<'a>(
    module: &'a ModuleFile,
    token: &str,
) -> Vec<(&'a Span, CallableIdentityKind)> {
    let mut matches = Vec::new();
    collect_callable_definition_spans(token, &module.definitions, &mut matches);
    for decl in &module.module_decls {
        if let Some(defs) = decl.definitions() {
            collect_callable_definition_spans(token, defs, &mut matches);
        }
    }
    matches
}

fn callable_identity(module: &ModuleFile, token: &str) -> Option<SameFileIdentity> {
    let matches = collect_callable_matches(module, token);
    if matches.len() != 1 {
        return None;
    }
    let (span, kind) = matches[0];
    Some(SameFileIdentity::Callable {
        name: token.to_string(),
        line: span.line,
        column: span.column,
        kind,
    })
}

fn is_macro_invocation_at(source: &str, offset: usize) -> bool {
    if source.get(offset..).is_none() {
        return false;
    }

    let mut start = offset;
    while start > 0 {
        let Some(prev) = source[..start].chars().next_back() else {
            break;
        };
        let prev_start = start - prev.len_utf8();
        if !is_ident_char(prev) {
            break;
        }
        start = prev_start;
    }

    let mut end = offset;
    while let Some(next) = source[end..].chars().next() {
        if !is_ident_char(next) {
            break;
        }
        end += next.len_utf8();
    }

    source[end..]
        .chars()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch == '!')
}

fn is_ordinary_call_at(source: &str, offset: usize) -> bool {
    let mut end = offset;
    while let Some(next) = source[end..].chars().next() {
        if !is_ident_char(next) {
            break;
        }
        end += next.len_utf8();
    }
    source[end..]
        .chars()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch == '(')
}

fn is_macro_declaration_name_at(source: &str, offset: usize) -> bool {
    let mut start = offset;
    while start > 0 {
        let Some(prev) = source[..start].chars().next_back() else {
            break;
        };
        let prev_start = start - prev.len_utf8();
        if !is_ident_char(prev) {
            break;
        }
        start = prev_start;
    }
    let Some(prefix) = source.get(..start) else {
        return false;
    };
    prefix
        .split(|ch: char| !is_ident_char(ch))
        .rfind(|word| !word.is_empty())
        .is_some_and(|word| word == "macro")
}

fn is_callable_declaration_name_at(source: &str, offset: usize) -> bool {
    let mut start = offset;
    while start > 0 {
        let Some(prev) = source[..start].chars().next_back() else {
            break;
        };
        let prev_start = start - prev.len_utf8();
        if !is_ident_char(prev) {
            break;
        }
        start = prev_start;
    }
    let Some(prefix) = source.get(..start) else {
        return false;
    };
    let words = prefix
        .split(|ch: char| !is_ident_char(ch))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    words.last().is_some_and(|word| *word == "fn")
}

fn same_file_identity_at(
    module: &ModuleFile,
    source: &str,
    token: &str,
    offset: usize,
) -> Option<SameFileIdentity> {
    if is_macro_invocation_at(source, offset) || is_macro_declaration_name_at(source, offset) {
        return macro_identity(module, token);
    }
    if is_ordinary_call_at(source, offset) || is_callable_declaration_name_at(source, offset) {
        return callable_identity(module, token);
    }
    None
}

fn is_ambiguous_callable_reference_at(
    module: &ModuleFile,
    source: &str,
    token: &str,
    offset: usize,
) -> bool {
    (is_ordinary_call_at(source, offset) || is_callable_declaration_name_at(source, offset))
        && collect_callable_matches(module, token).len() > 1
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

fn find_macro_decl_span<'a>(token: &str, decls: &'a [ModuleDecl]) -> Option<&'a Span> {
    for decl in decls {
        if let Some(defs) = decl.definitions()
            && let Some(span) = find_macro_definition_span(token, defs)
        {
            return Some(span);
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

    // Macro invocations are syntax-phase uses (`m!(...)`), so prefer a macro
    // declaration over an ordinary function with the same spelling. Cross-file
    // imported-summary navigation remains out of scope until a real source
    // location is available.
    if (is_macro_invocation_at(source, offset) || is_macro_declaration_name_at(source, offset))
        && let Some(span) = find_macro_decl_span(token, &module.module_decls)
            .or_else(|| find_macro_definition_span(token, &module.definitions))
    {
        let range = span_to_range(source, span)?;
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

/// Perform same-file find-references at the given cursor position.
///
/// Returns every span in `source` where the identifier at the cursor appears,
/// including the definition site. Results are sorted by source order.
/// Cross-file references are deferred.
#[must_use]
pub fn find_references(
    module: &ModuleFile,
    source: &str,
    uri: &lsp_types::Uri,
    line: u32,
    col: u32,
) -> Vec<Location> {
    let offset = match offset_from_line_col(source, line, col) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let token = match token_at_offset(source, offset) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let target_identity = same_file_identity_at(module, source, token, offset);
    if target_identity.is_none()
        && is_ambiguous_callable_reference_at(module, source, token, offset)
    {
        return Vec::new();
    }

    let mut locations = Vec::new();
    let mut prev_was_ident = false;
    for (idx, ch) in source.char_indices() {
        let at_token_start = !prev_was_ident && is_ident_char(ch);
        prev_was_ident = is_ident_char(ch);

        if !at_token_start {
            continue;
        }

        let mut end = idx + ch.len_utf8();
        for (_next_rel, next_ch) in source[end..].char_indices() {
            if is_ident_char(next_ch) {
                end += next_ch.len_utf8();
            } else {
                break;
            }
        }

        if source.get(idx..end) != Some(token) {
            continue;
        }
        if let Some(identity) = target_identity.as_ref()
            && same_file_identity_at(module, source, token, idx).as_ref() != Some(identity)
        {
            continue;
        }
        if let Some((start_line, start_col)) = line_col_from_offset(source, idx)
            && let Some((end_line, end_col)) = line_col_from_offset(source, end)
        {
            locations.push(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: start_line,
                        character: start_col,
                    },
                    end: Position {
                        line: end_line,
                        character: end_col,
                    },
                },
            });
        }
    }

    locations
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
    fn test_goto_entry_function() {
        let source = "fn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        let result = goto_definition(&module, source, &parse_uri(), 0, 3);
        assert!(result.is_some(), "should find entry function definition");
    }

    #[test]
    fn test_goto_function() {
        let source = "fn helper(x: Int) -> Int { x }\nfn main() -> Int { helper(1) }";
        let module = parse_surface_file(source).expect("parse ok");
        // "helper" starts at col 3 on line 0
        let result = goto_definition(&module, source, &parse_uri(), 0, 3);
        assert!(result.is_some(), "should find function definition");
    }

    #[test]
    fn test_goto_interface_definition() {
        let source = "interface Sensor { read() -> Int }\nfn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        let result = goto_definition(&module, source, &parse_uri(), 0, 11);
        assert!(result.is_some(), "should find interface definition");
    }

    #[test]
    fn test_goto_interface() {
        let source = "interface Store { get(String) -> String }\nfn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        // "Store" at col 11
        let result = goto_definition(&module, source, &parse_uri(), 0, 11);
        assert!(result.is_some(), "should find interface definition");
    }

    #[test]
    fn test_goto_nothing_on_whitespace() {
        let source = "fn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        let result = goto_definition(&module, source, &parse_uri(), 0, 2);
        assert!(result.is_none(), "whitespace should not resolve");
    }

    #[test]
    fn test_goto_reference_to_function() {
        let source = "fn helper() -> Int { 1 }\nfn main() -> Int { helper() }";
        let module = parse_surface_file(source).expect("parse ok");
        // "helper" on line 1 — find its column
        let line1_start = source.find('\n').unwrap() + 1;
        let helper_offset = source[line1_start..].find("helper").unwrap() + line1_start;
        let col = u32::try_from(helper_offset - line1_start).unwrap();
        let result = goto_definition(&module, source, &parse_uri(), 1, col);
        assert!(result.is_some(), "should resolve reference to function");
    }

    #[test]
    fn test_goto_macro_invocation_prefers_macro_over_same_named_function() {
        let source = "fn id() -> Int { 1 }\nmacro id(x) => x;\nfn main() -> Int { id!(1) }";
        let module = parse_surface_file(source).expect("parse ok");
        let line2_start = source.rfind('\n').unwrap() + 1;
        let id_offset = source[line2_start..].find("id!").unwrap() + line2_start;
        let col = u32::try_from(id_offset - line2_start).unwrap();

        let result = goto_definition(&module, source, &parse_uri(), 2, col)
            .expect("macro invocation resolves");
        let GotoDefinitionResponse::Scalar(location) = result else {
            panic!("expected scalar location");
        };
        assert_eq!(
            location.range.start.line, 1,
            "should select macro declaration line"
        );
    }

    #[test]
    fn test_goto_ordinary_call_prefers_function_over_same_named_macro() {
        let source = "macro id(x) => x;\nfn id() -> Int { 1 }\nfn main() -> Int { id() }";
        let module = parse_surface_file(source).expect("parse ok");
        let line2_start = source.rfind('\n').unwrap() + 1;
        let id_offset = source[line2_start..].find("id()").unwrap() + line2_start;
        let col = u32::try_from(id_offset - line2_start).unwrap();

        let result =
            goto_definition(&module, source, &parse_uri(), 2, col).expect("ordinary call resolves");
        let GotoDefinitionResponse::Scalar(location) = result else {
            panic!("expected scalar location");
        };
        assert_eq!(
            location.range.start.line, 1,
            "ordinary calls must not resolve to a same-named syntax-phase macro"
        );
    }

    #[test]
    fn test_find_references_function_definition_and_call() {
        let source = "fn helper() -> Int { 1 }\nfn main() -> Int { helper() }";
        let module = parse_surface_file(source).expect("parse ok");
        let line1_start = source.find('\n').unwrap() + 1;
        let helper_offset = source[line1_start..].find("helper").unwrap() + line1_start;
        let col = u32::try_from(helper_offset - line1_start).unwrap();
        let refs = find_references(&module, source, &parse_uri(), 1, col);
        assert_eq!(refs.len(), 2, "should find definition and call site");
    }

    #[test]
    fn test_find_references_no_substring_false_positives() {
        let source = "fn helper() -> Int { 1 }\nfn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        // "help" is a substring of "helper" but not a standalone token.
        let refs = find_references(&module, source, &parse_uri(), 0, 5);
        assert_eq!(refs.len(), 1, "should only match the full identifier");
    }

    #[test]
    fn test_find_references_splits_macro_invocation_from_same_named_function() {
        let source = "fn id() -> Int { 1 }\nmacro id(x) => x;\nfn main() -> Int { id!(1) + id() }";
        let module = parse_surface_file(source).expect("parse ok");
        let line2_start = source.rfind('\n').unwrap() + 1;
        let macro_offset = source[line2_start..].find("id!").unwrap() + line2_start;
        let macro_col = u32::try_from(macro_offset - line2_start).unwrap();
        let macro_refs = find_references(&module, source, &parse_uri(), 2, macro_col);
        assert_eq!(
            macro_refs.len(),
            2,
            "macro refs include declaration and invocation only"
        );
        assert!(macro_refs.iter().all(|loc| loc.range.start.line != 0));

        let call_offset = source[line2_start..].find("id() ").unwrap() + line2_start;
        let call_col = u32::try_from(call_offset - line2_start).unwrap();
        let call_refs = find_references(&module, source, &parse_uri(), 2, call_col);
        assert_eq!(
            call_refs.len(),
            2,
            "callable refs include function declaration and ordinary call only"
        );
        assert!(call_refs.iter().all(|loc| loc.range.start.line != 1));
    }

    #[test]
    fn test_find_references_ambiguous_duplicate_callable_fails_closed() {
        let source = "fn id() -> Int { 1 }\nfn id(x: Int) -> Int { x }\nfn main() -> Int { id() }";
        let module = parse_surface_file(source).expect("parse ok");
        let line2_start = source.rfind('\n').unwrap() + 1;
        let call_offset = source[line2_start..].find("id()").unwrap() + line2_start;
        let call_col = u32::try_from(call_offset - line2_start).unwrap();

        let refs = find_references(&module, source, &parse_uri(), 2, call_col);
        assert!(
            refs.is_empty(),
            "duplicate callable declarations are ambiguous, so semantic references fail closed"
        );
    }

    #[test]
    fn test_find_references_entry_function_name() {
        let source = "fn main() -> Int { 1 }";
        let module = parse_surface_file(source).expect("parse ok");
        let refs = find_references(&module, source, &parse_uri(), 0, 3);
        assert_eq!(refs.len(), 1, "should find entry function declaration");
    }
}
