# TASK-316: Fix --input to Fail Fast on File Path Attempts (Options B & C)

## Status: ⚪ Superseded by TASK-324

**Note:** This task has been superseded by TASK-324. The `--input` flag has been
removed entirely from the CLI rather than being fixed. Input handling will be
redesigned in a future phase with a clearer contract.

## Original Status: 🔴 Critical - Phase 50 Remediation

## Decision

**Implement Options B and C for --input handling:**

- **Option B (Fail Fast)**: Return clear error when input looks like a file path
- **Option C (Document Limitation)**: Document that only inline JSON is supported, with file paths and `--` syntax planned for future

## Rationale

- **Fail fast**: Users get immediate feedback instead of confusing "Invalid JSON" error
- **Clear contract**: Users understand current limitations vs future features
- **SPEC-005 honesty**: Document what works now, note future enhancements
- **Backward compatible**: Inline JSON continues to work

## SPEC-005 Changes Required

**Update SPEC-005 Section 5.2 (`ash run`):**

```
| `--input <json>` | Inline JSON input object (file paths: see Future Enhancements) |
```

**Add Future Enhancements section:**
```markdown
### Future Enhancements (Post-0.5.0)

- **File path input**: `--input data.json` will read from file
- **Positional input**: `ash run <wf> -- '<json>'` for shell-free JSON passing
- These features are planned but not yet implemented. Use shell substitution:
  `ash run workflow.ash --input "$(cat data.json)"`
```

## Implementation

### 1. Detect File Path Attempts (Option B)

Modify `parse_input()` to fail fast when input looks like a file path:

```rust
fn parse_input(input: &Option<String>) -> Result<HashMap<String, Value>> {
    match input {
        Some(input_str) => {
            // Fail fast if input looks like a file path
            if input_str.ends_with(".json") || input_str.contains('/') {
                return Err(anyhow::anyhow!(
                    "File path input not yet supported. Use inline JSON or shell substitution: \
                     --input \"$(cat {})\"",
                    input_str
                ));
            }
            // Parse as inline JSON
            let json_value: serde_json::Value =
                serde_json::from_str(input_str).context("Invalid JSON input")?;
            json_to_hashmap(json_value)
        }
        None => Ok(HashMap::new()),
    }
}
```

### 2. Update CLI Help Text (Option C)

```rust
/// Input parameters as inline JSON object (file paths not yet supported, use `$(cat file.json)`)
#[arg(short, long, value_name = "JSON")]
pub input: Option<String>,
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

$ ash run greet.ash --input data.json
Error: File path input not yet supported. Use inline JSON or shell substitution: --input "$(cat data.json)"
```

## Files to Modify

- `crates/ash-cli/src/commands/run.rs`:
  - Lines 37-39: Update help text
  - Lines 191-200: Add file path detection with clear error
- `docs/spec/SPEC-005-CLI.md`: Add Future Enhancements section

## Verification

```bash
# Inline JSON works
$ ash run workflow.ash --input '{"name":"test"}'

# File path fails with clear error
$ ash run workflow.ash --input data.json
Error: File path input not yet supported. Use inline JSON or shell substitution: --input "$(cat data.json)"

# Help text updated
$ ash run --help | grep -A2 "input"
--input <JSON>  Input parameters as inline JSON object (file paths not yet supported)
```

## Completion Checklist

- [ ] File path detection with clear error message
- [ ] Help text updated with limitation note
- [ ] SPEC-005 updated with Future Enhancements section
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2
**Priority:** Critical (API clarity)

## Related Tasks

- Supersedes TASK-308 (file path implementation - now documented as future enhancement)
- Aligns with fail-fast philosophy
