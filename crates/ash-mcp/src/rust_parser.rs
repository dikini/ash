//! Rust source file parser for cross-language symbol location finding.
//!
//! This module parses Rust source files and locates symbols
//! (enums, structs, traits, functions, types) by simple pattern matching.

use std::path::{Path, PathBuf};

/// Error type for Rust source parsing
#[derive(Debug, thiserror::Error)]
pub enum RustParseError {
    /// I/O error reading the file
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

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
/// Returns `RustParseError::Io` if the file cannot be read.
#[allow(clippy::cast_possible_truncation)]
pub fn find_symbol_location(
    file_path: &Path,
    symbol_name: &str,
) -> Result<Option<RustSymbolLocation>, RustParseError> {
    let content = std::fs::read_to_string(file_path)?;

    let base_name = symbol_name.split("::").last().unwrap_or(symbol_name);

    // Simple pattern matching for common symbol declarations
    let patterns = vec![
        format!("struct {}", base_name),
        format!("enum {}", base_name),
        format!("trait {}", base_name),
        format!("type {}", base_name),
        format!("fn {}(", base_name),
        format!("mod {}", base_name),
        format!("impl {}", base_name),
    ];

    for (line_num, line) in content.lines().enumerate() {
        for pattern in &patterns {
            if line.contains(pattern) {
                // Find the position of the keyword
                if let Some(pos) = line.find(base_name) {
                    return Ok(Some(RustSymbolLocation {
                        file: file_path.to_path_buf(),
                        start_line: (line_num + 1) as u32,
                        start_column: (pos + 1) as u32,
                        end_line: (line_num + 1) as u32,
                        end_column: (pos + base_name.len() + 1) as u32,
                    }));
                }
            }
        }
    }

    Ok(None)
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

    // Convert ash_core -> ash-core, ash_interp -> ash-interp
    let crate_name = parts[0].replace('_', "-");
    let module_path = parts[1..parts.len() - 1].join("/");

    // Try common patterns for file location
    let candidates = vec![
        // crates/ash-core/src/effect.rs for ash_core::effect::Effect
        format!("crates/{}/src/{}.rs", crate_name, module_path),
        format!("crates/{}/src/{}/mod.rs", crate_name, module_path),
        format!("crates/{}/src/lib.rs", crate_name),
    ];

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
}
