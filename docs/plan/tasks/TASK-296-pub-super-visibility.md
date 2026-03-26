# TASK-296: Fix pub(super) Visibility Implementation

## Status: 📝 Planned

## Description

Fix the issue where module visibility rules are not implemented correctly for `pub(super)` and restricted visibility. The current Visibility::Super logic does not encode parent-module semantics correctly, and related resolver behavior reportedly over-permits some imports. This is a SPEC-009 compliance issue.

## Specification Reference

- SPEC-009: Module System Specification

## Dependencies

- ✅ TASK-066: Parse visibility modifiers
- ✅ TASK-070: Visibility checking

## Critical File Locations

- `crates/ash-parser/src/surface.rs:146` - Visibility::Super implementation

## Requirements

### Functional Requirements

1. `pub(super)` must restrict visibility to parent module only
2. `pub(crate)` must restrict visibility to current crate
3. `pub(in path)` must restrict visibility to specified path
4. Private items must not be accessible from outside module
5. SPEC-009 compliance for visibility rules

### Current State (Broken)

**File:** `crates/ash-parser/src/surface.rs:146`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Super,  // WRONG: Doesn't encode parent module reference
    Crate,
    Restricted(String),  // Path not resolved
}
```

**File:** `crates/ash-typeck/src/visibility.rs`

```rust
fn check_visibility(item: &Item, from_module: &ModulePath) -> bool {
    match item.visibility {
        Visibility::Private => from_module == item.module,
        Visibility::Public => true,
        Visibility::Super => {
            // WRONG: No actual parent module check
            true  // Over-permissive!
        }
        Visibility::Crate => from_module.crate_root() == item.module.crate_root(),
        Visibility::Restricted(_) => {
            // Not properly implemented
            true
        }
    }
}
```

Problems:
1. `pub(super)` doesn't check parent relationship
2. Resolver over-permits some imports
3. SPEC-009 visibility rules not enforced

### Target State (Fixed)

```rust
// crates/ash-parser/src/surface.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    /// Visible to parent module and its descendants
    Super {
        /// How many levels up (1 = parent, 2 = grandparent, etc.)
        levels: usize,
    },
    /// Visible to entire crate
    Crate,
    /// Visible to specific module path
    Restricted {
        path: ModulePath,
    },
}

impl Visibility {
    /// Check if an item is visible from a given module
    pub fn is_visible_from(&self, item_module: &ModulePath, from_module: &ModulePath) -> bool {
        match self {
            Visibility::Private => item_module == from_module,
            Visibility::Public => true,
            Visibility::Super { levels } => {
                // FIX: Check if from_module is within `levels` ancestors of item_module
                let ancestors = item_module.ancestors(*levels);
                ancestors.iter().any(|ancestor| {
                    from_module == ancestor || from_module.is_descendant_of(ancestor)
                })
            }
            Visibility::Crate => {
                item_module.crate_id() == from_module.crate_id()
            }
            Visibility::Restricted { path } => {
                from_module == path || from_module.is_descendant_of(path)
            }
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/visibility_test.rs`

```rust
//! Tests for visibility checking

use ash_typeck::visibility::VisibilityChecker;
use ash_parser::surface::Visibility;

#[test]
fn test_private_not_visible_outside_module() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let other_module = ModulePath::parse("crate::baz");
    
    assert!(!checker.is_visible(
        &Visibility::Private,
        &item_module,
        &other_module
    ));
}

#[test]
fn test_private_visible_in_same_module() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    
    assert!(checker.is_visible(
        &Visibility::Private,
        &item_module,
        &item_module
    ));
}

#[test]
fn test_super_visible_from_parent() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let parent_module = ModulePath::parse("crate::foo");
    
    assert!(checker.is_visible(
        &Visibility::Super { levels: 1 },
        &item_module,
        &parent_module
    ));
}

#[test]
fn test_super_visible_from_sibling() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let sibling_module = ModulePath::parse("crate::foo::baz");
    
    // Sibling is also descendant of parent
    assert!(checker.is_visible(
        &Visibility::Super { levels: 1 },
        &item_module,
        &sibling_module
    ));
}

#[test]
fn test_super_not_visible_from_unrelated() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let unrelated_module = ModulePath::parse("crate::other");
    
    assert!(!checker.is_visible(
        &Visibility::Super { levels: 1 },
        &item_module,
        &unrelated_module
    ));
}

#[test]
fn test_super_levels_multiple() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::a::b::c::d");
    
    // Grandparent (2 levels up)
    let grandparent = ModulePath::parse("crate::a::b");
    assert!(checker.is_visible(
        &Visibility::Super { levels: 2 },
        &item_module,
        &grandparent
    ));
    
    // Great-grandparent (3 levels up)
    let great_grandparent = ModulePath::parse("crate::a");
    assert!(!checker.is_visible(
        &Visibility::Super { levels: 2 },
        &item_module,
        &great_grandparent
    ));
}

#[test]
fn test_crate_visible_within_crate() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::deep::nested");
    let other_module = ModulePath::parse("crate::other::module");
    
    assert!(checker.is_visible(
        &Visibility::Crate,
        &item_module,
        &other_module
    ));
}

#[test]
fn test_restricted_visibility() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let restricted_to = ModulePath::parse("crate::foo");
    
    // Visible from restricted module
    assert!(checker.is_visible(
        &Visibility::Restricted { path: restricted_to.clone() },
        &item_module,
        &restricted_to
    ));
    
    // Visible from descendant
    let descendant = ModulePath::parse("crate::foo::baz");
    assert!(checker.is_visible(
        &Visibility::Restricted { path: restricted_to },
        &item_module,
        &descendant
    ));
}

#[test]
fn test_restricted_not_visible_outside() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo::bar");
    let restricted_to = ModulePath::parse("crate::foo");
    let outside = ModulePath::parse("crate::other");
    
    assert!(!checker.is_visible(
        &Visibility::Restricted { path: restricted_to },
        &item_module,
        &outside
    ));
}

#[test]
fn test_public_always_visible() {
    let checker = VisibilityChecker::new();
    
    let item_module = ModulePath::parse("crate::foo");
    let any_module = ModulePath::parse("other_crate::anywhere");
    
    assert!(checker.is_visible(
        &Visibility::Public,
        &item_module,
        &any_module
    ));
}

#[test]
fn test_import_respects_visibility() {
    let mut checker = VisibilityChecker::new();
    
    // Item in foo with super visibility
    checker.add_item(
        "internal",
        ModulePath::parse("crate::foo::bar"),
        Visibility::Super { levels: 1 }
    );
    
    // Import from parent (should succeed)
    assert!(checker.can_import("internal", 
        &ModulePath::parse("crate::foo"),
        &ModulePath::parse("crate::foo::bar")
    ));
    
    // Import from unrelated (should fail)
    assert!(!checker.can_import("internal",
        &ModulePath::parse("crate::other"),
        &ModulePath::parse("crate::foo::bar")
    ));
}

proptest! {
    #[test]
    fn visibility_is_reflexive(vis in visibility_strategy(), module in module_path_strategy()) {
        let checker = VisibilityChecker::new();
        assert!(checker.is_visible(&vis, &module, &module));
    }
    
    #[test]
    fn private_is_strict(module1 in module_path_strategy(), module2 in module_path_strategy()) {
        let checker = VisibilityChecker::new();
        
        // Private is only visible from same module
        let visible = checker.is_visible(&Visibility::Private, &module1, &module2);
        
        if module1 == module2 {
            assert!(visible);
        } else {
            assert!(!visible);
        }
    }
}
```

### Step 2: Implement ModulePath

**File:** `crates/ash-core/src/module_path.rs`

```rust
//! Module path representation

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    components: Vec<String>,
    crate_id: CrateId,
}

impl ModulePath {
    pub fn root() -> Self {
        Self {
            components: vec![],
            crate_id: CrateId::current(),
        }
    }
    
    pub fn new(components: Vec<String>) -> Self {
        Self {
            components,
            crate_id: CrateId::current(),
        }
    }
    
    pub fn parse(s: &str) -> Self {
        let parts: Vec<_> = s.split("::").map(|s| s.to_string()).collect();
        Self::new(parts)
    }
    
    /// Get ancestors up to n levels
    pub fn ancestors(&self, levels: usize) -> Vec<ModulePath> {
        let mut result = vec![];
        for i in 1..=levels.min(self.components.len()) {
            result.push(ModulePath::new(
                self.components[..self.components.len() - i].to_vec()
            ));
        }
        result
    }
    
    /// Check if self is a descendant of other
    pub fn is_descendant_of(&self, other: &ModulePath) -> bool {
        if self.components.len() <= other.components.len() {
            return false;
        }
        self.components.starts_with(&other.components)
    }
    
    /// Get parent module
    pub fn parent(&self) -> Option<ModulePath> {
        if self.components.is_empty() {
            None
        } else {
            Some(ModulePath::new(
                self.components[..self.components.len()-1].to_vec()
            ))
        }
    }
    
    pub fn crate_root(&self) -> ModulePath {
        if self.components.is_empty() {
            self.clone()
        } else {
            ModulePath::new(vec![self.components[0].clone()])
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.components.join("::"))
    }
}
```

### Step 3: Implement Visibility Checker

**File:** `crates/ash-typeck/src/visibility.rs`

```rust
use ash_core::module_path::ModulePath;
use ash_parser::surface::Visibility;

pub struct VisibilityChecker {
    items: HashMap<String, (ModulePath, Visibility)>,
}

impl VisibilityChecker {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
    
    pub fn add_item(&mut self, name: &str, module: ModulePath, visibility: Visibility) {
        self.items.insert(name.to_string(), (module, visibility));
    }
    
    /// Check if an item is visible from a given module
    pub fn is_visible(
        &self,
        visibility: &Visibility,
        item_module: &ModulePath,
        from_module: &ModulePath,
    ) -> bool {
        match visibility {
            Visibility::Private => {
                item_module == from_module
            }
            Visibility::Public => {
                true
            }
            Visibility::Super { levels } => {
                // Get ancestors of item_module
                let ancestors = item_module.ancestors(*levels);
                
                // from_module must be one of those ancestors or a descendant of one
                ancestors.iter().any(|ancestor| {
                    from_module == ancestor || from_module.is_descendant_of(ancestor)
                })
            }
            Visibility::Crate => {
                item_module.crate_id() == from_module.crate_id()
            }
            Visibility::Restricted { path } => {
                from_module == path || from_module.is_descendant_of(path)
            }
        }
    }
    
    /// Check if an import is allowed
    pub fn can_import(
        &self,
        item_name: &str,
        importing_module: &ModulePath,
        item_module: &ModulePath,
    ) -> bool {
        if let Some((_, visibility)) = self.items.get(item_name) {
            self.is_visible(visibility, item_module, importing_module)
        } else {
            false
        }
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test visibility_test` passes
- [ ] `pub(super)` correctly restricts to parent
- [ ] `pub(crate)` correctly restricts to crate
- [ ] Private items not accessible outside module
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Correct visibility implementation
- SPEC-009 compliance

Required by:
- Module system encapsulation
- API boundaries

## Notes

**Critical Issue**: Visibility rules not properly enforced.

**Risk Assessment**: Medium - affects module encapsulation.

**Implementation Strategy**:
1. First: Implement ModulePath with ancestor checking
2. Second: Update Visibility enum
3. Third: Implement visibility checker
4. Fourth: Add comprehensive tests
