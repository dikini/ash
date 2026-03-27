# TASK-320: Fix Timeout Diagnostics

## Status: 🟡 Medium - Information Loss

## Problem

Timeout diagnostics lose the configured duration. User sees "timeout after 0s" instead of actual timeout.

**Current code (error.rs:196-198):**
```rust
} else if msg.contains("timeout") {
    // Try to extract seconds from message
    CliError::Timeout { seconds: 0 }  // Always 0!
}
```

**In run.rs:111:**
```rust
return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
```

The message contains the actual timeout, but the From impl ignores it and always uses 0.

## Root Cause

The `From<anyhow::Error>` implementation doesn't extract the timeout value from the error message.

## Files to Modify

- `crates/ash-cli/src/error.rs` - Lines 196-198

## Implementation

Parse the timeout value from the message:
```rust
} else if msg.contains("timeout") {
    // Extract seconds from message like "timeout after 5s"
    let seconds = msg
        .split("after ")
        .nth(1)
        .and_then(|s| s.split('s').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    CliError::Timeout { seconds }
}
```

## Verification

```bash
$ ash run --timeout 5 workflow.ash
timeout after 5s  # Should show actual timeout, not 0s
```

## Completion Checklist

- [ ] Timeout value extracted from error message
- [ ] User sees actual timeout duration
- [ ] Fallback to 0 only if parsing fails
- [ ] CHANGELOG.md updated

**Estimated Hours:** 1
**Priority:** Medium (diagnostics quality)
