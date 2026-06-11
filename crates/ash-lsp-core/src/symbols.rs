//! Document symbol extraction for Ash source files.
//!
//! Converts `ash_parser::surface::ModuleFile` into hierarchical
//! `lsp_types::DocumentSymbol` values.

#![allow(deprecated, clippy::missing_const_for_fn)]

use ash_parser::module::{ModuleDecl, ModuleSource};
use ash_parser::surface::{
    Definition, FnDef, ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, ModuleFile,
    WorkflowDef,
};
use ash_parser::token::Span;
use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// A symbol found in a workspace-wide search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    /// The symbol name.
    pub name: String,
    /// The LSP symbol kind.
    pub kind: SymbolKind,
    /// The absolute file path containing the symbol.
    pub file: std::path::PathBuf,
    /// The 1-indexed line of the symbol definition.
    pub line: u32,
    /// The 1-indexed column of the symbol definition.
    pub column: u32,
}

/// Search for symbols across all `.ash` files under `root`.
///
/// Performs a recursive directory scan, parses each `.ash` file, and returns
/// every top-level symbol whose name contains `query` as a case-insensitive
/// substring. Parse failures are skipped with a debug log rather than failing
/// the whole search.
#[must_use]
pub fn workspace_symbols(root: &std::path::Path, query: &str) -> Vec<WorkspaceSymbol> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    collect_workspace_symbols(root, &query_lower, &mut results);
    results.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
    });
    results
}

fn collect_workspace_symbols(
    root: &std::path::Path,
    query_lower: &str,
    out: &mut Vec<WorkspaceSymbol>,
) {
    let mut entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) => {
            tracing::debug!(path = %root.display(), error = %err, "unable to read workspace directory");
            return;
        }
    };

    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(err) => {
                tracing::debug!(path = %path.display(), error = %err, "unable to read file type");
                continue;
            }
        };

        if file_type.is_dir() {
            collect_workspace_symbols(&path, query_lower, out);
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "ash") {
            collect_file_symbols(&path, query_lower, out);
        }
    }
}

fn collect_file_symbols(path: &std::path::Path, query_lower: &str, out: &mut Vec<WorkspaceSymbol>) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            tracing::debug!(path = %path.display(), error = %err, "unable to read ash file");
            return;
        }
    };

    let module = match ash_parser::parse_surface_file(&content) {
        Ok(module) => module,
        Err(err) => {
            tracing::debug!(path = %path.display(), error = ?err, "unable to parse ash file");
            return;
        }
    };

    let file = path.to_path_buf();
    for symbol in document_symbols(&module) {
        collect_matching_symbols(&file, &symbol, query_lower, out);
    }
}

fn collect_matching_symbols(
    file: &std::path::Path,
    symbol: &DocumentSymbol,
    query_lower: &str,
    out: &mut Vec<WorkspaceSymbol>,
) {
    if symbol.name.to_lowercase().contains(query_lower) {
        out.push(WorkspaceSymbol {
            name: symbol.name.clone(),
            kind: symbol.kind,
            file: file.to_path_buf(),
            line: symbol.range.start.line + 1,
            column: symbol.range.start.character + 1,
        });
    }

    if let Some(children) = &symbol.children {
        for child in children {
            collect_matching_symbols(file, child, query_lower, out);
        }
    }
}

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
    DocumentSymbol {
        name,
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children,
    }
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

fn workflow_symbol(def: &WorkflowDef) -> DocumentSymbol {
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

fn definition_symbol(definition: &Definition) -> Option<DocumentSymbol> {
    match definition {
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
        Definition::Proxy(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::OBJECT,
            &def.span,
            None,
        )),
        Definition::Interface(def) => Some(interface_symbol(def)),
        Definition::CapabilityInterface(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::INTERFACE,
            &def.span,
            None,
        )),
        Definition::CapabilityImplementation(def) => Some(symbol(
            def.name.to_string(),
            SymbolKind::CLASS,
            &def.span,
            None,
        )),
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
            Definition::Capability(def) => def.span.start,
            Definition::Policy(def) => def.span.start,
            Definition::Role(def) => def.span.start,
            Definition::Proxy(def) => def.span.start,
            Definition::Interface(def) => def.span.start,
            Definition::CapabilityInterface(def) => def.span.start,
            Definition::CapabilityImplementation(def) => def.span.start,
            Definition::ResourceType(def) => def.span.start,
            Definition::Type(def) => def.span.start,
            Definition::DataKind(def) => def.span.start,
            Definition::TypeFn(def) => def.span.start,
            Definition::PropositionPredicate(def) => def.span.start,
            Definition::Impl(def) => def.span.start,
            Definition::Function(def) => def.span.start,
            Definition::BuiltinFn(def) => def.span.start,
            Definition::SealedDomain(def) => def.span.start,
            Definition::Law(def) => def.span.start,
            Definition::Proof(def) => def.span.start,
        };
        if let Some(sym) = definition_symbol(definition) {
            entries.push((start, sym));
        }
    }

    if let Some(workflow) = &module.workflow {
        entries.push((workflow.span.start, workflow_symbol(workflow)));
    }

    entries.sort_by_key(|(start, _)| *start);
    entries.into_iter().map(|(_, symbol)| symbol).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::module::ModuleDecl;
    use ash_parser::parse_utils::CommentTable;
    use ash_parser::surface::{
        CapabilityDef, EffectType, Expr, FnDef, InterfaceDef, InterfaceMethodSig, Literal, Param,
        Type, Visibility, Workflow,
    };

    fn span(start: usize, end: usize, line: usize, column: usize) -> Span {
        Span::new(start, end, line, column)
    }

    fn empty_module() -> ModuleFile {
        ModuleFile {
            definitions: vec![],
            module_decls: vec![],
            workflow: None,
            span: span(0, 1, 1, 1),
            comments: CommentTable::default(),
            path: None,
        }
    }

    #[test]
    fn test_document_symbols_includes_workflow_and_function() {
        let module = ModuleFile {
            definitions: vec![Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "helper".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(Type::Name("Int".into())),
                proposition_tail: None,
                contract: None,
                body: Expr::Literal(Literal::Int(1)),
                span: span(10, 20, 2, 1),
            })],
            workflow: Some(WorkflowDef {
                name: "main".into(),
                type_params: vec![],
                params: vec![],
                declared_return_type: None,
                plays_roles: vec![],
                capabilities: vec![],
                owned_resources: vec![],
                used_bindings: vec![],
                header_events: vec![],
                body: Workflow::Done {
                    span: span(30, 40, 4, 1),
                },
                contract: None,
                span: span(30, 60, 4, 1),
            }),
            ..empty_module()
        };

        let symbols = document_symbols(&module);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "helper");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[1].name, "main");
        assert_eq!(symbols[1].kind, SymbolKind::FUNCTION);
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
    fn test_document_symbols_sorted_by_source_order() {
        let module = ModuleFile {
            definitions: vec![Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "later".into(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                proposition_tail: None,
                contract: None,
                body: Expr::Literal(Literal::Int(1)),
                span: span(30, 40, 4, 1),
            })],
            workflow: Some(WorkflowDef {
                name: "first".into(),
                type_params: vec![],
                params: vec![],
                declared_return_type: None,
                plays_roles: vec![],
                capabilities: vec![],
                owned_resources: vec![],
                used_bindings: vec![],
                header_events: vec![],
                body: Workflow::Done {
                    span: span(10, 20, 2, 1),
                },
                contract: None,
                span: span(10, 25, 2, 1),
            }),
            ..empty_module()
        };

        let symbols = document_symbols(&module);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "first");
        assert_eq!(symbols[1].name, "later");
    }

    #[test]
    fn test_workspace_symbols_finds_top_level_names() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("lib.ash"), "fn helper() -> Int { 1 }\n").unwrap();

        let results = workspace_symbols(dir.path(), "helper");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "helper");
        assert_eq!(results[0].kind, SymbolKind::FUNCTION);
        assert_eq!(results[0].line, 1);
    }

    #[test]
    fn test_workspace_symbols_case_insensitive_substring() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("lib.ash"),
            "fn HelperFunction() -> Int { 1 }\n",
        )
        .unwrap();

        let results = workspace_symbols(dir.path(), "help");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "HelperFunction");
    }

    #[test]
    fn test_workspace_symbols_recurses_into_subdirectories() {
        let dir = tempfile::tempdir().expect("temp dir");
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("nested.ash"), "fn nested_helper() -> Int { 1 }\n").unwrap();

        let results = workspace_symbols(dir.path(), "nested");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "nested_helper");
    }

    #[test]
    fn test_workspace_symbols_ignores_non_ash_files() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("lib.txt"), "fn helper() -> Int { 1 }\n").unwrap();

        let results = workspace_symbols(dir.path(), "helper");
        assert!(results.is_empty());
    }

    #[test]
    fn test_workspace_symbols_returns_empty_for_no_matches() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("lib.ash"), "fn helper() -> Int { 1 }\n").unwrap();

        let results = workspace_symbols(dir.path(), "missing");
        assert!(results.is_empty());
    }

    #[test]
    fn test_workspace_symbols_includes_interface_children() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("lib.ash"),
            "interface Reader { read(String) -> String }\n",
        )
        .unwrap();

        let results = workspace_symbols(dir.path(), "read");
        assert_eq!(results.len(), 2);
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Reader"));
        assert!(names.contains(&"read"));
    }

    #[test]
    fn test_workspace_symbols_handles_parse_errors_gracefully() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("bad.ash"), "this is not valid ash {{\n").unwrap();
        std::fs::write(dir.path().join("good.ash"), "fn helper() -> Int { 1 }\n").unwrap();

        let results = workspace_symbols(dir.path(), "helper");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "helper");
    }
}
