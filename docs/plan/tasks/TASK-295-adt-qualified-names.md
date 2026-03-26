# TASK-295: Preserve ADT Qualified Names

## Status: 📝 Planned

## Description

Fix the issue where ADT/type qualification is lossy, so module-qualified names can collide. ADT resolution collapses to root names instead of preserving qualified names, which conflicts with SPEC-003 Section 3.3 and SPEC-020.

## Specification Reference

- SPEC-003: Type System Specification (Section 3.3)
- SPEC-020: ADT Specification

## Dependencies

- ✅ TASK-122: ADT runtime values
- ✅ TASK-127: Type check constructors
- ✅ TASK-174: Align ADT contracts

## Critical File Locations

- `crates/ash-typeck/src/check_expr.rs:206` - ADT resolution collapses to root names

## Requirements

### Functional Requirements

1. ADT names must preserve full module qualification
2. Type resolution must use qualified names
3. Name collisions across modules must be prevented
4. SPEC-003 Section 3.3 compliance

### Current State (Broken)

**File:** `crates/ash-typeck/src/check_expr.rs:206`

```rust
fn resolve_adt_type(&self, name: &str) -> Result<Type, TypeError> {
    // WRONG: Collapses to root name, loses qualification
    let root_name = name.split("::").last().unwrap();  // Line 206
    
    self.type_context.lookup_adt(root_name)
        .ok_or_else(|| TypeError::UnknownType { name: root_name.to_string() })
}
```

Problems:
1. `std::option::Option` and `my::option::Option` both become `Option`
2. Name collisions possible
3. Type safety compromised
4. SPEC-003 Section 3.3 violated

### Target State (Fixed)

```rust
fn resolve_adt_type(&self, name: &str, current_module: &ModulePath) -> Result<Type, TypeError> {
    // FIX: Preserve full qualification
    let qualified_name = if name.contains("::") {
        // Already qualified
        FullyQualifiedName::parse(name)?
    } else {
        // Unqualified, resolve relative to current module
        self.resolve_relative_name(name, current_module)?
    };
    
    self.type_context.lookup_adt(&qualified_name)
        .ok_or_else(|| TypeError::UnknownType { name: qualified_name.to_string() })
}

/// A fully qualified type name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullyQualifiedName {
    pub module_path: Vec<String>,
    pub name: String,
}

impl FullyQualifiedName {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let parts: Vec<_> = s.split("::").collect();
        if parts.is_empty() {
            return Err(ParseError::EmptyName);
        }
        
        Ok(Self {
            name: parts.last().unwrap().to_string(),
            module_path: parts[..parts.len()-1].iter().map(|s| s.to_string()).collect(),
        })
    }
    
    pub fn to_string(&self) -> String {
        if self.module_path.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.module_path.join("::"), self.name)
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/adt_qualified_names_test.rs`

```rust
//! Tests for ADT qualified name preservation

use ash_typeck::TypeChecker;
use ash_parser::parse_type;

#[test]
fn test_fully_qualified_name_parsing() {
    let name = FullyQualifiedName::parse("std::option::Option").unwrap();
    assert_eq!(name.module_path, vec!["std", "option"]);
    assert_eq!(name.name, "Option");
    
    let name = FullyQualifiedName::parse("MyType").unwrap();
    assert!(name.module_path.is_empty());
    assert_eq!(name.name, "MyType");
}

#[test]
fn test_qualified_name_roundtrip() {
    let original = "a::b::c::Type";
    let parsed = FullyQualifiedName::parse(original).unwrap();
    assert_eq!(parsed.to_string(), original);
}

#[test]
fn test_adt_resolution_preserves_qualification() {
    let mut checker = TypeChecker::new();
    
    // Register two ADTs with same name in different modules
    checker.register_adt(FullyQualifiedName::parse("std::result::Result").unwrap(), 
        vec!["T".to_string(), "E".to_string()]);
    
    checker.register_adt(FullyQualifiedName::parse("my::result::Result").unwrap(),
        vec!["Value".to_string()]);
    
    // Both should be resolvable by full qualification
    let std_result = checker.resolve_adt_type("std::result::Result", &ModulePath::root()).unwrap();
    let my_result = checker.resolve_adt_type("my::result::Result", &ModulePath::root()).unwrap();
    
    // They should be different types
    assert_ne!(std_result, my_result);
}

#[test]
fn test_unqualified_name_resolves_relative() {
    let mut checker = TypeChecker::new();
    
    // Register in current module
    checker.register_adt(FullyQualifiedName::parse("current::Option").unwrap(),
        vec!["T".to_string()]);
    
    // Unqualified name should resolve to current module
    let resolved = checker.resolve_adt_type("Option", &ModulePath::new(vec!["current"])).unwrap();
    
    // Should resolve to current::Option, not just Option
    assert!(resolved.name().contains("current"));
}

#[test]
fn test_collision_prevention() {
    let mut checker = TypeChecker::new();
    
    checker.register_adt(FullyQualifiedName::parse("mod1::Type").unwrap(), vec![]);
    checker.register_adt(FullyQualifiedName::parse("mod2::Type").unwrap(), vec![]);
    
    // Both should be independently resolvable
    let t1 = checker.resolve_adt_type("mod1::Type", &ModulePath::root()).unwrap();
    let t2 = checker.resolve_adt_type("mod2::Type", &ModulePath::root()).unwrap();
    
    assert_ne!(t1, t2);
}

#[test]
fn test_constructor_type_uses_qualified_name() {
    let mut checker = TypeChecker::new();
    
    checker.register_adt(FullyQualifiedName::parse("std::option::Option").unwrap(),
        vec!["T".to_string()]);
    
    // Check constructor type includes full qualification
    let some_type = checker.get_constructor_type("Some").unwrap();
    
    // The return type should be fully qualified
    match some_type {
        Type::Function(_, ret) => {
            assert!(ret.to_string().contains("std::option::Option"));
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_pattern_matching_preserves_qualification() {
    let checker = TypeChecker::new();
    
    // Pattern should match fully qualified type
    let pattern = parse_pattern("std::option::Option::Some(x)").unwrap();
    let ty = checker.check_pattern(&pattern, &Type::Adt("std::option::Option".to_string(), vec![Type::Int]));
    
    assert!(ty.is_ok());
}

#[test]
fn test_import_brings_name_into_scope() {
    let mut checker = TypeChecker::new();
    
    checker.register_adt(FullyQualifiedName::parse("std::option::Option").unwrap(),
        vec!["T".to_string()]);
    
    // Import brings qualified name into local scope
    checker.import("std::option::Option", "Option");
    
    // Now unqualified "Option" should resolve
    let resolved = checker.resolve_adt_type("Option", &ModulePath::root()).unwrap();
    assert!(resolved.name().contains("std::option"));
}

proptest! {
    #[test]
    fn qualified_name_equality_is_consistent(path in module_path_strategy(), name in "[A-Z][a-zA-Z0-9]*") {
        let full_name = if path.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", path.join("::"), name)
        };
        
        let parsed1 = FullyQualifiedName::parse(&full_name).unwrap();
        let parsed2 = FullyQualifiedName::parse(&full_name).unwrap();
        
        assert_eq!(parsed1, parsed2);
        assert_eq!(parsed1.to_string(), parsed2.to_string());
    }
}
```

### Step 2: Implement FullyQualifiedName

**File:** `crates/ash-core/src/names.rs`

```rust
//! Fully qualified name handling

use std::fmt;

/// A fully qualified type or value name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullyQualifiedName {
    /// Module path components (e.g., ["std", "option"] for std::option::Option)
    pub module_path: Vec<String>,
    /// The actual name (e.g., "Option")
    pub name: String,
}

impl FullyQualifiedName {
    /// Parse a string into a fully qualified name
    pub fn parse(s: &str) -> Result<Self, NameError> {
        if s.is_empty() {
            return Err(NameError::EmptyName);
        }
        
        let parts: Vec<_> = s.split("::").collect();
        
        if parts.iter().any(|p| p.is_empty()) {
            return Err(NameError::EmptyComponent);
        }
        
        Ok(Self {
            name: parts.last().unwrap().to_string(),
            module_path: parts[..parts.len()-1].iter().map(|s| s.to_string()).collect(),
        })
    }
    
    /// Create from components
    pub fn new(module_path: Vec<String>, name: String) -> Self {
        Self { module_path, name }
    }
    
    /// Get just the root name (for error messages)
    pub fn root_name(&self) -> &str {
        &self.name
    }
    
    /// Check if this is in a specific module
    pub fn is_in_module(&self, module: &[String]) -> bool {
        self.module_path == module
    }
    
    /// Join with a sub-name
    pub fn join(&self, sub_name: &str) -> Self {
        let mut path = self.module_path.clone();
        path.push(self.name.clone());
        Self {
            module_path: path,
            name: sub_name.to_string(),
        }
    }
}

impl fmt::Display for FullyQualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.module_path.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}::{}", self.module_path.join("::"), self.name)
        }
    }
}

#[derive(Debug, Error)]
pub enum NameError {
    #[error("Name cannot be empty")]
    EmptyName,
    #[error("Name cannot have empty components")]
    EmptyComponent,
}
```

### Step 3: Update Type Resolution

**File:** `crates/ash-typeck/src/check_expr.rs`

```rust
use ash_core::names::{FullyQualifiedName, NameError};

impl TypeChecker {
    pub fn resolve_type(&mut self, name: &str, current_module: &ModulePath) -> Result<Type, TypeError> {
        // Determine if already qualified
        let qualified = if name.contains("::") {
            FullyQualifiedName::parse(name).map_err(|e| TypeError::InvalidName(e))?
        } else {
            // Try to resolve as unqualified name
            self.resolve_unqualified_name(name, current_module)?
        };
        
        // Look up in type context using full qualification
        self.type_context.lookup_adt(&qualified)
            .ok_or_else(|| TypeError::UnknownType { name: qualified.to_string() })
    }
    
    fn resolve_unqualified_name(&self, name: &str, current_module: &ModulePath) 
        -> Result<FullyQualifiedName, TypeError> {
        // 1. Check imports
        if let Some(imported) = self.imports.get(name) {
            return Ok(imported.clone());
        }
        
        // 2. Check current module
        let current_qualified = FullyQualifiedName::new(
            current_module.path.clone(),
            name.to_string(),
        );
        if self.type_context.has_adt(&current_qualified) {
            return Ok(current_qualified);
        }
        
        // 3. Check prelude
        let prelude_qualified = FullyQualifiedName::new(
            vec!["std".to_string()],
            name.to_string(),
        );
        if self.type_context.has_adt(&prelude_qualified) {
            return Ok(prelude_qualified);
        }
        
        Err(TypeError::UnknownType { name: name.to_string() })
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test adt_qualified_names_test` passes
- [ ] Qualified names preserved through resolution
- [ ] Name collisions prevented
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Proper ADT qualified name handling
- SPEC-003 Section 3.3 compliance

Required by:
- Module system type safety
- Large codebase support

## Notes

**Critical Issue**: Type system loses qualification information.

**Risk Assessment**: Medium - affects type safety in multi-module code.

**Implementation Strategy**:
1. First: Implement FullyQualifiedName type
2. Second: Update type resolution
3. Third: Update constructor handling
4. Fourth: Add collision tests
