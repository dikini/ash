//! TASK-870: source-facing explicit associated-family projections in public type positions.

use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn task_870_module_loader_accepts_explicit_family_projection_in_public_type_alias() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub interface Iterator<I> { sealed type family Item: Type }
impl<T> Iterator<List<T>> { type Item = T; }
pub type Projected = <Iterator<List<String>>>::Item;
",
    );
    write_file(
        &caller,
        r"use provider::*
fn main() { 0 }
",
    );

    load_ordinary_file(&caller)
        .expect("explicit associated-family projection should lower in public type aliases");
}
