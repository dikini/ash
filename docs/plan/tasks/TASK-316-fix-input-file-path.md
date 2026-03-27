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

## Input JSON Format

The `--input` JSON should be an **object** where keys are workflow parameter names and values are the parameter values:

```json
{
  "param1": "string value",
  "param2": 42,
  "param3": true,
  "param4": ["item1", "item2"],
  "param5": {"nested": "object"}
}
```

### Example Workflow with Input

**Workflow file (greet.ash):**
```ash
workflow greet(name: String, times: Int) {
    ret "Hello, " + name;
}
```

**Input JSON file (input.json):**
```json
{"name": "World", "times": 3}
```

**Execution:**
```bash
$ ash run greet.ash --input input.json
Hello, World
```

### Input Processing

Current flow in `parse_input()`:
1. Parse JSON string → `serde_json::Value`
2. Convert to `HashMap<String, Value>` where key = param name
3. Bind to workflow parameters via `execute_with_input()`

The input bindings are injected into the workflow's execution context as initial variable bindings.

## Root Cause

1. `RunArgs.input` help text says "JSON object" not "file path"
2. `parse_input()` function tries to parse as JSON directly (line 195)
3. No clap configuration for trailing positional arguments after `--`

## Files to Modify

- `crates/ash-cli/src/commands/run.rs`:
  - Lines 37-39: Update arg definition (change value_name from "JSON" to "FILE")
  - Lines 191-200: Modify `parse_input()` to try file read first, then JSON parse
  - Add clap config for positional `[input]` after `--`

## Implementation

1. **File path detection**: Try `std::fs::read_to_string()` first
2. **Fallback to inline JSON**: If file read fails, try parsing as JSON
3. **Positional argument support**: Add clap config for trailing args
4. **Help text**: Update to "JSON input file or inline JSON object"

### Algorithm for parse_input():
```rust
fn parse_input(input: &Option<String>) -> Result<HashMap<String, Value>> {
    match input {
        Some(input_str) => {
            // Try as file path first
            if let Ok(content) = std::fs::read_to_string(input_str) {
                return parse_json(&content);
            }
            // Fall back to inline JSON
            parse_json(input_str)
        }
        None => Ok(HashMap::new()),
    }
}
```

## Verification

```bash
# File path should work
echo '{"name": "test"}' > data.json
$ ash run workflow.ash --input data.json

# Inline JSON should still work (backward compatibility)
$ ash run workflow.ash --input '{"name":"test"}'

# Positional after -- should work
$ ash run workflow.ash -- '{"name":"test"}'
```

## Completion Checklist

- [ ] `--input` accepts file path per SPEC-005
- [ ] `--input` still accepts inline JSON (backward compatibility)
- [ ] `-- <input>` positional syntax works
- [ ] Help text updated to clarify FILE or JSON
- [ ] Tests verify all three patterns
- [ ] CHANGELOG.md updated

**Estimated Hours:** 4
**Priority:** Critical (SPEC-005 compliance)
