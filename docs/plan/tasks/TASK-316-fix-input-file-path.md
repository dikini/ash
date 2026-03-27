# TASK-316: Fix ash run --input to Accept File Path (SPEC-005)

## Status: 🔴 Critical - Phase 50 Gap

## Problem

`ash run --input` is still parsed as inline JSON, not as a file path. Additionally, there's no support for trailing positional input after `--`.

**SPEC-005 requires:**
```bash
ash run <wf> --input data.json    # data.json is a FILE path
ash run <wf> -- '{"name":"test"}'  # Positional input after --
```

**Current behavior:**
```bash
$ ash run <wf> --input data.json
Invalid JSON input  # Tries to parse filename as JSON

$ ash run <wf> -- '{"name":"test"}'
error: unexpected argument  # Clap rejects --
```

## Root Cause

1. `RunArgs.input` help text says "JSON object" not "file path"
2. `parse_input()` function tries to parse as JSON directly
3. No clap configuration for trailing positional arguments after `--`

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Lines 37-39 (arg definition), 191+ (parse_input)
- May need to update clap argument configuration for positional args

## Implementation

1. Change `--input` to accept file path
2. Try reading as file first, fall back to inline JSON for backward compatibility
3. Add support for `-- <input>` positional syntax
4. Update help text to reflect file path behavior

## Verification

```bash
# File path should work
echo '{"name": "test"}' > data.json
$ ash run workflow.ash --input data.json

# Positional after -- should work
$ ash run workflow.ash -- '{"name":"test"}'
```

## Completion Checklist

- [ ] `--input` accepts file path per SPEC-005
- [ ] `-- <input>` positional syntax works
- [ ] Help text updated
- [ ] Tests verify both patterns
- [ ] CHANGELOG.md updated

**Estimated Hours:** 4
**Priority:** Critical (SPEC-005 compliance)
