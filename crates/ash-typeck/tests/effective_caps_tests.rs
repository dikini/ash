//! Integration tests for effective capability set composition (TASK-264)
//!
//! These tests verify the capability composition behavior per SPEC-024 Section 2,
//! ensuring workflows can compose capabilities from multiple roles.

use ash_parser::surface::{CapabilityDecl, ConstraintBlock, ConstraintField, ConstraintValue};
use ash_parser::token::Span;
use ash_typeck::effective_caps::{CapabilitySource, EffectiveCapabilitySet};

fn test_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn create_capability_decl(name: &str, constraints: Option<ConstraintBlock>) -> CapabilityDecl {
    CapabilityDecl {
        capability: name.into(),
        constraints,
        span: test_span(),
    }
}

fn create_constraint_block(fields: Vec<(&str, ConstraintValue)>) -> ConstraintBlock {
    let fields = fields
        .into_iter()
        .map(|(name, value)| ConstraintField {
            name: name.into(),
            value,
            span: test_span(),
        })
        .collect();

    ConstraintBlock {
        fields,
        span: test_span(),
    }
}

/// Test single capability added to effective set
#[test]
fn test_single_capability() {
    let mut effective = EffectiveCapabilitySet::new();

    let decl = create_capability_decl("file", None);
    let source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };

    assert!(effective.add(&decl, source).is_ok());
    assert!(effective.has_capability("file"));
    assert_eq!(effective.len(), 1);

    // Check the merged capability
    let merged = effective.get("file").unwrap();
    assert_eq!(merged.name, "file");
    assert!(merged.merged_constraints.is_none());
    assert_eq!(merged.sources.len(), 1);
}

/// Test same capability from multiple roles is merged
#[test]
fn test_multiple_sources_same_capability() {
    let mut effective = EffectiveCapabilitySet::new();

    // Add file capability from ai_agent role
    let decl1 = create_capability_decl("file", None);
    let source1 = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&decl1, source1).is_ok());

    // Add file capability from another role
    let decl2 = create_capability_decl("file", None);
    let source2 = CapabilitySource::Role {
        role_name: "file_processor".to_string(),
    };
    assert!(effective.add(&decl2, source2).is_ok());

    // Should still have only one file capability
    assert_eq!(effective.len(), 1);

    // But should have two sources
    let sources = effective.get_sources("file").unwrap();
    assert_eq!(sources.len(), 2);
    assert!(sources.contains(&CapabilitySource::Role {
        role_name: "ai_agent".to_string()
    }));
    assert!(sources.contains(&CapabilitySource::Role {
        role_name: "file_processor".to_string()
    }));
}

/// Test different capabilities from different roles are composed
#[test]
fn test_different_capabilities_composed() {
    let mut effective = EffectiveCapabilitySet::new();

    // Simulate a workflow like:
    // workflow processor
    //     plays role(ai_agent)      -- provides: file, process
    //     plays role(net_client)    -- provides: network, tls
    //     capabilities: [cache]      -- provides: cache
    // -- Effective: file, process, network, tls, cache

    // Add capabilities from ai_agent role
    let file_decl = create_capability_decl("file", None);
    let file_source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&file_decl, file_source).is_ok());

    let process_decl = create_capability_decl("process", None);
    let process_source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&process_decl, process_source).is_ok());

    // Add capabilities from net_client role
    let network_decl = create_capability_decl("network", None);
    let network_source = CapabilitySource::Role {
        role_name: "net_client".to_string(),
    };
    assert!(effective.add(&network_decl, network_source).is_ok());

    let tls_decl = create_capability_decl("tls", None);
    let tls_source = CapabilitySource::Role {
        role_name: "net_client".to_string(),
    };
    assert!(effective.add(&tls_decl, tls_source).is_ok());

    // Add direct workflow capability (implicit default)
    let cache_decl = create_capability_decl("cache", None);
    let cache_source = CapabilitySource::ImplicitDefault;
    assert!(effective.add(&cache_decl, cache_source).is_ok());

    // All capabilities should be present
    assert!(effective.has_capability("file"));
    assert!(effective.has_capability("process"));
    assert!(effective.has_capability("network"));
    assert!(effective.has_capability("tls"));
    assert!(effective.has_capability("cache"));
    assert_eq!(effective.len(), 5);
}

/// Test implicit default source (direct workflow capabilities)
#[test]
fn test_implicit_default_source() {
    let mut effective = EffectiveCapabilitySet::new();

    // Add capability from implicit default (direct workflow declaration)
    let decl = create_capability_decl("cache", None);
    let source = CapabilitySource::ImplicitDefault;

    assert!(effective.add(&decl, source.clone()).is_ok());
    assert!(effective.has_capability("cache"));

    let sources = effective.get_sources("cache").unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0], source);
    assert_eq!(sources[0].to_string(), "implicit");
}

/// Test composition idempotence - adding same capability multiple times
#[test]
fn test_composition_idempotence() {
    let mut effective = EffectiveCapabilitySet::new();

    // Add same capability twice from same source
    let decl = create_capability_decl("file", None);
    let source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };

    assert!(effective.add(&decl, source.clone()).is_ok());
    assert!(effective.add(&decl, source.clone()).is_ok());
    assert!(effective.add(&decl, source.clone()).is_ok());

    // Should still have one capability with one source
    assert_eq!(effective.len(), 1);
    let sources = effective.get_sources("file").unwrap();
    assert_eq!(sources.len(), 1);
}

/// Test constraint merging when same capability has different constraints
#[test]
fn test_constraint_merge_union() {
    let mut effective = EffectiveCapabilitySet::new();

    // First declaration with read constraint
    let decl1 = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "read",
            ConstraintValue::Bool(true),
        )])),
    );
    let source1 = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&decl1, source1).is_ok());

    // Second declaration with write constraint
    let decl2 = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "write",
            ConstraintValue::Bool(true),
        )])),
    );
    let source2 = CapabilitySource::Role {
        role_name: "file_processor".to_string(),
    };
    assert!(effective.add(&decl2, source2).is_ok());

    // Constraints should be merged (union)
    let constraint = effective.get_constraint("file").unwrap();
    assert_eq!(constraint.fields.len(), 2);

    // Both fields should be present
    let field_names: Vec<_> = constraint.fields.iter().map(|f| f.name.as_ref()).collect();
    assert!(field_names.contains(&"read"));
    assert!(field_names.contains(&"write"));
}

/// Test check_use with valid subset constraints
#[test]
fn test_check_use_valid_subset() {
    let mut effective = EffectiveCapabilitySet::new();

    // Add capability with array constraints
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![
            ("read", ConstraintValue::Bool(true)),
            (
                "paths",
                ConstraintValue::Array(vec![
                    ConstraintValue::String("/tmp/*".to_string()),
                    ConstraintValue::String("/home/*".to_string()),
                    ConstraintValue::String("/var/log/*".to_string()),
                ]),
            ),
        ])),
    );
    let source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&decl, source).is_ok());

    // Use with subset of paths should be valid
    let use_constraints = create_constraint_block(vec![
        ("read", ConstraintValue::Bool(true)),
        (
            "paths",
            ConstraintValue::Array(vec![
                ConstraintValue::String("/tmp/*".to_string()),
                ConstraintValue::String("/home/*".to_string()),
            ]),
        ),
    ]);

    assert!(effective.check_use("file", &use_constraints));
}

/// Test check_use with incompatible constraint value
#[test]
fn test_check_use_incompatible_value() {
    let mut effective = EffectiveCapabilitySet::new();

    // Add capability with read=true
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "read",
            ConstraintValue::Bool(true),
        )])),
    );
    let source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(effective.add(&decl, source).is_ok());

    // Use with read=false should be invalid
    let use_constraints = create_constraint_block(vec![("read", ConstraintValue::Bool(false))]);

    assert!(!effective.check_use("file", &use_constraints));
}

/// Test check_use for unknown capability
#[test]
fn test_check_use_unknown_capability() {
    let effective = EffectiveCapabilitySet::new();

    let use_constraints = create_constraint_block(vec![]);
    assert!(!effective.check_use("unknown_capability", &use_constraints));
}

/// Test empty capability set
#[test]
fn test_empty_capability_set() {
    let effective = EffectiveCapabilitySet::new();

    assert!(effective.is_empty());
    assert_eq!(effective.len(), 0);
    assert!(!effective.has_capability("anything"));
}

/// Test merging two effective capability sets
#[test]
fn test_merge_effective_sets() {
    let mut set1 = EffectiveCapabilitySet::new();
    let mut set2 = EffectiveCapabilitySet::new();

    // Add to set1
    let decl1 = create_capability_decl("file", None);
    let source1 = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };
    assert!(set1.add(&decl1, source1).is_ok());

    // Add to set2
    let decl2 = create_capability_decl("network", None);
    let source2 = CapabilitySource::Role {
        role_name: "net_client".to_string(),
    };
    assert!(set2.add(&decl2, source2).is_ok());

    // Merge set2 into set1
    assert!(set1.merge(&set2).is_ok());

    assert!(set1.has_capability("file"));
    assert!(set1.has_capability("network"));
    assert_eq!(set1.len(), 2);
}

/// Test capability_names iterator
#[test]
fn test_capability_names_iterator() {
    let mut effective = EffectiveCapabilitySet::new();

    effective
        .add(
            &create_capability_decl("file", None),
            CapabilitySource::ImplicitDefault,
        )
        .unwrap();
    effective
        .add(
            &create_capability_decl("network", None),
            CapabilitySource::ImplicitDefault,
        )
        .unwrap();
    effective
        .add(
            &create_capability_decl("cache", None),
            CapabilitySource::ImplicitDefault,
        )
        .unwrap();

    let names: Vec<_> = effective.capability_names().cloned().collect();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"file".to_string()));
    assert!(names.contains(&"network".to_string()));
    assert!(names.contains(&"cache".to_string()));
}

/// Test MergedCapability structure
#[test]
fn test_merged_capability_structure() {
    let mut effective = EffectiveCapabilitySet::new();

    let constraints = create_constraint_block(vec![("read", ConstraintValue::Bool(true))]);

    let decl = create_capability_decl("file", Some(constraints.clone()));
    let source = CapabilitySource::Role {
        role_name: "ai_agent".to_string(),
    };

    effective.add(&decl, source).unwrap();

    let merged = effective.get("file").unwrap();
    assert_eq!(merged.name, "file");
    assert!(merged.merged_constraints.is_some());
    assert_eq!(merged.sources.len(), 1);
}

/// Test complex workflow scenario with multiple roles and constraints
#[test]
fn test_complex_workflow_scenario() {
    // workflow data_processor
    //     plays role(ai_agent)
    //     plays role(storage_manager)
    //     capabilities: [logging @ { level: "info" }]
    //
    // ai_agent provides: file, network
    // storage_manager provides: database, cache
    // direct: logging

    let mut effective = EffectiveCapabilitySet::new();

    // ai_agent role
    effective
        .add(
            &create_capability_decl(
                "file",
                Some(create_constraint_block(vec![
                    ("read", ConstraintValue::Bool(true)),
                    ("write", ConstraintValue::Bool(false)),
                ])),
            ),
            CapabilitySource::Role {
                role_name: "ai_agent".to_string(),
            },
        )
        .unwrap();

    effective
        .add(
            &create_capability_decl("network", None),
            CapabilitySource::Role {
                role_name: "ai_agent".to_string(),
            },
        )
        .unwrap();

    // storage_manager role
    effective
        .add(
            &create_capability_decl("database", None),
            CapabilitySource::Role {
                role_name: "storage_manager".to_string(),
            },
        )
        .unwrap();

    effective
        .add(
            &create_capability_decl("cache", None),
            CapabilitySource::Role {
                role_name: "storage_manager".to_string(),
            },
        )
        .unwrap();

    // Direct workflow capability
    effective
        .add(
            &create_capability_decl(
                "logging",
                Some(create_constraint_block(vec![(
                    "level",
                    ConstraintValue::String("info".to_string()),
                )])),
            ),
            CapabilitySource::ImplicitDefault,
        )
        .unwrap();

    // Verify all capabilities are present
    assert_eq!(effective.len(), 5);
    assert!(effective.has_capability("file"));
    assert!(effective.has_capability("network"));
    assert!(effective.has_capability("database"));
    assert!(effective.has_capability("cache"));
    assert!(effective.has_capability("logging"));

    // Verify file has correct sources
    let file_sources = effective.get_sources("file").unwrap();
    assert_eq!(file_sources.len(), 1);
    assert_eq!(
        file_sources[0],
        CapabilitySource::Role {
            role_name: "ai_agent".to_string()
        }
    );

    // Verify logging has implicit source
    let logging_sources = effective.get_sources("logging").unwrap();
    assert_eq!(logging_sources.len(), 1);
    assert_eq!(logging_sources[0], CapabilitySource::ImplicitDefault);
}
