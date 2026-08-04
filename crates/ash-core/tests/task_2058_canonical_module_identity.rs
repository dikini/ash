//! TASK-2058 contract tests for canonical module keys and durable module artifacts.
//!
//! These tests deliberately exercise the new `module_graph` carrier rather
//! than the legacy allocation-backed `semantic_summary::ModuleIdentity`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::module_graph::{
    MODULE_ARTIFACT_SCHEMA_VERSION, ModuleArtifact, ModuleArtifactOrigin, ModuleKey,
};
use proptest::prelude::*;

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn root(crate_name: &str) -> ModuleKey {
    ModuleKey::root(crate_name).expect("fixture crate name is canonical")
}

fn child(parent: &ModuleKey, segment: &str) -> ModuleKey {
    parent
        .child(segment)
        .expect("fixture child segment is canonical")
}

fn is_reserved_module_keyword(segment: &str) -> bool {
    matches!(
        segment,
        "workflow"
            | "capability"
            | "policy"
            | "role"
            | "observe"
            | "orient"
            | "propose"
            | "decide"
            | "act"
            | "oblige"
            | "check"
            | "let"
            | "if"
            | "then"
            | "else"
            | "for"
            | "do"
            | "with"
            | "on"
            | "handle"
            | "maybe"
            | "must"
            | "attempt"
            | "retry"
            | "timeout"
            | "done"
            | "ret"
            | "epistemic"
            | "deliberative"
            | "evaluative"
            | "operational"
            | "authority"
            | "obligations"
            | "when"
            | "returns"
            | "where"
            | "law"
            | "proof"
            | "by_definition"
            | "permit"
            | "deny"
            | "require_approval"
            | "escalate"
            | "fn"
            | "panic"
            | "match"
            | "fail"
            | "with_error"
            | "requires"
            | "ensures"
            | "set"
            | "send"
            | "in"
            | "not"
            | "and"
            | "or"
            | "true"
            | "false"
            | "null"
    )
}

fn file_artifact(
    key: ModuleKey,
    parent: Option<ModuleKey>,
    path: &str,
    children: Vec<ModuleKey>,
) -> ModuleArtifact {
    ModuleArtifact::new(
        key,
        ModuleArtifactOrigin::File(path.into()),
        parent,
        children,
    )
    .expect("fixture artifact is structurally valid")
}

#[test]
fn canonical_key_is_crate_qualified_and_independent_of_file_layout() {
    let root = root("garden");
    let plants = child(&root, "plants");
    let direct_file = file_artifact(
        plants.clone(),
        Some(root.clone()),
        "src/plants.ash",
        Vec::new(),
    );
    let directory_file =
        file_artifact(plants.clone(), Some(root), "src/plants/mod.ash", Vec::new());

    assert_eq!(plants.to_string(), "garden::plants");
    assert_eq!(direct_file.key(), directory_file.key());
    assert_eq!(
        direct_file.key().cache_key(),
        directory_file.key().cache_key()
    );
    assert_ne!(direct_file.origin(), directory_file.origin());
}

#[test]
fn nested_keys_round_trip_through_parent_and_serde_without_allocation_identity() {
    let root = root("orchard");
    let ui = child(&root, "ui");
    let theme = child(&ui, "theme");

    assert_eq!(root.to_string(), "orchard");
    assert_eq!(theme.to_string(), "orchard::ui::theme");
    assert_eq!(theme.parent(), Some(ui.clone()));
    assert_eq!(ui.parent(), Some(root.clone()));
    assert_eq!(ModuleKey::root("orchard").expect("same root"), root.clone());

    let json = serde_json::to_string(&theme).expect("key serializes");
    let restored: ModuleKey = serde_json::from_str(&json).expect("key deserializes");
    assert_eq!(restored, theme);
    assert_eq!(hash_of(&restored), hash_of(&theme));
    assert_eq!(restored.cache_key(), theme.cache_key());
}

#[test]
fn different_crates_never_share_a_canonical_key_or_cache_key() {
    let first = child(&root("orchard"), "ui");
    let second = child(&root("greenhouse"), "ui");

    assert_ne!(first, second);
    assert_ne!(first.cache_key(), second.cache_key());
}

#[test]
fn empty_or_noncanonical_child_segments_are_rejected_before_identity_exists() {
    let root = root("orchard");

    assert!(ModuleKey::root("").is_err());
    for invalid in ["", "with space", "nested::path", "../escape", "-dash"] {
        assert!(
            root.child(invalid).is_err(),
            "{invalid:?} must not become a key"
        );
    }
}

#[test]
fn canonical_keys_accept_parser_valid_identifier_spellings() {
    let root = ModuleKey::root("Thing").expect("uppercase parser identifier is a crate key");
    let private = root
        .child("_private")
        .expect("leading underscore parser identifier is a child key");
    let hyphenated = private
        .child("with-error")
        .expect("hyphenated parser identifier is a child key");

    assert_eq!(hyphenated.to_string(), "Thing::_private::with-error");
    assert_eq!(hyphenated.parent(), Some(private));
    assert_eq!(
        hyphenated.cache_key(),
        "module-key/v1/Thing::_private::with-error"
    );
}

#[test]
fn module_child_keys_reject_reserved_parser_keywords() {
    let root = ModuleKey::root("Thing").expect("fixture root is valid");

    for reserved in ["fn", "let", "if", "true"] {
        assert!(
            root.child(reserved).is_err(),
            "reserved parser keyword {reserved:?} must not become a child key"
        );
    }
}

#[test]
fn root_key_accepts_the_current_crate_root_parser_domain() {
    let root = ModuleKey::root("42").expect("crate-root parser accepts numeric crate names");

    assert_eq!(root.to_string(), "42");
    assert_eq!(root.cache_key(), "module-key/v1/42");
    assert_eq!(root.parent(), None);
}

#[test]
fn module_key_wire_rejects_unknown_fields() {
    let wire = serde_json::json!({
        "crate_name": "orchard",
        "segments": [],
        "forged_cache_key": "module-key/v1/other",
    });

    assert!(serde_json::from_value::<ModuleKey>(wire).is_err());
}

#[test]
fn module_key_wire_rejects_invalid_segments() {
    let wire = serde_json::json!({
        "crate_name": "orchard",
        "segments": ["not::a::segment"],
    });

    assert!(serde_json::from_value::<ModuleKey>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_unknown_fields() {
    let root = root("orchard");
    let mut wire = serde_json::to_value(file_artifact(root, None, "src/lib.ash", Vec::new()))
        .expect("fixture artifact serializes");
    wire["forged_cache_key"] = serde_json::json!("module-key/v1/other");

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_invalid_key_segment() {
    let root = root("orchard");
    let mut wire = serde_json::to_value(file_artifact(root, None, "src/lib.ash", Vec::new()))
        .expect("fixture artifact serializes");
    wire["key"]["segments"] = serde_json::json!(["not::a::segment"]);

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_unsupported_schema() {
    let root = root("orchard");
    let mut wire = serde_json::to_value(file_artifact(root, None, "src/lib.ash", Vec::new()))
        .expect("fixture artifact serializes");
    wire["schema_version"] = serde_json::json!(MODULE_ARTIFACT_SCHEMA_VERSION + 1);

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_duplicate_child_keys() {
    let root = root("orchard");
    let child = child(&root, "child");
    let mut wire = serde_json::to_value(file_artifact(
        root,
        None,
        "src/lib.ash",
        vec![child.clone()],
    ))
    .expect("fixture artifact serializes");
    let child_wire = serde_json::to_value(child).expect("child key serializes");
    wire["child_keys"] = serde_json::json!([child_wire.clone(), child_wire]);

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_mismatched_inline_origin_parent() {
    let root = root("orchard");
    let inline = child(&root, "inline");
    let sibling = child(&root, "sibling");
    let artifact = ModuleArtifact::new(
        inline,
        ModuleArtifactOrigin::Inline {
            parent: root.clone(),
            declaration_offset: 41,
        },
        Some(root),
        Vec::new(),
    )
    .expect("fixture inline artifact is valid");
    let mut wire = serde_json::to_value(artifact).expect("fixture artifact serializes");
    wire["origin"]["inline"]["parent"] =
        serde_json::to_value(sibling).expect("sibling key serializes");

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn module_artifact_wire_rejects_unknown_nested_inline_origin_fields() {
    let root = root("orchard");
    let inline = child(&root, "inline");
    let artifact = ModuleArtifact::new(
        inline,
        ModuleArtifactOrigin::Inline {
            parent: root.clone(),
            declaration_offset: 41,
        },
        Some(root),
        Vec::new(),
    )
    .expect("fixture inline artifact is valid");
    let mut wire = serde_json::to_value(artifact).expect("fixture artifact serializes");
    wire["origin"]["inline"]["forged_source_text"] = serde_json::json!("forged");

    assert!(serde_json::from_value::<ModuleArtifact>(wire).is_err());
}

#[test]
fn inline_origin_retains_parent_key_and_offset_without_text_reconstruction() {
    let root = root("orchard");
    let inline = child(&root, "generated");
    let artifact = ModuleArtifact::new(
        inline.clone(),
        ModuleArtifactOrigin::Inline {
            parent: root.clone(),
            declaration_offset: 41,
        },
        Some(root.clone()),
        Vec::new(),
    )
    .expect("matching inline origin is publishable");

    assert_eq!(artifact.key(), &inline);
    assert_eq!(artifact.structural_parent(), Some(&root));
    assert_eq!(
        artifact.origin(),
        &ModuleArtifactOrigin::Inline {
            parent: root,
            declaration_offset: 41,
        }
    );
    let json = serde_json::to_string(&artifact).expect("artifact serializes");
    assert!(json.contains("inline"));
    assert!(!json.contains("source_text"));
}

#[test]
fn artifact_rejects_mismatched_child_parent_duplicate_child_and_inline_origin_parent() {
    let root = root("orchard");
    let branch = child(&root, "branch");
    let sibling = child(&root, "sibling");
    let leaf = child(&branch, "leaf");

    assert!(
        ModuleArtifact::new(
            root.clone(),
            ModuleArtifactOrigin::File("src/lib.ash".into()),
            None,
            vec![leaf.clone()],
        )
        .is_err()
    );
    assert!(
        ModuleArtifact::new(
            branch.clone(),
            ModuleArtifactOrigin::File("src/branch.ash".into()),
            Some(root.clone()),
            vec![leaf.clone(), leaf],
        )
        .is_err()
    );
    assert!(
        ModuleArtifact::new(
            child(&branch, "inline"),
            ModuleArtifactOrigin::Inline {
                parent: sibling,
                declaration_offset: 7,
            },
            Some(branch),
            Vec::new(),
        )
        .is_err()
    );
}

#[test]
fn artifact_canonicalizes_children_and_round_trips_schema_and_origin() {
    let root = root("orchard");
    let alpha = child(&root, "alpha");
    let beta = child(&root, "beta");
    let artifact = file_artifact(
        root.clone(),
        None,
        "src/lib.ash",
        vec![beta.clone(), alpha.clone()],
    );

    assert_eq!(artifact.schema_version(), MODULE_ARTIFACT_SCHEMA_VERSION);
    assert_eq!(artifact.child_keys(), &[alpha, beta]);
    let json = serde_json::to_string(&artifact).expect("artifact serializes");
    let restored: ModuleArtifact = serde_json::from_str(&json).expect("artifact deserializes");
    assert_eq!(restored, artifact);
    assert_eq!(restored.key().cache_key(), artifact.key().cache_key());
}

proptest! {
    #[test]
    fn generated_segments_produce_stable_descendants_and_parent_round_trips(
        crate_name in "[a-z][a-z0-9_]{0,8}",
        segments in proptest::collection::vec(
            "[a-z][a-z0-9_]{0,8}".prop_filter(
                "generated child segment must not be a reserved module keyword",
                |segment| !is_reserved_module_keyword(segment),
            ),
            1..8,
        ),
    ) {
        let root = ModuleKey::root(&crate_name).expect("generated crate name is valid");
        let mut key = root.clone();
        let mut parents = vec![root];

        for segment in &segments {
            key = key.child(segment).expect("generated segment is valid");
            parents.push(key.clone());
        }

        prop_assert_eq!(key.segments(), segments.as_slice());
        prop_assert_eq!(key.to_string(), format!("{crate_name}::{}", segments.join("::")));
        prop_assert_eq!(key.parent(), Some(parents[parents.len() - 2].clone()));
        prop_assert_eq!(
            serde_json::from_str::<ModuleKey>(&serde_json::to_string(&key).expect("serialize"))
                .expect("deserialize"),
            key,
        );
    }

    #[test]
    fn generated_children_are_sorted_and_remain_structurally_owned(
        crate_name in "[a-z][a-z0-9_]{0,8}",
        left in "a[a-z0-9_]{0,7}".prop_filter(
            "generated child segment must not be a reserved module keyword",
            |segment| !is_reserved_module_keyword(segment),
        ),
        right in "z[a-z0-9_]{0,7}".prop_filter(
            "generated child segment must not be a reserved module keyword",
            |segment| !is_reserved_module_keyword(segment),
        ),
    ) {
        let root = ModuleKey::root(&crate_name).expect("generated crate name is valid");
        let left = root.child(&left).expect("generated segment is valid");
        let right = root.child(&right).expect("generated segment is valid");
        let artifact = ModuleArtifact::new(
            root,
            ModuleArtifactOrigin::File("src/lib.ash".into()),
            None,
            vec![right.clone(), left.clone()],
        )
        .expect("generated children are distinct direct descendants");

        prop_assert_eq!(artifact.child_keys(), &[left, right]);
        for child in artifact.child_keys() {
            prop_assert_eq!(child.parent(), Some(artifact.key().clone()));
        }
    }
}
