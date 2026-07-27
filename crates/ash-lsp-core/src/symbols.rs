//! Document symbol extraction for Ash source files.
//!
//! Converts `ash_parser::surface::ModuleFile` into hierarchical
//! `lsp_types::DocumentSymbol` values.

#![allow(clippy::missing_const_for_fn)]

use ash_parser::module::{ModuleDecl, ModuleSource};
use ash_parser::parse_surface_file;
use ash_parser::surface::{
    Definition, FnDef, ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, ModuleFile,
};
use ash_parser::token::Span;
use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};
use serde_json::json;
use std::path::{Path, PathBuf};

const fn span_to_range(span: &Span) -> Range {
    let start_line = span.line.saturating_sub(1) as u32;
    let start_col = span.column.saturating_sub(1) as u32;
    let byte_width = span.end.saturating_sub(span.start);
    let end_col = span.column.saturating_sub(1).saturating_add(byte_width) as u32;
    Range {
        start: Position {
            line: start_line,
            character: start_col,
        },
        end: Position {
            line: start_line,
            character: end_col,
        },
    }
}

fn symbol(
    name: String,
    kind: SymbolKind,
    span: &Span,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    let range = span_to_range(span);
    let name = serde_json::to_value(name).expect("symbol name serializes");
    let children = serde_json::to_value(children).expect("symbol children serialize");
    serde_json::from_value(json!({
        "name": name,
        "kind": kind,
        "range": range,
        "selectionRange": range,
        "children": children,
    }))
    .expect("document symbol fixture uses current LSP shape")
}

fn interface_method_symbol(method: &InterfaceMethodSig) -> DocumentSymbol {
    symbol(
        method.name.to_string(),
        SymbolKind::METHOD,
        &method.span,
        None,
    )
}

fn impl_method_symbol(method: &ImplMethodDef) -> DocumentSymbol {
    symbol(
        method.name.to_string(),
        SymbolKind::METHOD,
        &method.span,
        None,
    )
}

fn fn_symbol(def: &FnDef) -> DocumentSymbol {
    symbol(def.name.to_string(), SymbolKind::FUNCTION, &def.span, None)
}

fn interface_symbol(def: &InterfaceDef) -> DocumentSymbol {
    let children = def
        .methods
        .iter()
        .map(interface_method_symbol)
        .collect::<Vec<_>>();
    symbol(
        def.name.to_string(),
        SymbolKind::INTERFACE,
        &def.span,
        (!children.is_empty()).then_some(children),
    )
}

fn impl_symbol(def: &ImplDef) -> DocumentSymbol {
    let children = def
        .methods
        .iter()
        .map(impl_method_symbol)
        .collect::<Vec<_>>();
    symbol(
        format!("impl {}", def.interface),
        SymbolKind::OBJECT,
        &def.span,
        (!children.is_empty()).then_some(children),
    )
}

// Exhaustively maps every source definition to its LSP symbol presentation.
#[allow(clippy::too_many_lines)]
fn definition_symbol(definition: &Definition) -> Option<DocumentSymbol> {
    match definition {
        Definition::Notation(def) => Some(symbol(
            def.pattern.raw.as_ref().to_string(),
            SymbolKind::OPERATOR,
            &def.span,
            None,
        )),
        Definition::Macro(def) => {
            let mut symbol = symbol(def.name.to_string(), SymbolKind::OPERATOR, &def.span, None);
            symbol.detail = Some("syntax-phase macro".to_string());
            Some(symbol)
        }
        Definition::Capability(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::FUNCTION,
            &def.span,
            None,
        )),
        Definition::Policy(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::Role(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::CLASS,
            &def.span,
            None,
        )),
        Definition::Interface(def) => Some(interface_symbol(def)),
        Definition::ResourceType(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::Type(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::Newtype(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::EffectAlias(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::EffectGroup(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::STRUCT,
            &def.span,
            None,
        )),
        Definition::DataKind(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::ENUM,
            &def.span,
            None,
        )),
        Definition::TypeFn(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::FUNCTION,
            &def.span,
            None,
        )),
        Definition::PropositionPredicate(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::FUNCTION,
            &def.span,
            None,
        )),
        Definition::Impl(def) => Some(impl_symbol(def)),
        Definition::Function(def) => Some(fn_symbol(def)),
        Definition::Handler(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::FUNCTION,
            &def.span,
            None,
        )),
        Definition::BuiltinFn(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::FUNCTION,
            &def.span,
            None,
        )),
        Definition::SealedDomain(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::ENUM,
            &def.span,
            None,
        )),
        Definition::Law(_) | Definition::Proof(_) => None,
    }
}

fn module_decl_symbol(module: &ModuleDecl) -> DocumentSymbol {
    let children = match &module.source {
        ModuleSource::File => None,
        ModuleSource::Inline(definitions) => {
            let children: Vec<DocumentSymbol> =
                definitions.iter().filter_map(definition_symbol).collect();
            (!children.is_empty()).then_some(children)
        }
    };

    symbol(
        module.name.to_string(),
        SymbolKind::MODULE,
        &module.span,
        children,
    )
}

#[must_use]
pub fn document_symbols(module: &ModuleFile) -> Vec<DocumentSymbol> {
    let mut entries: Vec<(usize, DocumentSymbol)> = Vec::new();

    for module_decl in &module.module_decls {
        entries.push((module_decl.span.start, module_decl_symbol(module_decl)));
    }

    for definition in &module.definitions {
        let start = match definition {
            Definition::Notation(def) => def.span.start,
            Definition::Macro(def) => def.span.start,
            Definition::Capability(def) => def.span.start,
            Definition::Policy(def) => def.span.start,
            Definition::Role(def) => def.span.start,
            Definition::Interface(def) => def.span.start,
            Definition::ResourceType(def) => def.span.start,
            Definition::Type(def) => def.span.start,
            Definition::Newtype(def) => def.span.start,
            Definition::EffectAlias(def) => def.span.start,
            Definition::EffectGroup(def) => def.span.start,
            Definition::DataKind(def) => def.span.start,
            Definition::TypeFn(def) => def.span.start,
            Definition::PropositionPredicate(def) => def.span.start,
            Definition::Impl(def) => def.span.start,
            Definition::Function(def) => def.span.start,
            Definition::Handler(def) => def.span.start,
            Definition::BuiltinFn(def) => def.span.start,
            Definition::SealedDomain(def) => def.span.start,
            Definition::Law(def) => def.span.start,
            Definition::Proof(def) => def.span.start,
        };
        if let Some(sym) = definition_symbol(definition) {
            entries.push((start, sym));
        }
    }

    entries.sort_by_key(|(start, _)| *start);
    entries.into_iter().map(|(_, symbol)| symbol).collect()
}

/// A workspace-wide symbol match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    /// Symbol name.
    pub name: String,
    /// LSP symbol kind.
    pub kind: SymbolKind,
    /// Absolute file path containing the symbol.
    pub file: PathBuf,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column (character offset within the line).
    pub column: u32,
}

/// Search recursively under `root` for `.ash` files and return top-level
/// symbols whose names contain `query` (case-insensitive).
///
/// # Errors
///
/// Returns an empty vector if `root` cannot be read or no files match.
#[must_use]
pub fn workspace_symbols(root: &Path, query: &str) -> Vec<WorkspaceSymbol> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    collect_workspace_symbol_files(root, query, &mut out);
    out.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.name.cmp(&right.name))
    });
    out
}

fn collect_workspace_symbols(
    file: &Path,
    module: &ModuleFile,
    query: &str,
    out: &mut Vec<WorkspaceSymbol>,
) {
    let needle = query.to_lowercase();
    let symbols = document_symbols(module);
    for symbol in &symbols {
        collect_symbol_match(file, symbol, &needle, out);
    }
}

fn collect_symbol_match(
    file: &Path,
    symbol: &DocumentSymbol,
    needle: &str,
    out: &mut Vec<WorkspaceSymbol>,
) {
    if symbol.name.to_lowercase().contains(needle) {
        out.push(WorkspaceSymbol {
            name: symbol.name.clone(),
            kind: symbol.kind,
            file: file.to_path_buf(),
            line: symbol.selection_range.start.line + 1,
            column: symbol.selection_range.start.character + 1,
        });
    }

    if let Some(children) = &symbol.children {
        for child in children {
            collect_symbol_match(file, child, needle, out);
        }
    }
}

fn collect_workspace_symbol_files(root: &Path, query: &str, out: &mut Vec<WorkspaceSymbol>) {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_workspace_symbol_files(&path, query, out);
        } else if path.extension().is_some_and(|ext| ext == "ash")
            && let Ok(source) = std::fs::read_to_string(&path)
            && let Ok(module) = parse_surface_file(&source)
        {
            collect_workspace_symbols(&path, &module, query, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::module::ModuleDecl;
    use ash_parser::parse_utils::CommentTable;
    use ash_parser::surface::{
        CapabilityDef, EffectType, InterfaceDef, InterfaceMethodSig, Param, Type, Visibility,
    };
    use std::io::Write;

    fn span(start: usize, end: usize, line: usize, column: usize) -> Span {
        Span::new(start, end, line, column)
    }

    fn empty_module() -> ModuleFile {
        ModuleFile {
            definitions: vec![],
            module_decls: vec![],
            span: span(0, 1, 1, 1),
            comments: CommentTable::default(),
            path: None,
        }
    }

    #[test]
    fn test_document_symbols_interface_has_method_children() {
        let module = ModuleFile {
            definitions: vec![Definition::Interface(InterfaceDef {
                visibility: Visibility::Inherited,
                name: "Reader".into(),
                type_params: vec![],
                evidence_constraints: vec![],
                associated_types: vec![],
                methods: vec![InterfaceMethodSig {
                    name: "read".into(),
                    params: vec![Type::Name("String".into())],
                    return_type: Type::Name("String".into()),
                    span: span(15, 25, 2, 5),
                }],
                laws: Vec::new(),
                span: span(10, 40, 2, 1),
            })],
            ..empty_module()
        };

        let symbols = document_symbols(&module);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Reader");
        assert_eq!(symbols[0].kind, SymbolKind::INTERFACE);
        let children = symbols[0].children.as_ref().expect("interface children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "read");
        assert_eq!(children[0].kind, SymbolKind::METHOD);
    }

    #[test]
    fn test_document_symbols_inline_module_has_children() {
        let inline = ModuleDecl::inline(
            "inner".into(),
            Visibility::Inherited,
            vec![Definition::Capability(CapabilityDef {
                visibility: Visibility::Inherited,
                name: "sensor".into(),
                effect: EffectType::Read,
                params: vec![Param {
                    name: "id".into(),
                    name_span: Span::default(),
                    ty: Type::Name("String".into()),
                }],
                return_type: Some(Type::Name("String".into())),
                constraints: vec![],
                target_provider: None,
                target_action: None,
                span: span(20, 40, 3, 3),
            })],
            span(10, 50, 2, 1),
        );

        let module = ModuleFile {
            module_decls: vec![inline],
            ..empty_module()
        };

        let symbols = document_symbols(&module);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "inner");
        assert_eq!(symbols[0].kind, SymbolKind::MODULE);
        let children = symbols[0]
            .children
            .as_ref()
            .expect("inline module children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "sensor");
        assert_eq!(children[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_document_symbols_marks_macro_as_operator_detail() {
        let module = parse_surface_file("macro id(x) => x;").expect("parse ok");
        let symbols = document_symbols(&module);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "id");
        assert_eq!(symbols[0].kind, SymbolKind::OPERATOR);
        assert_eq!(symbols[0].detail.as_deref(), Some("syntax-phase macro"));
    }

    fn write_ash(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).expect("create");
        f.write_all(content.as_bytes()).expect("write");
        path
    }

    #[test]
    fn test_workspace_symbols_finds_top_level_names() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "one.ash", "fn helper() -> Int { 1 }\n");
        write_ash(dir.path(), "two.ash", "fn main() -> Int { helper() }\n");

        let result = workspace_symbols(dir.path(), "helper");

        assert_eq!(result.len(), 1, "expected one match for 'helper'");
        assert_eq!(result[0].name, "helper");
        assert_eq!(result[0].kind, SymbolKind::FUNCTION);
        assert!(
            result[0]
                .file
                .as_os_str()
                .to_string_lossy()
                .contains("one.ash")
        );
        assert_eq!(result[0].line, 1);
        assert_eq!(result[0].column, 1);
    }

    #[test]
    fn test_workspace_symbols_case_insensitive_substring() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "alpha.ash", "fn HelperBee() -> Int { 1 }\n");

        let result = workspace_symbols(dir.path(), "bee");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "HelperBee");
    }

    #[test]
    fn test_workspace_symbols_recurses_into_subdirectories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).expect("mkdir");
        write_ash(&nested, "deep.ash", "interface Sensor { read() -> Int }\n");

        let result = workspace_symbols(dir.path(), "sensor");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Sensor");
        assert!(
            result[0]
                .file
                .as_os_str()
                .to_string_lossy()
                .contains("deep.ash")
        );
    }

    #[test]
    fn test_workspace_symbols_ignores_non_ash_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "code.ash", "fn real() -> Int { 1 }\n");
        let mut txt = dir.path().to_path_buf();
        txt.push("notes.txt");
        std::fs::write(&txt, "fn fake() -> Int {}").expect("write txt");

        let result = workspace_symbols(dir.path(), "fake");

        assert!(result.is_empty(), "should not match symbols in .txt files");
    }

    #[test]
    fn test_workspace_symbols_returns_empty_for_no_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "empty.ash", "fn only() -> Int { 1 }\n");

        let result = workspace_symbols(dir.path(), "missing");

        assert!(
            result.is_empty(),
            "expected empty result for non-matching query"
        );
    }

    #[test]
    fn test_workspace_symbols_includes_interface_children() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(
            dir.path(),
            "iface.ash",
            "interface Source {\n  read() -> String\n}\n",
        );

        let result = workspace_symbols(dir.path(), "read");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "read");
        assert_eq!(result[0].kind, SymbolKind::METHOD);
        assert_eq!(result[0].line, 2);
    }

    #[test]
    fn test_workspace_symbols_returns_sorted_by_file_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "z.ash", "fn alpha() -> Int { 1 }\n");
        write_ash(dir.path(), "a.ash", "fn alpha() -> Int { 2 }\n");

        let result = workspace_symbols(dir.path(), "alpha");

        assert_eq!(result.len(), 2);
        let names: Vec<_> = result.iter().map(|s| s.file.file_name().unwrap()).collect();
        assert_eq!(
            names,
            vec![std::ffi::OsStr::new("a.ash"), std::ffi::OsStr::new("z.ash")]
        );
    }

    #[test]
    fn test_workspace_symbols_handles_parse_errors_gracefully() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_ash(dir.path(), "good.ash", "fn ok() -> Int { 1 }\n");
        write_ash(dir.path(), "bad.ash", "fn broken { }\n");

        let result = workspace_symbols(dir.path(), "ok");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "ok");
    }
}
