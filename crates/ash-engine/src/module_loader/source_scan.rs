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

pub(super) fn resolve_child_module(module_root: &Path, name: &str) -> Result<PathBuf, EngineError> {
    // Try name.ash first, then name/mod.ash
    let file_candidate = module_root.join(format!("{name}.ash"));
    if file_candidate.is_file() {
        return Ok(file_candidate);
    }
    let mod_candidate = module_root.join(name).join("mod.ash");
    if mod_candidate.is_file() {
        return Ok(mod_candidate);
    }
    Err(EngineError::Parse(format!(
        "pub mod '{name}': module not found (searched {} and {})",
        file_candidate.display(),
        mod_candidate.display()
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
