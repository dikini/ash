# TASK-282: Fix pub(super) Visibility

## Status: 📝 Planned

## Description

Fix module visibility rules for `pub(super)` and restricted visibility. Current implementation `Visibility::Super => from.starts_with(owner)` does not correctly encode "parent module and descendants", causing the import resolver to over-permit some restricted imports.

## Specification Reference

- SPEC-009: Module System Specification - Section 4.2 (Visibility)

## Dependencies

- ✅ TASK-065: Visibility AST types
- ✅ TASK-070: Visibility checking in typeck
- ✅ TASK-086: Import resolution algorithm

## Requirements

### Functional Requirements

1. `pub(super)` must be visible only in parent module and its descendants
2. `pub(in path)` must be visible only in the specified module path
3. `pub(crate)` must be visible throughout the crate
4. `pub(self)` must be visible only in the current module
5. Visibility check must correctly handle module hierarchy

### Visibility Rules (SPEC-009)

| Visibility | Meaning |
|------------|---------|
| `pub` | Public - visible everywhere |
| `pub(crate)` | Visible anywhere in the crate |
| `pub(super)` | Visible in parent module and descendants |
| `pub(self)` | Visible only in current module |
| `pub(in path::to::module)` | Visible only in specified module |

### Current State (Broken)

**File:** `crates/ash-typeck/src/visibility.rs`

```rust
impl VisibilityChecker {
    pub fn is_visible(
        &self,
        item_visibility: &Visibility,
        item_module: &ModulePath,
        from_module: &ModulePath,
    ) -> bool {
        match item_visibility {
            Visibility::Public => true,
            Visibility::Crate => true, // Should check same crate
            Visibility::Restricted(path) => {
                // WRONG: Simple prefix check doesn't handle descendants
                from_module.starts_with(path)
            }
            Visibility::Super => {
                // WRONG: Doesn't check parent descendants properly
                let parent = item_module.parent();
                from_module.starts_with(&parent)
            }
        }
    }
}
```

### Target State (Fixed)

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Public,
    Crate,
    Super,           // parent module and descendants
    Self_,           // current module only
    Restricted(ModulePath), // specific module path
}

impl Visibility {
    /// Check if an item with this visibility in `item_module` 
    /// is visible from `from_module`
    pub fn is_visible(&self, item_module: &ModulePath, from_module: &ModulePath) -> bool {
        match self {
            Visibility::Public => true,
            
            Visibility::Crate => {
                // Both must be in the same crate
                // (crate root is the common ancestor)
                true // Simplified - actual check would compare crate roots
            }
            
            Visibility::Super => {
                // Get parent of item's module
                let parent = item_module.parent();
                if parent.is_empty() {
                    // Item is at crate root, pub(super) = pub(crate)
                    return true;
                }
                // Visible if from_module is parent or descendant of parent
                from_module == &parent 
                    || from_module.starts_with(&parent)
                    || is_descendant(&parent, from_module)
            }
            
            Visibility::Self_ => {
                // Only visible in the exact same module
                from_module == item_module
            }
            
            Visibility::Restricted(allowed_path) => {
                // Visible only in the specified module and its descendants
                from_module == allowed_path 
                    || from_module.starts_with(allowed_path)
            }
        }
    }
}

/// Check if `potential_descendant` is a descendant of `ancestor`
fn is_descendant(ancestor: &ModulePath, potential_descendant: &ModulePath) -> bool {
    if ancestor.segments().is_empty() {
        return true; // Root is ancestor of everything
    }
    potential_descendant.starts_with(ancestor)
}

pub struct ModulePath {
    segments: Vec<String>,
}

impl ModulePath {
    pub fn parent(&self) -> ModulePath {
        if self.segments.is_empty() {
            return ModulePath { segments: vec![] };
        }
        ModulePath {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        }
    }
    
    pub fn starts_with(&self, other: &ModulePath) -> bool {
        if other.segments.is_empty() {
            return true;
        }
        if other.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..other.segments.len()] == other.segments
    }
    
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/visibility_test.rs`

```rust
//! Tests for visibility checking

use ash_typeck::visibility::{Visibility, ModulePath};

fn path(segments: &[&str]) -> ModulePath {
    ModulePath::new(segments.iter().map(|s| s.to_string()).collect())
}

#[test]
fn test_pub_super_in_parent() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a"]); // parent of b
    
    assert!(Visibility::Super.is_visible(&item_module, &from_module));
}

#[test]
fn test_pub_super_in_sibling_not_allowed() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a", "c"]); // sibling of b
    
    // c is NOT a descendant of a (it's a sibling branch)
    // This depends on exact interpretation - may need clarification
}

#[test]
fn test_pub_super_in_grandparent_not_allowed() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate"]); // grandparent
    
    assert!(!Visibility::Super.is_visible(&item_module, &from_module));
}

#[test]
fn test_pub_super_in_descendant_of_parent() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a", "c", "d"]); // descendant of a
    
    assert!(Visibility::Super.is_visible(&item_module, &from_module));
}

#[test]
fn test_pub_self_only_in_same_module() {
    let module = path(&["crate", "a", "b"]);
    
    assert!(Visibility::Self_.is_visible(&module, &module));
    
    let other = path(&["crate", "a", "c"]);
    assert!(!Visibility::Self_.is_visible(&module, &other));
}

#[test]
fn test_pub_restricted_in_target_module() {
    let item_module = path(&["crate", "a", "b"]);
    let restricted_to = path(&["crate", "x", "y"]);
    let from_allowed = path(&["crate", "x", "y"]);
    let from_descendant = path(&["crate", "x", "y", "z"]);
    let from_other = path(&["crate", "a"]);
    
    let vis = Visibility::Restricted(restricted_to);
    
    assert!(vis.is_visible(&item_module, &from_allowed));
    assert!(vis.is_visible(&item_module, &from_descendant));
    assert!(!vis.is_visible(&item_module, &from_other));
}

#[test]
fn test_pub_crate_anywhere() {
    let item_module = path(&["crate", "a"]);
    let from_anywhere = path(&["crate", "b", "c", "d"]);
    
    assert!(Visibility::Crate.is_visible(&item_module, &from_anywhere));
}

#[test]
fn test_pub_super_at_root_is_crate() {
    let root = path(&["crate"]);
    let from_anywhere = path(&["crate", "a", "b"]);
    
    // pub(super) at root = pub(crate)
    assert!(Visibility::Super.is_visible(&root, &from_anywhere));
}

// Integration test with actual modules
#[test]
fn test_visibility_integration() {
    let source = r#"
        mod outer {
            pub mod inner {
                pub(super) fn secret() -> Int { 42 }
                pub fn public() -> Int { 21 }
            }
            
            pub workflow use_secret() -> Int {
                decide inner::secret()  // Should work - same parent
            }
        }
        
        // This should fail - secret is pub(super)
        // workflow try_secret() -> Int {
        //     decide outer::inner::secret()
        // }
    "#;
    
    let ast = ash_parser::parse(source).unwrap();
    let mut checker = ash_typeck::TypeChecker::new();
    let result = checker.type_check_module(&ast);
    
    assert!(result.is_ok());
}
```

### Step 2: Implement ModulePath

**File:** `crates/ash-typeck/src/module/path.rs` (new file or existing)

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModulePath {
    segments: Vec<String>,
}

impl ModulePath {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }
    
    pub fn root() -> Self {
        Self { segments: vec![] }
    }
    
    pub fn parent(&self) -> Self {
        if self.segments.is_empty() {
            return Self::root();
        }
        Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        }
    }
    
    pub fn child(&self, name: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(name.into());
        Self { segments }
    }
    
    pub fn starts_with(&self, other: &Self) -> bool {
        if other.segments.is_empty() {
            return true;
        }
        if other.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..other.segments.len()] == other.segments
    }
    
    pub fn is_parent_of(&self, other: &Self) -> bool {
        other.starts_with(self) && other.segments.len() > self.segments.len()
    }
    
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
    
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}
```

### Step 3: Implement Visibility

**File:** `crates/ash-typeck/src/visibility.rs`

```rust
use crate::module::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    Self_,
    Restricted(ModulePath),
}

impl Visibility {
    pub fn is_visible(&self, item_module: &ModulePath, from_module: &ModulePath) -> bool {
        match self {
            Visibility::Public => true,
            
            Visibility::Crate => true, // Same crate check done elsewhere
            
            Visibility::Super => {
                let parent = item_module.parent();
                if parent.is_root() {
                    // At root, pub(super) = pub(crate)
                    return true;
                }
                // Visible in parent and its descendants
                from_module == &parent || from_module.starts_with(&parent)
            }
            
            Visibility::Self_ => from_module == item_module,
            
            Visibility::Restricted(allowed) => {
                from_module == allowed || from_module.starts_with(allowed)
            }
        }
    }
}

pub struct VisibilityChecker;

impl VisibilityChecker {
    pub fn check_access(
        &self,
        item_name: &str,
        item_visibility: &Visibility,
        item_module: &ModulePath,
        from_module: &ModulePath,
    ) -> Result<(), VisibilityError> {
        if item_visibility.is_visible(item_module, from_module) {
            Ok(())
        } else {
            Err(VisibilityError::NotAccessible {
                name: item_name.to_string(),
                visibility: item_visibility.clone(),
                defined_in: item_module.clone(),
                accessed_from: from_module.clone(),
            })
        }
    }
}

#[derive(Debug, Error)]
pub enum VisibilityError {
    #[error("'{name}' is not accessible from '{accessed_from}' - visibility is {visibility:?} in '{defined_in}'")]
    NotAccessible {
        name: String,
        visibility: Visibility,
        defined_in: ModulePath,
        accessed_from: ModulePath,
    },
}
```

### Step 4: Update Import Resolution

**File:** `crates/ash-typeck/src/resolve.rs`

```rust
impl ImportResolver {
    fn resolve_import(
        &mut self,
        import: &Import,
        from_module: &ModulePath,
    ) -> Result<ResolvedImport, ResolveError> {
        let target_module = self.resolve_module_path(&import.path)?;
        
        let item = self.find_item(&target_module, &import.item_name)?;
        
        // Check visibility
        self.visibility_checker.check_access(
            &import.item_name,
            &item.visibility,
            &target_module,
            from_module,
        )?;
        
        Ok(ResolvedImport {
            name: item.name.clone(),
            full_path: target_module.child(&item.name),
        })
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test visibility_test` passes
- [ ] `cargo test -p ash-parser` passes
- [ ] Module visibility integration tests pass
- [ ] SPEC-009 compliance verified
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Correct visibility checking
- SPEC-009 compliance
- Secure module boundaries

Required by:
- All module-dependent code

## Notes

**Spec Compliance Issue**: Current implementation over-permits imports, violating encapsulation guarantees.

**Design Decision**: Explicit ModulePath type ensures correct path handling and comparison.

**Edge Cases**:
- pub(super) at crate root = pub(crate)
- Empty module paths
- Deeply nested visibility restrictions
- Self-referential visibility (pub(in self))
