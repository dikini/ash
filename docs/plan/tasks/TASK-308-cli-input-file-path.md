# TASK-308: Fix ash run --input to Accept File Path (SPEC-005)

## Status: 🔴 Critical - Phase 47/48/49 Gap

## Problem

`ash run --input` currently parses the argument as inline JSON, not as a JSON file path.

SPEC-005 requires:
```bash
ash run workflow.ash --input input.json  # input.json is a FILE path
```

Current behavior:
```bash
$ ash run greet.ash --input /tmp/input.json
error: Invalid JSON input  # Tries to parse the path string as JSON
```

## Root Cause

`crates/ash-cli/src/commands/run.rs:69`:
```rust
let input_values = parse_input(&args.input)?;
```

The `parse_input` function tries to parse the string as JSON directly, not as a file path.

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Lines 35-37, 65-70
- Update `RunArgs` struct documentation
- Modify `parse_input()` or add file reading logic

## Implementation Options

1. **Option A**: Detect if input is a file path (ends with .json or file exists)
2. **Option B**: Add separate `--input-file` flag
3. **Option C**: Try parse as JSON first, fall back to file read

SPEC-005 indicates `--input <file>` should be a file path, not inline JSON.

## Also Required

Support positional `-- <input>` form:
```bash
ash run workflow.ash -- input.json
```

## Verification

```bash
echo '{"name": "World"}' > /tmp/input.json
$ ash run greet.ash --input /tmp/input.json
Hello, World
```

## Completion Checklist

- [ ] `--input` accepts file path
- [ ] `-- <input>` positional form works
- [ ] Backward compatibility for inline JSON (if desired)
- [ ] Tests updated
- [ ] CHANGELOG.md updated

**Estimated Hours:** 6
**Priority:** Critical (SPEC-005 compliance gap)
