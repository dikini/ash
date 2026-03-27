# TASK-324: Remove --input Flag from CLI and SPEC-005

## Status: 🟡 High - API Cleanup

## Problem

The Phase 50 `--input` remediation did not fully land. The current behavior is inconsistent:

1. **Current code** (`crates/ash-cli/src/commands/run.rs:191`) still only attempts `serde_json::from_str` on the raw string
2. **No file-path fail-fast** - doesn't detect `.json` file paths
3. **No positional `-- '<json>'` handling** - not implemented
4. **SPEC-005 Section 5.2** still documents file input and positional input
5. **TASK-316** says the new contract should be inline JSON only with file-path error

The current behavior is **neither** the old documented contract **nor** the planned new one.

## Resolution

**Remove `--input` and inline input from SPEC-50.** Input handling will be redesigned in a future phase with a clearer contract.

## Rationale

1. **Incomplete implementation**: Neither the old nor new contract is properly implemented
2. **Confusing UX**: Current behavior is inconsistent with documentation
3. **Clean slate**: Remove now, design properly later
4. **No production impact**: Workflows can use `observe` or hardcoded values for now

## Implementation

### 1. Remove CLI Flag

Modify `crates/ash-cli/src/commands/run.rs`:

```rust
// REMOVE from RunArgs struct:
// #[arg(short, long, value_name = "JSON")]
// pub input: Option<String>,

// REMOVE parse_input() function (lines 191-216)
// REMOVE input handling from execute_workflow() calls
```

### 2. Remove Input Handling from Execution

Remove input binding from workflow execution path:

```rust
// REMOVE: execute_with_trace_and_input() or modify to not accept input_bindings
// REMOVE: Input parameter from execute_workflow() signature
```

### 3. Update SPEC-005

Modify `docs/spec/SPEC-005-CLI.md`:

**Remove from Section 5.2:**
```markdown
| `--input <json>` | Input parameters as JSON |
```

**Update usage:**
```bash
ash run [options] <file.ash>
```

**Remove examples with --input:**
```bash
# REMOVE: ash run workflow.ash --input '{"name": "test"}'
```

**Add note about input:**
```markdown
**Input Parameters:**

Workflow input parameters are not yet supported via CLI. Workflows should use
`observe` statements or hardcoded values. CLI input binding is planned for a
future release.
```

### 4. Update TASK-316

Mark TASK-316 as superseded by this task (removal instead of fix).

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Remove `--input` argument and handling
- `crates/ash-cli/src/args.rs` (if separate) - Remove input field from RunArgs
- `docs/spec/SPEC-005-CLI.md` - Remove `--input` documentation
- `docs/plan/tasks/TASK-316-fix-input-file-path.md` - Mark as superseded
- Any tests that use `--input` flag - Update or remove

## Verification

```bash
# --input flag no longer exists
$ ash run --help | grep input
# (no output)

# Running without --input still works
$ ash run workflow.ash

# Attempting to use --input fails with "unexpected argument"
$ ash run workflow.ash --input '{"name":"test"}'
error: unexpected argument '--input' found
```

## Completion Checklist

- [ ] `--input` flag removed from CLI argument parser
- [ ] `parse_input()` function removed
- [ ] Input handling removed from workflow execution
- [ ] SPEC-005 documentation updated (flag removed)
- [ ] CLI help text no longer shows `--input`
- [ ] TASK-316 marked as superseded
- [ ] Tests using `--input` updated or removed
- [ ] CHANGELOG.md updated with removal note

**Estimated Hours:** 2
**Priority:** High (API cleanup)
**Supersedes:** TASK-316-fix-input-file-path.md

## Related

- Original task: TASK-316-fix-input-file-path.md
- SPEC-005: CLI Specification
- Deferred: Proper CLI input design for future phase
