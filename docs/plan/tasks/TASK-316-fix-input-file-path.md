# TASK-316: Remove --input and -- Support from SPEC-005 (Design Decision)

## Status: ✅ Complete - Design Decision

## Decision

**Simplify the CLI by removing `--input` file path and `--` positional support rather than implementing them.**

The `--input` flag currently accepts inline JSON only. Rather than extending it to support file paths and positional arguments, we will:

1. **Keep `--input` as inline JSON only** (current behavior becomes the documented behavior)
2. **Remove `--input <file>` from SPEC-005** (file path support not implemented)
3. **Remove `-- <input>` positional syntax from SPEC-005** (positional input not implemented)

## Rationale

- **Simpler CLI surface**: One way to pass input (inline JSON) rather than three ways
- **Shell redirection adequate**: Users can use shell features for file input:
  ```bash
  # Instead of: ash run workflow.ash --input data.json
  # Use:        ash run workflow.ash --input "$(cat data.json)"
  ```
- **Avoids ambiguity**: No confusion about whether argument is file path or JSON
- **Consistent with design philosophy**: Prefer compositional shell patterns over CLI complexity

## SPEC-005 Changes Required

**Remove from SPEC-005 Section 5.2 (`ash run`):**

```diff
- | `--input <file>` | JSON input file |
+ | `--input <json>` | Inline JSON input object |
```

**Remove example:**
```diff
- # Run with JSON input
- ash run workflow.ash --input data.json
-
- # Run with inline input
- ash run workflow.ash -- '{"name": "test"}'
+ # Run with inline JSON input
+ ash run workflow.ash --input '{"name": "test"}'
```

## Current (Now Documented) Behavior

```bash
# Inline JSON (supported)
$ ash run workflow.ash --input '{"name": "test", "count": 5}'

# File path (not supported - use shell substitution)
$ ash run workflow.ash --input "$(cat data.json)"
```

## Input JSON Format

The `--input` JSON is an **object** where keys are workflow parameter names:

```json
{
  "param1": "string value",
  "param2": 42,
  "param3": true
}
```

### Example

**Workflow (greet.ash):**
```ash
workflow greet(name: String) {
    ret "Hello, " + name;
}
```

**Execution:**
```bash
$ ash run greet.ash --input '{"name": "World"}'
Hello, World
```

## Files to Modify

- `docs/spec/SPEC-005-CLI.md` - Update `--input` documentation
- `crates/ash-cli/src/commands/run.rs` - Update help text (line 37-39)

## Implementation

**No code changes required** - current behavior is correct. Only documentation updates:

1. Update SPEC-005 to reflect `--input` accepts inline JSON only
2. Update CLI help text from "JSON object" to "Inline JSON input object"
3. Remove file path and positional syntax from examples

## Verification

```bash
# Inline JSON works
$ ash run workflow.ash --input '{"name":"test"}'

# Help text updated
$ ash run --help | grep -A1 "input"
--input <json>  Inline JSON input object
```

## Completion Checklist

- [x] Design decision made: Keep inline JSON only
- [ ] SPEC-005 updated to remove file path and `--` syntax
- [ ] CLI help text updated
- [ ] CHANGELOG.md updated with design decision

**Estimated Hours:** 1 (documentation only)
**Priority:** Complete - No implementation required

## Related Tasks

- Supersedes TASK-308 (file path implementation - no longer needed)
- Aligns with simplification philosophy: prefer shell composition over CLI complexity
