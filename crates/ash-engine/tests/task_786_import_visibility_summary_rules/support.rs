#![allow(unused_imports)]

//! TASK-786 regression tests for import/pub-use/glob visibility and summary transport.

pub use ash_core::ast::{TypeBody, Visibility};
pub use ash_engine::module_loader::load_ordinary_file;

pub fn imported_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_type_defs
        .iter()
        .map(|type_def| type_def.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

pub fn semantic_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .map(|ty| ty.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

pub fn semantic_constructor_names(
    loaded: &ash_engine::module_loader::LoadedOrdinaryFile,
) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_constructors.iter())
        .map(|constructor| constructor.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

pub fn check_file(path: &std::path::Path) -> Result<(), String> {
    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|error| error.to_string())?;
    let mut workflow = engine.parse_file(path).map_err(|error| error.to_string())?;
    engine
        .check(&mut workflow)
        .map_err(|error| error.to_string())
}

pub fn check_module_file(path: &std::path::Path) -> Result<(), String> {
    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|error| error.to_string())?;
    let result = engine
        .check_module_file(path)
        .map_err(|error| error.to_string())?;
    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(result.errors.join("\n"))
    }
}
