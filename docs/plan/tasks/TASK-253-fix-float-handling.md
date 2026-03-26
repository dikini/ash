# TASK-253: Fix Float Handling

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Replace silent float truncation with explicit error or full float support.

**Spec Reference:** SPEC-002 (Surface Syntax), Numeric literal semantics

**File Locations:**
- `crates/ash-parser/src/lower.rs:531`
- `crates/ash-cli/src/commands/run.rs:90`

---

## Background

The audit found silent dangerous coercion:

```rust
// Parser lowering (line 531)
serde_json::Value::Number(n) => {
    // Silent truncation to Int!
    n.as_i64().map(Value::Int).unwrap_or(Value::Null)
}

// CLI input (line 90)
// JSON numbers that aren't integers become Value::Null
```

Options:
1. **Add full float support** - Add Float variant to Value
2. **Explicit error** - Reject non-integers with clear error

Decision: Option 2 for now (explicit error), Option 1 later if needed.

---

## Step 1: Understand Value Type

Check current Value enum:

```bash
grep -n "enum Value" crates/ash-core/src/value.rs
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-parser/tests/float_handling_tests.rs
use ash_parser::*;

#[test]
fn test_float_literal_rejected() {
    let result = parse_expr("3.14");
    // Should fail with explicit error about float support
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("float"));
}

#[test]
fn test_integer_literal_accepted() {
    let result = parse_expr("42");
    assert!(result.is_ok());
}

#[test]
fn test_json_float_rejected() {
    use serde_json::json;
    
    let result = convert_json_value(json!(3.14));
    // Should fail, not return Null
    assert!(result.is_err());
}
```

---

## Step 3: Implement Explicit Error

### Parser Lowering Fix

```rust
// crates/ash-parser/src/lower.rs:531
serde_json::Value::Number(n) => {
    if let Some(i) = n.as_i64() {
        Ok(Value::Int(i))
    } else {
        Err(LoweringError::UnsupportedNumericType {
            value: n.to_string(),
            note: "Float support not yet implemented".to_string(),
        })
    }
}
```

### CLI Input Fix

```rust
// crates/ash-cli/src/commands/run.rs:90
fn parse_input_value(input: &str) -> Result<Value, anyhow::Error> {
    let json: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
    
    match json {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else {
                Err(anyhow::anyhow!(
                    "Non-integer numbers not supported: {}. Use integer or string representation.",
                    n
                ))
            }
        }
        // ... handle other types ...
    }
}
```

---

## Step 4: Add Error Type

```rust
// crates/ash-parser/src/lower.rs
#[derive(Debug, Clone)]
pub enum LoweringError {
    UnsupportedNumericType {
        value: String,
        note: String,
    },
    // ... other errors ...
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoweringError::UnsupportedNumericType { value, note } => {
                write!(f, "Numeric value '{}' not supported: {}", value, note)
            }
            // ...
        }
    }
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-parser float_handling -v
cargo test --package ash-cli float -v
```

---

## Step 6: Commit

```bash
git add crates/ash-parser/src/lower.rs
git add crates/ash-cli/src/commands/run.rs
git add crates/ash-parser/tests/float_handling_tests.rs
git commit -m "fix: explicit error for float literals (TASK-253)

- Replace silent truncation with explicit LoweringError
- CLI rejects non-integer JSON numbers with clear message
- Add UnsupportedNumericType error variant
- Tests for float rejection and integer acceptance
- Prevents silent data loss from float coercion"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-253 implementation"
  context: |
    Files to verify:
    - crates/ash-parser/src/lower.rs (numeric conversion)
    - crates/ash-cli/src/commands/run.rs (CLI input)
    - crates/ash-parser/tests/float_handling_tests.rs
    
    Requirements:
    1. Float literals produce explicit error
    2. Integer literals work correctly
    3. No silent truncation to Null
    4. Error message mentions float support
    5. CLI input validates JSON numbers
    
    Run and report:
    1. cargo test --package ash-parser float
    2. cargo test --package ash-cli float
    3. cargo clippy --workspace --all-targets --all-features -- -D warnings
    4. cargo fmt --check
    5. Test explicit error for: echo '{"x": 3.14}' | ash run
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Value type reviewed
- [ ] Failing tests written
- [ ] Lowering error added
- [ ] Parser lowering fixed
- [ ] CLI input fixed
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** None
**Blocks:** None

**Note:** Full float support (Value::Float) can be added later as a feature.
