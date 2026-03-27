# TASK-317: Fix --capability Provider Binding

## Status: 🔴 Critical - Phase 50 Gap

## Problem

`--capability <name=uri>` is not actually binding providers:

1. **URI is parsed and discarded** (`_uri` at line 148)
2. **Unknown capability names are silently ignored** (lines 164-168)
3. **Documented `db=postgres://...` form cannot work**

**Current code:**
```rust
for cap_str in &args.capability {
    let (name, _uri) = parse_capability(cap_str);  // URI discarded!
    match name {
        "stdio" => {},
        "fs" => {},
        "http" => {},
        _ => {
            // Unknown capability - silently ignored!
        }
    }
}
```

## Root Cause

The implementation only handles built-in providers by name. URI-based provider binding is not implemented, and unknown capabilities are silently ignored instead of returning an error.

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Lines 147-170

## Implementation

1. **Option A**: Implement URI-based provider binding for known URI schemes
2. **Option B**: Return error for unknown capabilities instead of silently ignoring
3. **Option C**: Document that only built-in providers are supported

## Recommendation

Implement Option B immediately (fail fast for unknown capabilities), then Option A for full SPEC-005 compliance.

## Verification

```bash
# Should error clearly
$ ash run workflow.ash --capability db=postgres://...
Error: Unknown capability 'db'. Supported: stdio, fs, http

# Should work
$ ash run workflow.ash --capability http
```

## Completion Checklist

- [ ] Unknown capabilities return clear error (not silent)
- [ ] URI is used for provider binding (not discarded)
- [ ] Help text reflects actual capability support
- [ ] Tests verify error handling
- [ ] CHANGELOG.md updated

**Estimated Hours:** 6
**Priority:** Critical (API contract gap)
