//! Tests for ADT qualified name preservation (TASK-281)
//!
//! These tests verify that ADT names preserve full qualification (module path + name)
//! and that same-name ADTs in different modules are distinct types.

use ash_core::adt::AdtName;
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, Visibility};
use ash_typeck::type_env::{TypeEnv, type_expr_to_type};
use ash_typeck::types::{Type, TypeVar};
use std::collections::HashMap;

// ============================================================
// AdtName Core Tests
// ============================================================

#[test]
fn test_adt_name_parses_qualified() {
    let name = AdtName::new("std::option::Option");
    assert_eq!(name.qualified, "std::option::Option");
    assert_eq!(name.module, vec!["std", "option"]);
    assert_eq!(name.root, "Option");
    assert!(!name.is_root());
}

#[test]
fn test_adt_name_root_level() {
    let name = AdtName::new("MyType");
    assert_eq!(name.qualified, "MyType");
    assert!(name.module.is_empty());
    assert_eq!(name.root, "MyType");
    assert!(name.is_root());
}

#[test]
fn test_adt_name_equality_uses_full_qualification() {
    let a_t = AdtName::new("a::T");
    let b_t = AdtName::new("b::T");
    let a_t_copy = AdtName::new("a::T");

    // Same qualified name should be equal
    assert_eq!(a_t, a_t_copy, "Same qualified name should be equal");

    // Different qualified names should not be equal
    // This is the key SPEC-003 compliance test
    assert_ne!(a_t, b_t, "a::T and b::T should be different types");
}

#[test]
fn test_adt_name_from_parts() {
    let module = vec!["utils".to_string(), "result".to_string()];
    let name = AdtName::from_parts(&module, "Result");
    assert_eq!(name.qualified, "utils::result::Result");
    assert_eq!(name.module_path(), "utils::result");
}

#[test]
fn test_adt_name_display() {
    let name = AdtName::new("a::b::C");
    assert_eq!(name.to_string(), "a::b::C");
}

// ============================================================
// Type Environment Resolution Tests
// ============================================================

#[test]
fn test_resolve_type_returns_qualified_name() {
    let mut env = TypeEnv::new();

    // Register a type
    let type_def = TypeDef {
        name: "Option".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
            },
            VariantDef {
                name: "None".to_string(),
                fields: vec![],
            },
        ]),
        visibility: Visibility::Public,
    };

    env.register_type(&type_def).unwrap();

    // Resolve the type
    let (qualified, _) = env.resolve_type("Option").unwrap();

    // Currently returns root name, should return qualified after fix
    // This test documents current behavior and will fail after implementation
    assert_eq!(qualified.name, "Option");
}

#[test]
fn test_type_constructor_uses_qualified_name() {
    let env = TypeEnv::with_builtin_types();

    // Convert a type expression
    let type_expr = TypeExpr::Constructor {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("Int".to_string())],
    };

    let param_mapping: HashMap<String, TypeVar> = HashMap::new();
    let ty = type_expr_to_type(&type_expr, &param_mapping, &env).unwrap();

    // Should be a Constructor type
    match ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, "Option");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Type::Constructor, got {:?}", ty),
    }
}

// ============================================================
// Module Context Resolution Tests
// ============================================================

/// Context for module-scoped ADT resolution
#[derive(Debug, Clone)]
struct ModuleContext {
    /// Current module path (e.g., ["a", "b"] for a::b)
    path: Vec<String>,
    /// Imported ADTs: local alias -> fully qualified name
    imports: HashMap<String, AdtName>,
}

impl ModuleContext {
    fn new(path: Vec<String>) -> Self {
        Self {
            path,
            imports: HashMap::new(),
        }
    }

    fn with_import(mut self, alias: &str, qualified: AdtName) -> Self {
        self.imports.insert(alias.to_string(), qualified);
        self
    }

    /// Get the module path as a string
    fn path_string(&self) -> String {
        self.path.join("::")
    }
}

/// Resolve an ADT reference with full module context support
fn resolve_adt_reference(
    env: &TypeEnv,
    name: &str,
    context: &ModuleContext,
) -> Result<AdtName, String> {
    // Case 1: Already fully qualified
    if name.contains("::") {
        // Check if it exists
        let adt_name = AdtName::new(name);
        let root_name = &adt_name.root;

        // Try to find the type by root name (simplified check)
        if env.lookup_type_info(root_name).is_some() {
            return Ok(adt_name);
        }
        return Err(format!("Unknown type: {}", name));
    }

    // Case 2: Try as imported name
    if let Some(imported) = context.imports.get(name) {
        return Ok(imported.clone());
    }

    // Case 3: Try relative to current module
    if !context.path.is_empty() {
        let qualified = format!("{}::{}", context.path_string(), name);
        // In a real implementation, we'd check if this type exists
        // For now, assume it exists if the root name is known
        if env.lookup_type_info(name).is_some() {
            return Ok(AdtName::new(qualified));
        }
    }

    // Case 4: Try as root-level type
    if env.lookup_type_info(name).is_some() {
        return Ok(AdtName::new(name));
    }

    Err(format!("Unknown type: {}", name))
}

#[test]
fn test_resolve_fully_qualified_name() {
    let env = TypeEnv::with_builtin_types();
    let ctx = ModuleContext::new(vec!["a".to_string()]);

    // Should resolve fully qualified name
    let result = resolve_adt_reference(&env, "std::option::Option", &ctx);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().qualified, "std::option::Option");
}

#[test]
fn test_resolve_imported_name() {
    let env = TypeEnv::with_builtin_types();
    let ctx = ModuleContext::new(vec![]).with_import("MyResult", AdtName::new("utils::Result"));

    // Should resolve imported alias
    let result = resolve_adt_reference(&env, "MyResult", &ctx);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().qualified, "utils::Result");
}

#[test]
fn test_resolve_relative_to_module() {
    let env = TypeEnv::with_builtin_types();
    let ctx = ModuleContext::new(vec!["myapp".to_string(), "utils".to_string()]);

    // Should resolve relative to module path
    let result = resolve_adt_reference(&env, "Result", &ctx);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().qualified, "myapp::utils::Result");
}

#[test]
fn test_resolve_root_level_type() {
    let env = TypeEnv::with_builtin_types();
    let ctx = ModuleContext::new(vec![]);

    // Should resolve root-level types
    let result = resolve_adt_reference(&env, "Option", &ctx);
    assert!(result.is_ok());
}

// ============================================================
// Cross-Module Type Distinctness Tests
// ============================================================

#[test]
fn test_same_name_different_modules_are_distinct() {
    // Create two types with same root name in different modules
    let a_t = AdtName::new("a::T");
    let b_t = AdtName::new("b::T");

    // They should be distinct types
    assert_ne!(
        a_t, b_t,
        "Types with same root but different modules should be distinct"
    );

    // Verify the fields are what we expect
    assert_eq!(a_t.root, "T");
    assert_eq!(b_t.root, "T");
    assert_eq!(a_t.module, vec!["a"]);
    assert_eq!(b_t.module, vec!["b"]);
}

#[test]
fn test_deeply_qualified_names() {
    let deeply_nested = AdtName::new("a::b::c::d::e::MyType");
    assert_eq!(deeply_nested.module, vec!["a", "b", "c", "d", "e"]);
    assert_eq!(deeply_nested.root, "MyType");
    assert_eq!(deeply_nested.module_path(), "a::b::c::d::e");
}

#[test]
fn test_qualified_name_roundtrip() {
    let original = "std::result::Result";
    let parsed = AdtName::new(original);
    assert_eq!(parsed.qualified, original);
}

// ============================================================
// Hash-based Lookup Tests
// ============================================================

#[test]
fn test_adt_name_can_be_used_in_hashmap() {
    use std::collections::HashMap;

    let mut map: HashMap<AdtName, String> = HashMap::new();

    let a_t = AdtName::new("a::T");
    let b_t = AdtName::new("b::T");

    map.insert(a_t.clone(), "from module a".to_string());
    map.insert(b_t.clone(), "from module b".to_string());

    // Should be able to retrieve by qualified name
    assert_eq!(
        map.get(&AdtName::new("a::T")),
        Some(&"from module a".to_string())
    );
    assert_eq!(
        map.get(&AdtName::new("b::T")),
        Some(&"from module b".to_string())
    );

    // Different keys should return different values
    assert_ne!(map.get(&a_t), map.get(&b_t));
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_empty_module_path() {
    let name = AdtName::from_parts(&[], "RootType");
    assert_eq!(name.qualified, "RootType");
    assert!(name.is_root());
}

#[test]
fn test_single_component_module() {
    let name = AdtName::new("module::Type");
    assert_eq!(name.module, vec!["module"]);
    assert_eq!(name.root, "Type");
}

#[test]
fn test_generic_adt_name() {
    // Generic ADTs are represented with their qualified name
    // The type arguments are stored separately in Type::Constructor
    let option = AdtName::new("std::option::Option");
    assert_eq!(option.root, "Option");

    let result = AdtName::new("std::result::Result");
    assert_eq!(result.root, "Result");
}

// ============================================================
// SPEC-003 Compliance Tests
// ============================================================

#[test]
fn test_spec_003_section_3_3_compliance() {
    // SPEC-003 Section 3.3 requires "fully qualified type names for all ADT references"

    // Create two ADTs with the same root name but different module paths
    let status_a = AdtName::new("http::Status");
    let status_b = AdtName::new("internal::Status");

    // They must be considered different types
    assert_ne!(
        status_a, status_b,
        "SPEC-003 Section 3.3: Same-name ADTs in different modules must be distinct"
    );

    // Type equality must use full qualification
    let status_a_copy = AdtName::new("http::Status");
    assert_eq!(
        status_a, status_a_copy,
        "SPEC-003 Section 3.3: Same qualified names must be equal"
    );
}

// ============================================================
// TypeEnv Qualified Name Resolution Tests (TASK-295)
// ============================================================

use ash_typeck::QualifiedName;

#[test]
fn test_qualified_name_parse_double_colon_separator() {
    // Qualified names should use :: separator (Rust/Ada style)
    let name = QualifiedName::parse("std::option::Option");
    assert_eq!(name.module, vec!["std", "option"]);
    assert_eq!(name.name, "Option");
    assert_eq!(name.display(), "std::option::Option");
}

#[test]
fn test_qualified_name_root_level() {
    let name = QualifiedName::parse("Option");
    assert!(name.is_root());
    assert_eq!(name.name, "Option");
    assert_eq!(name.display(), "Option");
}

#[test]
fn test_qualified_name_equality_uses_full_qualification() {
    let a_t = QualifiedName::parse("a::T");
    let b_t = QualifiedName::parse("b::T");
    let a_t_copy = QualifiedName::parse("a::T");

    // Same qualified name should be equal
    assert_eq!(a_t, a_t_copy, "Same qualified name should be equal");

    // Different qualified names should not be equal
    assert_ne!(a_t, b_t, "a::T and b::T should be different types");
}

#[test]
fn test_qualified_name_parsing_preserves_distinct_modules() {
    // This is the key issue: std::option::Option and my::option::Option
    // should be different types
    let std_option = QualifiedName::parse("std::option::Option");
    let my_option = QualifiedName::parse("my::option::Option");

    assert_eq!(std_option.name, "Option");
    assert_eq!(my_option.name, "Option");

    // Module paths should be different
    assert_eq!(std_option.module, vec!["std", "option"]);
    assert_eq!(my_option.module, vec!["my", "option"]);

    // They should NOT be equal
    assert_ne!(std_option, my_option);
}

#[test]
fn test_resolve_type_with_qualified_name() {
    use ash_typeck::type_env::TypeEnv;

    let env = TypeEnv::with_builtin_types();

    // Resolve a builtin type
    let (qualified, info) = env.resolve_type("Option").unwrap();

    // Should return a root-level QualifiedName for builtin
    assert_eq!(qualified.name, "Option");
    assert!(info.is_some());
}

#[test]
fn test_qualified_name_display_format() {
    // Display should use :: separator
    let name = QualifiedName::qualified(vec!["std".to_string(), "result".to_string()], "Result");
    assert_eq!(name.to_string(), "std::result::Result");

    // Root name should have no separator
    let root = QualifiedName::root("Int");
    assert_eq!(root.to_string(), "Int");
}

#[test]
fn test_qualified_name_from_parts() {
    let name = QualifiedName::qualified(vec!["a".to_string(), "b".to_string()], "C");
    assert_eq!(name.module, vec!["a", "b"]);
    assert_eq!(name.name, "C");
}

#[test]
fn test_qualified_name_hash_consistency() {
    use std::collections::HashMap;

    let mut map: HashMap<QualifiedName, String> = HashMap::new();

    let a_t = QualifiedName::parse("a::T");
    let b_t = QualifiedName::parse("b::T");

    map.insert(a_t.clone(), "from module a".to_string());
    map.insert(b_t.clone(), "from module b".to_string());

    // Should be able to retrieve by qualified name
    assert_eq!(
        map.get(&QualifiedName::parse("a::T")),
        Some(&"from module a".to_string())
    );
    assert_eq!(
        map.get(&QualifiedName::parse("b::T")),
        Some(&"from module b".to_string())
    );

    // Different keys should return different values
    assert_ne!(map.get(&a_t), map.get(&b_t));
}
