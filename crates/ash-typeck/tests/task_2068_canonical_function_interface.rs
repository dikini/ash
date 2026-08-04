//! TASK-2068 RED contracts for the canonical closed-function interface leaf.
//!
//! The covered M-CHECK leaf consumes already-acquired canonical parser units.
//! It admits only self-contained leaf modules with simple closed primitive
//! function signatures, then atomically publishes a fresh Typeck-owned public
//! projection. It is neither import/binder authority nor a Core, CPS, or
//! Engine capability.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Spanned};
use ash_typeck::{CanonicalModuleCheckError, Type, check_closed_function_modules};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real parser fixture whose drop implementation removes its source tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2068-function-interface-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary parser fixture tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create fixture parent directory");
        fs::write(&path, source).expect("write parser fixture source");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn parsed_graph(source: &str, label: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn parsed_function<'a>(
    graph: &'a CanonicalModuleGraph,
    module: &ModuleKey,
    name: &str,
) -> &'a FnDef {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .expect("fixture function remains in the parser-owned module unit")
}

fn int_signature(parameters: Vec<Type>) -> Type {
    Type::Fn(parameters, Box::new(Type::Int))
}

fn assert_restricted_function_is_private(
    graph: &CanonicalModuleGraph,
    module_key: &ModuleKey,
    name: &str,
    expected_signature: &Type,
) {
    let artifact = graph
        .module_unit(module_key)
        .expect("fixture module is retained by the canonical parser graph")
        .artifact()
        .clone();
    let parsed = parsed_function(graph, module_key, name);

    let checked = check_closed_function_modules(graph)
        .expect("a closed primitive restricted function must check privately");
    let module = checked
        .module(module_key)
        .expect("the checked set is keyed by the canonical module identity");
    let private = module
        .private_function(name)
        .expect("the private checked view retains the restricted function");

    assert_eq!(private.defining_identity().module_key(), module_key);
    assert_eq!(private.defining_identity().name(), name);
    assert_eq!(private.declaration_span(), parsed.span);
    assert_eq!(private.body_span(), parsed.body.span());
    assert_eq!(private.origin(), artifact.origin());
    assert_eq!(private.visibility(), &parsed.visibility);
    assert_eq!(private.signature(), expected_signature);
    assert_eq!(private.body_type(), &Type::Int);
    assert!(
        module.public_interface().exported_function(name).is_none(),
        "a restricted function must never acquire a public projection"
    );
}

#[test]
fn closed_leaf_functions_publish_a_fresh_public_projection_with_checked_parser_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            fn increment(value: Int) -> Int { value + 1 }
            pub fn answer() -> Int { increment(41) }
        "#,
        "positive",
    );
    let root_unit = graph
        .module_unit(&root_key)
        .expect("the root unit is retained by the canonical parser graph");
    let root_artifact = root_unit.artifact().clone();
    let increment = parsed_function(&graph, &root_key, "increment");
    let answer = parsed_function(&graph, &root_key, "answer");

    let checked = check_closed_function_modules(&graph)
        .expect("closed primitive sibling functions must check and publish atomically");
    let module = checked
        .module(&root_key)
        .expect("the checked set is keyed by the canonical root identity");
    let private_increment = module
        .private_function("increment")
        .expect("the private checked view retains the inherited sibling function");
    let public = module.public_interface();
    let public_answer = public
        .exported_function("answer")
        .expect("the public projection retains the public checked function");

    assert_eq!(module.artifact(), &root_artifact);
    assert_eq!(
        private_increment.defining_identity().module_key(),
        &root_key,
        "the private checked function retains its canonical defining module"
    );
    assert_eq!(
        private_increment.defining_identity().name(),
        "increment",
        "the private checked function retains its defining name"
    );
    assert_eq!(private_increment.declaration_span(), increment.span);
    assert_eq!(private_increment.body_span(), increment.body.span());
    assert_eq!(private_increment.origin(), root_artifact.origin());
    assert_eq!(private_increment.visibility(), &increment.visibility);
    assert_eq!(
        private_increment.signature(),
        &int_signature(vec![Type::Int])
    );
    assert_eq!(private_increment.body_type(), &Type::Int);

    assert_eq!(public.module_key(), &root_key);
    assert_eq!(public.origin(), root_artifact.origin());
    assert!(
        public.exported_function("increment").is_none(),
        "the public interface must not flatten an inherited function into its exports"
    );
    assert_eq!(public_answer.defining_identity().module_key(), &root_key);
    assert_eq!(public_answer.defining_identity().name(), "answer");
    assert_eq!(public_answer.declaration_span(), answer.span);
    assert_eq!(public_answer.body_span(), answer.body.span());
    assert_eq!(public_answer.origin(), root_artifact.origin());
    assert_eq!(public_answer.visibility(), &answer.visibility);
    assert_eq!(public_answer.signature(), &int_signature(Vec::new()));
    assert_eq!(public_answer.body_type(), &Type::Int);
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-CRATE
#[test]
fn crate_visible_closed_leaf_function_is_checked_privately_without_a_public_projection() {
    let (root_key, graph) = parsed_graph(
        "pub(crate) fn helper(value: Int) -> Int { value + 1 }",
        "crate-visible-private-function",
    );
    assert_restricted_function_is_private(
        &graph,
        &root_key,
        "helper",
        &int_signature(vec![Type::Int]),
    );
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SUPER
#[test]
fn super_visible_closed_leaf_function_is_checked_privately_without_a_public_projection() {
    let (root_key, graph) = parsed_graph(
        "pub(super) fn helper(value: Int) -> Int { value + 1 }",
        "super-visible-private-function",
    );

    assert_restricted_function_is_private(
        &graph,
        &root_key,
        "helper",
        &int_signature(vec![Type::Int]),
    );
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-IN-CRATE
#[test]
fn crate_path_visible_closed_leaf_function_is_checked_privately_without_a_public_projection() {
    let (root_key, graph) = parsed_graph(
        "pub(in crate::internal) fn helper(value: Int) -> Int { value + 1 }",
        "crate-path-visible-private-function",
    );

    assert_restricted_function_is_private(
        &graph,
        &root_key,
        "helper",
        &int_signature(vec![Type::Int]),
    );
}

#[test]
fn non_crate_restricted_path_is_rejected_as_unsupported_visibility() {
    let (root_key, graph) = parsed_graph(
        "pub(in self::internal) fn helper(value: Int) -> Int { value + 1 }",
        "non-crate-restricted-visibility",
    );
    let helper = parsed_function(&graph, &root_key, "helper");

    let error = check_closed_function_modules(&graph)
        .expect_err("a non-crate restricted path is outside the closed-function leaf domain");

    match error {
        CanonicalModuleCheckError::UnsupportedFunctionFeature {
            module,
            function,
            declaration_span,
            reason,
        } => {
            assert_eq!(module, root_key);
            assert_eq!(function, "helper");
            assert_eq!(declaration_span, helper.span);
            assert_eq!(
                reason,
                "restricted declaration visibility is outside the closed-function leaf domain"
            );
        }
        other => panic!("expected unsupported restricted-visibility rejection, got {other:?}"),
    }
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUB-SELF
#[test]
fn self_visible_closed_leaf_function_is_checked_privately_without_a_public_projection() {
    let (root_key, graph) = parsed_graph(
        "pub(self) fn helper(value: Int) -> Int { value + 1 }",
        "self-visible-private-function",
    );

    assert_restricted_function_is_private(
        &graph,
        &root_key,
        "helper",
        &int_signature(vec![Type::Int]),
    );
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-IDENTITY-PROVENANCE
#[test]
fn restricted_checked_function_preserves_fresh_identity_provenance_and_public_projection_split() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub(super) fn restricted(value: Int) -> Int { value + 1 }
            pub fn public_answer() -> Int { 42 }
        "#,
        "restricted-identity-provenance",
    );
    let artifact = graph
        .module_unit(&root_key)
        .expect("the root unit is retained by the canonical parser graph")
        .artifact()
        .clone();
    let restricted = parsed_function(&graph, &root_key, "restricted");
    let public_answer = parsed_function(&graph, &root_key, "public_answer");

    let checked = check_closed_function_modules(&graph)
        .expect("restricted and public primitive sibling functions must check atomically");
    let module = checked
        .module(&root_key)
        .expect("the checked set retains the root module");
    let private = module
        .private_function("restricted")
        .expect("the restricted function remains an internal checked fact");
    let public = module.public_interface();
    let projected_public = public
        .exported_function("public_answer")
        .expect("the separately public function remains in the public projection");

    assert_eq!(private.defining_identity().module_key(), &root_key);
    assert_eq!(private.defining_identity().name(), "restricted");
    assert_eq!(private.declaration_span(), restricted.span);
    assert_eq!(private.body_span(), restricted.body.span());
    assert_eq!(private.origin(), artifact.origin());
    assert_eq!(private.visibility(), &restricted.visibility);
    assert_eq!(private.signature(), &int_signature(vec![Type::Int]));
    assert_eq!(private.body_type(), &Type::Int);
    assert!(
        public.exported_function("restricted").is_none(),
        "the restricted identity must not be flattened into the public projection"
    );
    assert_eq!(projected_public.defining_identity().module_key(), &root_key);
    assert_eq!(projected_public.defining_identity().name(), "public_answer");
    assert_eq!(projected_public.declaration_span(), public_answer.span);
    assert_eq!(projected_public.body_span(), public_answer.body.span());
    assert_eq!(projected_public.origin(), artifact.origin());
    assert_eq!(projected_public.visibility(), &public_answer.visibility);
    assert_eq!(projected_public.signature(), &int_signature(Vec::new()));
    assert_eq!(projected_public.body_type(), &Type::Int);
}

#[test]
fn a_mismatched_closed_function_body_returns_an_anchored_body_check_error() {
    let (root_key, graph) = parsed_graph("pub fn broken() -> Int { true }", "broken-body");
    let broken = parsed_function(&graph, &root_key, "broken");

    let error = check_closed_function_modules(&graph).expect_err(
        "a body whose inferred type violates its declared signature must reject atomically",
    );

    match error {
        CanonicalModuleCheckError::BodyCheck {
            module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(module, root_key);
            assert_eq!(function, "broken");
            assert_eq!(declaration_span, broken.span);
        }
        other => panic!("expected anchored closed-function body check error, got {other:?}"),
    }
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-SIGNATURE-BODY-DIAGNOSTICS
#[test]
fn restricted_signature_and_body_failures_remain_anchored_at_their_declarations() {
    let (signature_root, signature_graph) = parsed_graph(
        "pub(crate) fn unsupported(value: Option<Int>) -> Option<Int> { value }",
        "restricted-signature-diagnostic",
    );
    let unsupported = parsed_function(&signature_graph, &signature_root, "unsupported");

    let signature_error = check_closed_function_modules(&signature_graph).expect_err(
        "a restricted nonprimitive signature must reject before a public projection exists",
    );
    match signature_error {
        CanonicalModuleCheckError::UnsupportedFunctionFeature {
            module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(module, signature_root);
            assert_eq!(function, "unsupported");
            assert_eq!(declaration_span, unsupported.span);
        }
        other => panic!("expected anchored restricted-signature rejection, got {other:?}"),
    }

    let (body_root, body_graph) = parsed_graph(
        "pub(self) fn broken() -> Int { true }",
        "restricted-body-diagnostic",
    );
    let broken = parsed_function(&body_graph, &body_root, "broken");

    let body_error = check_closed_function_modules(&body_graph).expect_err(
        "a restricted body whose inferred type violates its declaration must reject atomically",
    );
    match body_error {
        CanonicalModuleCheckError::BodyCheck {
            module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(module, body_root);
            assert_eq!(function, "broken");
            assert_eq!(declaration_span, broken.span);
        }
        other => panic!("expected anchored restricted-body rejection, got {other:?}"),
    }
}

#[test]
fn public_nonprimitive_signature_is_rejected_at_the_closed_interface_boundary() {
    let (root_key, graph) = parsed_graph(
        "pub fn expose(value: Option<Int>) -> Option<Int> { value }",
        "public-option-signature",
    );
    let expose = parsed_function(&graph, &root_key, "expose");

    let error = check_closed_function_modules(&graph).expect_err(
        "a public nonprimitive signature must not be misrepresented as a general export-closed interface",
    );

    match error {
        CanonicalModuleCheckError::PublicSignatureOutsideClosedSlice {
            module,
            function,
            declaration_span,
        } => {
            assert_eq!(module, root_key);
            assert_eq!(function, "expose");
            assert_eq!(declaration_span, expose.span);
        }
        other => panic!("expected closed public-signature boundary error, got {other:?}"),
    }
}

#[test]
fn a_late_broken_function_rejects_the_whole_module_without_publishing_an_earlier_public_function() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub fn answer() -> Int { 42 }
            fn broken() -> Int { true }
        "#,
        "atomic-body-failure",
    );
    let broken = parsed_function(&graph, &root_key, "broken");

    let error = check_closed_function_modules(&graph).expect_err(
        "a late body-check failure must reject atomically instead of publishing the earlier public function",
    );

    match error {
        CanonicalModuleCheckError::BodyCheck {
            module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(module, root_key);
            assert_eq!(function, "broken");
            assert_eq!(declaration_span, broken.span);
        }
        other => panic!("expected atomic body-check failure, got {other:?}"),
    }
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-ATOMICITY
#[test]
fn a_late_invalid_restricted_sibling_rejects_atomically_without_a_checked_module_set() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub fn earlier_public() -> Int { 42 }
            pub(in crate::internal) fn late_restricted() -> Int { true }
        "#,
        "restricted-sibling-atomicity",
    );
    let late_restricted = parsed_function(&graph, &root_key, "late_restricted");

    let error = check_closed_function_modules(&graph).expect_err(
        "a late invalid restricted sibling must return no checked module set or partial public projection",
    );

    match error {
        CanonicalModuleCheckError::BodyCheck {
            module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(module, root_key);
            assert_eq!(function, "late_restricted");
            assert_eq!(declaration_span, late_restricted.span);
        }
        other => panic!("expected atomic restricted sibling body-check failure, got {other:?}"),
    }
}

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY
#[test]
fn restricted_leaf_source_form_boundary_allows_a_file_root_and_rejects_an_inline_child_before_projection()
 {
    let (file_root, file_graph) = parsed_graph(
        "pub(crate) fn helper(value: Int) -> Int { value + 1 }",
        "restricted-file-root-source-form",
    );
    assert_restricted_function_is_private(
        &file_graph,
        &file_root,
        "helper",
        &int_signature(vec![Type::Int]),
    );

    let tree = TempTree::new("restricted-inline-child-source-form");
    let root_path = tree.write(
        "src/main.ash",
        "pub mod leaf { pub(crate) fn helper(value: Int) -> Int { value + 1 } }",
    );
    let inline_root = ModuleKey::root("app").expect("fixture crate key is canonical");
    let inline_graph = CanonicalModuleGraphResolver::new()
        .resolve_root(inline_root.clone(), root_path)
        .expect("inline child fixture resolves through the canonical parser graph");
    let inline_declaration = inline_graph
        .module_unit(&inline_root)
        .expect("the inline fixture retains the root unit")
        .body()
        .module_decls()
        .first()
        .expect("the inline fixture contains one child declaration")
        .span;

    let error = check_closed_function_modules(&inline_graph).expect_err(
        "the closed leaf API must reject an inline child before it can claim a projected parity result",
    );
    match error {
        CanonicalModuleCheckError::UnsupportedModuleShape {
            module,
            span,
            reason,
        } => {
            assert_eq!(module, inline_root);
            assert_eq!(span, inline_declaration);
            assert_eq!(
                reason,
                "nested module declarations are outside the closed-function leaf domain"
            );
        }
        other => panic!("expected inline source-form boundary rejection, got {other:?}"),
    }
}

#[test]
fn parsed_use_rejects_the_closed_leaf_module_domain_before_type_checking() {
    let (root_key, graph) = parsed_graph(
        r#"
            use crate::api::greet as greet;
            pub fn answer() -> Int { 42 }
        "#,
        "use-outside-domain",
    );
    let use_span = graph
        .module_unit(&root_key)
        .expect("fixture root unit is retained by the parser graph")
        .body()
        .uses()
        .first()
        .expect("fixture contains a parsed use declaration")
        .span;

    let error = check_closed_function_modules(&graph)
        .expect_err("parsed imports are outside the sealed closed-function module domain");

    match error {
        CanonicalModuleCheckError::UnsupportedModuleShape {
            module,
            span,
            reason,
        } => {
            assert_eq!(module, root_key);
            assert_eq!(span, use_span);
            assert!(
                !reason.is_empty(),
                "the sealed-domain rejection retains a stable reason"
            );
        }
        other => panic!("expected sealed-domain use rejection, got {other:?}"),
    }
}

#[test]
fn nested_module_declaration_rejects_during_global_leaf_preflight_at_the_root_declaration_anchor() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub fn root() -> Int { 42 }
            mod child { pub fn child() -> Int { 1 } }
        "#,
        "nested-module-outside-domain",
    );
    let declaration_span = graph
        .module_unit(&root_key)
        .expect("fixture root unit is retained by the parser graph")
        .body()
        .module_decls()
        .first()
        .expect("fixture root has one parsed nested module declaration")
        .span;

    let error = check_closed_function_modules(&graph).expect_err(
        "a nested module must reject the whole graph during leaf-domain preflight before child bodies are checked",
    );

    match error {
        CanonicalModuleCheckError::UnsupportedModuleShape {
            module,
            span,
            reason,
        } => {
            assert_eq!(module, root_key);
            assert_eq!(
                span, declaration_span,
                "the rejection is anchored at the root's parsed nested-module declaration"
            );
            assert!(
                !reason.is_empty(),
                "the global-preflight rejection retains a stable reason"
            );
        }
        other => panic!("expected nested-module leaf-domain rejection, got {other:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_closed_public_integer_functions_keep_root_identity_type_and_origin(
        value in 0i64..10_000,
    ) {
        let function_name = format!("f_{value}");
        let source = format!("pub fn {function_name}() -> Int {{ {value} }}");
        let (root_key, graph) = parsed_graph(&source, "generated-public-integer");
        let root_origin = graph
            .module_unit(&root_key)
            .expect("fixture root unit remains graph-owned")
            .artifact()
            .origin()
            .clone();

        let checked = check_closed_function_modules(&graph)
            .expect("a generated closed integer function must check");
        let function = checked
            .module(&root_key)
            .expect("the root checked module is present")
            .public_interface()
            .exported_function(&function_name)
            .expect("the generated public function is exported exactly once");

        prop_assert_eq!(function.defining_identity().module_key(), &root_key);
        prop_assert_eq!(function.defining_identity().name(), function_name.as_str());
        prop_assert_eq!(function.signature(), &int_signature(Vec::new()));
        prop_assert_eq!(function.body_type(), &Type::Int);
        prop_assert_eq!(function.origin(), &root_origin);
    }

    // TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PROPERTY
    #[test]
    fn generated_restricted_primitive_functions_remain_private_across_visibility_signature_body_and_source_layout(
        visibility_form in 0usize..4,
        has_parameter in any::<bool>(),
        value in 0i64..10_000,
        multiline_source in any::<bool>(),
    ) {
        let visibility = match visibility_form {
            0 => "pub(crate)",
            1 => "pub(super)",
            2 => "pub(in crate::internal)",
            _ => "pub(self)",
        };
        let function_name = format!("restricted_{visibility_form}_{value}");
        let (parameters, body, signature) = if has_parameter {
            (
                "value: Int",
                format!("value + {value}"),
                int_signature(vec![Type::Int]),
            )
        } else {
            ("", value.to_string(), int_signature(Vec::new()))
        };
        let source = if multiline_source {
            format!("\n    {visibility} fn {function_name}({parameters}) -> Int {{ {body} }}\n")
        } else {
            format!("{visibility} fn {function_name}({parameters}) -> Int {{ {body} }}")
        };
        let (root_key, graph) = parsed_graph(&source, "generated-restricted-function");
        let artifact = graph
            .module_unit(&root_key)
            .expect("generated root unit remains graph-owned")
            .artifact()
            .clone();
        let parsed = parsed_function(&graph, &root_key, &function_name);

        let checked = check_closed_function_modules(&graph)
            .expect("a generated restricted primitive function must check privately");
        let module = checked
            .module(&root_key)
            .expect("the generated root checked module is present");
        let private = module
            .private_function(&function_name)
            .expect("the generated restricted function remains private");

        prop_assert_eq!(private.defining_identity().module_key(), &root_key);
        prop_assert_eq!(private.defining_identity().name(), function_name.as_str());
        prop_assert_eq!(private.declaration_span(), parsed.span);
        prop_assert_eq!(private.body_span(), parsed.body.span());
        prop_assert_eq!(private.origin(), artifact.origin());
        prop_assert_eq!(private.visibility(), &parsed.visibility);
        prop_assert_eq!(private.signature(), &signature);
        prop_assert_eq!(private.body_type(), &Type::Int);
        prop_assert!(module.public_interface().exported_function(&function_name).is_none());
    }
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

// TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-PUBLIC-PROJECTION-AUTHORITY-FENCE
#[test]
fn canonical_public_projection_has_no_import_binder_final_interface_or_runtime_authority() {
    let interface_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/canonical_function_interface.rs");
    let interface_source = fs::read_to_string(&interface_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated canonical closed-function interface at {}: {error}",
            interface_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "type_check_program",
        "type_check_program_in_env",
        "type_check_program_in_env_with_config",
        "type_check_program_in_env_for_module",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "validate_core_program",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "TypeEnvModuleInterfaceCollection",
        "InterfaceImportResolver",
        "NameBinder",
        "ModuleGraph",
        "CanonicalBoundModuleSet",
        "Engine",
        "LegacyModuleResolver",
        "ModuleResolver",
        "RuntimeValue",
        "Cli",
        "Daemon",
    ] {
        assert!(
            !contains_exact_identifier(&interface_source, forbidden_identifier),
            "the canonical closed-function interface must not depend on legacy, final-interface, binder, or runtime authority: {forbidden_identifier}"
        );
    }
    for forbidden_bypass in [
        "module_interface_finalization",
        "interface_import_resolver",
        "canonical_module_binder",
        "module_core_cps_lowering",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !interface_source.contains(forbidden_bypass),
            "the canonical closed-function interface must consume only CanonicalModuleGraph, ModuleKey, TypeEnv, and typed parser/checker payloads: {forbidden_bypass}"
        );
    }
}
