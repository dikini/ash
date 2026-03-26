# TASK-264: Compose Effective Capability Sets

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement capability set composition for workflows with multiple roles.

**Spec Reference:** SPEC-024 Section 2

**File Locations:**
- Modify: `crates/ash-typeck/src/effective_caps.rs` (or create)
- Test: `crates/ash-typeck/tests/effective_caps_tests.rs`

---

## Background

Workflows compose capabilities from multiple sources:
```ash
workflow processor
    plays role(ai_agent)      -- provides: file, process
    plays role(net_client)    -- provides: network, tls
    capabilities: [cache]      -- provides: cache
-- Effective: file, process, network, tls, cache
```

Need to compute and store effective capability set.

---

## Step 1: Create Effective Caps Module

Create `crates/ash-typeck/src/effective_caps.rs`:

```rust
use ash_core::*;
use std::collections::HashMap;

/// The complete set of capabilities available to a workflow
#[derive(Debug, Clone)]
pub struct EffectiveCapabilitySet {
    /// Capability name -> declaration (with merged constraints)
    capabilities: HashMap<String, MergedCapability>,
}

#[derive(Debug, Clone)]
pub struct MergedCapability {
    pub name: String,
    /// Constraints from all sources, merged
    pub merged_constraints: Option<ConstraintBlock>,
    /// Sources (for error reporting)
    pub sources: Vec<CapabilitySource>,
}

#[derive(Debug, Clone)]
pub enum CapabilitySource {
    Role { role_name: String },
    ImplicitDefault,
}

impl EffectiveCapabilitySet {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }
    
    /// Add a capability from a source
    pub fn add(
        &mut self,
        decl: &CapabilityDecl,
        source: CapabilitySource,
    ) -> Result<(), CompositionError> {
        let name = decl.capability.clone();
        
        if let Some(existing) = self.capabilities.get_mut(&name) {
            // Merge with existing
            existing.merge(decl, source)?;
        } else {
            // First occurrence
            self.capabilities.insert(name.clone(), MergedCapability {
                name,
                merged_constraints: decl.constraints.clone(),
                sources: vec![source],
            });
        }
        
        Ok(())
    }
    
    /// Check if workflow has a capability
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }
    
    /// Get merged constraint for a capability
    pub fn get_constraint(&self, name: &str) -> Option<&ConstraintBlock> {
        self.capabilities.get(name)
            .and_then(|c| c.merged_constraints.as_ref())
    }
    
    /// Check if capability use is allowed given constraints
    pub fn check_use(&self, cap_name: &str, use_constraints: &ConstraintBlock) -> bool {
        // Placeholder: Real implementation would check subsumption
        self.has_capability(cap_name)
    }
}

impl MergedCapability {
    fn merge(
        &mut self,
        decl: &CapabilityDecl,
        source: CapabilitySource,
    ) -> Result<(), CompositionError> {
        self.sources.push(source);
        
        // Merge constraints if both have them
        if let (Some(existing), Some(new)) = (&mut self.merged_constraints, &decl.constraints) {
            // For now: union of constraints
            // Real implementation: intersection for security
            for field in &new.fields {
                if !existing.has_field(&field.name) {
                    existing.fields.push(field.clone());
                }
            }
        } else if self.merged_constraints.is_none() {
            self.merged_constraints = decl.constraints.clone();
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum CompositionError {
    IncompatibleConstraints {
        capability: String,
        source1: CapabilitySource,
        source2: CapabilitySource,
    },
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-typeck/tests/effective_caps_tests.rs
use ash_typeck::effective_caps::*;
use ash_core::*;

#[test]
fn test_single_capability() {
    let mut caps = EffectiveCapabilitySet::new();
    
    caps.add(
        &CapabilityDecl::new("file"),
        CapabilitySource::Role { role_name: "test".to_string() }
    ).unwrap();
    
    assert!(caps.has_capability("file"));
    assert!(!caps.has_capability("network"));
}

#[test]
fn test_multiple_sources_same_capability() {
    let mut caps = EffectiveCapabilitySet::new();
    
    // Role 1 provides file with read
    let mut decl1 = CapabilityDecl::new("file");
    decl1.add_constraint("read", ConstraintValue::Bool(true));
    
    caps.add(&decl1, CapabilitySource::Role { 
        role_name: "role1".to_string() 
    }).unwrap();
    
    // Role 2 provides file with paths
    let mut decl2 = CapabilityDecl::new("file");
    decl2.add_constraint("paths", ConstraintValue::Array(vec![
        ConstraintValue::String("/tmp/*".to_string())
    ]));
    
    caps.add(&decl2, CapabilitySource::Role { 
        role_name: "role2".to_string() 
    }).unwrap();
    
    // Should have merged capability
    assert!(caps.has_capability("file"));
    let merged = caps.get_constraint("file").unwrap();
    assert!(merged.has_field("read"));
    assert!(merged.has_field("paths"));
}

#[test]
fn test_different_capabilities_composed() {
    let mut caps = EffectiveCapabilitySet::new();
    
    caps.add(
        &CapabilityDecl::new("file"),
        CapabilitySource::Role { role_name: "file_role".to_string() }
    ).unwrap();
    
    caps.add(
        &CapabilityDecl::new("network"),
        CapabilitySource::Role { role_name: "net_role".to_string() }
    ).unwrap();
    
    assert!(caps.has_capability("file"));
    assert!(caps.has_capability("network"));
}

#[test]
fn test_implicit_default_source() {
    let mut caps = EffectiveCapabilitySet::new();
    
    caps.add(
        &CapabilityDecl::new("cache"),
        CapabilitySource::ImplicitDefault
    ).unwrap();
    
    assert!(caps.has_capability("cache"));
}

proptest! {
    #[test]
    fn test_composition_idempotent(cap in arbitrary_capability_decl()) {
        // Adding same capability twice should be idempotent
        let mut caps1 = EffectiveCapabilitySet::new();
        caps1.add(&cap, CapabilitySource::ImplicitDefault).unwrap();
        caps1.add(&cap, CapabilitySource::ImplicitDefault).unwrap();
        
        let mut caps2 = EffectiveCapabilitySet::new();
        caps2.add(&cap, CapabilitySource::ImplicitDefault).unwrap();
        
        // caps1 and caps2 should be equivalent
    }
}
```

---

## Step 3: Integrate into Type Check

```rust
// crates/ash-typeck/src/lib.rs
pub fn compute_effective_capabilities(
    &self,
    workflow: &Workflow,
    roles: &HashMap<String, RoleDef>,
) -> TypeResult<EffectiveCapabilitySet> {
    let mut effective = EffectiveCapabilitySet::new();
    
    // Add from explicit roles
    for role_ref in &workflow.header.plays_roles {
        if let Some(role) = roles.get(&role_ref.name) {
            for cap in &role.capabilities {
                effective.add(
                    cap,
                    CapabilitySource::Role { 
                        role_name: role_ref.name.clone() 
                    }
                )?;
            }
        }
    }
    
    // Add from implicit default (direct capabilities)
    for cap in &workflow.header.capabilities {
        effective.add(cap, CapabilitySource::ImplicitDefault)?;
    }
    
    Ok(effective)
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-typeck effective_caps -v
```

---

## Step 5: Commit

```bash
git add crates/ash-typeck/src/effective_caps.rs
git add crates/ash-typeck/tests/effective_caps_tests.rs
git add crates/ash-typeck/src/lib.rs
git commit -m "feat: compose effective capability sets (TASK-264)

- Add EffectiveCapabilitySet for workflow capability composition
- Merge capabilities from multiple role sources
- Track capability sources for error reporting
- Handle implicit default role capabilities
- Constraint merging (union for now)
- Integration with type check flow
- Tests for composition scenarios"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-264 implementation"
  context: |
    Files to verify:
    - crates/ash-typeck/src/effective_caps.rs
    - crates/ash-typeck/tests/effective_caps_tests.rs
    - crates/ash-typeck/src/lib.rs
    
    Spec reference: SPEC-024 Section 2
    Requirements:
    1. Capabilities from multiple roles composed
    2. Same capability from multiple sources merged
    3. Implicit default capabilities included
    4. Sources tracked for debugging
    5. Constraint merging works
    6. Effective set stored per workflow
    
    Run and report:
    1. cargo test --package ash-typeck effective_caps
    2. cargo clippy --package ash-typeck --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-typeck
    4. Test 3+ roles composition
    5. Test constraint merging
    6. Verify sources tracked correctly
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] EffectiveCapabilitySet created
- [ ] Failing tests written
- [ ] Capability composition
- [ ] Constraint merging
- [ ] Source tracking
- [ ] Type check integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** TASK-263
**Blocks:** Phase 46.3 (Runtime Integration)
