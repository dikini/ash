# TASK-323: Remove --capability Flag from CLI and SPEC-005

## Status: 🟡 High - API Cleanup

## Problem

The `ash run --capability <name=uri>` flag remains in the CLI despite being a contract gap:

1. **URI is parsed and discarded** (`_uri` at `crates/ash-cli/src/commands/run.rs:148`)
2. **Unknown capability names silently do nothing** (lines 164-168)
3. **Users can believe a provider was bound when the workflow actually runs without it**

This directly contradicts:
- `docs/plan/tasks/TASK-317-fix-capability-binding.md:7` (intended fix)
- `docs/plan/PLAN-INDEX.md:1371` (Phase 50 closeout claim)
- `docs/spec/SPEC-005-CLI.md:110` (documented contract)

## Resolution

**Remove `--capability` from SPEC-50.** Capabilities will be defined in:
- Ash source files (via `capability` declarations)
- Libraries (imported modules)
- Default engine capabilities (stdio, fs)

The `--capability` CLI flag will be removed entirely rather than fixed.

## Rationale

1. **Simpler mental model**: Capabilities are declared in code, not CLI args
2. **Type safety**: Capability declarations are type-checked at compile time
3. **Consistency**: Aligns with the reduced syntax philosophy of SPEC-024
4. **Avoids confusion**: No silent failures or misleading behavior

## Implementation

### 1. Remove CLI Flag

Modify `crates/ash-cli/src/commands/run.rs`:

```rust
// REMOVE from RunArgs struct:
// #[arg(long, value_name = "NAME=URI")]
// pub capability: Vec<String>,

// REMOVE build_engine() capability handling loop (lines 146-170)
// Keep only default stdio/fs capabilities
```

### 2. Update SPEC-005

Modify `docs/spec/SPEC-005-CLI.md`:

**Remove from Section 5.2:**
```markdown
| `--capability <name=uri>` | Bind capability to provider |
```

**Remove examples:**
```bash
# Run with custom capability binding
ash run workflow.ash --capability db=postgres://localhost/mydb
```

**Add note about capability providers:**
```markdown
**Capability Providers:**

Capabilities are defined in Ash source files using `capability` declarations.
Built-in providers (stdio, filesystem) are enabled by default. Custom providers
must be declared in the workflow or imported from libraries.
```

### 3. Update Help Text

The CLI help will no longer show `--capability` option.

## Files to Modify

- `crates/ash-cli/src/commands/run.rs` - Remove `--capability` argument and handling
- `crates/ash-cli/src/args.rs` (if separate) - Remove capability field from RunArgs
- `docs/spec/SPEC-005-CLI.md` - Remove `--capability` documentation
- `docs/plan/tasks/TASK-317-fix-capability-binding.md` - Mark as superseded by this task

## Verification

```bash
# --capability flag no longer exists
$ ash run --help | grep capability
# (no output)

# Running without --capability still works (defaults only)
$ ash run workflow.ash

# Attempting to use --capability fails with "unexpected argument"
$ ash run workflow.ash --capability http
error: unexpected argument '--capability' found
```

## Completion Checklist

- [ ] `--capability` flag removed from CLI argument parser
- [ ] `build_engine()` no longer processes capability bindings
- [ ] SPEC-005 documentation updated (flag removed)
- [ ] CLI help text no longer shows `--capability`
- [ ] TASK-317 marked as superseded
- [ ] CHANGELOG.md updated with removal note

**Estimated Hours:** 2
**Priority:** High (API cleanup)
**Supersedes:** TASK-317-fix-capability-binding.md

## Related

- Original task: TASK-317-fix-capability-binding.md
- SPEC-005: CLI Specification
- Design philosophy: SPEC-024 reduced surface syntax
