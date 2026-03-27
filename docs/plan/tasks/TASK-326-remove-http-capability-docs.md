# TASK-326: Remove HTTP Capability Documentation from SPEC-010

## Status: 🟡 Medium - Documentation Correction

## Problem

The HTTP capability API now fails fast (correctly), but the public contract is still misleading:

**Code** (`crates/ash-engine/src/lib.rs:449-455`):
```rust
// Register HTTP provider if configured
// Note: HTTP provider is not yet implemented.
if self.http_config.is_some() {
    return Err(EngineError::Configuration(
        "HTTP provider not yet implemented. Use with_custom_provider() to add your own HTTP implementation.".to_string(),
    ));
}
```

**Documentation** (`docs/spec/SPEC-010-EMBEDDING.md:53`):
```rust
let engine = Engine::new()
    .with_stdio_capabilities()    -- Add print/read_line
    .with_fs_capabilities()       -- Add file operations
    .with_custom_provider(MyProvider)
    .build()?;
```

Note: SPEC-010 doesn't actually document `with_http_capabilities()` directly in the builder example, but it does document the method exists elsewhere.

**CHANGELOG.md:184**:
```markdown
- Implemented `with_http_capabilities(config)` to configure HTTP client capabilities with timeout, redirect, and SSL settings
```

The code is safer than the earlier no-op, but the docs and changelog still advertise functionality that does not exist.

## Resolution

**Remove HTTP capabilities from SPEC-010.** The method exists but returns an error. Documentation should reflect this reality.

## Implementation

### 1. Update SPEC-010

Modify `docs/spec/SPEC-010-EMBEDDING.md`:

**Add section about unimplemented capabilities:**
```markdown
### 4.3 Unimplemented Capabilities

**HTTP Capabilities:**

The `with_http_capabilities()` method exists in the API but returns a configuration error
when called. HTTP support is planned for a future release. For now, use `with_custom_provider()`
to add your own HTTP implementation:

```rust
let engine = Engine::new()
    .with_custom_provider("http", Arc::new(MyHttpProvider))
    .build()?;
```
```

**Update builder pattern example** to remove implication that HTTP is available:
```rust
let engine = Engine::new()
    .with_stdio_capabilities()    -- Add print/read_line
    .with_fs_capabilities()       -- Add file operations
    -- HTTP available via with_custom_provider() until native support added
    .with_custom_provider(MyProvider)
    .build()?;
```

### 2. Update CHANGELOG.md

Correct the entry at line 184:

**Current:**
```markdown
- Implemented `with_http_capabilities(config)` to configure HTTP client capabilities with timeout, redirect, and SSL settings
```

**New:**
```markdown
- Added `with_http_capabilities(config)` method that returns a configuration error with guidance to use `with_custom_provider()` instead. Native HTTP provider implementation is planned for a future release.
```

### 3. Update Method Documentation

Ensure the rustdoc for `with_http_capabilities()` is clear:

```rust
/// Configure HTTP capabilities (not yet implemented)
///
/// # Errors
///
/// This method currently returns a `Configuration` error as the HTTP provider
/// is not yet implemented. Use `with_custom_provider()` to add a custom HTTP
/// implementation.
///
/// # Example
///
/// ```rust
/// let result = Engine::new()
///     .with_http_capabilities(HttpConfig::new())
///     .build();
/// assert!(result.is_err()); // HTTP provider not yet implemented
/// ```
pub fn with_http_capabilities(mut self, config: HttpConfig) -> Self {
    self.http_config = Some(config);
    self
}
```

## Files to Modify

- `docs/spec/SPEC-010-EMBEDDING.md` - Add "Unimplemented Capabilities" section
- `CHANGELOG.md` - Correct HTTP capability entry (line ~184)
- `crates/ash-engine/src/lib.rs` - Update rustdoc for `with_http_capabilities()`

## Verification

```rust
// Test that documentation example shows the error
#[test]
fn test_http_capabilities_documented_behavior() {
    let result = Engine::new()
        .with_http_capabilities(HttpConfig::new())
        .build();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not yet implemented"));
    assert!(err.contains("with_custom_provider"));
}
```

## Completion Checklist

- [ ] SPEC-010 updated with "Unimplemented Capabilities" section
- [ ] CHANGELOG.md entry corrected to reflect actual behavior
- [ ] Rustdoc for `with_http_capabilities()` updated
- [ ] No documentation claims HTTP provider is fully implemented
- [ ] Users are directed to `with_custom_provider()` as workaround

**Estimated Hours:** 1
**Priority:** Medium (documentation honesty)

## Related

- Implementation: TASK-319-fix-http-noop.md (returns error)
- Previous task: TASK-312-http-provider-noop.md (documented the no-op)
- Related spec: SPEC-010-EMBEDDING.md
