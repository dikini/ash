# TASK-281: Preserve ADT Qualified Names

## Status: 📝 Planned

## Description

Fix ADT resolution to preserve qualified names instead of collapsing to root names. Currently conflicts with SPEC-003 Section 3.3 and SPEC-020's module-safe ADT model where module-qualified names collide.

## Specification Reference

- SPEC-003: Type System Specification - Section 3.3 (Name Resolution)
- SPEC-020: ADT Specification

## Dependencies

- ✅ TASK-160: Canonicalize ADT contracts
- ✅ TASK-069: Module resolution algorithm
- ✅ TASK-086: Import resolution algorithm

## Requirements

### Functional Requirements

1. ADT names must preserve full qualification (module path + name)
2. Same-name ADTs in different modules must be distinct types
3. Type equality must compare qualified names, not just root names
4. Pattern matching must resolve constructors using qualified paths
5. Imports must bring in qualified ADT names

### Current State (Broken)

**File:** `crates/ash-typeck/src/resolve.rs`

```rust
impl TypeResolver {
    pub fn resolve_adt(&mut self, name: &str) -> Result<Type, TypeError> {
        // WRONG: Only looks up root name, ignores module path
        if let Some(adt) = self.adt_definitions.get(name) {
            Ok(Type::Adt(name.to_string())) // "MyType" not "module::MyType"
        } else {
            Err(TypeError::UnknownType { name: name.to_string() })
        }
    }
}

// Type equality ignores qualification
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Adt(a), Type::Adt(b)) => {
                // WRONG: Compares only root names
                a.split("::").last() == b.split("::").last()
            }
            // ...
        }
    }
}
```

### Target State (Fixed)

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AdtName {
    /// Full qualified path: "std::option::Option"
    pub qualified: String,
    /// Module path: "std::option"
    pub module: String,
    /// Root name: "Option"
    pub root: String,
}

impl AdtName {
    pub fn new(qualified: &str) -> Self {
        let parts: Vec<_> = qualified.split("::").collect();
        let root = parts.last().unwrap().to_string();
        let module = parts[..parts.len()-1].join("::");
        Self {
            qualified: qualified.to_string(),
            module,
            root,
        }
    }
}

impl TypeResolver {
    pub fn resolve_adt(&mut self, name: &str, context: &ModulePath) -> Result<Type, TypeError> {
        // Try to resolve fully qualified name first
        if let Some(adt) = self.adt_definitions.get(name) {
            return Ok(Type::Adt(adt.name.clone()));
        }
        
        // Try with context module path
        let qualified = if name.contains("::") {
            name.to_string()
        } else {
            format!("{}::{}", context, name)
        };
        
        if let Some(adt) = self.adt_definitions.get(&qualified) {
            return Ok(Type::Adt(adt.name.clone()));
        }
        
        // Try imported names
        if let Some(imported) = self.imported_adts.get(name) {
            return Ok(Type::Adt(imported.clone()));
        }
        
        Err(TypeError::UnknownType { name: name.to_string() })
    }
}

// Type equality uses full qualification
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Adt(a), Type::Adt(b)) => {
                a.qualified == b.qualified // Full comparison
            }
            // ...
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
use ash_parser::parse;
use ash_core::Type;

#[test]
fn test_same_name_different_modules_are_distinct() {
    let source = r#"
        mod a {
            pub type T = A | B;
        }
        mod b {
            pub type T = X | Y;
        }
        
        workflow test(x: a::T, y: b::T) {
            // a::T and b::T should be different types
        }
    "#;
    
    let ast = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_ok());
    
    // Verify types are distinct
    let a_t = checker.resolve_type("a::T").unwrap();
    let b_t = checker.resolve_type("b::T").unwrap();
    assert_ne!(a_t, b_t);
}

#[test]
fn test_qualified_constructor_resolution() {
    let source = r#"
        mod option {
            pub type Option<T> = Some(T) | None;
        }
        
        workflow test() {
            let x = option::Some(42);
            let y = option::None;
        }
    "#;
    
    let ast = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_import_brings_qualified_name() {
    let source = r#"
        mod utils {
            pub type Result<T> = Ok(T) | Err(String);
        }
        
        use utils::Result;
        
        workflow test() -> Result<Int> {
            // Result here should be utils::Result
            decide Ok(42)
        }
    "#;
    
    let ast = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_ok());
    
    // Verify the return type is fully qualified
    let workflow = checker.get_workflow("test").unwrap();
    assert!(matches!(&workflow.return_type, 
        Type::Adt(name) if name.qualified == "utils::Result"));
}

#[test]
fn test_unqualified_name_resolves_in_scope() {
    let source = r#"
        mod inner {
            pub type Local = A | B;
            
            pub workflow use_local() -> Local {
                decide A  // Should resolve to inner::Local
            }
        }
    "#;
    
    let ast = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_cross_module_type_mismatch_error() {
    let source = r#"
        mod a { pub type T = A; }
        mod b { pub type T = B; }
        
        workflow test(x: a::T) -> b::T {
            decide x  // ERROR: a::T is not b::T
        }
    "#;
    
    let ast = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("a::T") || err.to_string().contains("b::T"));
}

proptest! {
    #[test]
    fn qualified_name_roundtrip(qual_name in qualified_name_strategy()) {
        let parsed = AdtName::new(&qual_name);
        assert_eq!(parsed.qualified, qual_name);
    }
}
```

### Step 2: Implement AdtName Type

**File:** `crates/ash-core/src/adt.rs`

```rust
#[derive(Clone, Debug, Eq, Hash)]
pub struct AdtName {
    /// Fully qualified name: "std::option::Option"
    pub qualified: String,
    /// Module path components
    pub module: Vec<String>,
    /// Root name without module
    pub root: String,
}

impl AdtName {
    pub fn new(qualified: impl Into<String>) -> Self {
        let qualified = qualified.into();
        let parts: Vec<_> = qualified.split("::").map(String::from).collect();
        let root = parts.last().cloned().unwrap_or_default();
        let module = parts[..parts.len().saturating_sub(1)].to_vec();
        
        Self { qualified, module, root }
    }
    
    pub fn from_parts(module: &[String], root: &str) -> Self {
        let mut qualified = module.join("::");
        if !qualified.is_empty() {
            qualified.push_str("::");
        }
        qualified.push_str(root);
        
        Self {
            qualified,
            module: module.to_vec(),
            root: root.to_string(),
        }
    }
}

impl PartialEq for AdtName {
    fn eq(&self, other: &Self) -> bool {
        self.qualified == other.qualified
    }
}

impl std::fmt::Display for AdtName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.qualified)
    }
}
```

### Step 3: Update Type Enum

**File:** `crates/ash-core/src/types.rs`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    // ... existing variants ...
    
    /// Algebraic data type with qualified name
    Adt(AdtName),
    
    /// Type constructor application
    AdtApp { name: AdtName, args: Vec<Type> },
    // ...
}

impl Type {
    pub fn adt(name: impl Into<AdtName>) -> Self {
        Type::Adt(name.into())
    }
}
```

### Step 4: Update Type Resolver

**File:** `crates/ash-typeck/src/resolve.rs`

```rust
impl TypeResolver {
    pub fn resolve_adt_reference(
        &mut self,
        name: &str,
        context: &ModuleContext,
    ) -> Result<AdtName, TypeError> {
        // Already fully qualified
        if name.contains("::") {
            if self.adt_definitions.contains_key(name) {
                return Ok(AdtName::new(name));
            }
            return Err(TypeError::UnknownType { 
                name: name.to_string(),
                context: context.path.clone(),
            });
        }
        
        // Try relative to current module
        let qualified = if context.path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", context.path.join("::"), name)
        };
        
        if self.adt_definitions.contains_key(&qualified) {
            return Ok(AdtName::new(qualified));
        }
        
        // Try imports
        if let Some(imported) = context.imported_adts.get(name) {
            return Ok(imported.clone());
        }
        
        // Try prelude
        if let Some(prelude) = self.prelude_adts.get(name) {
            return Ok(prelude.clone());
        }
        
        Err(TypeError::UnknownType { 
            name: name.to_string(),
            context: context.path.clone(),
        })
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test adt_qualified_names_test` passes
- [ ] `cargo test -p ash-core` passes
- [ ] `cargo test -p ash-parser` passes
- [ ] Cross-module ADT tests pass
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Qualified ADT name preservation
- Module-safe ADT model
- Correct type equality

Required by:
- All ADT-dependent code

## Notes

**Spec Conflict**: Current implementation violates SPEC-003 Section 3.3 which requires "fully qualified type names for all ADT references."

**Design Decision**: Use dedicated AdtName type to ensure qualification is always available and correctly compared.

**Edge Cases**:
- Empty module path (root level)
- Deeply nested modules (a::b::c::Type)
- Generic ADTs with qualified names
- Type aliases preserving qualification
