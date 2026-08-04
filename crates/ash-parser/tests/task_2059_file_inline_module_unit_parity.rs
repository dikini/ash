//! TASK-2059 RED integration evidence for source-kind-independent module units.
//!
//! The resolver is expected to perform only source acquisition. `use` items
//! remain parsed syntax here; binding, checking, lowering, and runtime are
//! deliberately outside this target's boundary.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::module::{ModuleBody, ModuleDecl, ModuleItem, ModuleSource};
use ash_parser::surface::{Definition, Expr, Visibility, expand_surface_module};
use ash_parser::token::Span;
use ash_parser::use_tree::UsePath;
use ash_parser::{Fs, ModuleUnitResolver, ResolveError, parse_surface_file};

const CHILD_BODY: &str = r#"
use crate::shared::Thing;
fn local() {}
mod nested { fn inner() {} }
"#;

struct CountedFs {
    files: HashMap<PathBuf, String>,
    reads: Arc<Mutex<HashMap<PathBuf, usize>>>,
    probes: Arc<Mutex<HashMap<PathBuf, usize>>>,
}

impl CountedFs {
    fn new(
        reads: Arc<Mutex<HashMap<PathBuf, usize>>>,
        probes: Arc<Mutex<HashMap<PathBuf, usize>>>,
    ) -> Self {
        Self {
            files: HashMap::new(),
            reads,
            probes,
        }
    }

    fn with_file(mut self, path: impl AsRef<Path>, source: impl Into<String>) -> Self {
        self.files
            .insert(path.as_ref().to_path_buf(), source.into());
        self
    }
}

impl Fs for CountedFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        let mut reads = self.reads.lock().expect("test read counter lock");
        *reads.entry(path.to_path_buf()).or_default() += 1;
        self.files.get(path).cloned()
    }

    fn file_exists(&self, path: &Path) -> bool {
        let mut probes = self.probes.lock().expect("test probe counter lock");
        *probes.entry(path.to_path_buf()).or_default() += 1;
        self.files.contains_key(path)
    }
}

fn child_declaration(source: &str) -> ModuleDecl {
    let module_file = parse_surface_file(source).expect("fixture parent module should parse");
    assert_eq!(module_file.module_decls.len(), 1, "fixture has one child");
    module_file
        .module_decls
        .into_iter()
        .next()
        .expect("fixture child declaration exists")
}

fn counter_snapshot(counter: &Arc<Mutex<HashMap<PathBuf, usize>>>) -> HashMap<PathBuf, usize> {
    counter.lock().expect("test counter lock").clone()
}

fn item_kinds(body: &ModuleBody) -> Vec<&'static str> {
    body.items()
        .iter()
        .map(|item| match item {
            ModuleItem::Use(_) => "use",
            ModuleItem::Definition(_) => "definition",
            ModuleItem::ModuleDecl(_) => "module",
        })
        .collect()
}

fn assert_complete_source_order(body: &ModuleBody) {
    match body.items() {
        [
            ModuleItem::Use(import),
            ModuleItem::Definition(Definition::Function(function)),
            ModuleItem::ModuleDecl(nested),
        ] => {
            assert_eq!(
                import.path,
                UsePath::Simple(ash_parser::use_tree::SimplePath {
                    segments: vec!["crate".into(), "shared".into(), "Thing".into()],
                })
            );
            assert!(import.alias.is_none());
            assert_eq!(function.name.as_ref(), "local");
            assert_eq!(nested.name.as_ref(), "nested");
            assert!(nested.is_inline());
        }
        items => {
            panic!("expected use, definition, and nested module in source order; got {items:?}")
        }
    }
}

fn inline_body(declaration: &ModuleDecl) -> &ModuleBody {
    declaration
        .body()
        .expect("fixture declaration should be inline")
}

fn block_tail(expression: &Expr) -> &Expr {
    match expression {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail,
        expression => expression,
    }
}

fn assert_macro_notation_expands_to(body: &ModuleBody, function_name: &str, target: &str) {
    let function = body
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == function_name => {
                Some(function)
            }
            _ => None,
        })
        .expect("fixture function should be retained in its module body");

    match block_tail(&function.body) {
        Expr::FnDef { body, .. } => {
            assert!(
                matches!(body.as_ref(), Expr::Call { func, .. } if func.as_ref() == target),
                "{function_name} should elaborate its local notation to {target}"
            );
        }
        expression => panic!(
            "{function_name} should expand its local macro and notation section, got {expression:?}"
        ),
    }
}

#[test]
fn file_and_inline_children_materialize_the_same_ordered_body_after_acquisition() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let file_declaration = child_declaration("mod child;");
    let inline_declaration = child_declaration(&format!("mod child {{{CHILD_BODY}}}"));
    let inline_offset = inline_declaration.span.start;
    let reads = Arc::new(Mutex::new(HashMap::new()));
    let probes = Arc::new(Mutex::new(HashMap::new()));
    let resolver = ModuleUnitResolver::with_fs(Box::new(
        CountedFs::new(Arc::clone(&reads), Arc::clone(&probes))
            .with_file("src/child.ash", CHILD_BODY)
            .with_file("src/child/mod.ash", "fn directory_fallback() {}"),
    ));

    let file_unit = resolver
        .acquire_child(&parent, Path::new("src/root.ash"), &file_declaration)
        .expect("direct file child should materialize");

    assert_complete_source_order(file_unit.body());
    assert_eq!(
        file_unit.artifact().origin(),
        &ModuleArtifactOrigin::File("src/child.ash".into())
    );
    assert_eq!(
        file_unit.artifact().key(),
        &parent.child("child").expect("fixture child key is valid")
    );
    assert_eq!(file_unit.source_path(), Some("src/child.ash"));
    assert_eq!(
        counter_snapshot(&reads).get(Path::new("src/child.ash")),
        Some(&1),
        "the selected file source is read exactly once"
    );
    assert_eq!(
        counter_snapshot(&reads).get(Path::new("src/child/mod.ash")),
        None,
        "the fallback source is not read when child.ash exists"
    );
    assert_eq!(
        counter_snapshot(&probes).get(Path::new("src/child/mod.ash")),
        None,
        "child.ash must be preferred before probing child/mod.ash"
    );

    let reads_before_inline = counter_snapshot(&reads);
    let probes_before_inline = counter_snapshot(&probes);
    let inline_unit = resolver
        .acquire_child(&parent, Path::new("src/root.ash"), &inline_declaration)
        .expect("inline child should materialize from the parsed declaration");

    assert_complete_source_order(inline_unit.body());
    assert_eq!(
        inline_unit.artifact().origin(),
        &ModuleArtifactOrigin::Inline {
            parent: parent.clone(),
            declaration_offset: inline_offset,
        }
    );
    assert_eq!(item_kinds(file_unit.body()), item_kinds(inline_unit.body()));
    assert_eq!(
        file_unit.artifact().child_keys(),
        inline_unit.artifact().child_keys(),
        "source acquisition must not change canonical child identities"
    );
    assert_eq!(
        inline_unit.source_path(),
        Some("src/root.ash"),
        "an inline unit retains its enclosing source path for diagnostics"
    );
    assert_eq!(
        counter_snapshot(&reads),
        reads_before_inline,
        "inline acquisition must not read any filesystem source"
    );
    assert_eq!(
        counter_snapshot(&probes),
        probes_before_inline,
        "inline acquisition must not probe any filesystem candidate"
    );
}

#[test]
fn missing_or_malformed_file_acquisition_returns_no_module_unit() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let missing_declaration = child_declaration("mod missing;");
    let malformed_declaration = child_declaration("mod malformed;");
    let resolver = ModuleUnitResolver::with_fs(Box::new(
        CountedFs::new(
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        )
        .with_file("src/malformed.ash", "fn broken( {}"),
    ));

    assert!(
        resolver
            .acquire_child(&parent, Path::new("src/root.ash"), &missing_declaration,)
            .is_err(),
        "a missing child must not return a partially published module unit"
    );
    assert!(
        resolver
            .acquire_child(&parent, Path::new("src/root.ash"), &malformed_declaration,)
            .is_err(),
        "a malformed child must not return a partially published module unit"
    );
}

#[test]
fn missing_module_unit_is_anchored_to_the_parent_declaration() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let declaration = child_declaration("\nmod missing;");
    assert_eq!(declaration.span.line, 2, "fixture declaration line");
    assert_eq!(declaration.span.column, 1, "fixture declaration column");
    let resolver = ModuleUnitResolver::with_fs(Box::new(CountedFs::new(
        Arc::new(Mutex::new(HashMap::new())),
        Arc::new(Mutex::new(HashMap::new())),
    )));

    let result = resolver.acquire_child(&parent, Path::new("src/root.ash"), &declaration);

    assert!(
        matches!(
            result,
            Err(ResolveError::ModuleUnitNotFound {
                module_name,
                parent_path,
                declaration_span,
                expected_path,
            }) if module_name == "missing"
                && parent_path.as_path() == Path::new("src/root.ash")
                && declaration_span == declaration.span
                && expected_path.as_path() == Path::new("src/missing.ash")
        ),
        "a missing module unit must name its parent declaration, not only its child path"
    );
}

#[test]
fn parser_valid_hyphenated_module_key_materializes() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let declaration = child_declaration("\nmod with-error;");
    assert_eq!(declaration.span.line, 2, "fixture declaration line");
    assert_eq!(declaration.span.column, 1, "fixture declaration column");
    let resolver = ModuleUnitResolver::with_fs(Box::new(
        CountedFs::new(
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        )
        .with_file("src/with-error.ash", "fn materialized() {}"),
    ));

    let unit = resolver
        .acquire_child(&parent, Path::new("src/root.ash"), &declaration)
        .expect("a parser-valid ModuleKey spelling must materialize");

    assert!(
        matches!(
            unit.artifact().key(),
            key if key == &parent.child("with-error").expect("ModuleKey accepts hyphens")
        ),
        "source acquisition must preserve parser-valid ModuleKey spellings"
    );
}

#[test]
fn invalid_module_unit_key_is_anchored_to_the_parent_declaration() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let declaration = ModuleDecl {
        name: "bad!".into(),
        visibility: Visibility::Inherited,
        source: ModuleSource::File,
        span: Span::new(17, 21, 3, 5),
    };
    let resolver = ModuleUnitResolver::with_fs(Box::new(CountedFs::new(
        Arc::new(Mutex::new(HashMap::new())),
        Arc::new(Mutex::new(HashMap::new())),
    )));

    let result = resolver.acquire_child(&parent, Path::new("src/root.ash"), &declaration);

    assert!(
        matches!(
            result,
            Err(ResolveError::InvalidModuleUnitIdentity {
                module_name,
                parent_path,
                declaration_span,
                ..
            }) if module_name == "bad!"
                && parent_path.as_path() == Path::new("src/root.ash")
                && declaration_span == declaration.span
        ),
        "an invalid ModuleKey spelling must retain its parent declaration anchor"
    );
}

#[test]
fn directory_module_fallback_materializes_once_when_direct_file_is_absent() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let declaration = child_declaration("mod child;");
    let reads = Arc::new(Mutex::new(HashMap::new()));
    let probes = Arc::new(Mutex::new(HashMap::new()));
    let resolver = ModuleUnitResolver::with_fs(Box::new(
        CountedFs::new(Arc::clone(&reads), Arc::clone(&probes))
            .with_file("src/child/mod.ash", CHILD_BODY),
    ));

    let unit = resolver
        .acquire_child(&parent, Path::new("src/root.ash"), &declaration)
        .expect("directory fallback should materialize");

    assert_eq!(
        unit.artifact().origin(),
        &ModuleArtifactOrigin::File("src/child/mod.ash".into())
    );
    assert_eq!(unit.source_path(), Some("src/child/mod.ash"));
    assert_eq!(
        counter_snapshot(&reads).get(Path::new("src/child/mod.ash")),
        Some(&1),
        "the fallback source is read exactly once"
    );
    assert_eq!(
        counter_snapshot(&reads).get(Path::new("src/child.ash")),
        None,
        "the missing direct candidate is never read"
    );
    assert_eq!(
        counter_snapshot(&probes).get(Path::new("src/child.ash")),
        Some(&1),
        "the direct candidate is probed first"
    );
    assert_eq!(
        counter_snapshot(&probes).get(Path::new("src/child/mod.ash")),
        Some(&1),
        "the directory fallback is probed after the direct candidate"
    );
}

#[test]
fn duplicate_nested_children_reject_before_file_or_inline_unit_return() {
    let parent = ModuleKey::root("app").expect("fixture crate name is valid");
    let file_declaration = child_declaration("mod child;");
    let duplicate_body = "mod duplicate {}\nmod duplicate {}";
    let parsed_file_body = parse_surface_file(duplicate_body).expect("fixture file body parses");
    let first_file_span = parsed_file_body.module_decls[0].span;
    let second_file_span = parsed_file_body.module_decls[1].span;
    let file_resolver = ModuleUnitResolver::with_fs(Box::new(
        CountedFs::new(
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        )
        .with_file("src/child.ash", duplicate_body),
    ));

    let file_result =
        file_resolver.acquire_child(&parent, Path::new("src/root.ash"), &file_declaration);
    assert!(
        matches!(
            file_result,
            Err(ResolveError::DuplicateModuleDeclaration {
                module_name,
                path,
                first_line,
                first_column,
                line,
                column,
            }) if module_name == "duplicate"
                && path.as_path() == Path::new("src/child.ash")
                && first_line == first_file_span.line
                && first_column == first_file_span.column
                && line == second_file_span.line
                && column == second_file_span.column
        ),
        "duplicate file children must reject before returning a module unit"
    );

    let inline_declaration = child_declaration(&format!("mod child {{\n{duplicate_body}\n}}"));
    let inline_body = inline_body(&inline_declaration);
    let first_inline_span = inline_body.module_decls()[0].span;
    let second_inline_span = inline_body.module_decls()[1].span;
    let inline_resolver = ModuleUnitResolver::with_fs(Box::new(CountedFs::new(
        Arc::new(Mutex::new(HashMap::new())),
        Arc::new(Mutex::new(HashMap::new())),
    )));

    let inline_result =
        inline_resolver.acquire_child(&parent, Path::new("src/root.ash"), &inline_declaration);
    assert!(
        matches!(
            inline_result,
            Err(ResolveError::DuplicateModuleDeclaration {
                module_name,
                path,
                first_line,
                first_column,
                line,
                column,
            }) if module_name == "duplicate"
                && path.as_path() == Path::new("src/root.ash")
                && first_line == first_inline_span.line
                && first_column == first_inline_span.column
                && line == second_inline_span.line
                && column == second_inline_span.column
        ),
        "duplicate inline children must reject before returning a module unit"
    );
}

#[test]
fn depth_two_inline_scopes_expand_inner_macros_and_elaborate_inner_notation() {
    let module = parse_surface_file(
        r"
        mod outer {
            infixl 6 <+> = outer_combine
            macro apply(x) => (<+> x);
            fn outer_value(n: Int) -> Int { apply!(n) }
            mod inner {
                infixl 6 <+> = inner_combine
                macro apply(x) => (<+> x);
                fn inner_value(n: Int) -> Int { apply!(n) }
            }
        }
        ",
    )
    .expect("depth-two inline fixture parses");

    let expanded = expand_surface_module(module).expect("all inline syntax scopes expand");
    let outer = &expanded.module.module_decls[0];
    let outer_body = inline_body(outer);
    let inner = &outer_body.module_decls()[0];

    assert_macro_notation_expands_to(outer_body, "outer_value", "outer_combine");
    assert_macro_notation_expands_to(inline_body(inner), "inner_value", "inner_combine");
}
