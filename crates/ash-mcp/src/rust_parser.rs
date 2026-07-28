//! Rust source file parser for cross-language symbol location finding.
//!
//! This module parses Rust source files with `syn` and locates symbols
//! (enums, structs, traits, functions, type aliases, and modules) from the
//! syntax tree rather than from comments or string literals.

use std::path::{Path, PathBuf};

use proc_macro2::Span;

/// Error type for Rust source parsing
#[derive(Debug, thiserror::Error)]
pub enum RustParseError {
    /// I/O error reading the file
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Rust source could not be parsed
    #[error("Rust parse error: {0}")]
    Parse(#[from] syn::Error),

    /// Symbol was not found in the file
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
}

/// Location of a symbol in a Rust source file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustSymbolLocation {
    /// Absolute path to the Rust source file
    pub file: PathBuf,
    /// Start line (1-indexed)
    pub start_line: u32,
    /// Start column (1-indexed)
    pub start_column: u32,
    /// End line (1-indexed)
    pub end_line: u32,
    /// End column (1-indexed)
    pub end_column: u32,
}

/// Parse a Rust source file and find the location of a symbol.
///
/// Returns the line/column range of the symbol declaration if found.
///
/// # Errors
///
/// Returns `RustParseError::Io` if the file cannot be read, or
/// `RustParseError::Parse` if the Rust source cannot be parsed by `syn`.
pub fn find_symbol_location(
    file_path: &Path,
    symbol_name: &str,
) -> Result<Option<RustSymbolLocation>, RustParseError> {
    let content = std::fs::read_to_string(file_path)?;
    let syntax = syn::parse_file(&content)?;
    let query = RustSymbolQuery::new(symbol_name);

    find_in_items(file_path, &syntax.items, &query)
}

#[derive(Debug)]
struct RustSymbolQuery<'a> {
    container: Option<&'a str>,
    symbol: &'a str,
}

impl<'a> RustSymbolQuery<'a> {
    fn new(symbol_name: &'a str) -> Self {
        let mut parts = symbol_name.rsplitn(2, "::");
        let symbol = parts.next().unwrap_or(symbol_name);
        let container = parts.next();
        Self { container, symbol }
    }
}

fn find_in_items(
    file_path: &Path,
    items: &[syn::Item],
    query: &RustSymbolQuery<'_>,
) -> Result<Option<RustSymbolLocation>, RustParseError> {
    for item in items {
        if let Some(span) = item_symbol_span(item, query) {
            return Ok(Some(location_from_span(file_path, span)));
        }

        match item {
            syn::Item::Mod(module) => {
                if let Some((_, nested_items)) = &module.content
                    && let Some(location) = find_in_items(file_path, nested_items, query)?
                {
                    return Ok(Some(location));
                }
            }
            syn::Item::Impl(impl_block) => {
                if query
                    .container
                    .is_none_or(|container| impl_self_type_matches(impl_block, container))
                {
                    for impl_item in &impl_block.items {
                        if let Some(span) = impl_item_symbol_span(impl_item, query.symbol) {
                            return Ok(Some(location_from_span(file_path, span)));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn item_symbol_span(item: &syn::Item, query: &RustSymbolQuery<'_>) -> Option<Span> {
    if let Some(container) = query.container {
        return match item {
            syn::Item::Enum(item) if item.ident == container => item
                .variants
                .iter()
                .find(|variant| variant.ident == query.symbol)
                .map(|variant| variant.ident.span()),
            syn::Item::Trait(item) if item.ident == container => item
                .items
                .iter()
                .find_map(|trait_item| trait_item_symbol_span(trait_item, query.symbol)),
            _ => None,
        };
    }

    match item {
        syn::Item::Struct(item) if item.ident == query.symbol => Some(item.ident.span()),
        syn::Item::Enum(item) if item.ident == query.symbol => Some(item.ident.span()),
        syn::Item::Trait(item) if item.ident == query.symbol => Some(item.ident.span()),
        syn::Item::Type(item) if item.ident == query.symbol => Some(item.ident.span()),
        syn::Item::Fn(item) if item.sig.ident == query.symbol => Some(item.sig.ident.span()),
        syn::Item::Mod(item) if item.ident == query.symbol => Some(item.ident.span()),
        _ => None,
    }
}

fn impl_self_type_matches(impl_block: &syn::ItemImpl, container: &str) -> bool {
    match impl_block.self_ty.as_ref() {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == container),
        _ => false,
    }
}

fn trait_item_symbol_span(item: &syn::TraitItem, symbol_name: &str) -> Option<Span> {
    match item {
        syn::TraitItem::Fn(item) if item.sig.ident == symbol_name => Some(item.sig.ident.span()),
        syn::TraitItem::Type(item) if item.ident == symbol_name => Some(item.ident.span()),
        syn::TraitItem::Const(item) if item.ident == symbol_name => Some(item.ident.span()),
        _ => None,
    }
}

fn impl_item_symbol_span(item: &syn::ImplItem, symbol_name: &str) -> Option<Span> {
    match item {
        syn::ImplItem::Fn(item) if item.sig.ident == symbol_name => Some(item.sig.ident.span()),
        syn::ImplItem::Type(item) if item.ident == symbol_name => Some(item.ident.span()),
        syn::ImplItem::Const(item) if item.ident == symbol_name => Some(item.ident.span()),
        _ => None,
    }
}

fn location_from_span(file_path: &Path, span: Span) -> RustSymbolLocation {
    let start = span.start();
    let end = span.end();
    RustSymbolLocation {
        file: file_path.to_path_buf(),
        start_line: usize_to_u32(start.line),
        start_column: usize_to_u32(start.column.saturating_add(1)),
        end_line: usize_to_u32(end.line),
        end_column: usize_to_u32(end.column.saturating_add(1)),
    }
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// Find a Rust source file corresponding to a `crate::module::symbol` path.
///
/// Resolves crate names (e.g., `ash_core` → `ash-core`) and attempts common file patterns.
///
/// # Errors
///
/// Returns `RustParseError::Io` if the workspace cannot be accessed.
pub fn find_rust_file_for_symbol(
    workspace_root: &Path,
    qualified_symbol: &str,
) -> Result<Option<PathBuf>, RustParseError> {
    let parts: Vec<&str> = qualified_symbol.split("::").collect();

    if parts.len() < 2 {
        return Ok(None);
    }

    // Convert ash_core -> ash-core, ash_runtime -> ash-runtime
    let crate_name = parts[0].replace('_', "-");

    // Try progressively shorter module paths. Associated items such as
    // ash_core::effect::Effect::join should resolve to effect.rs, not
    // effect/Effect.rs.
    let mut candidates = Vec::new();
    for end in (2..parts.len()).rev() {
        let module_path = parts[1..end].join("/");
        candidates.push(format!("crates/{crate_name}/src/{module_path}.rs"));
        candidates.push(format!("crates/{crate_name}/src/{module_path}/mod.rs"));
    }
    candidates.push(format!("crates/{crate_name}/src/lib.rs"));

    for candidate in candidates {
        let full_path = workspace_root.join(&candidate);
        if full_path.exists() {
            return Ok(Some(full_path));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_rust(content: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(".rs")
            .tempfile()
            .expect("create temp rust file");
        file.write_all(content.as_bytes())
            .expect("write temp rust file");
        file
    }

    #[test]
    fn test_find_rust_file_for_symbol() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = find_rust_file_for_symbol(workspace, "ash_core::effect::Effect");
        assert!(result.unwrap().is_some());

        let result = find_rust_file_for_symbol(workspace, "nonexistent::foo::Bar");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_symbol_location_in_real_file() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let file_path = workspace.join("crates/ash-core/src/effect.rs");
        if !file_path.exists() {
            return; // Skip test if file doesn't exist
        }

        let result = find_symbol_location(&file_path, "Effect");
        if let Ok(Some(loc)) = result {
            assert!(loc.start_line > 0);
            assert!(loc.start_column > 0);
        }
    }

    #[test]
    fn test_syn_parser_ignores_comment_and_string_false_positives() {
        let file = write_temp_rust(
            r#"
// struct FakeComment;
const TEXT: &str = "enum FakeString { Value }";

pub enum RealSymbol {
    Value,
}
"#,
        );

        let comment = find_symbol_location(file.path(), "FakeComment").unwrap();
        assert!(comment.is_none(), "comments must not produce symbols");

        let string = find_symbol_location(file.path(), "FakeString").unwrap();
        assert!(string.is_none(), "string literals must not produce symbols");

        let real = find_symbol_location(file.path(), "RealSymbol").unwrap();
        assert!(real.is_some(), "real enum declaration should be found");
    }

    #[test]
    fn test_syn_parser_finds_trait_items_and_container_specific_impl_items() {
        let file = write_temp_rust(
            r"
pub trait Runnable { fn new(&self); }
pub struct A;
pub struct B;
impl A { pub fn new() -> Self { Self } }
impl B { pub fn new() -> Self { Self } }
",
        );

        let trait_method = find_symbol_location(file.path(), "Runnable::new")
            .unwrap()
            .expect("trait method should be found");
        assert_eq!(trait_method.start_line, 2);

        let b_method = find_symbol_location(file.path(), "B::new")
            .unwrap()
            .expect("B::new should be found");
        assert_eq!(b_method.start_line, 6);
    }

    #[test]
    fn test_syn_parser_finds_enum_variants_with_container() {
        let file = write_temp_rust(
            r"
pub enum Effect {
    Epistemic,
    Operational,
}
",
        );

        let variant = find_symbol_location(file.path(), "Effect::Operational")
            .unwrap()
            .expect("enum variant should be found");
        assert_eq!(variant.start_line, 4);
    }

    #[test]
    fn test_syn_parser_finds_supported_item_kinds() {
        let file = write_temp_rust(
            r"
pub struct Widget;
pub enum Mode { A }
pub trait Runnable { fn run(&self); }
pub type WidgetId = u64;
pub fn helper() {}
pub mod nested {}
impl Widget { pub fn new() -> Self { Self } }
",
        );

        for symbol in [
            "Widget", "Mode", "Runnable", "WidgetId", "helper", "nested", "new",
        ] {
            let location = find_symbol_location(file.path(), symbol).unwrap();
            assert!(location.is_some(), "expected to find {symbol}");
        }
    }
}
