//! TASK-2067 RED integration evidence for the canonical parser module graph.
//!
//! The graph must be built from parsed `ModuleDecl` values, key every
//! structural edge by `ModuleKey`, and retain the acquired `ModuleUnit` rather
//! than rediscovering source from a path or spelling.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::module::{ModuleItem, ModuleSource};
use ash_parser::surface::{BinaryOp, Definition, Expr, Literal, Type, Visibility};
use ash_parser::use_tree::UsePath;
use ash_parser::{
    CanonicalModuleGraphError, CanonicalModuleGraphResolver, CanonicalModuleState,
    CanonicalStructuralDiagnostic, Fs, Span, parse_surface_file,
};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real filesystem fixture whose drop implementation removes its tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2067-{label}-{}-{serial}",
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

/// A filesystem fixture that permits each configured file body to be read once.
///
/// It continues to report the path as existing after consumption. That makes a
/// missed active-source check fail as a bounded acquisition failure rather than
/// recursively reparsing the same root source.
struct OneReadFs {
    unread: Mutex<HashMap<PathBuf, String>>,
    existing: HashSet<PathBuf>,
}

impl OneReadFs {
    fn with_file(path: impl Into<PathBuf>, source: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            unread: Mutex::new(HashMap::from([(path.clone(), source.into())])),
            existing: HashSet::from([path]),
        }
    }
}

impl Fs for OneReadFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        self.unread
            .lock()
            .expect("test filesystem lock should not be poisoned")
            .remove(path)
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.existing.contains(path)
    }
}

fn child_segments(graph: &ash_parser::CanonicalModuleGraph, key: &ModuleKey) -> Vec<String> {
    graph
        .children(key)
        .expect("parsed graph entry has child edges")
        .iter()
        .map(|child| {
            child
                .segments()
                .last()
                .expect("a structural child has one segment")
                .clone()
        })
        .collect()
}

fn assert_failed_keys(
    error: &CanonicalModuleGraphError,
    expected_keys: &[ModuleKey],
    context: &str,
) {
    assert_eq!(
        error.failed_keys(),
        expected_keys,
        "{context} must report exactly the canonical keys whose structural state failed"
    );
    for key in expected_keys {
        assert_eq!(
            error.failed_state(key),
            Some(CanonicalModuleState::Failed),
            "{context} must retain `Failed` for {key} without publishing a partial graph"
        );
    }
}

/// Produces source-form-independent identities for the ordered items exercised
/// by the file/inline transport-parity fixture below.
fn ordered_item_semantics(unit: &ash_parser::ModuleUnit) -> Vec<String> {
    unit.body()
        .items()
        .iter()
        .map(|item| match item {
            ModuleItem::Use(import) => match &import.path {
                UsePath::Simple(path) => format!(
                    "use:{}",
                    path.segments
                        .iter()
                        .map(|segment| segment.as_ref())
                        .collect::<Vec<_>>()
                        .join("::")
                ),
                path => panic!("fixture requires a simple parsed use path, got {path:?}"),
            },
            ModuleItem::Definition(Definition::Function(function)) => {
                format!("fn:{}", function.name)
            }
            ModuleItem::Definition(definition) => {
                panic!("fixture requires an ordinary function definition, got {definition:?}")
            }
            ModuleItem::ModuleDecl(declaration) => format!("mod:{}", declaration.name),
        })
        .collect()
}

/// A source-span- and provenance-free view of the parser payloads that the
/// graph must transport unchanged. This is deliberately test-local: TASK-2067
/// proves parser-carrier retention, not a new normalized module representation
/// or any import-binding semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OrderedPayload {
    Use {
        visibility: String,
        path: UsePathPayload,
        alias: Option<String>,
    },
    Function {
        visibility: String,
        name: String,
        parameters: Vec<(String, String)>,
        return_type: Option<String>,
        body: ExprPayload,
    },
    Module {
        visibility: String,
        name: String,
        source_form: ModuleSourceForm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UsePathPayload {
    Simple(Vec<String>),
    Glob(Vec<String>),
    Nested {
        base: Vec<String>,
        items: Vec<(String, Option<String>)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleSourceForm {
    File,
    Inline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExprPayload {
    Variable(String),
    Int(i64),
    Add(Box<ExprPayload>, Box<ExprPayload>),
    Block(Box<ExprPayload>),
}

fn visibility_payload(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Inherited => "inherited".to_owned(),
        Visibility::Public => "public".to_owned(),
        Visibility::Crate => "crate".to_owned(),
        Visibility::Super { levels } => format!("super:{levels}"),
        Visibility::Self_ => "self".to_owned(),
        Visibility::Restricted { path } => format!("restricted:{path}"),
    }
}

fn type_payload(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        other => panic!("fixture requires named parameter/return types, got {other:?}"),
    }
}

fn path_segments_payload(segments: &[Box<str>]) -> Vec<String> {
    segments.iter().map(ToString::to_string).collect()
}

fn use_path_payload(path: &UsePath) -> UsePathPayload {
    match path {
        UsePath::Simple(path) => UsePathPayload::Simple(path_segments_payload(&path.segments)),
        UsePath::Glob(path) => UsePathPayload::Glob(path_segments_payload(&path.segments)),
        UsePath::Nested(path, items) => UsePathPayload::Nested {
            base: path_segments_payload(&path.segments),
            items: items
                .iter()
                .map(|item| {
                    (
                        item.name.to_string(),
                        item.alias.as_deref().map(ToString::to_string),
                    )
                })
                .collect(),
        },
        UsePath::Notation { .. } => {
            panic!("TASK-2067 fixture does not exercise TASK-2074 notation imports")
        }
    }
}

fn expr_payload(expression: &Expr) -> ExprPayload {
    match expression {
        Expr::Variable { name, .. } => ExprPayload::Variable(name.to_string()),
        Expr::Literal(Literal::Int(value)) => ExprPayload::Int(*value),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
            ..
        } => ExprPayload::Add(Box::new(expr_payload(left)), Box::new(expr_payload(right))),
        Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty() => ExprPayload::Block(Box::new(expr_payload(tail_expr))),
        other => panic!("fixture requires a simple `value + integer` function body, got {other:?}"),
    }
}

fn ordered_payloads(unit: &ash_parser::ModuleUnit) -> Vec<OrderedPayload> {
    unit.body()
        .items()
        .iter()
        .map(|item| match item {
            ModuleItem::Use(import) => OrderedPayload::Use {
                visibility: visibility_payload(&import.visibility),
                path: use_path_payload(&import.path),
                alias: import.alias.as_deref().map(ToString::to_string),
            },
            ModuleItem::Definition(Definition::Function(function)) => OrderedPayload::Function {
                visibility: visibility_payload(&function.visibility),
                name: function.name.to_string(),
                parameters: function
                    .params
                    .iter()
                    .map(|parameter| (parameter.name.to_string(), type_payload(&parameter.ty)))
                    .collect(),
                return_type: function.return_type.as_ref().map(type_payload),
                body: expr_payload(&function.body),
            },
            ModuleItem::Definition(definition) => {
                panic!("fixture requires an ordinary function definition, got {definition:?}")
            }
            ModuleItem::ModuleDecl(declaration) => OrderedPayload::Module {
                visibility: visibility_payload(&declaration.visibility),
                name: declaration.name.to_string(),
                source_form: match declaration.source {
                    ModuleSource::File => ModuleSourceForm::File,
                    ModuleSource::Inline(_) => ModuleSourceForm::Inline,
                },
            },
        })
        .collect()
}

#[test]
fn parsed_declarations_publish_canonical_edges_and_real_file_inline_units() {
    let tree = TempTree::new("paired-units");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            mod file_child;
            mod inline_child {
                fn shared() {}
                mod nested { fn leaf() {} }
            }
            -- mod comment_lookalike;
            fn main() { "mod string_lookalike;" }
        "#,
    );
    tree.write(
        "src/file_child.ash",
        r#"
            fn shared() {}
            mod nested { fn leaf() {} }
        "#,
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let file_key = root_key.child("file_child").expect("fixture child key");
    let inline_key = root_key.child("inline_child").expect("fixture child key");
    let file_nested = file_key.child("nested").expect("fixture grandchild key");
    let inline_nested = inline_key.child("nested").expect("fixture grandchild key");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), &root_path)
        .expect("only parsed module declarations should create structural graph edges");

    assert_eq!(graph.root_key(), &root_key);
    assert_eq!(
        graph.children(&root_key),
        Some([file_key.clone(), inline_key.clone()].as_slice()),
        "the root topology must contain only parsed declarations, never source-text lookalikes"
    );
    assert_eq!(graph.children(&file_key), Some([file_nested].as_slice()));
    assert_eq!(
        graph.children(&inline_key),
        Some([inline_nested].as_slice())
    );
    assert_eq!(
        child_segments(&graph, &file_key),
        child_segments(&graph, &inline_key),
        "source kind must not change the canonical relative topology"
    );

    let root_unit = graph
        .module_unit(&root_key)
        .expect("the root is retained as an acquired module unit");
    let file_unit = graph
        .module_unit(&file_key)
        .expect("the file child is retained as its acquired module unit");
    let inline_unit = graph
        .module_unit(&inline_key)
        .expect("the inline child is retained as its acquired module unit");

    assert_eq!(root_unit.artifact().key(), &root_key);
    assert_eq!(file_unit.artifact().key(), &file_key);
    assert_eq!(inline_unit.artifact().key(), &inline_key);
    assert_eq!(
        file_unit.artifact().origin(),
        &ModuleArtifactOrigin::File(
            root_path
                .parent()
                .expect("root path has a source directory")
                .join("file_child.ash")
                .display()
                .to_string(),
        )
    );
    assert!(matches!(
        inline_unit.artifact().origin(),
        ModuleArtifactOrigin::Inline { parent, .. } if *parent == root_key
    ));
    assert_eq!(
        file_unit.artifact().child_keys(),
        graph.children(&file_key).expect("file topology exists")
    );
    assert_eq!(
        inline_unit.artifact().child_keys(),
        graph.children(&inline_key).expect("inline topology exists")
    );
    assert_eq!(
        file_unit
            .body()
            .module_decls()
            .iter()
            .map(|decl| decl.name.as_ref())
            .collect::<Vec<_>>(),
        inline_unit
            .body()
            .module_decls()
            .iter()
            .map(|decl| decl.name.as_ref())
            .collect::<Vec<_>>(),
        "the acquired file and inline units retain equivalent parsed child declarations"
    );
    assert_eq!(graph.state(&root_key), Some(CanonicalModuleState::Parsed));
    assert_eq!(graph.state(&file_key), Some(CanonicalModuleState::Parsed));
    assert_eq!(graph.state(&inline_key), Some(CanonicalModuleState::Parsed));
}

#[test]
fn missing_child_diagnostic_uses_the_parsed_declaration_key_and_anchor() {
    let missing_tree = TempTree::new("missing");
    let missing_source = "\nmod actual_child;\n";
    let missing_root = missing_tree.write("src/main.ash", missing_source);
    let missing_span = parse_surface_file(missing_source)
        .expect("missing-child fixture parses")
        .module_decls[0]
        .span;
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let missing_key = root_key
        .child("actual_child")
        .expect("fixture child key is canonical");

    let missing = CanonicalModuleGraphResolver::new().resolve_root(root_key.clone(), &missing_root);
    assert!(
        missing.is_err(),
        "a missing structural child must reject rather than publish a canonical graph"
    );
    let missing_error = missing
        .as_ref()
        .expect_err("missing-child resolution must produce an error-side failure report");
    assert_failed_keys(
        missing_error,
        std::slice::from_ref(&missing_key),
        "the missing-child error",
    );
    match missing {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic: CanonicalStructuralDiagnostic::MissingChild { child_key, .. },
            ..
        }) => {
            assert_eq!(
                parent_key, root_key,
                "the missing-child failure must remain anchored to its declaring parent"
            );
            assert_eq!(
                declaration_span, missing_span,
                "the missing-child failure must retain the parsed declaration span"
            );
            assert_eq!(
                child_key, missing_key,
                "the missing-child diagnostic must identify the key derived from the parsed declaration"
            );
        }
        other => panic!("expected an anchored missing-child structural error, got {other:?}"),
    }
}

#[test]
fn duplicate_root_children_are_anchored_at_the_second_parsed_declaration() {
    let duplicate_tree = TempTree::new("duplicate-root-child");
    let duplicate_source = "\nmod duplicate;\nmod duplicate;\n";
    let duplicate_root = duplicate_tree.write("src/main.ash", duplicate_source);
    let parsed_root = parse_surface_file(duplicate_source).expect("duplicate fixture parses");
    assert_eq!(
        parsed_root.module_decls.len(),
        2,
        "the fixture must contain two parsed duplicate declarations"
    );
    let first_declaration_span = parsed_root.module_decls[0].span;
    let second_declaration_span = parsed_root.module_decls[1].span;
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let duplicate_key = root_key.child("duplicate").expect("fixture child key");

    let resolution =
        CanonicalModuleGraphResolver::new().resolve_root(root_key.clone(), &duplicate_root);
    assert!(
        resolution.is_err(),
        "duplicate structural declarations must reject rather than publish a canonical graph"
    );
    let duplicate_error = resolution
        .as_ref()
        .expect_err("duplicate-child resolution must produce an error-side failure report");
    assert_failed_keys(
        duplicate_error,
        std::slice::from_ref(&duplicate_key),
        "the duplicate-child error",
    );

    match resolution {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic:
                CanonicalStructuralDiagnostic::DuplicateChild {
                    child_key,
                    first_declaration_span: reported_first_declaration_span,
                },
            ..
        }) => {
            assert_eq!(
                parent_key, root_key,
                "the duplicate failure must remain anchored to the declaring root"
            );
            assert_eq!(
                declaration_span, second_declaration_span,
                "the duplicate failure must anchor the later parsed declaration"
            );
            assert_eq!(
                child_key, duplicate_key,
                "the duplicate diagnostic must identify the canonical child key"
            );
            assert_eq!(
                reported_first_declaration_span, first_declaration_span,
                "the duplicate diagnostic must retain the first parsed declaration anchor"
            );
        }
        other => panic!("expected an anchored duplicate-child structural error, got {other:?}"),
    }
}

#[test]
fn duplicate_nested_children_are_anchored_at_the_second_child_declaration() {
    let duplicate_tree = TempTree::new("duplicate-nested-child");
    let root_path = duplicate_tree.write("src/main.ash", "mod child;\n");
    let child_source = "\nmod duplicate;\nmod duplicate;\n";
    duplicate_tree.write("src/child.ash", child_source);
    let parsed_child = parse_surface_file(child_source).expect("nested duplicate fixture parses");
    assert_eq!(
        parsed_child.module_decls.len(),
        2,
        "the child fixture must contain two parsed duplicate declarations"
    );
    let first_declaration_span = parsed_child.module_decls[0].span;
    let second_declaration_span = parsed_child.module_decls[1].span;
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let expected_child_key = root_key.child("child").expect("fixture child key");
    let duplicate_key = expected_child_key
        .child("duplicate")
        .expect("fixture grandchild key");

    let resolution = CanonicalModuleGraphResolver::new().resolve_root(root_key, &root_path);
    assert!(
        resolution.is_err(),
        "a nested duplicate structural declaration must reject rather than publish a canonical graph"
    );
    let duplicate_error = resolution
        .as_ref()
        .expect_err("nested duplicate resolution must produce an error-side failure report");
    assert_failed_keys(
        duplicate_error,
        std::slice::from_ref(&duplicate_key),
        "the nested duplicate-child error",
    );

    match resolution {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic:
                CanonicalStructuralDiagnostic::DuplicateChild {
                    child_key,
                    first_declaration_span: reported_first_declaration_span,
                },
            ..
        }) => {
            assert_eq!(
                parent_key, expected_child_key,
                "the nested duplicate failure must retain its declaring canonical parent"
            );
            assert_eq!(
                declaration_span, second_declaration_span,
                "the nested duplicate failure must anchor the later parsed child declaration"
            );
            assert_eq!(
                child_key, duplicate_key,
                "the nested duplicate diagnostic must identify the canonical grandchild key"
            );
            assert_eq!(
                reported_first_declaration_span, first_declaration_span,
                "the nested duplicate diagnostic must retain the first parsed declaration anchor"
            );
        }
        other => {
            panic!("expected an anchored nested duplicate-child structural error, got {other:?}")
        }
    }
}

#[test]
fn reentrant_file_child_cycle_is_closed_and_anchored_at_the_reentrant_declaration() {
    let cycle_tree = TempTree::new("cycle");
    let cycle_root = cycle_tree.write("src/main.ash", "mod loop_child;");
    let cycle_child_source = "\nmod loop_child;\n";
    cycle_tree.write("src/loop_child.ash", cycle_child_source);
    let cycle_span = parse_surface_file(cycle_child_source)
        .expect("cycle child fixture parses")
        .module_decls[0]
        .span;
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let loop_key = root_key.child("loop_child").expect("fixture child key");

    let cycle = CanonicalModuleGraphResolver::new().resolve_root(root_key, &cycle_root);
    assert!(
        cycle.is_err(),
        "a structural cycle must reject rather than publish a canonical graph"
    );
    match &cycle {
        Err(
            error @ CanonicalModuleGraphError::Structural {
                parent_key,
                declaration_span,
                diagnostic: CanonicalStructuralDiagnostic::Cycle { cycle },
                ..
            },
        ) => {
            assert_eq!(
                parent_key.value(),
                &loop_key,
                "the cycle failure must remain anchored to the reentrant declaration parent"
            );
            assert_eq!(
                declaration_span.value(),
                &cycle_span,
                "the cycle failure must retain the reentrant parsed declaration span"
            );
            assert!(
                !cycle.is_empty(),
                "a structural-cycle diagnostic must report a nonempty canonical path"
            );
            assert_eq!(
                cycle.first(),
                cycle.last(),
                "a structural-cycle diagnostic must close its canonical path"
            );
            let unique_cycle_keys = cycle.iter().cloned().collect::<HashSet<_>>();
            let reported_failed_keys = error.failed_keys().iter().cloned().collect::<HashSet<_>>();
            assert_eq!(
                reported_failed_keys, unique_cycle_keys,
                "the cycle error must retain every unique canonical cycle key as failed"
            );
            for key in &unique_cycle_keys {
                assert_eq!(
                    error.failed_state(key),
                    Some(CanonicalModuleState::Failed),
                    "the cycle error must retain `Failed` for every cycle member without publishing a graph"
                );
            }
        }
        other => panic!("expected an anchored structural-cycle error, got {other:?}"),
    }
}

#[test]
fn inline_child_does_not_hide_an_active_root_source_reentrancy() {
    let source = "mod inline_child { fn leaf() {} }\nmod main;\n";
    let parsed_root = parse_surface_file(source).expect("reentrancy fixture parses");
    let reentrant_declaration_span = parsed_root.module_decls[1].span;
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");

    let resolution =
        CanonicalModuleGraphResolver::with_fs(Box::new(OneReadFs::with_file("main.ash", source)))
            .resolve_root(root_key.clone(), "main.ash");

    match resolution {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic: CanonicalStructuralDiagnostic::Cycle { cycle },
            ..
        }) => {
            assert_eq!(
                parent_key, root_key,
                "the reentrant root declaration must retain its parsed parent identity"
            );
            assert_eq!(
                declaration_span, reentrant_declaration_span,
                "the cycle must anchor the root's parsed `mod main;` declaration"
            );
            assert_eq!(
                cycle.first(),
                cycle.last(),
                "the reported canonical reentrancy cycle must be closed"
            );
            assert_eq!(
                cycle.first(),
                Some(&root_key),
                "the active root source must be the canonical cycle identity"
            );
        }
        other => panic!(
            "expected immediate anchored structural cycle before rereading the active root, got {other:?}"
        ),
    }
}

#[test]
fn malformed_inline_root_body_is_anchored_and_retains_a_failed_child_state() {
    let malformed_tree = TempTree::new("malformed-inline-root");
    let source = "mod broken { fn invalid( {}";
    let root_path = malformed_tree.write("src/main.ash", source);
    let inline_header = "mod broken {";
    let inline_header_span = Span::new(0, inline_header.len(), 1, 1);
    let parser_error_span = parse_surface_file(source)
        .expect_err("the malformed fixture must enter the inline-module header before failing")
        .first()
        .expect("the malformed fixture must retain a parser-owned error span")
        .span;
    assert!(
        parser_error_span.start >= inline_header_span.end,
        "the malformed fixture must fail inside the inline module body, after its header"
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let broken_key = root_key.child("broken").expect("fixture child key");

    let resolution = CanonicalModuleGraphResolver::new().resolve_root(root_key.clone(), &root_path);
    assert!(
        resolution.is_err(),
        "a malformed inline body must reject rather than publish a canonical graph"
    );
    let malformed_error = resolution
        .as_ref()
        .expect_err("malformed-inline resolution must produce an error-side failure report");
    assert_failed_keys(
        malformed_error,
        std::slice::from_ref(&broken_key),
        "the malformed-inline error",
    );

    match resolution {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic:
                CanonicalStructuralDiagnostic::MalformedInline {
                    child_key,
                    error_span,
                },
            ..
        }) => {
            assert_eq!(
                parent_key, root_key,
                "the malformed-inline failure must retain the declaring canonical parent"
            );
            assert_eq!(
                declaration_span, inline_header_span,
                "the malformed-inline failure must be anchored at the parser-owned inline header"
            );
            assert_eq!(
                child_key, broken_key,
                "the malformed-inline diagnostic must identify the canonical child key"
            );
            assert_eq!(
                error_span, parser_error_span,
                "the malformed-inline diagnostic must preserve the parser-owned body error span"
            );
        }
        other => panic!("expected an anchored malformed-inline structural error, got {other:?}"),
    }
}

#[test]
fn malformed_inline_nested_body_is_anchored_at_its_parsed_file_child() {
    let malformed_tree = TempTree::new("malformed-inline-nested");
    let root_path = malformed_tree.write("src/main.ash", "mod child;\n");
    let child_source = "\nmod broken { fn invalid( {}";
    malformed_tree.write("src/child.ash", child_source);

    let inline_header = "mod broken {";
    let inline_header_start = child_source
        .find(inline_header)
        .expect("the child fixture must contain its inline header");
    let inline_header_span = Span::new(
        inline_header_start,
        inline_header_start + inline_header.len(),
        child_source[..inline_header_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1,
        inline_header_start
            - child_source[..inline_header_start]
                .rfind('\n')
                .map_or(0, |newline| newline + 1)
            + 1,
    );
    let parser_error_span = parse_surface_file(child_source)
        .expect_err("the nested malformed fixture must fail after parsing the inline header")
        .first()
        .expect("the nested malformed fixture must retain a parser-owned error span")
        .span;
    assert!(
        parser_error_span.start >= inline_header_span.end,
        "the malformed nested fixture must fail in the inline body after its header"
    );

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let child_key = root_key.child("child").expect("fixture child key");
    let broken_key = child_key.child("broken").expect("fixture inline child key");

    let resolution = CanonicalModuleGraphResolver::new().resolve_root(root_key, &root_path);
    assert!(
        resolution.is_err(),
        "a nested malformed inline body must reject rather than publish a canonical graph"
    );
    let malformed_error = resolution
        .as_ref()
        .expect_err("nested malformed-inline resolution must report its failed canonical child");
    assert_failed_keys(
        malformed_error,
        std::slice::from_ref(&broken_key),
        "the nested malformed-inline error",
    );

    match resolution {
        Err(CanonicalModuleGraphError::Structural {
            parent_key,
            declaration_span,
            diagnostic:
                CanonicalStructuralDiagnostic::MalformedInline {
                    child_key: reported_child_key,
                    error_span,
                },
            ..
        }) => {
            assert_eq!(
                parent_key, child_key,
                "the malformed inline failure must retain the canonical file-child parent"
            );
            assert_eq!(
                declaration_span, inline_header_span,
                "the nested malformed inline failure must be anchored at its parsed header"
            );
            assert_eq!(
                reported_child_key, broken_key,
                "the nested malformed inline diagnostic must identify the canonical inline child"
            );
            assert_eq!(
                error_span, parser_error_span,
                "the nested malformed inline diagnostic must preserve the parser-owned body error span"
            );
        }
        other => {
            panic!("expected an anchored nested malformed-inline structural error, got {other:?}")
        }
    }
}

#[test]
fn invalid_module_name_source_is_rejected_before_a_canonical_edge_or_failed_child_exists() {
    let invalid_tree = TempTree::new("invalid-module-name");
    let source = "mod bad!;\n";
    let root_path = invalid_tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");

    assert!(
        parse_surface_file(source).is_err(),
        "the parser must reject an invalid module name before producing a ModuleDecl"
    );

    let resolution = CanonicalModuleGraphResolver::new().resolve_root(root_key, root_path);
    let error = resolution.expect_err(
        "the canonical graph resolver must surface the parser rejection rather than fabricate a child",
    );
    assert!(
        error.failed_keys().is_empty(),
        "without a parsed ModuleDecl there is no canonical child transition to report as Failed"
    );
    assert!(
        matches!(error, CanonicalModuleGraphError::RootAcquisition { .. }),
        "the invalid source belongs to root acquisition, not a structural graph diagnostic"
    );
}

#[test]
fn file_and_inline_units_retain_equivalent_ordered_item_semantics_and_child_topology() {
    let parity_tree = TempTree::new("ordered-file-inline-parity");
    let shared_body = r#"
        use crate::shared::Thing;
        fn local() {}
        mod nested { fn inner() {} }
    "#;
    let root_path = parity_tree.write(
        "src/main.ash",
        format!("mod file_child;\nmod inline_child {{\n{shared_body}\n}}\n").as_str(),
    );
    parity_tree.write("src/file_child.ash", shared_body);

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let file_key = root_key.child("file_child").expect("fixture file key");
    let inline_key = root_key.child("inline_child").expect("fixture inline key");
    let file_nested_key = file_key.child("nested").expect("fixture file nested key");
    let inline_nested_key = inline_key
        .child("nested")
        .expect("fixture inline nested key");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("equivalent parsed file and inline units should build one canonical graph");
    let file_unit = graph
        .module_unit(&file_key)
        .expect("the graph retains the acquired file ModuleUnit");
    let inline_unit = graph
        .module_unit(&inline_key)
        .expect("the graph retains the acquired inline ModuleUnit");

    assert_eq!(
        ordered_item_semantics(file_unit),
        ["use:crate::shared::Thing", "fn:local", "mod:nested"],
        "the file unit must retain the parsed use, definition, and child declaration in source order"
    );
    assert_eq!(
        ordered_item_semantics(file_unit),
        ordered_item_semantics(inline_unit),
        "source origin must not change ordered parsed item semantics or their identifiers"
    );
    assert_eq!(
        graph.children(&file_key),
        Some([file_nested_key.clone()].as_slice()),
        "the file unit retains its nested canonical child edge"
    );
    assert_eq!(
        graph.children(&inline_key),
        Some([inline_nested_key.clone()].as_slice()),
        "the inline unit retains its nested canonical child edge"
    );
    assert_eq!(
        child_segments(&graph, &file_key),
        child_segments(&graph, &inline_key),
        "file and inline source forms preserve equivalent relative child topology"
    );
    assert!(
        graph.module_unit(&file_nested_key).is_some()
            && graph.module_unit(&inline_nested_key).is_some(),
        "the graph retains both nested units acquired through their respective parsed declarations"
    );
}

#[test]
fn canonical_root_acquisition_preserves_parsed_crate_metadata_and_child_unit() {
    let tree = TempTree::new("root-metadata");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            crate app;
            dependency util from "../util/main.ash";

            mod child;
        "#,
    );
    tree.write("src/child.ash", "fn child() {}\n");

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let child_key = root_key
        .child("child")
        .expect("fixture child key is canonical");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), &root_path)
        .expect("a root preamble and parsed child declaration must build one canonical graph");

    let metadata = graph
        .root_crate_metadata()
        .expect("canonical root acquisition must retain the parser-owned crate metadata carrier");
    assert_eq!(
        metadata.crate_name.as_ref(),
        "app",
        "the graph must retain the parsed root crate name rather than recover it from the path"
    );
    assert_eq!(
        metadata.dependencies.len(),
        1,
        "the graph must retain the parser-owned root dependency metadata"
    );
    assert_eq!(
        metadata.dependencies[0].alias.as_ref(),
        "util",
        "the dependency alias must come from the parsed metadata carrier"
    );
    assert_eq!(
        metadata.dependencies[0].root_path.as_ref(),
        "../util/main.ash",
        "the dependency source path must remain attached to parsed root metadata"
    );
    assert_eq!(
        graph.children(&root_key),
        Some([child_key.clone()].as_slice()),
        "the root preamble must not prevent canonical parsed child-edge acquisition"
    );
    assert!(
        graph.module_unit(&child_key).is_some(),
        "the graph must retain the child ModuleUnit acquired from its parsed declaration"
    );
}

#[test]
fn file_and_inline_units_preserve_complete_ordered_payloads_and_mutations() {
    let parity_tree = TempTree::new("complete-ordered-file-inline-payloads");
    let file_body = r#"
        pub use crate::shared::{Thing as LocalThing, Other};
        pub fn local(value: Int) -> Int { value + 1 }
        pub mod nested { pub fn leaf(value: Int) -> Int { value + 1 } }
    "#;
    let root_path = parity_tree.write(
        "src/main.ash",
        format!("mod file_child;\npub mod inline_child {{\n{file_body}\n}}\n").as_str(),
    );
    parity_tree.write("src/file_child.ash", file_body);

    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let file_key = root_key
        .child("file_child")
        .expect("fixture file child key");
    let inline_key = root_key
        .child("inline_child")
        .expect("fixture inline child key");
    let file_nested_key = file_key.child("nested").expect("fixture file nested key");
    let inline_nested_key = inline_key
        .child("nested")
        .expect("fixture inline nested key");

    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("paired complete file and inline bodies should build a canonical graph");
    let file_unit = graph
        .module_unit(&file_key)
        .expect("the graph retains the acquired file ModuleUnit");
    let inline_unit = graph
        .module_unit(&inline_key)
        .expect("the graph retains the acquired inline ModuleUnit");
    let expected_payloads = vec![
        OrderedPayload::Use {
            visibility: "public".to_owned(),
            path: UsePathPayload::Nested {
                base: vec!["crate".to_owned(), "shared".to_owned()],
                items: vec![
                    ("Thing".to_owned(), Some("LocalThing".to_owned())),
                    ("Other".to_owned(), None),
                ],
            },
            alias: None,
        },
        OrderedPayload::Function {
            visibility: "public".to_owned(),
            name: "local".to_owned(),
            parameters: vec![("value".to_owned(), "Int".to_owned())],
            return_type: Some("Int".to_owned()),
            body: ExprPayload::Block(Box::new(ExprPayload::Add(
                Box::new(ExprPayload::Variable("value".to_owned())),
                Box::new(ExprPayload::Int(1)),
            ))),
        },
        OrderedPayload::Module {
            visibility: "public".to_owned(),
            name: "nested".to_owned(),
            source_form: ModuleSourceForm::Inline,
        },
    ];

    assert_eq!(
        ordered_payloads(file_unit),
        expected_payloads,
        "the file graph entry must retain all ordered parsed import/function/module payload fields"
    );
    assert_eq!(
        ordered_payloads(file_unit),
        ordered_payloads(inline_unit),
        "a file and inline unit with equivalent parsed source must deliver the same ordered payloads"
    );
    assert_eq!(
        child_segments(&graph, &file_key),
        child_segments(&graph, &inline_key),
        "equivalent source forms must preserve their relative nested topology"
    );
    for key in [&file_key, &inline_key, &file_nested_key, &inline_nested_key] {
        assert_eq!(
            graph.state(key),
            Some(CanonicalModuleState::Parsed),
            "equivalent source forms must preserve the parsed state for {key}"
        );
    }

    let mutation_tree = TempTree::new("complete-ordered-inline-mutation");
    let mutation_root = mutation_tree.write(
        "src/main.ash",
        r#"
            mod file_child;
            pub mod inline_child {
                pub use crate::shared::{Thing as MutatedThing, Other};
                pub fn local(value: Int) -> Int { value + 2 }
                pub mod nested { pub fn leaf(value: Int) -> Int { value + 1 } }
            }
        "#,
    );
    mutation_tree.write("src/file_child.ash", file_body);
    let mutated_graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), mutation_root)
        .expect("a payload-only inline mutation must retain the canonical graph topology");
    let mutated_file_unit = mutated_graph
        .module_unit(&file_key)
        .expect("the mutated graph retains its file ModuleUnit");
    let mutated_inline_unit = mutated_graph
        .module_unit(&inline_key)
        .expect("the mutated graph retains its inline ModuleUnit");
    let mutated_payloads = vec![
        OrderedPayload::Use {
            visibility: "public".to_owned(),
            path: UsePathPayload::Nested {
                base: vec!["crate".to_owned(), "shared".to_owned()],
                items: vec![
                    ("Thing".to_owned(), Some("MutatedThing".to_owned())),
                    ("Other".to_owned(), None),
                ],
            },
            alias: None,
        },
        OrderedPayload::Function {
            visibility: "public".to_owned(),
            name: "local".to_owned(),
            parameters: vec![("value".to_owned(), "Int".to_owned())],
            return_type: Some("Int".to_owned()),
            body: ExprPayload::Block(Box::new(ExprPayload::Add(
                Box::new(ExprPayload::Variable("value".to_owned())),
                Box::new(ExprPayload::Int(2)),
            ))),
        },
        OrderedPayload::Module {
            visibility: "public".to_owned(),
            name: "nested".to_owned(),
            source_form: ModuleSourceForm::Inline,
        },
    ];

    assert_eq!(
        ordered_payloads(mutated_inline_unit),
        mutated_payloads,
        "the inline mutation must remain visible in the real acquired ModuleUnit payload"
    );
    assert_ne!(
        ordered_payloads(mutated_file_unit),
        ordered_payloads(mutated_inline_unit),
        "the graph must not overwrite an inline unit's alias or function literal with its file sibling's payload"
    );
    assert_eq!(
        child_segments(&mutated_graph, &file_key),
        child_segments(&mutated_graph, &inline_key),
        "a payload-only mutation must not alter canonical relative topology"
    );
    for key in [&file_key, &inline_key, &file_nested_key, &inline_nested_key] {
        assert_eq!(
            mutated_graph.state(key),
            Some(CanonicalModuleState::Parsed),
            "a payload-only mutation must not alter the parsed graph state for {key}"
        );
    }
}
