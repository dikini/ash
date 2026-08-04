//! TASK-2074 completion evidence for normalized source-form parity, mutation,
//! atomic rejection, and the parser-only expansion architecture boundary.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::module::{ModuleBody, ModuleItem};
use ash_parser::surface::{
    Definition, ExpansionDiagnosticKind, ExpansionError, Expr, IdentifierHygieneContext, Literal,
    SurfaceOrigin, Type, Visibility, visit_expr,
};
use ash_parser::use_tree::UsePath;
use ash_parser::{
    CanonicalExpandedModuleGraph, CanonicalModuleGraph, CanonicalModuleGraphResolver,
    CanonicalSyntaxImportFailureKind, Span,
};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2074-completion-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2074 completion fixture directory");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2074 completion fixture parent");
        fs::write(&path, source).expect("write TASK-2074 completion fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn resolve_graph(
    root_source: &str,
    files: &[(&str, &str)],
    label: &str,
) -> (CanonicalModuleGraph, ModuleKey) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    for (relative, source) in files {
        tree.write(relative, source);
    }
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture builds one canonical parsed graph");
    (graph, root_key)
}

fn parity_fixture(
    file_backed: bool,
    provider_first: bool,
    alias: &str,
    increment: i64,
    function_count: usize,
    depth: usize,
    label: &str,
) -> (CanonicalModuleGraph, ModuleKey) {
    assert!((1..=2).contains(&depth), "fixture depth is one or two");
    let provider_body = format!("pub macro inc(x) => add(x, {increment});");
    let provider_path = if depth == 1 {
        "crate::provider::inc"
    } else {
        "crate::layer::provider::inc"
    };
    let functions = (0..function_count)
        .map(|index| format!("fn run_{index}(n: Int) -> Int {{ {alias}!(n) }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let consumer_body = format!("use {provider_path} as {alias};\n{functions}\n");
    let (first, second) = if provider_first {
        ("pub mod provider", "pub mod consumer")
    } else {
        ("pub mod consumer", "pub mod provider")
    };
    if file_backed {
        let declarations = format!("{first};\n{second};\n");
        if depth == 1 {
            resolve_graph(
                &declarations,
                &[
                    ("src/provider.ash", provider_body.as_str()),
                    ("src/consumer.ash", consumer_body.as_str()),
                ],
                label,
            )
        } else {
            resolve_graph(
                "pub mod layer;\n",
                &[
                    ("src/layer.ash", declarations.as_str()),
                    ("src/provider.ash", provider_body.as_str()),
                    ("src/consumer.ash", consumer_body.as_str()),
                ],
                label,
            )
        }
    } else {
        let provider = format!("pub mod provider {{ {provider_body} }}");
        let consumer = format!("pub mod consumer {{ {consumer_body} }}");
        let declarations = if provider_first {
            format!("{provider}\n{consumer}\n")
        } else {
            format!("{consumer}\n{provider}\n")
        };
        let root = if depth == 1 {
            declarations
        } else {
            format!("pub mod layer {{ {declarations} }}")
        };
        resolve_graph(&root, &[], label)
    }
}

fn direct_module_declaration_names(body: &ModuleBody) -> Vec<String> {
    body.items()
        .iter()
        .filter_map(|item| match item {
            ModuleItem::ModuleDecl(declaration) => Some(declaration.name.to_string()),
            ModuleItem::Use(_) | ModuleItem::Definition(_) => None,
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
enum NormalizedUsePath {
    Simple(Vec<String>),
    Glob(Vec<String>),
    Nested {
        base: Vec<String>,
        members: Vec<(String, Option<String>)>,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct NormalizedUse {
    visibility: Visibility,
    path: NormalizedUsePath,
    alias: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum FixtureType {
    Name(String),
}

#[derive(Debug, PartialEq, Eq)]
enum FixtureExpr {
    LiteralInt(i64),
    Variable(String),
    Call {
        module: Option<String>,
        function: String,
        arguments: Vec<FixtureExpr>,
    },
    Block {
        has_statements: bool,
        tail: Option<Box<FixtureExpr>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct FunctionProjection {
    visibility: Visibility,
    name: String,
    type_parameter_count: usize,
    parameters: Vec<(String, FixtureType)>,
    return_type: Option<FixtureType>,
    has_proposition_tail: bool,
    has_contract: bool,
    body: FixtureExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct MacroProjection {
    visibility: Visibility,
    name: String,
    parameters: Vec<String>,
    has_typed_signature: bool,
    template: FixtureExpr,
}

#[derive(Debug, PartialEq, Eq)]
enum NormalizedItem {
    Use(NormalizedUse),
    Function(FunctionProjection),
    Macro(MacroProjection),
    ModuleDecl {
        visibility: Visibility,
        name: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum NormalizedSurfaceOrigin {
    Source,
    MacroExpansion { expansion_name: String },
    NotationExpansion { target: String },
    OperatorSection,
    Desugaring { rule: String },
}

#[derive(Debug, PartialEq, Eq)]
struct NormalizedExpandedOrigin {
    expansion_id: u32,
    origin: NormalizedSurfaceOrigin,
    parent: Option<Box<NormalizedSurfaceOrigin>>,
}

#[derive(Debug, PartialEq, Eq)]
struct NormalizedExpandedProjection {
    ordered_items: Vec<NormalizedItem>,
    diagnostics: Vec<ExpansionDiagnosticKind>,
    origins: Vec<NormalizedExpandedOrigin>,
    hygiene: Vec<(String, IdentifierHygieneContext, Option<u32>)>,
    syntax_imports: Vec<(ModuleKey, String, String)>,
}

fn normalize_use(use_declaration: &ash_parser::use_tree::Use) -> NormalizedUse {
    let segments = |segments: &[Box<str>]| segments.iter().map(ToString::to_string).collect();
    let path = match &use_declaration.path {
        UsePath::Simple(path) => NormalizedUsePath::Simple(segments(&path.segments)),
        UsePath::Glob(path) => NormalizedUsePath::Glob(segments(&path.segments)),
        UsePath::Nested(path, members) => NormalizedUsePath::Nested {
            base: segments(&path.segments),
            members: members
                .iter()
                .map(|member| {
                    (
                        member.name.to_string(),
                        member.alias.as_deref().map(ToOwned::to_owned),
                    )
                })
                .collect(),
        },
    };
    NormalizedUse {
        visibility: use_declaration.visibility.clone(),
        path,
        alias: use_declaration.alias.as_deref().map(ToOwned::to_owned),
    }
}

fn project_fixture_type(ty: &Type) -> FixtureType {
    match ty {
        Type::Name(name) => FixtureType::Name(name.to_string()),
        other => panic!("TASK-2074 parity fixture unexpectedly used non-name type {other:?}"),
    }
}

fn project_fixture_expr(expression: &Expr) -> FixtureExpr {
    match expression {
        Expr::Literal(Literal::Int(value)) => FixtureExpr::LiteralInt(*value),
        Expr::Variable { name, .. } => FixtureExpr::Variable(name.to_string()),
        Expr::Call {
            func, module, args, ..
        } => FixtureExpr::Call {
            module: module.as_deref().map(ToOwned::to_owned),
            function: func.to_string(),
            arguments: args.iter().map(project_fixture_expr).collect(),
        },
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            assert!(
                statements.is_empty(),
                "TASK-2074 bounded parity fixture does not omit statement semantics"
            );
            FixtureExpr::Block {
                has_statements: false,
                tail: tail_expr.as_deref().map(project_fixture_expr).map(Box::new),
            }
        }
        other => panic!("TASK-2074 parity fixture unexpectedly used expression {other:?}"),
    }
}

fn normalize_definition(definition: &Definition) -> NormalizedItem {
    match definition {
        Definition::Function(function) => NormalizedItem::Function(FunctionProjection {
            visibility: function.visibility.clone(),
            name: function.name.to_string(),
            type_parameter_count: function.type_params.len(),
            parameters: function
                .params
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.to_string(),
                        project_fixture_type(&parameter.ty),
                    )
                })
                .collect(),
            return_type: function.return_type.as_ref().map(project_fixture_type),
            has_proposition_tail: function.proposition_tail.is_some(),
            has_contract: function.contract.is_some(),
            body: project_fixture_expr(&function.body),
        }),
        Definition::Macro(definition) => NormalizedItem::Macro(MacroProjection {
            visibility: definition.visibility.clone(),
            name: definition.name.to_string(),
            parameters: definition.params.iter().map(ToString::to_string).collect(),
            has_typed_signature: definition.typed_signature.is_some(),
            template: project_fixture_expr(&definition.body),
        }),
        other => panic!("TASK-2074 bounded parity fixture used unexpected definition {other:?}"),
    }
}

fn normalize_surface_origin(origin: &SurfaceOrigin) -> NormalizedSurfaceOrigin {
    match origin {
        SurfaceOrigin::Source { .. } => NormalizedSurfaceOrigin::Source,
        SurfaceOrigin::MacroExpansion { expansion_id, .. } => {
            NormalizedSurfaceOrigin::MacroExpansion {
                expansion_name: expansion_id.to_string(),
            }
        }
        SurfaceOrigin::NotationExpansion { target, .. } => {
            NormalizedSurfaceOrigin::NotationExpansion {
                target: target.to_string(),
            }
        }
        SurfaceOrigin::OperatorSection { .. } => NormalizedSurfaceOrigin::OperatorSection,
        SurfaceOrigin::Desugaring { rule, .. } => NormalizedSurfaceOrigin::Desugaring {
            rule: rule.to_string(),
        },
    }
}

fn normalized_projection(
    module: ash_parser::CanonicalExpandedModuleRef<'_>,
) -> NormalizedExpandedProjection {
    let ordered_items = module
        .body()
        .items()
        .iter()
        .map(|item| match item {
            ModuleItem::Use(use_declaration) => NormalizedItem::Use(normalize_use(use_declaration)),
            ModuleItem::Definition(definition) => normalize_definition(definition),
            ModuleItem::ModuleDecl(declaration) => NormalizedItem::ModuleDecl {
                visibility: declaration.visibility.clone(),
                name: declaration.name.to_string(),
            },
        })
        .collect();
    let diagnostics = module
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.kind.clone())
        .collect();
    let origins = module
        .origins()
        .iter()
        .map(|origin| NormalizedExpandedOrigin {
            expansion_id: origin.expansion_id.0,
            origin: normalize_surface_origin(&origin.origin),
            parent: origin
                .parent
                .as_deref()
                .map(normalize_surface_origin)
                .map(Box::new),
        })
        .collect();
    let hygiene = module
        .hygiene()
        .iter()
        .map(|entry| {
            (
                entry.name.to_string(),
                entry.context,
                entry.expansion_id.map(|identity| identity.0),
            )
        })
        .collect();
    let syntax_imports = module
        .syntax_imports()
        .iter()
        .map(|import| {
            (
                import.provider_key().clone(),
                import.exported_name().to_owned(),
                import.local_name().to_owned(),
            )
        })
        .collect();
    NormalizedExpandedProjection {
        ordered_items,
        diagnostics,
        origins,
        hygiene,
        syntax_imports,
    }
}

fn expanded_add_literals(body: &ModuleBody) -> Vec<i64> {
    let mut literals = Vec::new();
    for definition in body.definitions() {
        let Definition::Function(function) = definition else {
            continue;
        };
        visit_expr(&function.body, &mut |expression| {
            let Expr::Call { func, args, .. } = expression else {
                return;
            };
            if func.as_ref() != "add" {
                return;
            }
            if let Some(Expr::Literal(Literal::Int(value))) = args.get(1) {
                literals.push(*value);
            }
        });
    }
    literals
}

fn contains_macro_invocation(body: &ModuleBody) -> bool {
    body.definitions().iter().any(|definition| {
        let Definition::Function(function) = definition else {
            return false;
        };
        let mut found = false;
        visit_expr(&function.body, &mut |expression| {
            found |= matches!(expression, Expr::MacroInvocation { .. });
        });
        found
    })
}

#[test]
fn intended_file_inline_and_notation_nonactivation_fixtures_parse_before_expansion() {
    let (file_graph, root_key) = parity_fixture(true, true, "plus_one", 1, 1, 1, "parse-file");
    let (inline_graph, _) = parity_fixture(false, true, "plus_one", 1, 1, 1, "parse-inline");
    let child_key = root_key.child("consumer").expect("consumer key");
    assert!(file_graph.module_unit(&child_key).is_some());
    assert!(inline_graph.module_unit(&child_key).is_some());

    let (notation_graph, notation_root) = resolve_graph(
        r#"
            pub mod provider {
                pub fn combine(left: Int, right: Int) -> Int { add(left, right) }
                pub infixl 6 <+> = combine
            }
            pub mod consumer {
                use crate::provider::combine;
                fn direct(left: Int, right: Int) -> Int { combine(left, right) }
                fn forbidden(value: Int) { (value <+>) }
            }
        "#,
        &[],
        "parse-notation-nonactivation",
    );
    assert!(
        notation_graph
            .module_unit(&notation_root.child("consumer").expect("consumer key"))
            .is_some()
    );
}

#[test]
fn equal_file_and_inline_children_have_equal_normalized_expanded_projections() {
    let (file_graph, root_key) = parity_fixture(true, true, "plus_one", 1, 1, 1, "parity-file");
    let (inline_graph, _) = parity_fixture(false, true, "plus_one", 1, 1, 1, "parity-inline");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");

    let file_expanded = CanonicalExpandedModuleGraph::try_expand(file_graph)
        .expect("file-backed declarations expand");
    let inline_expanded = CanonicalExpandedModuleGraph::try_expand(inline_graph)
        .expect("equal inline declarations expand");

    for key in [&provider_key, &consumer_key] {
        let file_projection = normalized_projection(
            file_expanded
                .module(key)
                .expect("file-backed expanded module exists"),
        );
        let inline_projection = normalized_projection(
            inline_expanded
                .module(key)
                .expect("inline expanded module exists"),
        );
        assert_eq!(
            file_projection, inline_projection,
            "source path, artifact origin, and source spans are deliberately outside the normalized semantic projection"
        );
    }
}

#[test]
fn expansion_uses_acquired_typed_units_after_file_sources_are_removed() {
    let tree = TempTree::new("post-acquisition-no-reread");
    let root_path = tree.write("src/main.ash", "pub mod provider;\npub mod consumer;\n");
    let provider_path = tree.write("src/provider.ash", "pub macro inc(x) => add(x, 1);");
    let consumer_path = tree.write(
        "src/consumer.ash",
        "use crate::provider::inc as plus_one;\nfn run_0(n: Int) -> Int { plus_one!(n) }\n",
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key");
    let parsed_file = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file graph is fully acquired before the mutation");
    let provider_key = root_key.child("provider").expect("provider key");
    let consumer_key = root_key.child("consumer").expect("consumer key");
    assert!(matches!(
        parsed_file
            .module_unit(&provider_key)
            .expect("provider was acquired")
            .artifact()
            .origin(),
        ModuleArtifactOrigin::File(_)
    ));

    fs::write(&provider_path, "this replacement is not valid Ash")
        .expect("overwrite provider after acquisition");
    fs::write(&consumer_path, "this replacement is also not valid Ash")
        .expect("overwrite consumer after acquisition");
    fs::remove_dir_all(&tree.root).expect("remove every acquired source before expansion");

    let (parsed_inline, inline_root_key) = parity_fixture(
        false,
        true,
        "plus_one",
        1,
        1,
        1,
        "post-acquisition-inline-baseline",
    );
    assert_eq!(inline_root_key, root_key);
    let expanded_file = CanonicalExpandedModuleGraph::try_expand(parsed_file)
        .expect("canonical expansion must not reread removed source files");
    let expanded_inline = CanonicalExpandedModuleGraph::try_expand(parsed_inline)
        .expect("equivalent inline baseline expands");

    for key in [&provider_key, &consumer_key] {
        assert_eq!(
            normalized_projection(
                expanded_file
                    .module(key)
                    .expect("file-expanded module exists"),
            ),
            normalized_projection(
                expanded_inline
                    .module(key)
                    .expect("inline-expanded module exists"),
            ),
            "removed source files cannot affect the acquired typed expansion projection"
        );
    }
}

#[test]
fn alias_and_provider_definition_mutations_change_the_observable_projection() {
    let (baseline_graph, root_key) =
        parity_fixture(false, true, "plus_one", 1, 1, 1, "mutation-baseline");
    let (alias_graph, _) = parity_fixture(false, true, "bump", 1, 1, 1, "mutation-alias");
    let (definition_graph, _) =
        parity_fixture(false, true, "plus_one", 2, 1, 1, "mutation-definition");
    let consumer_key = root_key.child("consumer").expect("consumer key");

    let baseline = CanonicalExpandedModuleGraph::try_expand(baseline_graph)
        .expect("baseline expansion succeeds");
    let alias =
        CanonicalExpandedModuleGraph::try_expand(alias_graph).expect("alias mutation succeeds");
    let definition = CanonicalExpandedModuleGraph::try_expand(definition_graph)
        .expect("definition mutation succeeds");

    let baseline_module = baseline.module(&consumer_key).expect("consumer exists");
    let alias_module = alias.module(&consumer_key).expect("consumer exists");
    let definition_module = definition.module(&consumer_key).expect("consumer exists");
    assert_eq!(expanded_add_literals(baseline_module.body()), [1]);
    assert_eq!(expanded_add_literals(definition_module.body()), [2]);
    let baseline = normalized_projection(baseline_module);
    let alias = normalized_projection(alias_module);
    let definition = normalized_projection(definition_module);
    assert_ne!(
        baseline, alias,
        "the retained local syntax alias must be observable"
    );
    assert_ne!(
        baseline, definition,
        "a provider-template mutation must change the expanded consumer body"
    );
}

#[test]
fn importing_only_a_notation_target_callable_does_not_activate_provider_notation() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod provider {
                pub fn combine(left: Int, right: Int) -> Int { add(left, right) }
                pub infixl 6 <+> = combine
            }
            pub mod consumer {
                use crate::provider::combine;
                fn direct(left: Int, right: Int) -> Int { combine(left, right) }
                fn forbidden(value: Int) { (value <+>) }
            }
        "#,
        &[],
        "target-callable-is-not-notation",
    );
    let consumer_key = root_key.child("consumer").expect("consumer key");
    let consumer = parsed
        .module_unit(&consumer_key)
        .expect("parsed consumer exists");
    let expected_span = first_operator_section_span(consumer.body());

    let error = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect_err("an ordinary callable import must not activate provider notation");
    let failure = error
        .expansion_failure()
        .expect("late notation rejection is anchored to its consumer module");
    assert_eq!(failure.module_key(), &consumer_key);
    assert_eq!(failure.span(), expected_span);
    assert!(matches!(
        failure.expansion_error(),
        ExpansionError::UnresolvedOperatorSection { span, operator }
            if *span == expected_span && operator.as_ref() == "<+>"
    ));
}

#[test]
fn graph_wide_prepass_rejects_non_macro_syntax_edge_with_exact_anchors() {
    let (parsed, root_key) = resolve_graph(
        r#"
            pub mod a_valid {
                macro local(x) => add(x, 1);
                fn succeeds(n: Int) -> Int { local!(n) }
            }
            pub mod m_provider {
                pub fn ordinary(n: Int) -> Int { n }
            }
            pub mod z_broken {
                use crate::m_provider::ordinary as imported_syntax;
                fn fails(n: Int) -> Int { imported_syntax!(n) }
            }
        "#,
        &[],
        "late-non-macro-edge",
    );
    let valid_key = root_key.child("a_valid").expect("valid sibling key");
    let provider_key = root_key.child("m_provider").expect("provider key");
    let broken_key = root_key.child("z_broken").expect("broken sibling key");
    let valid = parsed
        .module_unit(&valid_key)
        .expect("valid sibling exists");
    assert!(
        contains_macro_invocation(valid.body()),
        "the valid sibling would perform observable expansion if a partial graph leaked"
    );
    let broken = parsed
        .module_unit(&broken_key)
        .expect("broken sibling exists");
    let use_span = broken.body().uses()[0].span;
    let declaration_span = parsed
        .module_unit(&provider_key)
        .expect("provider exists")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "ordinary" => {
                Some(function.span)
            }
            _ => None,
        })
        .expect("ordinary declaration exists");

    let result = CanonicalExpandedModuleGraph::try_expand(parsed);
    assert!(
        result.is_err(),
        "the public API returns only one whole error and cannot expose the expanded valid sibling"
    );
    let error = result.expect_err("non-macro syntax edge rejects atomically");
    let failure = error
        .syntax_import_failure()
        .expect("invalid syntax edge exposes anchored import facts");
    assert_eq!(
        failure.kind(),
        CanonicalSyntaxImportFailureKind::NonMacroDeclaration
    );
    assert_eq!(failure.consumer_key(), &broken_key);
    assert_eq!(failure.provider_key(), Some(&provider_key));
    assert_eq!(failure.use_span(), use_span);
    assert_eq!(failure.declaration_span(), Some(declaration_span));
}

fn first_operator_section_span(body: &ModuleBody) -> Span {
    let mut span = None;
    for definition in body.definitions() {
        let Definition::Function(function) = definition else {
            continue;
        };
        visit_expr(&function.body, &mut |expression| {
            if let Expr::OperatorSection { section } = expression {
                span.get_or_insert(section.span);
            }
        });
    }
    span.expect("fixture contains one unresolved operator section")
}

fn rust_code_without_comments_or_literals(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment(usize),
        String,
        RawString(usize),
    }
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut state = State::Code;
    let mut index = 0;
    while index < bytes.len() {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            State::Code if current == b'/' && next == Some(b'/') => {
                output.push_str("  ");
                index += 2;
                state = State::LineComment;
            }
            State::Code if current == b'/' && next == Some(b'*') => {
                output.push_str("  ");
                index += 2;
                state = State::BlockComment(1);
            }
            State::Code if current == b'r' => {
                let hash_count = bytes[index + 1..]
                    .iter()
                    .take_while(|byte| **byte == b'#')
                    .count();
                if bytes.get(index + 1 + hash_count) == Some(&b'"') {
                    let width = hash_count + 2;
                    output.extend(std::iter::repeat_n(' ', width));
                    index += width;
                    state = State::RawString(hash_count);
                } else {
                    output.push('r');
                    index += 1;
                }
            }
            State::Code if current == b'"' => {
                output.push(' ');
                index += 1;
                state = State::String;
            }
            State::Code => {
                output.push(current as char);
                index += 1;
            }
            State::LineComment if current == b'\n' => {
                output.push('\n');
                index += 1;
                state = State::Code;
            }
            State::LineComment => {
                output.push(' ');
                index += 1;
            }
            State::BlockComment(depth) if current == b'/' && next == Some(b'*') => {
                output.push_str("  ");
                index += 2;
                state = State::BlockComment(depth + 1);
            }
            State::BlockComment(depth) if current == b'*' && next == Some(b'/') => {
                output.push_str("  ");
                index += 2;
                state = if depth == 1 {
                    State::Code
                } else {
                    State::BlockComment(depth - 1)
                };
            }
            State::BlockComment(depth) => {
                output.push(if current == b'\n' { '\n' } else { ' ' });
                index += 1;
                state = State::BlockComment(depth);
            }
            State::String if current == b'\\' && next.is_some() => {
                output.push_str("  ");
                index += 2;
            }
            State::String if current == b'"' => {
                output.push(' ');
                index += 1;
                state = State::Code;
            }
            State::String => {
                output.push(if current == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
            State::RawString(hash_count) if current == b'"' => {
                let closes = bytes[index + 1..]
                    .iter()
                    .take(hash_count)
                    .all(|byte| *byte == b'#')
                    && bytes.len() >= index + 1 + hash_count;
                if closes {
                    let width = hash_count + 1;
                    output.extend(std::iter::repeat_n(' ', width));
                    index += width;
                    state = State::Code;
                } else {
                    output.push(' ');
                    index += 1;
                }
            }
            State::RawString(hash_count) => {
                output.push(if current == b'\n' { '\n' } else { ' ' });
                index += 1;
                state = State::RawString(hash_count);
            }
        }
    }
    output
}

fn contains_exact_identifier(source: &str, identifier: &str) -> bool {
    source.match_indices(identifier).any(|(start, _)| {
        let before = source[..start].chars().next_back();
        let end = start + identifier.len();
        let after = source[end..].chars().next();
        let is_identifier = |character: char| character.is_ascii_alphanumeric() || character == '_';
        before.is_none_or(|character| !is_identifier(character))
            && after.is_none_or(|character| !is_identifier(character))
    })
}

#[test]
fn canonical_expansion_direct_orchestration_has_no_loader_scanner_or_later_layer_dependency() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_paths = [
        manifest_dir.join("src/canonical_expanded_module_graph.rs"),
        manifest_dir.join("src/canonical_syntax_dependencies.rs"),
    ];
    for source_path in source_paths {
        let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!(
                "read canonical expansion source {}: {error}",
                source_path.display()
            )
        });
        let code = rust_code_without_comments_or_literals(&source);
        for forbidden_path in [
            "ash_engine",
            "module_loader",
            "std::fs",
            "tokio::fs",
            "read_to_string",
            "read_file",
            "file_exists",
            "scan_source",
            "scan_text",
            "raw_source",
            "source_text",
        ] {
            assert!(
                !code.contains(forbidden_path),
                "{} must not contain the forbidden dependency `{forbidden_path}`",
                source_path.display()
            );
        }
        for forbidden_identifier in [
            "Engine",
            "ModuleLoader",
            "ModuleResolver",
            "LegacyModuleResolver",
            "RawSource",
            "SourceText",
            "SourceScanner",
            "Regex",
            "Lexer",
            "Path",
            "PathBuf",
            "Core",
            "CoreExpr",
            "RawCoreProgram",
            "Cps",
            "CpsProgram",
            "Runtime",
            "RuntimeValue",
            "ExecutionResult",
            "ImportResolver",
            "CanonicalProvisionalNameView",
            "CheckedInterface",
        ] {
            assert!(
                !contains_exact_identifier(&code, forbidden_identifier),
                "{} must not reference the forbidden carrier `{forbidden_identifier}`",
                source_path.display()
            );
        }
    }

    let manifest = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("ash-parser manifest remains available to the architecture fence");
    assert!(
        !manifest.lines().any(|line| {
            let dependency = line.split('#').next().unwrap_or_default();
            dependency.trim_start().starts_with("ash-engine")
                || dependency.trim_start().starts_with("ash_engine")
        }),
        "ash-parser must not depend on ash-engine"
    );
}

#[test]
fn exhaustive_sixty_four_case_depth_source_order_alias_definition_and_count_projection() {
    for case in 0u8..64 {
        let file_backed = case & 0b0001 != 0;
        let provider_first = case & 0b0010 != 0;
        let alias = if case & 0b0100 != 0 {
            "bump"
        } else {
            "plus_one"
        };
        let increment = if case & 0b1000 != 0 { 2 } else { 1 };
        let function_count = if case & 0b1_0000 != 0 { 2 } else { 1 };
        let depth = if case & 0b10_0000 != 0 { 2 } else { 1 };
        let (parsed, root_key) = parity_fixture(
            file_backed,
            provider_first,
            alias,
            increment,
            function_count,
            depth,
            "generated-projection",
        );
        let declaration_owner_key = if depth == 1 {
            root_key.clone()
        } else {
            root_key.child("layer").expect("layer key")
        };
        let consumer_key = declaration_owner_key
            .child("consumer")
            .expect("consumer key");
        let provider_key = declaration_owner_key
            .child("provider")
            .expect("provider key");
        for key in [&provider_key, &consumer_key] {
            let origin = parsed
                .module_unit(key)
                .expect("generated parsed module exists")
                .artifact()
                .origin();
            assert_eq!(
                matches!(origin, ModuleArtifactOrigin::File(_)),
                file_backed,
                "case {case} must make the encoded source form observable before normalization"
            );
            assert_eq!(
                matches!(origin, ModuleArtifactOrigin::Inline { .. }),
                !file_backed,
                "case {case} must retain the encoded inline source form before normalization"
            );
        }

        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("every generated supported graph expands atomically");
        let consumer = expanded.module(&consumer_key).expect("consumer exists");
        let provider = expanded.module(&provider_key).expect("provider exists");
        let declaration_owner = expanded
            .module(&declaration_owner_key)
            .expect("module-declaration owner exists");
        let projection = normalized_projection(consumer);
        let keys = expanded
            .modules()
            .map(|module| module.key().clone())
            .collect::<Vec<_>>();
        let mut expected_keys = vec![root_key.clone(), consumer_key, provider_key.clone()];
        if depth == 2 {
            expected_keys.push(declaration_owner_key);
        }
        expected_keys.sort();
        let expected_declaration_order = if provider_first {
            vec!["provider".to_owned(), "consumer".to_owned()]
        } else {
            vec!["consumer".to_owned(), "provider".to_owned()]
        };

        assert_eq!(keys, expected_keys, "case {case}: exact canonical keys");
        assert_eq!(
            direct_module_declaration_names(declaration_owner.body()),
            expected_declaration_order,
            "case {case}: encoded declaration order"
        );
        if depth == 2 {
            let root = expanded.module(&root_key).expect("expanded root exists");
            assert_eq!(
                direct_module_declaration_names(root.body()),
                vec!["layer".to_owned()],
                "case {case}: nested owner remains under the root"
            );
        }
        assert!(!contains_macro_invocation(consumer.body()), "case {case}");
        assert!(provider.origins().is_empty(), "case {case}");
        assert!(provider.hygiene().is_empty(), "case {case}");
        assert!(provider.syntax_imports().is_empty(), "case {case}");
        assert_eq!(consumer.origins().len(), function_count, "case {case}");
        assert_eq!(
            expanded_add_literals(consumer.body()),
            vec![increment; function_count],
            "case {case}: template and function-count dimensions"
        );
        assert_eq!(
            projection.syntax_imports,
            vec![(provider_key, "inc".to_owned(), alias.to_owned())],
            "case {case}: alias dimension"
        );
    }
}
