//! Tests for TASK-539: Sibling type cross-reference resolution via pre-declaration.
//!
//! These tests verify that TypeEnv::declare_type_name() allows forward-referencing
//! sibling types by inserting a placeholder that resolve_type can find, and that
//! register_type() upgrades placeholders to full definitions.

use ash_core::ast::{
    Name, TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility,
};
use ash_typeck::TypeEnv;

/// Helper: create a unit-enum TypeDef (e.g., `pub type Role = A | B | C;`)
fn unit_enum_def(name: &str, variants: &[&str]) -> TypeDef {
    TypeDef {
        name: Name::from(name),
        params: vec![],
        body: TypeBody::Enum(
            variants
                .iter()
                .map(|v| VariantDef {
                    name: Name::from(*v),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                })
                .collect(),
        ),
        visibility: Visibility::Public,
    }
}

/// Helper: create a struct TypeDef with named fields (e.g., `pub type Msg = Msg { role: Role, content: String };`)
fn struct_def(name: &str, fields: Vec<(&str, &str)>) -> TypeDef {
    TypeDef {
        name: Name::from(name),
        params: vec![],
        body: TypeBody::Struct(
            fields
                .into_iter()
                .map(|(n, t)| (Name::from(n), TypeExpr::Named(Name::from(t))))
                .collect(),
        ),
        visibility: Visibility::Public,
    }
}

/// Helper: create a struct with a generic field (e.g., `children: List<Tree>`)
fn struct_def_with_generic(name: &str, fields: Vec<(&str, TypeExpr)>) -> TypeDef {
    TypeDef {
        name: Name::from(name),
        params: vec![],
        body: TypeBody::Struct(
            fields
                .into_iter()
                .map(|(n, t)| (Name::from(n), t))
                .collect(),
        ),
        visibility: Visibility::Public,
    }
}

/// ST-1: Two types with forward reference register without error.
///
/// Message references Role, but Message is registered first.
/// With pre-declaration, both should succeed in any order.
#[test]
fn test_forward_reference_succeeds_with_predeclare() {
    let role_def = unit_enum_def("Role", &["System", "User", "Assistant", "Tool"]);
    let msg_def = struct_def("Message", vec![("role", "Role"), ("content", "String")]);

    let mut env = TypeEnv::with_builtin_types();

    // Pre-declare both names so resolve_type can find them
    env.declare_type_name("Role");
    env.declare_type_name("Message");

    // Register Message first (references Role which is only pre-declared)
    env.register_type(&msg_def).expect("Message should register with Role pre-declared");
    env.register_type(&role_def).expect("Role should register (upgrade placeholder)");

    // Verify both resolved
    assert!(env.resolve_type("Role").is_ok(), "Role should resolve");
    assert!(env.resolve_type("Message").is_ok(), "Message should resolve");
}

/// ST-2: All 11 SPEC-029 types register without error when pre-declared.
#[test]
fn test_all_spec029_types_register() {
    let type_defs: Vec<TypeDef> = vec![
        unit_enum_def("Role", &["System", "User", "Assistant", "Tool"]),
        struct_def("ToolCall", vec![("id", "String"), ("function_name", "String"), ("arguments", "String")]),
        struct_def("ToolCallDelta", vec![("index", "Int"), ("id", "String"), ("function_name", "String"), ("arguments", "String")]),
        struct_def("Message", vec![("role", "Role"), ("content", "String"), ("tool_calls", "ToolCall"), ("tool_call_id", "String")]),
        struct_def("ToolDef", vec![("name", "String"), ("description", "String"), ("parameters", "String")]),
        struct_def("Usage", vec![("prompt_tokens", "Int"), ("completion_tokens", "Int"), ("total_tokens", "Int")]),
        struct_def("ChatResponse", vec![("content", "String"), ("finish_reason", "String"), ("usage", "Usage"), ("model", "String")]),
        struct_def("ChatChunk", vec![("delta_content", "String"), ("delta_tool_calls", "ToolCallDelta"), ("finish_reason", "String")]),
        struct_def("Embedding", vec![("embedding", "String"), ("index", "Int"), ("model", "String")]),
        struct_def("CompletionParams", vec![("temperature", "Int"), ("top_p", "Int"), ("max_tokens", "Int"), ("stop", "String"), ("seed", "Int")]),
        struct_def("ProviderConfig", vec![("name", "String"), ("api_base", "String"), ("api_key", "String"), ("default_model", "String")]),
    ];

    let mut env = TypeEnv::with_builtin_types();

    // Pre-declare all names
    for def in &type_defs {
        env.declare_type_name(&def.name.clone());
    }

    // Register all (order should not matter)
    for def in &type_defs {
        env.register_type(def).unwrap_or_else(|e| {
            panic!("Failed to register {}: {e}", def.name);
        });
    }

    // All should resolve
    for def in &type_defs {
        assert!(env.resolve_type(&def.name).is_ok(), "{} should resolve", def.name);
    }
}

/// ST-3: Reference to truly unbound type produces descriptive error.
#[test]
fn test_unbound_type_still_errors() {
    let bad_def = struct_def("BadType", vec![("field", "NoSuchType")]);

    let mut env = TypeEnv::with_builtin_types();

    // Pre-declare only BadType, not NoSuchType
    env.declare_type_name("BadType");

    let result = env.register_type(&bad_def);
    assert!(result.is_err(), "Should fail with unbound type");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Unbound") || err_msg.contains("unbound") || err_msg.contains("not found"),
        "Error should mention unbound type: {err_msg}"
    );
}

/// ST-4: Self-referential type (Tree { children: List<Tree> }) registers.
#[test]
fn test_self_referential_type_registers() {
    let tree_def = struct_def_with_generic(
        "Tree",
        vec![(
            "children",
            TypeExpr::Constructor {
                name: Name::from("List"),
                args: vec![TypeExpr::Named(Name::from("Tree"))],
            },
        )],
    );

    let mut env = TypeEnv::with_builtin_types();
    env.declare_type_name("Tree");

    env.register_type(&tree_def).expect("Self-referential Tree should register");
    assert!(env.resolve_type("Tree").is_ok(), "Tree should resolve");
}

/// ST-5: Generic references (List<Role>, Option<Message>) resolve with builtin+imported types.
#[test]
fn test_generic_reference_resolves() {
    let role_def = unit_enum_def("Role", &["System", "User", "Assistant", "Tool"]);
    let msg_def = struct_def("Message", vec![("role", "Role"), ("content", "String")]);

    // Container type that uses List<Role> and Option<Message>
    let container_def = TypeDef {
        name: Name::from("Container"),
        params: vec![],
        body: TypeBody::Struct(vec![
            (
                Name::from("roles"),
                TypeExpr::Constructor {
                    name: Name::from("List"),
                    args: vec![TypeExpr::Named(Name::from("Role"))],
                },
            ),
            (
                Name::from("last_msg"),
                TypeExpr::Constructor {
                    name: Name::from("Option"),
                    args: vec![TypeExpr::Named(Name::from("Message"))],
                },
            ),
        ]),
        visibility: Visibility::Public,
    };

    let mut env = TypeEnv::with_builtin_types();

    // Pre-declare all
    env.declare_type_name("Role");
    env.declare_type_name("Message");
    env.declare_type_name("Container");

    // Register
    env.register_type(&role_def).expect("Role should register");
    env.register_type(&msg_def).expect("Message should register");
    env.register_type(&container_def).expect("Container with generics should register");

    assert!(env.resolve_type("Container").is_ok(), "Container should resolve");
}

/// Non-placeholder duplicate still errors.
#[test]
fn test_non_placeholder_duplicate_still_errors() {
    let def1 = unit_enum_def("Dup", &["A", "B"]);
    let def2 = struct_def("Dup", vec![("x", "Int")]);

    let mut env = TypeEnv::with_builtin_types();

    // Register normally (not via declare+upgrade)
    env.register_type(&def1).expect("First registration should succeed");

    // Try to register again without pre-declaration
    let result = env.register_type(&def2);
    assert!(result.is_err(), "Non-placeholder duplicate should error");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("already defined") || err_msg.contains("Duplicate"),
        "Error should mention duplicate: {err_msg}"
    );
}
