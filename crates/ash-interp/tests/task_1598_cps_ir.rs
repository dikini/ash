//! TASK-1598: Row validation scaffold tests
//!
//! Tests for EffectRow validation: no duplicate items, valid kinds.

use ash_core::cps::*;

#[test]
fn test_validate_row_empty_is_valid() {
    let row = EffectRow::default();
    assert!(row.validate_row().is_ok());
}

#[test]
fn test_validate_row_single_item_valid() {
    let row = EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: "fs.read".to_string(),
            kind: EffectItemKind::Capability,
        }],
    };
    assert!(row.validate_row().is_ok());
}

#[test]
fn test_validate_row_multiple_different_items_valid() {
    let row = EffectRow {
        items: vec![
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            EffectItem {
                namespace: "role".to_string(),
                name: "admin".to_string(),
                kind: EffectItemKind::Role,
            },
            EffectItem {
                namespace: "policy".to_string(),
                name: "audit".to_string(),
                kind: EffectItemKind::Policy,
            },
        ],
    };
    assert!(row.validate_row().is_ok());
}

#[test]
fn test_validate_row_duplicate_items_fails() {
    let row = EffectRow {
        items: vec![
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
        ],
    };
    let result = row.validate_row();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Duplicate"),
        "Error should mention duplicates: {}",
        err
    );
}

#[test]
fn test_validate_row_duplicate_same_namespace_different_kind_fails() {
    // Same namespace+name but different kind should still be a duplicate
    let row = EffectRow {
        items: vec![
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Role,
            },
        ],
    };
    let result = row.validate_row();
    assert!(result.is_err());
}

#[test]
fn test_validate_row_duplicate_different_namespace_ok() {
    // Same name but different namespace is NOT a duplicate
    let row = EffectRow {
        items: vec![
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            EffectItem {
                namespace: "role".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Role,
            },
        ],
    };
    assert!(row.validate_row().is_ok());
}

#[test]
fn test_validate_row_all_valid_kinds() {
    let kinds = vec![
        EffectItemKind::Capability,
        EffectItemKind::Role,
        EffectItemKind::Policy,
        EffectItemKind::Contract,
        EffectItemKind::Channel,
        EffectItemKind::Alias,
        EffectItemKind::Group,
    ];
    for kind in &kinds {
        let kind_clone = *kind;
        let row = EffectRow {
            items: vec![EffectItem {
                namespace: "test".to_string(),
                name: format!("item_{:?}", kind_clone).to_lowercase(),
                kind: kind_clone,
            }],
        };
        assert!(
            row.validate_row().is_ok(),
            "Kind {:?} should be valid",
            kind
        );
    }
}

#[test]
fn test_validate_row_error_message_contains_item_info() {
    let row = EffectRow {
        items: vec![
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            EffectItem {
                namespace: "cap".to_string(),
                name: "fs.read".to_string(),
                kind: EffectItemKind::Capability,
            },
        ],
    };
    let result = row.validate_row();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("cap"),
        "Error should mention namespace: {}",
        err
    );
    assert!(
        err.contains("fs.read"),
        "Error should mention name: {}",
        err
    );
}
