#![allow(unused_imports)]

pub use std::fs;
pub use std::path::PathBuf;

pub use ash_parser::surface::{Expr, FnDef, Type as SurfaceType};
pub use ash_parser::{
    Definition, Workflow, input::new_input, parse_module::parse_fn_definition,
    parse_type_def::parse_type_def, parse_use, parse_utils::skip_whitespace_and_comments,
    workflow_def,
};
pub use winnow::prelude::*;

/// Get the path to the stdlib source directory
pub fn stdlib_src_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("src");
    path
}

/// Helper to read and return a stdlib file's content
pub fn read_stdlib_file(filename: &str) -> String {
    let path = stdlib_src_path().join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

pub fn normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn contains_public_callable(source: &str, name: &str) -> bool {
    source.contains(&format!("pub fn {name}")) || source.contains(&format!("pub builtin fn {name}"))
}

pub fn extract_public_fn_sources(source: &str) -> Vec<String> {
    let mut functions = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut brace_depth = 0usize;
    let mut saw_open_brace = false;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if !capturing && trimmed.starts_with("pub fn ") {
            capturing = true;
            current.clear();
            brace_depth = 0;
            saw_open_brace = false;
        }

        if capturing {
            current.push(line);
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        saw_open_brace = true;
                    }
                    '}' => {
                        brace_depth = brace_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }

            if saw_open_brace && brace_depth == 0 {
                functions.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }

    functions
}

pub fn parse_public_functions(filename: &str) -> Vec<FnDef> {
    extract_public_fn_sources(&read_stdlib_file(filename))
        .into_iter()
        .map(|source| {
            let mut input = new_input(&source);
            skip_whitespace_and_comments(&mut input);
            match parse_fn_definition.parse_next(&mut input) {
                Ok(Definition::Function(function)) => function,
                Ok(other) => panic!("expected function definition, got {other:?}"),
                Err(error) => panic!("failed to parse function definition {source:?}: {error}"),
            }
        })
        .collect()
}
