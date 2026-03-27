# TASK-312: Fix EngineBuilder HTTP Provider No-Op

## Status: 🟡 Medium - API Mismatch

## Problem

`EngineBuilder::with_http_capabilities()` is a silent no-op. The public API, docs, and tests present HTTP as a built-in provider, but `build()` just checks `http_config.is_some()` and does nothing.

`crates/ash-engine/src/lib.rs:444-448`:
```rust
// Register HTTP provider if configured
if self.http_config.is_some() {
    // HTTP provider would be registered here when implemented
    // For now, we just acknowledge the config was provided
}
```

This is misleading API design and a SPEC-010 embedding contract gap.

## Options

1. **Implement HTTP provider** (significant work)
2. **Remove the method** until implemented (breaking change)
3. **Document as unimplemented** and return error
4. **Add runtime warning** when called

## Recommendation

Option 3: Return an error or panic in debug mode to make the unimplemented status obvious.

## Files to Modify

- `crates/ash-engine/src/lib.rs` - Lines 444-448
- Update documentation to reflect actual status

## Completion Checklist

- [ ] HTTP provider implemented OR method removed/documented
- [ ] Tests updated
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2 (for documentation/error approach)
**Priority:** Medium (API honesty)
