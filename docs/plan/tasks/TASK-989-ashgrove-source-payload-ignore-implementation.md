# TASK-989: Ashgrove source payload ignore implementation

## Status: 🟡 Ready

## Description

Implement the SPEC-074 source-root payload policy in ashgrove. Source-root digesting and isolated build copying must share one membership layer that excludes gitignored/local-state files while preserving fail-closed behavior for nonignored source payload changes.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §6-§10
- [PLAN-124](../PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §6-§7
- Audit handoff: `docs/plan/audits/TASK-988-ashgrove-source-payload-audit-gate.md`

## Dependencies

- ✅ TASK-988: Audit gate completed and froze classification, focused test names, metadata decision, and verification commands.

## Requirements

### Functional Requirements

1. Add a source-input classification helper and live source-root payload membership helper in `crates/ashgrove/src/lib.rs`.
2. Make live source-root digesting and live source-root isolated build copy consume the same membership list.
3. For git source roots, exclude gitignored local state from payload digest and copy.
4. For non-git source roots, exclude the SPEC-074 built-in local-state set.
5. Preserve source-archive release metadata and digest behavior for both non-source-root archives and source-shaped archives carrying `release-source.toml`.
6. Preserve dirty nonignored source rejection without `--allow-dirty-source`, while fencing the legacy `.dirty` sentinel so gitignored `.dirty` in git roots cannot bypass gitignore-aware cleanliness.
7. Preserve fail-closed post-build rejection when a nonignored source payload file changes during build.
8. Improve the post-build mismatch diagnostic to say source payload changed during build.
9. Record `source_payload_digest_policy` and `source_payload_digest` for live source-root installs without overloading `source_archive_digest`.
10. Prove `ashgrove update --from source` uses the same payload policy as source install.
11. Use fail-closed `git ls-files --cached --others --exclude-standard -z` membership for live git source roots; do not use optional git helpers that silently convert membership failures into fallback filesystem walks.

### Property Requirements

No proptest is required. Required invariants:

```text
source_root_digest_inputs == source_root_copy_inputs
ignored_local_state_change_during_build does not change payload_digest
nonignored_payload_change_during_build changes payload_digest and fails before publish
source_archive_attestation_policy remains unchanged
source_update_uses_same_payload_policy_as_install
```

## TDD Steps

### Step 1: Write ignored-local-state source install regression

**File:** `crates/ashgrove/tests/task_989_source_payload_ignore.rs`

Add `task_989_gitignored_agents_state_can_change_during_source_install`. Create a git source fixture with `.gitignore` ignoring `/.agents/` committed before install. Put a fake `cargo` earlier in `PATH`; fake cargo receives the original source root through a test-controlled environment variable, mutates `.agents/status/dashboard.json` in the original source root, records its isolated-copy `$PWD`, verifies/records that `$PWD/.agents/status/dashboard.json` is absent from the isolated copy, creates executable `ash`/`ashgrove` fixture binaries under `$CARGO_TARGET_DIR/debug`, and exits 0. Run source install without `--allow-dirty-source` and assert success.

### Step 2: Write nested-target exclusion regression

Add `task_989_gitignored_nested_target_is_excluded_from_digest_and_copy`. In the same test file, create `crates/ash-bench/target/generated.txt` or equivalent and commit a `.gitignore` rule for that nested target path before install. Verify the file is excluded from the isolated build copy and does not affect install success when changed during fake cargo execution.

### Step 3: Write nonignored mutation regression

Add `task_989_nonignored_payload_mutation_fails_before_publish`. Use fake cargo to mutate a nonignored payload file during build. Assert install fails before publish with a source-payload-changed diagnostic and no installed toolchain directory is published.

### Step 4: Write dirty-source preservation regression

Add `task_989_nonignored_dirty_source_still_rejects_without_override`. Create a nonignored untracked or modified file before install. Assert source install still rejects without `--allow-dirty-source`.

### Step 5: Write update parity regression

Add `task_989_update_from_source_uses_same_payload_policy_as_install`. Exercise `ashgrove update --from source --path <fixture> --to <expected-id>` with ignored local-state churn and prove it uses the same payload policy as source install.

### Step 6: Write source-archive non-regression

Add `task_989_source_archive_digest_policy_does_not_use_source_root_ignores`, or cite and rerun an existing focused source-archive test plus add any missing assertion needed to prove source-archive release metadata remains fail-closed and source-archive digest/record behavior is not replaced by source-root payload metadata.

### Step 7: Implement shared source-root payload membership

**File:** `crates/ashgrove/src/lib.rs`

Replace the shallow `source_digest_skip_path` path with explicit source-input classification and a policy-aware live-source-root payload membership helper. The helper must be used by both live-source-root digest and `copy_source_tree_for_build`; source-archive digesting must remain separate so source-shaped archives carrying `release-source.toml` are not accidentally filtered through source-root ignore policy.

### Step 8: Implement diagnostics/metadata

Patch post-build mismatch diagnostics and add `source_payload_digest_policy` / `source_payload_digest` for live source-root records according to TASK-988. Keep `source_archive_digest` reserved for attested source-archive payloads.

### Step 9: Run focused tests

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
  - git diff --check
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - cargo fmt --all --check
checklist:
  - [ ] Ignored `.agents/` mutation during source install succeeds without dirty override.
  - [ ] Nested `target/` local state is excluded from digest and isolated build copy.
  - [ ] Nonignored source payload mutation still fails before publish.
  - [ ] Dirty nonignored source root still rejects without `--allow-dirty-source`.
  - [ ] Source archive attestation behavior remains fail-closed and archive digest policy does not use source-root ignores.
  - [ ] Digest/copy source-root membership is shared, not duplicated.
  - [ ] `ashgrove update --from source` uses the same source-root payload policy.
  - [ ] Live git source-root membership fails closed on git membership errors.
  - [ ] Legacy `.dirty` sentinel does not force dirty override for gitignored local state in git roots.
```

## Dependencies for Next Task

TASK-990 uses TASK-989 evidence for phase closeout, SPEC status reconciliation, and independent review.

## Notes

Do not add arbitrary user ignore globs in this task. If a CLI flag becomes necessary, record it in SPEC-074 and install identity first.
