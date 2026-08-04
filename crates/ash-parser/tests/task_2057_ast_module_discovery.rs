//! TASK-2057 integration evidence for AST-owned file-module discovery.
//!
//! The resolver must obtain structural file-child declarations exclusively from
//! the parser's `ModuleFile`; text inside comments and literals is not a
//! declaration authority.

use ash_core::module_graph::ModuleSource as GraphModuleSource;
use ash_parser::{
    DiscoveredModuleSource, Fs, LegacyModuleResolver, ModuleSource, ResolveError, Visibility,
    discover_module_declarations, parse_surface_file,
};
use proptest::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A small public resolver test filesystem.
struct MockFs {
    files: HashMap<PathBuf, String>,
}

impl MockFs {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn with_file(mut self, path: impl AsRef<Path>, content: impl Into<String>) -> Self {
        self.files
            .insert(path.as_ref().to_path_buf(), content.into());
        self
    }
}

impl Fs for MockFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }
}

/// A filesystem probe that makes root-source acquisition observable.
struct CountedFs {
    files: HashMap<PathBuf, String>,
    reads: Arc<Mutex<HashMap<PathBuf, usize>>>,
    existence_checks: Arc<Mutex<HashMap<PathBuf, usize>>>,
}

impl CountedFs {
    fn with_file(
        path: impl AsRef<Path>,
        content: impl Into<String>,
        reads: Arc<Mutex<HashMap<PathBuf, usize>>>,
        existence_checks: Arc<Mutex<HashMap<PathBuf, usize>>>,
    ) -> Self {
        let mut files = HashMap::new();
        files.insert(path.as_ref().to_path_buf(), content.into());
        Self {
            files,
            reads,
            existence_checks,
        }
    }
}

impl Fs for CountedFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        let mut reads = self.reads.lock().expect("test read counter lock");
        *reads.entry(path.to_path_buf()).or_default() += 1;
        self.files.get(path).cloned()
    }

    fn file_exists(&self, path: &Path) -> bool {
        let mut existence_checks = self
            .existence_checks
            .lock()
            .expect("test existence counter lock");
        *existence_checks.entry(path.to_path_buf()).or_default() += 1;
        self.files.contains_key(path)
    }
}

#[test]
fn parsed_module_file_exposes_structural_declarations_for_later_module_tasks() {
    let source = r#"
        pub(crate) mod file_child;
        pub(in crate::nested) mod inline_child { interface Child { read() -> Unit } }
    "#;
    let module_file = parse_surface_file(source).expect("module declarations should parse");
    let discovered = discover_module_declarations(&module_file, Path::new("src/root.ash"))
        .expect("parsed declarations should become a structural handoff");

    assert_eq!(discovered.len(), 2);

    let file = &discovered[0];
    assert_eq!(file.name.as_ref(), "file_child");
    assert_eq!(file.visibility, Visibility::Crate);
    assert_eq!(file.source, DiscoveredModuleSource::File);
    assert_eq!(file.span, module_file.module_decls[0].span);
    assert_eq!(file.path, Path::new("src/root.ash"));

    let inline = &discovered[1];
    assert_eq!(inline.name.as_ref(), "inline_child");
    assert_eq!(
        inline.visibility,
        Visibility::Restricted {
            path: "crate::nested".into(),
        }
    );
    assert_eq!(inline.source, DiscoveredModuleSource::Inline);
    assert_eq!(inline.span, module_file.module_decls[1].span);
    assert_eq!(inline.path, Path::new("src/root.ash"));
}

#[test]
fn parsed_file_declarations_create_the_resolver_child_edges() {
    let source = r#"
        mod private_child;
        pub mod public_child;
        pub(crate) mod crate_child;
        pub(in crate::nested) mod restricted_child;
        fn Main() {}
    "#;
    let module_file = parse_surface_file(source).expect("valid file declarations should parse");
    let parsed_declarations: Vec<_> = module_file
        .module_decls
        .iter()
        .map(|declaration| {
            (
                declaration.name.as_ref(),
                declaration.visibility.clone(),
                declaration.source.clone(),
            )
        })
        .collect();
    assert_eq!(
        parsed_declarations,
        vec![
            ("private_child", Visibility::Inherited, ModuleSource::File,),
            ("public_child", Visibility::Public, ModuleSource::File),
            ("crate_child", Visibility::Crate, ModuleSource::File),
            (
                "restricted_child",
                Visibility::Restricted {
                    path: "crate::nested".into(),
                },
                ModuleSource::File,
            ),
        ]
    );

    let resolver = LegacyModuleResolver::with_fs(Box::new(
        MockFs::new()
            .with_file("main.ash", source)
            .with_file("private_child.ash", "fn Private() {}")
            .with_file("public_child.ash", "fn Public() {}")
            .with_file("crate_child.ash", "fn Crate() {}")
            .with_file("restricted_child.ash", "fn Restricted() {}"),
    ));

    let graph = resolver
        .resolve_crate("main.ash")
        .expect("parsed file declarations should resolve");
    let root = graph
        .get_root_node()
        .expect("resolved graph should have a root");
    let child_names: Vec<_> = root
        .children
        .iter()
        .map(|child| {
            graph
                .get_node(*child)
                .expect("resolver child edge should reference a graph node")
                .name
                .as_str()
        })
        .collect();
    assert_eq!(
        child_names,
        vec![
            "private_child",
            "public_child",
            "crate_child",
            "restricted_child",
        ]
    );
}

#[test]
fn comment_and_string_lookalikes_do_not_publish_module_edges() {
    let resolver = LegacyModuleResolver::with_fs(Box::new(
        MockFs::new()
            .with_file(
                "main.ash",
                r#"
                    mod declared;
                    -- mod comment_lookalike;
                    fn Main() { "mod string_lookalike;" }
                "#,
            )
            .with_file("declared.ash", "fn Declared() {}")
            .with_file("comment_lookalike.ash", "fn CommentLookalike() {}")
            .with_file("string_lookalike.ash", "fn StringLookalike() {}"),
    ));

    let graph = resolver
        .resolve_crate("main.ash")
        .expect("non-declaration text must not be resolved as a child module");
    let root = graph
        .get_root_node()
        .expect("resolved graph should have a root");
    let child_names: Vec<_> = root
        .children
        .iter()
        .map(|child| {
            graph
                .get_node(*child)
                .expect("resolver child edge should reference a graph node")
                .name
                .as_str()
        })
        .collect();

    assert_eq!(child_names, vec!["declared"]);
}

#[test]
fn malformed_module_syntax_returns_a_parser_error_before_resolution() {
    let source = "mod child\nfn Main() {}";
    assert!(
        parse_surface_file(source).is_err(),
        "the parser must reject malformed module syntax"
    );

    let resolver = LegacyModuleResolver::with_fs(Box::new(
        MockFs::new()
            .with_file("main.ash", source)
            .with_file("child.ash", "fn Child() {}"),
    ));

    let result = resolver.resolve_crate("main.ash");
    assert!(
        matches!(result, Err(ResolveError::ParseError { ref path, .. }) if path == Path::new("main.ash")),
        "the resolver must return a parser error before constructing a graph: {result:?}"
    );
}

#[test]
fn duplicate_file_module_declarations_are_rejected() {
    let source = "mod child;\nmod child;\nfn Main() {}";
    let module_file =
        parse_surface_file(source).expect("duplicate names remain parseable AST input");
    let first = module_file.module_decls[0].span;
    let duplicate = module_file.module_decls[1].span;
    let resolver = LegacyModuleResolver::with_fs(Box::new(
        MockFs::new()
            .with_file("main.ash", source)
            .with_file("child.ash", "fn Child() {}"),
    ));

    let result = resolver.resolve_crate("main.ash");
    assert!(
        matches!(
            result,
            Err(ResolveError::DuplicateModuleDeclaration {
                ref module_name,
                ref path,
                first_line,
                first_column,
                line,
                column,
            }) if module_name == "child"
                && path == Path::new("main.ash")
                && first_line == first.line
                && first_column == first.column
                && line == duplicate.line
                && column == duplicate.column
        ),
        "duplicate diagnostics must retain the declaration name, source path, and both AST anchors: {result:?}"
    );
}

#[test]
fn inline_declarations_publish_inline_structural_children_without_file_lookups() {
    let source = "mod inline { interface Child { read() -> Unit } }\nfn Main() {}";
    let module_file = parse_surface_file(source).expect("inline declaration should parse");
    let declaration_offset = module_file.module_decls[0].span.start;
    let reads = Arc::new(Mutex::new(HashMap::new()));
    let existence_checks = Arc::new(Mutex::new(HashMap::new()));
    let resolver = LegacyModuleResolver::with_fs(Box::new(CountedFs::with_file(
        "main.ash",
        source,
        Arc::clone(&reads),
        Arc::clone(&existence_checks),
    )));

    let graph = resolver
        .resolve_crate("main.ash")
        .expect("inline declarations must create structural children without a file source");
    let root_id = graph.root.expect("resolved graph should have a root");
    let root = graph.get_node(root_id).expect("root node should exist");
    assert_eq!(root.children.len(), 1);
    let inline = graph
        .get_node(root.children[0])
        .expect("inline structural child should exist");
    assert_eq!(inline.name, "inline");
    assert_eq!(
        inline.source,
        GraphModuleSource::Inline {
            parent: root_id,
            offset: declaration_offset,
        }
    );
    assert_eq!(
        existence_checks
            .lock()
            .expect("test existence counter lock")
            .get(Path::new("inline.ash")),
        None,
        "an inline declaration must never probe a file child"
    );
}

#[test]
fn crate_root_source_is_read_once_for_metadata_and_module_discovery() {
    let source = "crate app;\nfn Main() {}";
    let reads = Arc::new(Mutex::new(HashMap::new()));
    let existence_checks = Arc::new(Mutex::new(HashMap::new()));
    let resolver = LegacyModuleResolver::with_fs(Box::new(CountedFs::with_file(
        "main.ash",
        source,
        Arc::clone(&reads),
        existence_checks,
    )));

    resolver
        .resolve_crate("main.ash")
        .expect("a crate root must share one parsed source carrier");

    assert_eq!(
        reads
            .lock()
            .expect("test read counter lock")
            .get(Path::new("main.ash")),
        Some(&1),
        "crate metadata and ModuleFile discovery must not independently read the root source"
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn comments_and_literals_never_change_discovered_file_child_keys(
        lookalike in "[a-z][a-z0-9_]{0,15}",
        use_string_literal in any::<bool>(),
    ) {
        let non_declaration_line = if use_string_literal {
            format!("fn Main() {{ \"mod {lookalike};\" }}")
        } else {
            format!("-- mod {lookalike};\nfn Main() {{}}")
        };
        let source = format!("mod declared;\n{non_declaration_line}");
        let resolver = LegacyModuleResolver::with_fs(Box::new(
            MockFs::new()
                .with_file("main.ash", source)
                .with_file("declared.ash", "fn Declared() {}")
                .with_file(format!("{lookalike}.ash"), "fn Lookalike() {}"),
        ));

        let graph = resolver
            .resolve_crate("main.ash")
            .expect("non-declarations must not create resolution work");
        let root = graph.get_root_node().expect("resolved graph should have a root");
        let child_names: Vec<_> = root
            .children
            .iter()
            .map(|child| {
                graph
                    .get_node(*child)
                    .expect("resolver child edge should reference a graph node")
                    .name
                    .clone()
            })
            .collect();

        prop_assert_eq!(child_names, vec!["declared"]);
    }
}
