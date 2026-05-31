# TASK-989: Ashgrove source payload ignore implementation

## Status: 📝 Planned

## Description

Implement the SPEC-074 source-root payload policy in ashgrove. Source-root digesting and isolated build copying must share one membership layer that excludes gitignored/local-state files while preserving fail-closed behavior for nonignored source payload changes.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §6-§10
- [PLAN-124](../PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §6-§7
- Audit handoff: `docs/plan/audits/TASK-988-ashgrove-source-payload-audit-gate.md`

## Dependencies

- 🟡 TASK-988: Audit gate must complete and freeze focused test names before implementation starts.

## Requirements

### Functional Requirements

1. Add a source-root payload membership helper in `crates/ashgrove/src/lib.rs`.
2. Make source-root digesting and source-root isolated build copy consume the same membership list.
3. For git source roots, exclude gitignored local state from payload digest and copy.
4. For non-git source roots, exclude the SPEC-074 built-in local-state set.
5. Preserve source-archive release metadata and digest behavior unless TASK-988 explicitly amends the strategy.
6. Preserve dirty nonignored source rejection without `--allow-dirty-source`.
7. Preserve fail-closed post-build rejection when a nonignored source payload file changes during build.
8. Improve the post-build mismatch diagnostic to say source payload changed during build.
9. Record payload digest policy in install metadata if TASK-988 selected a metadata change.

### Property Requirements

No proptest is required. Required invariants:

```text
source_root_digest_inputs == source_root_copy_inputs
ignored_local_state_change_during_build does not change payload_digest
nonignored_payload_change_during_build changes payload_digest and fails before publish
source_archive_attestation_policy remains unchanged
```

## TDD Steps

### Step 1: Write ignored-local-state source install regression

**File:** `crates/ashgrove/tests/task_989_source_payload_ignore.rs`

Add a test that creates a git source fixture with `.gitignore` ignoring `/.agents/`. Put a fake `cargo` earlier in `PATH`; fake cargo mutates `.agents/status/dashboard.json` in the original source root while creating executable `ash`/`ashgrove` fixture binaries under `$CARGO_TARGET_DIR/debug`. Run source install without `--allow-dirty-source` and assert success.

### Step 2: Write nested-target exclusion regression

In the same test file, create `crates/ash-bench/target/generated.txt` or equivalent. Verify it is excluded from the isolated build copy and does not affect install success when changed during fake cargo execution.

### Step 3: Write nonignored mutation regression

Use fake cargo to mutate a nonignored payload file during build. Assert install fails before publish with a source-payload-changed diagnostic and no installed toolchain directory is published.

### Step 4: Write dirty-source preservation regression

Create a nonignored untracked or modified file before install. Assert source install still rejects without `--allow-dirty-source`.

### Step 5: Write source-archive non-regression

Either add a focused test or cite and rerun existing source-archive tests proving missing/invalid `release-source.toml` remains fail-closed for source archives.

### Step 6: Implement shared source-root payload membership

**File:** `crates/ashgrove/src/lib.rs`

Replace the shallow `source_digest_skip_path` path with a policy-aware source-root payload membership helper. The helper must be used by both `source_tree_digest` and `copy_source_tree_for_build`, or those functions must be split into source-root/source-archive variants that share the source-root membership function.

### Step 7: Implement diagnostics/metadata

Patch post-build mismatch diagnostics and install-record metadata according to TASK-988 decisions.

### Step 8: Run focused tests

Run the exact focused command frozen by TASK-988. Expected: all TASK-989 tests pass and source-archive non-regression passes.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - false # TASK-988 must replace with exact focused TASK-989 source-payload test command before implementation starts.
  - false # TASK-988 must replace with exact source-archive non-regression command before implementation starts.
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - cargo fmt --all --check
checklist:
  - [ ] Ignored `.agents/` mutation during source install succeeds without dirty override.
  - [ ] Nested `target/` local state is excluded from digest and isolated build copy.
  - [ ] Nonignored source payload mutation still fails before publish.
  - [ ] Dirty nonignored source root still rejects without `--allow-dirty-source`.
  - [ ] Source archive attestation behavior remains fail-closed.
  - [ ] Digest/copy source-root membership is shared, not duplicated.
```

## Dependencies for Next Task

TASK-990 uses TASK-989 evidence for phase closeout, SPEC status reconciliation, and independent review.

## Notes

Do not add arbitrary user ignore globs in this task. If a CLI flag becomes necessary, record it in SPEC-074 and install identity first.
