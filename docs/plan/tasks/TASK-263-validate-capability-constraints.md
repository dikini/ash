# TASK-263: Validate Capability Constraints

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Type check capability constraints against capability definitions.

**Spec Reference:** SPEC-017, SPEC-024

**File Locations:**
- Modify: `crates/ash-typeck/src/constraint_checking.rs` (or create)
- Test: `crates/ash-typeck/tests/constraint_type_tests.rs`

---

## Background

Capabilities have constraints:
```ash
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}

-- Usage with constraints
workflow processor capabilities: [
    file @ { paths: ["/tmp/*"], read: true }  -- Must match schema
]
```

Type system must validate constraints match capability definition schema.

---

## Step 1: Create Constraint Checking Module

Create `crates/ash-typeck/src/constraint_checking.rs`:

```rust
use ash_core::*;
use crate::{TypeError, TypeResult};

pub struct ConstraintChecker<'a> {
    capability_defs: &'a HashMap<String, CapabilityDef>,
}

impl<'a> ConstraintChecker<'a> {
    pub fn new(capability_defs: &'a HashMap<String, CapabilityDef>) -> Self {
        Self { capability_defs }
    }
    
    /// Validate a capability declaration against its definition
    pub fn check_capability_decl(&self, decl: &CapabilityDecl) -> TypeResult<()> {
        let def = self.capability_defs.get(&decl.capability)
            .ok_or_else(|| TypeError::UnknownCapability {
                name: decl.capability.clone(),
                span: decl.span,
            })?;
        
        if let Some(constraints) = &decl.constraints {
            self.check_constraints(&decl.capability, constraints, def)?;
        }
        
        Ok(())
    }
    
    fn check_constraints(
        &self,
        cap_name: &str,
        constraints: &ConstraintBlock,
        def: &CapabilityDef,
    ) -> TypeResult<()> {
        for field in &constraints.fields {
            // Check field is valid for this capability
            if !self.is_valid_constraint_field(cap_name, &field.name) {
                return Err(TypeError::InvalidConstraintField {
                    capability: cap_name.to_string(),
                    field: field.name.clone(),
                    span: field.span,
                });
            }
            
            // Check value type matches expected
            self.check_constraint_value(&field.name, &field.value, def)?;
        }
        
        Ok(())
    }
    
    fn is_valid_constraint_field(&self, cap_name: &str, field: &str) -> bool {
        // Based on capability type
        match cap_name {
            "file" => matches!(field, "paths" | "read" | "write"),
            "network" => matches!(field, "hosts" | "ports" | "protocols"),
            "process" => matches!(field, "spawn" | "kill" | "signal"),
            _ => true, // Unknown capability, defer runtime check
        }
    }
    
    fn check_constraint_value(
        &self,
        field_name: &str,
        value: &ConstraintValue,
        def: &CapabilityDef,
    ) -> TypeResult<()> {
        // Check value type matches field expectation
        match (field_name, value) {
            ("read" | "write" | "spawn" | "kill" | "signal", ConstraintValue::Bool(_)) => Ok(()),
            ("paths" | "hosts", ConstraintValue::Array(arr)) => {
                // Check array elements are strings
                for elem in arr {
                    if !matches!(elem, ConstraintValue::String(_)) {
                        return Err(TypeError::ConstraintTypeMismatch {
                            field: field_name.to_string(),
                            expected: "string array".to_string(),
                            found: elem.type_name(),
                        });
                    }
                }
                Ok(())
            }
            ("ports", ConstraintValue::Array(arr)) => {
                // Check array elements are integers
                for elem in arr {
                    if !matches!(elem, ConstraintValue::Int(_)) {
                        return Err(TypeError::ConstraintTypeMismatch {
                            field: field_name.to_string(),
                            expected: "integer array".to_string(),
                            found: elem.type_name(),
                        });
                    }
                }
                Ok(())
            }
            _ => Ok(()), // Unknown field, defer to runtime
        }
    }
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-typeck/tests/constraint_type_tests.rs
use ash_typeck::*;
use ash_core::*;

fn file_capability_def() -> CapabilityDef {
    CapabilityDef {
        name: "file".to_string(),
        effect: Effect::Operational,
        fields: vec![
            FieldDef::new("paths", Type::Array(Box::new(Type::String))),
            FieldDef::new("read", Type::Bool),
            FieldDef::new("write", Type::Bool),
        ],
    }
}

#[test]
fn test_valid_file_constraints() {
    let mut caps = HashMap::new();
    caps.insert("file".to_string(), file_capability_def());
    
    let checker = ConstraintChecker::new(&caps);
    let decl = CapabilityDecl::new("file")
        .with_constraint("paths", ConstraintValue::Array(vec![
            ConstraintValue::String("/tmp/*".to_string())
        ]))
        .with_constraint("read", ConstraintValue::Bool(true));
    
    assert!(checker.check_capability_decl(&decl).is_ok());
}

#[test]
fn test_invalid_constraint_field() {
    let mut caps = HashMap::new();
    caps.insert("file".to_string(), file_capability_def());
    
    let checker = ConstraintChecker::new(&caps);
    let decl = CapabilityDecl::new("file")
        .with_constraint("invalid_field", ConstraintValue::Bool(true));
    
    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());
}

#[test]
fn test_constraint_type_mismatch() {
    let mut caps = HashMap::new();
    caps.insert("file".to_string(), file_capability_def());
    
    let checker = ConstraintChecker::new(&caps);
    let decl = CapabilityDecl::new("file")
        .with_constraint("read", ConstraintValue::String("yes".to_string()));  // Should be bool
    
    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, TypeError::ConstraintTypeMismatch { .. }));
}

#[test]
fn test_unknown_capability() {
    let caps = HashMap::new();
    
    let checker = ConstraintChecker::new(&caps);
    let decl = CapabilityDecl::new("unknown_cap");
    
    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn test_valid_bool_constraint(b in prop::bool::ANY) {
        // Bool fields accept bool values
    }
}
```

---

## Step 3: Integrate into Type Checker

```rust
// crates/ash-typeck/src/lib.rs
pub fn type_check_module(module: &Module) -> TypeResult<TypeCheckedModule> {
    // ... role checking ...
    
    // Check capability constraints
    let constraint_checker = ConstraintChecker::new(&module.capabilities);
    
    // Check role capability declarations
    for role in &module.roles {
        for cap in &role.capabilities {
            constraint_checker.check_capability_decl(cap)?;
        }
    }
    
    // Check workflow capability declarations
    for workflow in &module.workflows {
        for cap in &workflow.header.capabilities {
            constraint_checker.check_capability_decl(cap)?;
        }
    }
    
    // ... rest of type checking ...
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-typeck constraint -v
```

---

## Step 5: Commit

```bash
git add crates/ash-typeck/src/constraint_checking.rs
git add crates/ash-typeck/tests/constraint_type_tests.rs
git add crates/ash-typeck/src/lib.rs
git commit -m "feat: validate capability constraints (TASK-263)

- Add ConstraintChecker for constraint validation
- Check constraints match capability schema
- Validate field names are valid for capability type
- Check value types match expected (bool, string array, etc.)
- Error messages for unknown capabilities, invalid fields, type mismatches
- Integration with main type check flow
- Tests for valid and invalid constraints"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-263 implementation"
  context: |
    Files to verify:
    - crates/ash-typeck/src/constraint_checking.rs
    - crates/ash-typeck/tests/constraint_type_tests.rs
    - crates/ash-typeck/src/lib.rs
    
    Spec reference: SPEC-017, SPEC-024
    Requirements:
    1. Valid constraints accepted
    2. Invalid field names rejected
    3. Type mismatches detected (bool vs string)
    4. Unknown capabilities rejected
    5. Error messages clear and helpful
    6. Works for role and workflow declarations
    
    Run and report:
    1. cargo test --package ash-typeck constraint
    2. cargo clippy --package ash-typeck --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-typeck
    4. Check error quality for each error type
    5. Test all capability types (file, network, process)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] ConstraintChecker created
- [ ] Failing tests written
- [ ] Field validation implemented
- [ ] Type checking implemented
- [ ] Type check integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-262
**Blocks:** TASK-264 (effective capability sets)
