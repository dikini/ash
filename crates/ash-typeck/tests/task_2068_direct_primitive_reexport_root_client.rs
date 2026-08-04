//! TASK-2068 RED contracts for direct primitive re-export root clients.
//!
//! The dedicated route is the only path that may bind an explicit public
//! re-export alias into one private root client. Generic parsed-import and
//! provider/client routes remain fail-closed for source `pub use`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, Expr, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Spanned};
use ash_typeck::{
    CanonicalDirectPrimitiveReexportRootClient, CanonicalDirectPrimitiveReexportRootClientError,
    CanonicalDirectPrimitiveReexportRootClientPlanError, CanonicalModuleBindError, Type,
    bind_simple_parsed_uses, check_direct_primitive_reexport_root_client,
    check_primitive_provider_client, resolve_direct_primitive_interface_imports,
    resolve_direct_primitive_reexport_root_client_plan, resolve_simple_parsed_imports,
};
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
            "ash-task-2068-direct-root-client-{label}-{}-{serial}",
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

fn inline_root_client_graph(
    provider_body: &str,
    root_body: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("pub mod api {{ {provider_body} }} {root_body}"),
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("inline root-client fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_root_client_graph(
    root_source: &str,
    provider_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write("src/api.ash", provider_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file root-client fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn function<'a>(graph: &'a CanonicalModuleGraph, module: &ModuleKey, name: &str) -> &'a FnDef {
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
        .expect("fixture function remains in its parser-owned module unit")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> ash_parser::Span {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture root contains the explicit public re-export")
        .span
}

fn direct_call_span(function: &FnDef, name: &str) -> ash_parser::Span {
    let expression = match &function.body {
        Expr::Block {
            statements,
            tail_expr: Some(tail_expression),
            ..
        } if statements.is_empty() => tail_expression.as_ref(),
        body => panic!(
            "fixture private root body must contain only the direct {name} call, got {body:?}"
        ),
    };
    match expression {
        Expr::Call { func, span, .. } if func.as_ref() == name => *span,
        body => panic!("fixture private root body must be the direct {name} call, got {body:?}"),
    }
}

fn primitive_type(name: &str) -> Type {
    match name {
        "Int" => Type::Int,
        "Bool" => Type::Bool,
        "String" => Type::String,
        "Float" => Type::Float,
        other => panic!("test primitive generator produced unsupported type {other}"),
    }
}

fn unary_primitive_signature(name: &str) -> Type {
    Type::Fn(vec![primitive_type(name)], Box::new(primitive_type(name)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedRootClientFacts {
    child_module: ModuleKey,
    public_alias: String,
    target_module: ModuleKey,
    target_name: String,
    target_signature: Type,
    private_root_module: ModuleKey,
    private_root_name: String,
    private_root_signature: Type,
    private_root_body_type: Type,
    private_root_visibility: Visibility,
    local_alias: String,
    local_target_module: ModuleKey,
    local_target_name: String,
    local_signature: Type,
    local_visibility: Visibility,
}

fn normalize(checked: &CanonicalDirectPrimitiveReexportRootClient) -> NormalizedRootClientFacts {
    let fragments = checked.fragments();
    let child = fragments
        .public_child("api")
        .expect("the dedicated route retains its one public child");
    let reexport = fragments
        .reexport("welcome")
        .expect("the dedicated route retains its explicit public alias");
    let root_function = checked
        .private_root_function("internal_entry")
        .expect("the dedicated route retains its checked private root client");
    let local_alias = checked
        .local_alias_binding("welcome")
        .expect("the dedicated route retains its checked local alias binding");

    NormalizedRootClientFacts {
        child_module: child.module_key().clone(),
        public_alias: reexport.visible_name().to_owned(),
        target_module: reexport.defining_identity().module_key().clone(),
        target_name: reexport.defining_identity().name().to_owned(),
        target_signature: reexport.signature().clone(),
        private_root_module: root_function.defining_identity().module_key().clone(),
        private_root_name: root_function.defining_identity().name().to_owned(),
        private_root_signature: root_function.signature().clone(),
        private_root_body_type: root_function.body_type().clone(),
        private_root_visibility: root_function.visibility().clone(),
        local_alias: local_alias.local_name().to_owned(),
        local_target_module: local_alias.defining_identity().module_key().clone(),
        local_target_name: local_alias.defining_identity().name().to_owned(),
        local_signature: local_alias.signature().clone(),
        local_visibility: local_alias.visibility().clone(),
    }
}

#[test]
fn private_root_client_binds_the_explicit_public_alias_without_changing_target_identity() {
    let (root_key, graph) = inline_root_client_graph(
        r#"
            fn normalize(value: Int) -> Int { value }
            pub fn greet(value: Int) -> Int { normalize(value) }
        "#,
        r#"
            pub use crate::api::greet as welcome;
            fn internal_entry(value: Int) -> Int { welcome(value) }
        "#,
        "positive",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture provider key is canonical");
    let root_artifact = graph
        .module_unit(&root_key)
        .expect("root artifact is graph-owned")
        .artifact()
        .clone();
    let api_artifact = graph
        .module_unit(&api_key)
        .expect("provider artifact is graph-owned")
        .artifact()
        .clone();
    let greet = function(&graph, &api_key, "greet");
    let internal_entry = function(&graph, &root_key, "internal_entry");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect("the exact public re-export and private root client admit a dedicated plan");

    let checked = check_direct_primitive_reexport_root_client(&graph, &plan)
        .expect("the private root client checks after the explicit local alias is bound");
    let fragments = checked.fragments();
    let reexport = fragments
        .reexport("welcome")
        .expect("the checked result retains the explicit public fragment alias");
    let root_function = checked
        .private_root_function("internal_entry")
        .expect("the private root function is retained only in the opaque handoff");
    let local_alias = checked
        .local_alias_binding("welcome")
        .expect("the selected public re-export is bound for this private root client");

    assert_eq!(fragments.root_artifact(), &root_artifact);
    assert_eq!(reexport.defining_identity().module_key(), &api_key);
    assert_eq!(reexport.defining_identity().name(), "greet");
    assert_eq!(reexport.declaration_span(), greet.span);
    assert_eq!(reexport.origin(), api_artifact.origin());
    assert_eq!(reexport.signature(), &unary_primitive_signature("Int"));
    assert_eq!(reexport.use_span(), use_span);
    assert_eq!(root_function.defining_identity().module_key(), &root_key);
    assert_eq!(root_function.defining_identity().name(), "internal_entry");
    assert_eq!(root_function.declaration_span(), internal_entry.span);
    assert_eq!(root_function.body_span(), internal_entry.body.span());
    assert_eq!(root_function.origin(), root_artifact.origin());
    assert_eq!(root_function.visibility(), &Visibility::Inherited);
    assert_eq!(root_function.signature(), &unary_primitive_signature("Int"));
    assert_eq!(root_function.body_type(), &Type::Int);
    assert_eq!(local_alias.local_name(), "welcome");
    assert_eq!(local_alias.use_span(), use_span);
    assert_eq!(
        local_alias.defining_identity(),
        reexport.defining_identity(),
        "the root-local alias preserves api::greet rather than receiving a root identity"
    );
    assert_eq!(local_alias.declaration_span(), greet.span);
    assert_eq!(local_alias.origin(), api_artifact.origin());
    assert_eq!(local_alias.visibility(), &Visibility::Public);
    assert_eq!(local_alias.signature(), &unary_primitive_signature("Int"));
    assert!(
        fragments.reexport("internal_entry").is_none(),
        "a private root client must never become a public re-export"
    );
}

#[test]
fn file_and_inline_root_clients_have_equal_normalized_fragment_provider_root_and_alias_facts() {
    let provider_source = r#"
        fn normalize(value: Int) -> Int { value }
        pub fn greet(value: Int) -> Int { normalize(value) }
    "#;
    let root_body = r#"
        pub use crate::api::greet as welcome;
        fn internal_entry(value: Int) -> Int { welcome(value) }
    "#;
    let (_, inline_graph) =
        inline_root_client_graph(provider_source, root_body, "inline-normalized-parity");
    let (_, file_graph) = file_root_client_graph(
        &format!("pub mod api; {root_body}"),
        provider_source,
        "file-normalized-parity",
    );
    let inline_plan = resolve_direct_primitive_reexport_root_client_plan(&inline_graph)
        .expect("the inline source admits the dedicated root-client plan");
    let file_plan = resolve_direct_primitive_reexport_root_client_plan(&file_graph)
        .expect("the file source admits the dedicated root-client plan");
    let inline_checked = check_direct_primitive_reexport_root_client(&inline_graph, &inline_plan)
        .expect("the inline source checks through the dedicated route");
    let file_checked = check_direct_primitive_reexport_root_client(&file_graph, &file_plan)
        .expect("the file source checks through the dedicated route");

    assert_eq!(normalize(&inline_checked), normalize(&file_checked));
}

#[test]
fn root_local_alias_name_collision_rejects_before_the_local_binding_is_published() {
    let (root_key, graph) = inline_root_client_graph(
        "pub fn greet(value: Int) -> Int { value }",
        r#"
            pub use crate::api::greet as welcome;
            fn welcome(value: Int) -> Int { value }
            fn internal_entry(value: Int) -> Int { welcome(value) }
        "#,
        "local-alias-collision",
    );
    let local = function(&graph, &root_key, "welcome");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect("the exact structural plan remains valid before root-local binding admission");

    let error = check_direct_primitive_reexport_root_client(&graph, &plan)
        .expect_err("a root declaration must not be overwritten by the local alias");

    match error {
        CanonicalDirectPrimitiveReexportRootClientError::LocalAliasCollision {
            root_module,
            local_name,
            local_declaration_span,
            use_span: collision_use_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(local_name, "welcome");
            assert_eq!(local_declaration_span, local.span);
            assert_eq!(collision_use_span, use_span);
        }
        other => panic!("expected anchored root-local alias collision, got {other:?}"),
    }
}

#[test]
fn public_root_client_is_rejected_before_a_dedicated_plan_is_published() {
    let (root_key, graph) = inline_root_client_graph(
        "pub fn greet(value: Int) -> Int { value }",
        r#"
            pub use crate::api::greet as welcome;
            pub fn internal_entry(value: Int) -> Int { welcome(value) }
        "#,
        "public-root-client",
    );
    let internal_entry = function(&graph, &root_key, "internal_entry");

    let error = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect_err("a public root function lies outside the dedicated private root-client plan");

    match error {
        CanonicalDirectPrimitiveReexportRootClientPlanError::PublicRootFunction {
            root_module,
            function,
            declaration_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(function, "internal_entry");
            assert_eq!(declaration_span, internal_entry.span);
        }
        other => panic!("expected anchored public-root plan rejection, got {other:?}"),
    }
}

#[test]
fn incompatible_local_alias_call_reports_the_exact_call_site_without_a_result() {
    let (root_key, graph) = inline_root_client_graph(
        "pub fn greet(value: Int) -> Int { value }",
        r#"
            pub use crate::api::greet as welcome;
            fn internal_entry(value: Int) -> Int { welcome(true) }
        "#,
        "local-alias-body-diagnostic",
    );
    let internal_entry = function(&graph, &root_key, "internal_entry");
    let welcome_call_span = direct_call_span(internal_entry, "welcome");
    let plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect("the incompatible root call preserves a valid structural direct plan");

    let error = check_direct_primitive_reexport_root_client(&graph, &plan)
        .expect_err("an incompatible local alias call must reject without publishing a handoff");

    match error {
        CanonicalDirectPrimitiveReexportRootClientError::RootBodyCheck {
            root_module,
            function,
            declaration_span,
            source,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(function, "internal_entry");
            assert_eq!(declaration_span, internal_entry.span);
            assert_eq!(source.call_or_body_anchor(), welcome_call_span);
        }
        other => panic!("expected anchored private root body diagnostic, got {other:?}"),
    }
}

#[test]
fn dedicated_plan_from_another_artifact_snapshot_rejects_before_root_client_checking() {
    let root_source = r#"
        pub mod api;
        pub use crate::api::greet as welcome;
        fn internal_entry(value: Int) -> Int { welcome(value) }
    "#;
    let (_, planned_graph) = file_root_client_graph(
        root_source,
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "planned-artifact",
    );
    let plan = resolve_direct_primitive_reexport_root_client_plan(&planned_graph)
        .expect("the original graph produces a dedicated root-client plan");
    let (root_key, changed_graph) = file_root_client_graph(
        root_source,
        "pub fn greet(value: Int) -> Int { value + 2 }",
        "changed-artifact",
    );
    assert_eq!(changed_graph.root_key(), &root_key);
    assert_ne!(
        planned_graph
            .module_unit(planned_graph.root_key())
            .expect("planned root unit exists")
            .artifact(),
        changed_graph
            .module_unit(&root_key)
            .expect("changed root unit exists")
            .artifact(),
        "same keys must retain distinct artifacts when the source changes"
    );

    let error = check_direct_primitive_reexport_root_client(&changed_graph, &plan)
        .expect_err("a dedicated root-client plan cannot be replayed against other artifacts");

    assert!(matches!(
        error,
        CanonicalDirectPrimitiveReexportRootClientError::PlanArtifactMismatch { .. }
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_private_root_clients_bind_only_the_selected_public_target_identity(
        suffix in "[a-z0-9_]{0,12}",
        primitive in prop_oneof![Just("Int"), Just("Bool"), Just("String"), Just("Float")],
    ) {
        let alias = format!("alias_{suffix}");
        let provider = format!(
            "fn normalize(value: {primitive}) -> {primitive} {{ value }} pub fn greet(value: {primitive}) -> {primitive} {{ normalize(value) }}"
        );
        let root = format!(
            "pub use crate::api::greet as {alias}; fn internal_entry(value: {primitive}) -> {primitive} {{ {alias}(value) }}"
        );
        let (root_key, graph) = inline_root_client_graph(
            &provider,
            &root,
            "generated-private-root-client",
        );
        let api_key = root_key.child("api").expect("fixture provider key is canonical");
        let plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
            .expect("each generated exact source admits the dedicated plan");
        let checked = check_direct_primitive_reexport_root_client(&graph, &plan)
            .expect("each generated private root client checks through the local alias");
        let reexport = checked
            .fragments()
            .reexport(&alias)
            .expect("the selected public re-export remains in the fragment");
        let root_function = checked
            .private_root_function("internal_entry")
            .expect("the generated private root client is retained");
        let local_alias = checked
            .local_alias_binding(&alias)
            .expect("the generated public alias is bound locally");

        prop_assert_eq!(reexport.defining_identity().module_key(), &api_key);
        prop_assert_eq!(reexport.defining_identity().name(), "greet");
        prop_assert_eq!(reexport.signature(), &unary_primitive_signature(primitive));
        prop_assert_eq!(root_function.defining_identity().module_key(), &root_key);
        prop_assert_eq!(root_function.defining_identity().name(), "internal_entry");
        prop_assert_eq!(root_function.visibility(), &Visibility::Inherited);
        prop_assert_eq!(root_function.signature(), &unary_primitive_signature(primitive));
        prop_assert_eq!(local_alias.local_name(), alias);
        prop_assert_eq!(local_alias.defining_identity(), reexport.defining_identity());
        prop_assert_eq!(local_alias.signature(), reexport.signature());
        prop_assert_eq!(local_alias.visibility(), &Visibility::Public);
        prop_assert!(checked.fragments().reexport("normalize").is_none());
    }
}

#[test]
fn late_invalid_private_root_client_rejects_atomically_without_returning_staged_facts() {
    let (root_key, graph) = inline_root_client_graph(
        "pub fn greet(value: Int) -> Int { value }",
        r#"
            pub use crate::api::greet as welcome;
            fn internal_entry(value: Int) -> Int { welcome(value) }
            fn late_broken() -> Int { true }
        "#,
        "late-invalid-root-client",
    );
    let late_broken = function(&graph, &root_key, "late_broken");
    let plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect("the late root failure preserves the exact dedicated plan shape");

    let error = check_direct_primitive_reexport_root_client(&graph, &plan)
        .expect_err("a late root body failure must publish neither fragments nor local aliases");

    match error {
        CanonicalDirectPrimitiveReexportRootClientError::RootBodyCheck {
            root_module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(function, "late_broken");
            assert_eq!(declaration_span, late_broken.span);
        }
        other => panic!("expected atomic late private-root rejection, got {other:?}"),
    }
}

#[test]
fn generic_routes_reject_source_public_reexports_while_the_dedicated_plan_stays_distinct() {
    let (_, graph) = inline_root_client_graph(
        "pub fn greet(value: Int) -> Int { value }",
        r#"
            pub use crate::api::greet as welcome;
            fn internal_entry(value: Int) -> Int { welcome(value) }
        "#,
        "generic-route-fence",
    );

    assert!(matches!(
        resolve_simple_parsed_imports(&graph),
        Err(CanonicalModuleBindError::Unsupported { .. })
    ));
    assert!(matches!(
        bind_simple_parsed_uses(&graph),
        Err(CanonicalModuleBindError::Unsupported { .. })
    ));

    let public_fragment_plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the legacy fragment planner remains structural-only evidence");
    assert!(
        check_primitive_provider_client(&graph, &public_fragment_plan).is_err(),
        "the generic provider/client checker must reject source pub use instead of accepting it"
    );

    let dedicated_plan = resolve_direct_primitive_reexport_root_client_plan(&graph)
        .expect("only the dedicated resolver admits this explicit public re-export form");
    let checked = check_direct_primitive_reexport_root_client(&graph, &dedicated_plan)
        .expect("the dedicated checker consumes its own opaque plan kind");
    assert!(checked.local_alias_binding("welcome").is_some());
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
fn dedicated_root_client_checker_has_no_compatibility_lowering_admission_or_runtime_authority() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/canonical_primitive_interface_fragments.rs");
    let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated root-client checker in {}: {error}",
            source_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "bind_simple_parsed_uses",
        "CanonicalBoundModuleSet",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
        "Default",
        "eval",
        "evaluate",
        "execute",
    ] {
        assert!(
            !contains_exact_identifier(&source, forbidden_identifier),
            "the dedicated root-client checker must not depend on compatibility, final-interface, lowering, admission, evaluator, or runtime authority: {forbidden_identifier}"
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
            !source.contains(forbidden_bypass),
            "the dedicated root-client checker may consume only canonical graph, its opaque direct plan, primitive facts, and ordinary Type/provenance data: {forbidden_bypass}"
        );
    }
    assert!(
        !source.contains("pub fn new("),
        "the root-client opaque result must not expose a public constructor"
    );
}
