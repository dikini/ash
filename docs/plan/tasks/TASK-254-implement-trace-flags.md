# TASK-254: Implement Trace Flags or Remove

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Make `--lineage` and `--verify` trace flags functional or remove them.

**Spec Reference:** SPEC-005 (CLI), Trace command semantics

**File Locations:**
- `crates/ash-cli/src/commands/trace.rs:32,72`

---

## Background

The audit found:

```rust
// crates/ash-cli/src/commands/trace.rs:32,72
// --lineage and --verify are parsed but _args is unused
```

Options:
1. **Implement the flags** - Add lineage tracking and verification
2. **Remove the flags** - Don't advertise what doesn't work

Decision: Check if underlying infrastructure exists, then decide.

---

## Step 1: Check Infrastructure

```bash
grep -rn "lineage\|Lineage" crates/ash-core/src/
grep -rn "verify\|Verify" crates/ash-cli/src/ | grep -v "mod\|use"
```

Check provenance module:

```bash
grep -n "pub fn" crates/ash-provenance/src/lib.rs
```

---

## Step 2: Decide Implement vs Remove

### If Lineage Infrastructure Exists → IMPLEMENT

```rust
// In trace command
if args.lineage {
    let lineage = provenance.compute_lineage(&trace);
    output.insert("lineage", json!(lineage));
}

if args.verify {
    let verification = provenance.verify_integrity(&trace);
    output.insert("verification", json!(verification));
}
```

### If Infrastructure Missing → REMOVE

```rust
// Remove from TraceArgs struct
// Remove from help text
// Don't advertise unavailable features
```

---

## Step 3: Implementation Path

Assuming infrastructure exists or can be added:

### Add Lineage Export

```rust
// crates/ash-provenance/src/lib.rs
impl ProvenanceLog {
    pub fn compute_lineage(&self, trace_id: &TraceId) -> Lineage {
        // Build lineage graph from trace events
        Lineage::from_events(&self.events)
    }
}
```

### Add Verification

```rust
impl ProvenanceLog {
    pub fn verify_integrity(&self) -> VerificationResult {
        // Check Merkle tree integrity
        // Verify all hashes
        VerificationResult {
            valid: self.check_hashes(),
            errors: self.find_corruptions(),
        }
    }
}
```

### Update Trace Command

```rust
// crates/ash-cli/src/commands/trace.rs
pub fn execute(args: TraceArgs) -> Result<()> {
    let trace = load_trace(&args.trace_file)?;
    let provenance = load_provenance(&args.trace_file)?;
    
    let mut output = json!({
        "trace": trace,
    });
    
    if args.lineage {
        let lineage = provenance.compute_lineage(&trace.id);
        output["lineage"] = json!(lineage);
    }
    
    if args.verify {
        let verification = provenance.verify_integrity();
        output["verification"] = json!(verification);
    }
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
```

---

## Step 4: Write Tests

```rust
// crates/ash-cli/tests/trace_tests.rs
#[test]
fn test_trace_with_lineage() {
    let output = Command::new("ash")
        .args(["trace", "--lineage", "test.trace"])
        .output()
        .unwrap();
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("lineage").is_some());
}

#[test]
fn test_trace_with_verify() {
    let output = Command::new("ash")
        .args(["trace", "--verify", "test.trace"])
        .output()
        .unwrap();
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("verification").is_some());
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-cli trace -v
```

---

## Step 6: Commit

```bash
git add crates/ash-cli/src/commands/trace.rs
git add crates/ash-provenance/src/lib.rs  # if modified
git add crates/ash-cli/tests/trace_tests.rs
git commit -m "feat: implement --lineage and --verify trace flags (TASK-254)

- Add lineage computation from provenance log
- Add integrity verification for traces
- Connect flags to actual functionality
- Tests for lineage and verification output
- Removes placeholder gap between CLI and implementation"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-254 implementation"
  context: |
    Files to verify:
    - crates/ash-cli/src/commands/trace.rs (flag handling)
    - crates/ash-provenance/src/lib.rs (lineage/verify)
    - crates/ash-cli/tests/trace_tests.rs
    
    Requirements:
    1. --lineage produces lineage output
    2. --verify produces verification output
    3. Both flags can be used together
    4. No more unused _args
    5. Help text matches behavior
    
    Run and report:
    1. cargo test --package ash-cli trace
    2. cargo build --release
    3. Test: ./target/release/ash trace --lineage test.trace
    4. Test: ./target/release/ash trace --verify test.trace
    5. cargo clippy --package ash-cli --all-targets --all-features -- -D warnings
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Infrastructure checked
- [ ] Implement/Remove decision made
- [ ] Code implemented (or flags removed)
- [ ] Tests written
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 4 (if remove) / 8 (if implement)
**Blocked by:** None
**Blocks:** None
