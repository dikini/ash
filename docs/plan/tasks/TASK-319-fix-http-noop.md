# TASK-319: Fix EngineBuilder HTTP Provider No-Op

## Status: 🟡 High - API Misleading

## Problem

`EngineBuilder::with_http_capabilities()` remains a **silent no-op**:

- Public API presents HTTP as built-in provider
- `build()` never registers an HTTP provider
- Users get misleading success when calling `with_http_capabilities()`

**Current code:**
```rust
if self.http_config.is_some() {
    // HTTP provider would be registered here when implemented
    // For now, we just acknowledge the config was provided
}
```

## Options

1. **Implement HTTP provider** (significant work)
2. **Return error** when HTTP is requested (fail fast)
3. **Add deprecation warning** and document limitation
4. **Remove the method** until implemented (breaking change)

## Recommendation

Option 2 (return error) for immediate fix:
```rust
if self.http_config.is_some() {
    return Err(EngineError::Configuration {
        message: "HTTP provider not yet implemented. Use with_custom_provider() instead."
    });
}
```

## Files to Modify

- `crates/ash-engine/src/lib.rs` - Lines 444-448
- `crates/ash-engine/src/error.rs` - Add Configuration error variant if needed

## Verification

```rust
let result = Engine::new()
    .with_http_capabilities(HttpConfig::new())
    .build();
assert!(result.is_err());  // Should fail clearly
```

## Completion Checklist

- [ ] HTTP config returns clear error (not silent success)
- [ ] Error message suggests workaround
- [ ] Tests updated to expect error
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2
**Priority:** High (API honesty)
