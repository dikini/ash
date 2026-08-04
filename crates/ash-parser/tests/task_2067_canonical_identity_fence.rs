//! TASK-2067 mutation and implementation-boundary evidence for canonical identity.
//!
//! The behavioral tests exercise a real parsed module graph. The source-layout
//! assertion deliberately checks an implementation boundary; it is not a
//! language-source scan and does not grant language-semantic authority.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::{CanonicalModuleGraphResolver, CanonicalModuleState};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real filesystem fixture whose drop implementation removes its tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2067-identity-fence-{label}-{}-{serial}",
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

#[test]
fn path_shaped_canonical_key_cannot_address_a_parsed_file_or_inline_graph_entry() {
    let tree = TempTree::new("wrong-key");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            mod file_child;
            mod inline_child { fn inline_leaf() {} }
        "#,
    );
    tree.write("src/file_child.ash", "fn file_leaf() {}\n");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let wrong_path_shaped_key = root_key
        .child("src-main")
        .expect("path-shaped segment remains a valid canonical module key");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key, &root_path)
        .expect("the parsed file and inline children form a canonical graph");

    assert!(
        graph.children(&wrong_path_shaped_key).is_none(),
        "a path-shaped key must not be repaired into a structural child lookup"
    );
    assert!(
        graph.module_unit(&wrong_path_shaped_key).is_none(),
        "a path-shaped key must not retrieve a source-acquired module unit"
    );
    assert_eq!(
        graph.state(&wrong_path_shaped_key),
        None,
        "a path-shaped key must not retrieve an inferred graph state"
    );
    assert_eq!(
        graph.state_or_absent(&wrong_path_shaped_key),
        CanonicalModuleState::Absent,
        "the canonical state machine must report a nondeclared key as absent without exposing a partial graph"
    );
}

#[test]
fn reparsing_a_renamed_declaration_with_one_resolver_rewrites_only_canonical_graph_entries() {
    let tree = TempTree::new("parsed-rewrite");
    let root_path = tree.write("src/main.ash", "mod old;\n");
    tree.write("src/old.ash", "fn old_leaf() {}\n");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let old_key = root_key.child("old").expect("fixture old key is canonical");
    let renamed_key = root_key
        .child("renamed")
        .expect("fixture renamed key is canonical");
    let resolver = CanonicalModuleGraphResolver::new();

    let before_rename = resolver
        .resolve_root(root_key.clone(), &root_path)
        .expect("initial parsed declaration resolves through the canonical graph");
    assert_eq!(
        before_rename.children(&root_key),
        Some([old_key.clone()].as_slice())
    );
    assert!(before_rename.module_unit(&old_key).is_some());
    assert_eq!(
        before_rename.state(&old_key),
        Some(CanonicalModuleState::Parsed)
    );

    tree.write("src/main.ash", "mod renamed;\n");
    tree.write("src/renamed.ash", "fn renamed_leaf() {}\n");

    let after_rename = resolver
        .resolve_root(root_key.clone(), &root_path)
        .expect("the rewritten parsed declaration resolves through the same resolver instance");

    assert_eq!(
        after_rename.children(&root_key),
        Some([renamed_key.clone()].as_slice()),
        "the parsed rewrite must replace topology with the renamed canonical child"
    );
    assert!(
        after_rename.module_unit(&renamed_key).is_some(),
        "the rewritten declaration must retain only its newly acquired unit"
    );
    assert_eq!(
        after_rename.state(&renamed_key),
        Some(CanonicalModuleState::Parsed)
    );
    assert!(
        after_rename.children(&old_key).is_none(),
        "the old canonical key must not survive a source-declaration rewrite"
    );
    assert!(
        after_rename.module_unit(&old_key).is_none(),
        "the old canonical key must not retrieve a stale source unit"
    );
    assert_eq!(
        after_rename.state(&old_key),
        None,
        "the old canonical key must not retrieve a stale graph state"
    );
}

fn function_body<'source>(source: &'source str, function_name: &str) -> &'source str {
    let signature = format!("fn {function_name}");
    let function_start = source
        .find(&signature)
        .unwrap_or_else(|| panic!("resolver source must define `{signature}`"));
    let body_start = source[function_start..]
        .find('{')
        .map(|offset| function_start + offset)
        .unwrap_or_else(|| panic!("resolver function `{function_name}` must have a body"));

    let mut depth = 0_usize;
    for (offset, byte) in source[body_start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start + 1..body_start + offset];
                }
            }
            _ => {}
        }
    }

    panic!("resolver function `{function_name}` must have balanced braces");
}

fn compact_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn assert_canonical_duplicate_tracking(function_name: &str, body: &str) {
    let compact = compact_whitespace(body);
    let canonical_map_type = [
        "HashMap<ModuleKey,Span>",
        "HashMap::<ModuleKey,Span>",
        "BTreeMap<ModuleKey,Span>",
        "BTreeMap::<ModuleKey,Span>",
    ]
    .iter()
    .any(|candidate| compact.contains(candidate));
    assert!(
        canonical_map_type,
        "{function_name} must track duplicate declarations by `ModuleKey`, not a raw declaration name"
    );
    assert!(
        !compact.contains(".insert(declaration.name.clone(),")
            && !compact.contains(".insert(declaration.name,"),
        "{function_name} must not use `declaration.name` as the duplicate-tracking identity"
    );
}

#[test]
fn module_unit_duplicate_tracking_uses_canonical_module_keys() {
    // This is a test-only implementation-boundary check, not a language-source
    // scan. Its job is to prevent raw-name duplicate identity from silently
    // returning beneath the parser's canonical graph route.
    let resolver_source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/resolver.rs");
    let resolver_source = fs::read_to_string(&resolver_source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2067 resolver source must remain available at {}: {error}",
            resolver_source_path.display()
        )
    });

    for function_name in ["root_unit_from_body", "build_artifact"] {
        let body = function_body(&resolver_source, function_name);
        assert_canonical_duplicate_tracking(function_name, body);
    }
}
