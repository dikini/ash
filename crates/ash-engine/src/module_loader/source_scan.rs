//! Source scanning helpers for module loading.

use std::path::{Path, PathBuf};

use super::import_needs_more_lines;
use crate::error::EngineError;

pub(super) fn extract_pub_mod_declarations(source: &str) -> Vec<String> {
    extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub mod "))
        .iter()
        .filter_map(|snippet| {
            let trimmed = snippet.trim();
            trimmed
                .strip_prefix("pub mod ")
                .map(str::trim)
                .filter(|rest| !rest.contains('{'))
                .map(|rest| rest.trim_end_matches(';').trim().to_string())
        })
        .filter(|name| !name.is_empty())
        .collect()
}

pub(super) fn resolve_child_module(module_path: &Path, name: &str) -> Result<PathBuf, EngineError> {
    let module_root = module_path.parent().unwrap_or_else(|| Path::new("."));

    // If parent is a file module (not `mod.ash`), child modules live in a
    // subdirectory named after the parent module. For example, `test.ash`
    // with `pub mod quickcheck;` looks for `test/quickcheck.ash` or
    // `test/quickcheck/mod.ash`.
    let parent_is_dir_module = module_path
        .file_name()
        .is_some_and(|name| name == "mod.ash");
    let search_dir = if parent_is_dir_module {
        module_root.to_path_buf()
    } else if let Some(stem) = module_path.file_stem() {
        module_root.join(stem)
    } else {
        module_root.to_path_buf()
    };

    // Try name.ash first, then name/mod.ash in the subdirectory
    let file_candidate = search_dir.join(format!("{name}.ash"));
    if file_candidate.is_file() {
        return Ok(file_candidate);
    }
    let mod_candidate = search_dir.join(name).join("mod.ash");
    if mod_candidate.is_file() {
        return Ok(mod_candidate);
    }

    // Fallback: try sibling module in the same directory (flat structure)
    let sibling_file = module_root.join(format!("{name}.ash"));
    if sibling_file.is_file() {
        return Ok(sibling_file);
    }
    let sibling_mod = module_root.join(name).join("mod.ash");
    if sibling_mod.is_file() {
        return Ok(sibling_mod);
    }

    Err(EngineError::Parse(format!(
        "pub mod '{name}': module not found (searched {}, {}, {}, {})",
        file_candidate.display(),
        mod_candidate.display(),
        sibling_file.display(),
        sibling_mod.display()
    )))
}

pub(super) fn extract_import_snippets(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") || trimmed.starts_with("pub use ") {
            index += 1;
            continue;
        }

        if trimmed.starts_with("use ") {
            let mut snippet = lines[index].to_string();
            while import_needs_more_lines(&snippet) {
                index += 1;
                if index >= lines.len() {
                    break;
                }
                snippet.push('\n');
                snippet.push_str(lines[index]);
            }
            snippets.push(snippet);
        }

        index += 1;
    }

    snippets
}

pub(super) fn extract_pub_use_snippets(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") {
            index += 1;
            continue;
        }
        if trimmed.starts_with("pub use ") {
            let mut snippet = lines[index].to_string();
            while import_needs_more_lines(&snippet) {
                index += 1;
                if index >= lines.len() {
                    break;
                }
                snippet.push('\n');
                snippet.push_str(lines[index]);
            }
            snippets.push(snippet);
        }
        index += 1;
    }

    snippets
}

pub(super) fn extract_semicolon_snippets<F>(source: &str, predicate: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") {
            index += 1;
            continue;
        }

        if predicate(trimmed) {
            let mut snippet = String::new();
            while index < lines.len() {
                if !snippet.is_empty() {
                    snippet.push('\n');
                }
                snippet.push_str(lines[index]);
                if lines[index].contains(';') {
                    snippets.push(snippet);
                    break;
                }
                index += 1;
            }
        }

        index += 1;
    }

    snippets
}

/// Extract braced code snippets from a source string whose first line matches
/// the given predicate.
///
/// Walks the source line-by-line looking for lines that satisfy `predicate`,
/// then consumes the matching brace-delimited block and returns it as a
/// standalone snippet string.
pub fn extract_braced_snippets<F>(source: &str, predicate: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") {
            index += 1;
            continue;
        }

        if predicate(trimmed) {
            let mut snippet = String::new();
            let mut brace_depth = 0usize;
            let mut seen_open = false;

            while index < lines.len() {
                if !snippet.is_empty() {
                    snippet.push('\n');
                }
                snippet.push_str(lines[index]);

                for ch in lines[index].chars() {
                    match ch {
                        '{' => {
                            brace_depth += 1;
                            seen_open = true;
                        }
                        '}' if brace_depth > 0 => {
                            brace_depth -= 1;
                        }
                        _ => {}
                    }
                }

                if seen_open && brace_depth == 0 {
                    snippets.push(snippet);
                    break;
                }

                index += 1;
            }
        }

        index += 1;
    }

    snippets
}
