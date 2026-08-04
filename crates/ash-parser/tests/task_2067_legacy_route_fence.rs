//! TASK-2067 architecture-fence evidence for the canonical module graph.
//!
//! This is deliberately not a language-semantics test.  The source-layout
//! assertion below keeps the canonical graph implementation separate from the
//! compatibility-only legacy resolver so the old path cannot silently become
//! an adapter used by the new structural route.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::CanonicalModuleGraphResolver;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A filesystem fixture whose drop implementation removes its tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2067-legacy-fence-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary module tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create fixture parent directory");
        fs::write(&path, source).expect("write module fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn imports_legacy_module_resolver(source: &str) -> bool {
    let mut remaining = source;
    while let Some(import_start) = remaining.find("use crate::resolver") {
        let import = &remaining[import_start..];
        let statement_end = import.find(';').unwrap_or(import.len());
        let statement = &import[..statement_end];
        if contains_exact_identifier(statement, "ModuleResolver") {
            return true;
        }
        remaining = &import[statement_end..];
    }
    false
}

fn contains_exact_identifier(source: &str, identifier: &str) -> bool {
    source.match_indices(identifier).any(|(start, _)| {
        let before_is_identifier = source[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        let after_is_identifier = source[start + identifier.len()..]
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        !before_is_identifier && !after_is_identifier
    })
}

#[test]
fn canonical_resolution_uses_real_file_and_inline_units_without_legacy_graph_types() {
    let tree = TempTree::new("canonical-smoke");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            mod file_child;
            mod inline_child { mod nested { fn leaf() {} } }
        "#,
    );
    tree.write("src/file_child.ash", "mod nested { fn leaf() {} }");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let file_key = root_key.child("file_child").expect("fixture child key");
    let inline_key = root_key.child("inline_child").expect("fixture child key");
    let file_nested = file_key.child("nested").expect("fixture grandchild key");
    let inline_nested = inline_key.child("nested").expect("fixture grandchild key");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), &root_path)
        .expect("parsed source should resolve through the canonical graph route");

    assert_eq!(graph.root_key(), &root_key);
    assert_eq!(
        graph.children(&root_key),
        Some([file_key.clone(), inline_key.clone()].as_slice())
    );
    assert_eq!(graph.children(&file_key), Some([file_nested].as_slice()));
    assert_eq!(
        graph.children(&inline_key),
        Some([inline_nested].as_slice())
    );
    assert_eq!(
        graph
            .module_unit(&file_key)
            .expect("file child is an acquired canonical unit")
            .artifact()
            .key(),
        &file_key
    );
    assert_eq!(
        graph
            .module_unit(&inline_key)
            .expect("inline child is an acquired canonical unit")
            .artifact()
            .key(),
        &inline_key
    );
}

#[test]
fn canonical_graph_implementation_is_isolated_from_the_deprecated_legacy_surface() {
    // This architecture fence intentionally inspects source layout rather than
    // language behavior.  Behavioral semantics belong in the canonical graph
    // and SPEC-103 fixtures, not in this dependency-boundary assertion.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canonical_source_path = manifest_dir.join("src/canonical_module_graph.rs");
    let canonical_source = fs::read_to_string(&canonical_source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2067 requires a dedicated canonical graph implementation at {}: {error}",
            canonical_source_path.display()
        )
    });

    for forbidden_legacy_identifier in [
        "ModuleGraph",
        "ModuleId",
        "ModuleNode",
        "LegacyModuleResolver",
    ] {
        assert!(
            !contains_exact_identifier(&canonical_source, forbidden_legacy_identifier),
            "the canonical graph module must not mention or adapt the legacy route: {forbidden_legacy_identifier}"
        );
    }
    for forbidden_legacy_adapter in [
        "into_legacy",
        "from_legacy",
        "legacy_adapter",
        "legacy_conversion",
    ] {
        assert!(
            !canonical_source.contains(forbidden_legacy_adapter),
            "the canonical graph module must not mention or adapt the legacy route: {forbidden_legacy_adapter}"
        );
    }
    assert!(
        !canonical_source.contains("crate::resolver::ModuleResolver"),
        "the canonical graph module must not reference the legacy resolver directly"
    );
    assert!(
        !imports_legacy_module_resolver(&canonical_source),
        "the canonical graph module must not import the legacy resolver from `crate::resolver`"
    );

    let lib_source_path = manifest_dir.join("src/lib.rs");
    let lib_source = fs::read_to_string(&lib_source_path)
        .expect("ash-parser lib source must remain available to the architecture fence");
    let lines = lib_source.lines().collect::<Vec<_>>();
    let legacy_surface = lines
        .iter()
        .position(|line| line.contains("LegacyModuleResolver"))
        .expect("lib.rs must expose a named `LegacyModuleResolver` compatibility surface");
    assert!(
        lines[..legacy_surface]
            .iter()
            .rev()
            .take(4)
            .any(|line| line.trim_start().starts_with("///")),
        "the named legacy compatibility surface must be documented"
    );

    let deprecated_alias = lines.iter().enumerate().find_map(|(index, line)| {
        line.contains("ModuleResolver")
            .then_some(index)
            .filter(|index| {
                lines[..*index]
                    .iter()
                    .rev()
                    .take(4)
                    .any(|line| line.trim_start().starts_with("#[deprecated"))
            })
    });
    let Some(deprecated_alias) = deprecated_alias else {
        panic!("lib.rs must retain an explicitly deprecated `ModuleResolver` alias or re-export");
    };
    assert!(
        lines[..deprecated_alias]
            .iter()
            .rev()
            .take(6)
            .any(|line| line.trim_start().starts_with("///")),
        "the retained deprecated `ModuleResolver` alias or re-export must be documented"
    );
}
