# TASK-247: Implement Stub Providers

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Replace stub provider implementations with real functionality for stdio and filesystem.

**Spec Reference:** SPEC-010 (Embedding API), SPEC-014 (Behaviours)

**File Locations:**
- Modify: `crates/ash-engine/src/providers.rs:82,95,133,146`
- Test: `crates/ash-engine/tests/provider_execution_tests.rs` (create)

---

## Background

The audit found provider methods discard inputs and return `Value::Null`:

```rust
// Current stubs
fn read_file(&self, _path: &str) -> Value {
    Value::Null  // Actually does nothing
}

fn write_file(&self, _path: &str, _content: Value) -> Value {
    Value::Null
}
```

This violates SPEC-010 and SPEC-014's capability contracts.

---

## Step 1: Audit Provider Implementations

Find all stub methods:

```bash
grep -n "Value::Null" crates/ash-engine/src/providers.rs
grep -n "fn.*->.*Value" crates/ash-engine/src/providers.rs
```

Review Provider trait:

```bash
grep -n "trait Provider" crates/ash-engine/src/providers.rs
```

---

## Step 2: Design Real Providers

### Filesystem Provider

```rust
pub struct FilesystemProvider {
    base_paths: Vec<PathBuf>,  // Allowed paths (capability constraint)
    allow_write: bool,
}

impl Provider for FilesystemProvider {
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        match operation {
            "read" => self.read_file(args),
            "write" => {
                if !self.allow_write {
                    return Err(ProviderError::PermissionDenied);
                }
                self.write_file(args)
            }
            "list" => self.list_directory(args),
            _ => Err(ProviderError::UnknownOperation),
        }
    }
}
```

### Stdio Provider

```rust
pub struct StdioProvider;

impl Provider for StdioProvider {
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        match operation {
            "print" => self.print(args),
            "read_line" => self.read_line(),
            "eprint" => self.eprint(args),
            _ => Err(ProviderError::UnknownOperation),
        }
    }
}
```

---

## Step 3: Write Failing Tests

```rust
// crates/ash-engine/tests/provider_execution_tests.rs
use ash_engine::*;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_filesystem_read() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std::fs::write(&file_path, "hello").unwrap();
    
    let provider = FilesystemProvider::new()
        .allow_path(temp.path())
        .read_only();
    
    let result = provider.invoke("read", json!({"path": file_path}));
    
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_filesystem_write() {
    let temp = TempDir::new().unwrap();
    let provider = FilesystemProvider::new()
        .allow_path(temp.path())
        .read_write();
    
    provider.invoke("write", json!({
        "path": temp.path().join("out.txt"),
        "content": "world"
    })).unwrap();
    
    let content = std::fs::read_to_string(temp.path().join("out.txt")).unwrap();
    assert_eq!(content, "world");
}

#[test]
fn test_stdio_print() {
    let provider = StdioProvider::new();
    
    // Capture stdout
    let mut output = Vec::new();
    {
        let mut handle = provider.capture_stdout(&mut output);
        provider.invoke("print", json!("hello")).unwrap();
    }
    
    assert_eq!(String::from_utf8(output).unwrap(), "hello");
}

#[test]
fn test_capability_constraints_enforced() {
    let temp = TempDir::new().unwrap();
    let allowed = temp.path().join("allowed");
    let forbidden = temp.path().join("forbidden");
    
    let provider = FilesystemProvider::new()
        .allow_path(&allowed)  // Only this path allowed
        .read_write();
    
    // Writing to allowed path works
    provider.invoke("write", json!({
        "path": allowed.join("file.txt"),
        "content": "ok"
    })).unwrap();
    
    // Writing to forbidden path fails
    let result = provider.invoke("write", json!({
        "path": forbidden.join("file.txt"),
        "content": "not ok"
    }));
    
    assert!(result.is_err());
}
```

---

## Step 4: Implement Filesystem Provider

```rust
// crates/ash-engine/src/providers/filesystem.rs
use std::path::{Path, PathBuf};

pub struct FilesystemProvider {
    allowed_paths: Vec<PathBuf>,
    allow_write: bool,
}

impl FilesystemProvider {
    pub fn new() -> Self {
        Self {
            allowed_paths: Vec::new(),
            allow_write: false,
        }
    }
    
    pub fn allow_path(mut self, path: impl AsRef<Path>) -> Self {
        self.allowed_paths.push(path.as_ref().to_path_buf());
        self
    }
    
    pub fn read_only(mut self) -> Self {
        self.allow_write = false;
        self
    }
    
    pub fn read_write(mut self) -> Self {
        self.allow_write = true;
        self
    }
    
    fn check_path_allowed(&self, path: &Path) -> Result<(), ProviderError> {
        let canonical = path.canonicalize()
            .map_err(|_| ProviderError::InvalidPath)?;
        
        for allowed in &self.allowed_paths {
            if canonical.starts_with(allowed) {
                return Ok(());
            }
        }
        
        Err(ProviderError::PathNotAllowed)
    }
    
    fn read_file(&self, args: Value) -> Result<Value, ProviderError> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or(ProviderError::MissingArgument("path"))?;
        
        let path = Path::new(path);
        self.check_path_allowed(path)?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProviderError::IoError(e.to_string()))?;
        
        Ok(Value::String(content))
    }
    
    fn write_file(&self, args: Value) -> Result<Value, ProviderError> {
        if !self.allow_write {
            return Err(ProviderError::PermissionDenied);
        }
        
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or(ProviderError::MissingArgument("path"))?;
        
        let content = args.get("content")
            .ok_or(ProviderError::MissingArgument("content"))?;
        
        let path = Path::new(path);
        self.check_path_allowed(path)?;
        
        let content_str = match content {
            Value::String(s) => s,
            other => other.to_string(),
        };
        
        std::fs::write(path, content_str)
            .map_err(|e| ProviderError::IoError(e.to_string()))?;
        
        Ok(Value::Null)
    }
}

impl Provider for FilesystemProvider {
    fn capability_name(&self) -> &str { "filesystem" }
    
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        match operation {
            "read" => self.read_file(args),
            "write" => self.write_file(args),
            _ => Err(ProviderError::UnknownOperation(operation.to_string())),
        }
    }
}
```

---

## Step 5: Implement Stdio Provider

```rust
// crates/ash-engine/src/providers/stdio.rs
use std::io::{self, Write};

pub struct StdioProvider;

impl StdioProvider {
    pub fn new() -> Self {
        Self
    }
    
    fn print(&self, args: Value) -> Result<Value, ProviderError> {
        let message = match &args {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        
        println!("{}", message);
        io::stdout().flush().ok();
        
        Ok(Value::Null)
    }
    
    fn eprint(&self, args: Value) -> Result<Value, ProviderError> {
        let message = match &args {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        
        eprintln!("{}", message);
        io::stderr().flush().ok();
        
        Ok(Value::Null)
    }
    
    fn read_line(&self, _args: Value) -> Result<Value, ProviderError> {
        use std::io::BufRead;
        
        let stdin = io::stdin();
        let mut line = String::new();
        
        stdin.lock().read_line(&mut line)
            .map_err(|e| ProviderError::IoError(e.to_string()))?;
        
        // Trim newline
        line.pop();
        
        Ok(Value::String(line))
    }
}

impl Provider for StdioProvider {
    fn capability_name(&self) -> &str { "stdio" }
    
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        match operation {
            "print" => self.print(args),
            "eprint" => self.eprint(args),
            "read_line" => self.read_line(args),
            _ => Err(ProviderError::UnknownOperation(operation.to_string())),
        }
    }
}
```

---

## Step 6: Update Provider Registry

```rust
// crates/ash-engine/src/providers.rs
mod filesystem;
mod stdio;

pub use filesystem::FilesystemProvider;
pub use stdio::StdioProvider;
```

---

## Step 7: Run Tests

```bash
cargo test --package ash-engine provider_execution -v
```

---

## Step 8: Commit

```bash
git add crates/ash-engine/src/providers/
git add crates/ash-engine/tests/provider_execution_tests.rs
git commit -m "feat: implement real filesystem and stdio providers (TASK-247)

- Replace Value::Null stubs with actual implementations
- FilesystemProvider with path constraints (capability security)
- StdioProvider for print/eprint/read_line operations
- Capability constraint enforcement (path allowlists)
- Error handling for permission denied, invalid paths
- Tests for read, write, and constraint enforcement
- Aligns with SPEC-010 and SPEC-014 contracts"
```

---

## Step 9: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-247 implementation"
  context: |
    Files to verify:
    - crates/ash-engine/src/providers/filesystem.rs
    - crates/ash-engine/src/providers/stdio.rs
    - crates/ash-engine/tests/provider_execution_tests.rs
    
    Spec reference: SPEC-010, SPEC-014
    Requirements:
    1. Filesystem read actually reads files
    2. Filesystem write actually writes files
    3. Path constraints enforced (security)
    4. Stdio print/eprint output to stdout/stderr
    5. Read_line reads from stdin
    6. No more Value::Null stubs
    7. Proper error handling
    
    Run and report:
    1. cargo test --package ash-engine provider_execution
    2. cargo clippy --package ash-engine --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-engine
    4. Verify no stub implementations remain
    5. Check capability constraint enforcement
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Stubs audited
- [ ] Failing tests written
- [ ] FilesystemProvider implemented
- [ ] StdioProvider implemented
- [ ] Path constraints enforced
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 12
**Blocked by:** None (can parallel with TASK-246)
**Blocks:** None
