# TASK-266: Capability Constraint Enforcement

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime enforcement of capability constraints.

**Spec Reference:** SPEC-017, SPEC-024

**File Locations:**
- Modify: `crates/ash-interp/src/constraint_enforcement.rs` (or create)
- Test: `crates/ash-interp/tests/constraint_enforcement_tests.rs`

---

## Background

Constraints declared at workflow definition must be enforced at runtime:
```ash
workflow processor capabilities: [
    file @ { paths: ["/tmp/*"], read: true }
]
```

At runtime:
- File reads to `/tmp/data.txt` should succeed
- File reads to `/etc/passwd` should fail

---

## Step 1: Create Constraint Enforcement Module

Create `crates/ash-interp/src/constraint_enforcement.rs`:

```rust
use ash_core::*;
use glob::glob; // or similar pattern matching

/// Enforce capability constraints at runtime
pub struct ConstraintEnforcer;

impl ConstraintEnforcer {
    /// Check if operation satisfies constraints
    pub fn check(
        operation: &str,
        args: &Value,
        constraints: &ConstraintBlock,
    ) -> Result<(), ConstraintViolation> {
        for field in &constraints.fields {
            match field.name.as_str() {
                "paths" => Self::check_paths(operation, args, &field.value)?,
                "hosts" => Self::check_hosts(operation, args, &field.value)?,
                "ports" => Self::check_ports(operation, args, &field.value)?,
                "read" | "write" | "spawn" | "kill" => {
                    Self::check_permission(operation, &field.name, &field.value)?
                }
                _ => {} // Unknown constraint, allow
            }
        }
        Ok(())
    }
    
    fn check_paths(
        operation: &str,
        args: &Value,
        allowed_paths: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        if operation != "read" && operation != "write" {
            return Ok(()); // Path constraints only for file ops
        }
        
        // Extract path from args
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or(ConstraintViolation::MissingArgument("path"))?;
        
        // Check against allowed patterns
        let patterns = match allowed_paths {
            ConstraintValue::Array(arr) => arr,
            _ => return Err(ConstraintViolation::InvalidConstraint("paths must be array")),
        };
        
        for pattern_val in patterns {
            let pattern = match pattern_val {
                ConstraintValue::String(s) => s,
                _ => continue,
            };
            
            // Use glob matching
            if Self::path_matches(path, pattern) {
                return Ok(());
            }
        }
        
        Err(ConstraintViolation::PathNotAllowed {
            path: path.to_string(),
            allowed: patterns.iter()
                .filter_map(|v| match v {
                    ConstraintValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
        })
    }
    
    fn path_matches(path: &str, pattern: &str) -> bool {
        // Use glob crate or simple pattern matching
        // For now: simple prefix matching
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len()-2];
            path.starts_with(prefix)
        } else {
            path == pattern
        }
    }
    
    fn check_hosts(
        operation: &str,
        args: &Value,
        allowed_hosts: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        let host = args.get("host")
            .and_then(|v| v.as_str())
            .ok_or(ConstraintViolation::MissingArgument("host"))?;
        
        let patterns = match allowed_hosts {
            ConstraintValue::Array(arr) => arr,
            _ => return Err(ConstraintViolation::InvalidConstraint("hosts must be array")),
        };
        
        for pattern_val in patterns {
            let pattern = match pattern_val {
                ConstraintValue::String(s) => s,
                _ => continue,
            };
            
            if Self::host_matches(host, pattern) {
                return Ok(());
            }
        }
        
        Err(ConstraintViolation::HostNotAllowed {
            host: host.to_string(),
            allowed: patterns.iter()
                .filter_map(|v| match v {
                    ConstraintValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
        })
    }
    
    fn host_matches(host: &str, pattern: &str) -> bool {
        // Simple glob matching for hosts
        // *.example.com matches api.example.com
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            host.ends_with(suffix)
        } else {
            host == pattern
        }
    }
    
    fn check_permission(
        operation: &str,
        permission: &str,
        value: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        let allowed = match value {
            ConstraintValue::Bool(b) => *b,
            _ => return Ok(()), // Non-bool, assume allowed
        };
        
        // Map operation to permission
        let required_perm = match operation {
            "read" | "get" => "read",
            "write" | "put" | "post" => "write",
            "spawn" => "spawn",
            "kill" => "kill",
            _ => return Ok(()),
        };
        
        if required_perm == permission && !allowed {
            return Err(ConstraintViolation::PermissionDenied {
                operation: operation.to_string(),
                permission: permission.to_string(),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ConstraintViolation {
    MissingArgument(&'static str),
    InvalidConstraint(&'static str),
    PathNotAllowed { path: String, allowed: Vec<String> },
    HostNotAllowed { host: String, allowed: Vec<String> },
    PermissionDenied { operation: String, permission: String },
}

impl std::fmt::Display for ConstraintViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintViolation::PathNotAllowed { path, allowed } => {
                write!(f, "Path '{}' not allowed. Allowed paths: {:?}", path, allowed)
            }
            ConstraintViolation::HostNotAllowed { host, allowed } => {
                write!(f, "Host '{}' not allowed. Allowed hosts: {:?}", host, allowed)
            }
            ConstraintViolation::PermissionDenied { operation, permission } => {
                write!(f, "Operation '{}' requires '{}' permission", operation, permission)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/constraint_enforcement_tests.rs
use ash_interp::constraint_enforcement::*;
use ash_core::*;

#[test]
fn test_path_constraint_allows_matching() {
    let constraints = constraint_block! {
        paths: ["/tmp/*"]
    };
    
    let args = json!({ "path": "/tmp/data.txt" });
    
    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_ok());
}

#[test]
fn test_path_constraint_denies_non_matching() {
    let constraints = constraint_block! {
        paths: ["/tmp/*"]
    };
    
    let args = json!({ "path": "/etc/passwd" });
    
    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, ConstraintViolation::PathNotAllowed { .. }));
}

#[test]
fn test_host_constraint_allows_matching() {
    let constraints = constraint_block! {
        hosts: ["*.example.com"]
    };
    
    let args = json!({ "host": "api.example.com" });
    
    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(result.is_ok());
}

#[test]
fn test_permission_denied() {
    let constraints = constraint_block! {
        read: true,
        write: false
    };
    
    let args = json!({ "path": "/tmp/test.txt" });
    
    // Read should succeed
    assert!(ConstraintEnforcer::check("read", &args, &constraints).is_ok());
    
    // Write should fail
    let result = ConstraintEnforcer::check("write", &args, &constraints);
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn test_path_matching_properties(path in "/[a-z/]*") {
        // Property: /tmp/* matches /tmp/x but not /var/x
    }
}
```

---

## Step 3: Integrate into Provider Calls

```rust
// In provider invocation
pub fn invoke_capability(
    &self,
    capability: &str,
    operation: &str,
    args: &Value,
) -> Result<Value, ExecError> {
    // Check capability granted
    let grant = self.capabilities.get_grant(capability)
        .ok_or(ExecError::CapabilityNotGranted(capability.to_string()))?;
    
    // Check constraints
    if let Some(constraints) = &grant.constraints {
        ConstraintEnforcer::check(operation, args, constraints)
            .map_err(|e| ExecError::ConstraintViolation(e))?;
    }
    
    // Invoke provider
    self.providers.invoke(capability, operation, args)
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-interp constraint_enforcement -v
```

---

## Step 5: Commit

```bash
git add crates/ash-interp/src/constraint_enforcement.rs
git add crates/ash-interp/tests/constraint_enforcement_tests.rs
git add crates/ash-interp/src/lib.rs
git commit -m "feat: runtime capability constraint enforcement (TASK-266)

- Add ConstraintEnforcer for runtime checks
- Path constraint checking with glob patterns
- Host constraint checking with wildcard support
- Permission-based constraints (read/write/spawn/kill)
- ConstraintViolation errors with context
- Integration with provider invocation
- Tests for allow/deny scenarios"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-266 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/constraint_enforcement.rs
    - crates/ash-interp/tests/constraint_enforcement_tests.rs
    - crates/ash-interp/src/lib.rs (integration)
    
    Spec reference: SPEC-017, SPEC-024
    Requirements:
    1. Paths constraints enforced with patterns
    2. Host constraints enforced with wildcards
    3. Permission constraints work
    4. Allowed operations succeed
    5. Denied operations fail with clear error
    6. Error messages helpful
    
    Run and report:
    1. cargo test --package ash-interp constraint
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Test boundary cases (/tmp/* matching /tmp)
    5. Test error message quality
    6. Test integration with provider calls
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] ConstraintEnforcer created
- [ ] Failing tests written
- [ ] Path constraint enforcement
- [ ] Host constraint enforcement
- [ ] Permission enforcement
- [ ] Provider integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-265
**Blocks:** TASK-267 (yield routing)
