# TASK-988 Ashgrove Source Payload Audit Gate

## Status

Complete for the Phase 129 pre-implementation gate. This audit freezes the source-payload membership strategy, source-root/source-archive boundary, install-record metadata decision, focused TASK-989 verification names, and downstream verification commands before Rust implementation starts.

## Review findings addressed

This audit incorporates the Phase 129 review blockers:

1. TASK-988 must exist as a real audit artifact before TASK-989 starts.
2. TASK-989/TASK-990 verification placeholders must be replaced by exact non-zero commands.
3. Source-root payload policy must not bleed into source-archive digest/attestation semantics.
4. Source-shaped archives can satisfy `is_source_root`; TASK-989 must classify them explicitly.
5. Install and update source paths need parity evidence.
6. Existing `.dirty` sentinel handling can bypass gitignore-aware cleanliness and must be fenced.
7. Git membership failures must fail closed, not fall back through optional git helpers.
8. Fake-cargo tests need concrete observation plumbing for isolated-copy and original-root mutation.

## Live implementation seam map

| Consumer / seam | Live file/function | Current behavior | TASK-989 policy owner |
| --- | --- | --- | --- |
| CLI source flags | `crates/ashgrove/src/lib.rs` `InstallArgs` / `UpdateArgs` | `--allow-dirty-source` and `--allow-unidentified-source`; no source payload policy CLI. | Keep no broad ignore CLI. |
| Early legacy dirty sentinel | `install_from_source` | Rejects `source/.dirty` before classifying source roots. | Fence so git source roots use git cleanliness/payload policy; keep intentional non-git/archive behavior only if explicitly justified by code path. |
| Source kind classification | `is_source_root` | `Cargo.toml` plus `std/src` makes both live source roots and extracted source archives look source-shaped. | Introduce explicit source-input classification before choosing digest policy. |
| Git identity/dirty metadata | `SourceRootMetadata::inspect` | Uses git rev/status; ignored files do not make `git status --porcelain` dirty. | Preserve dirty rejection for nonignored tracked/untracked changes; consider path-scoped status if source can be nested in a larger worktree. |
| Pre-build source-root digest | `install_from_source_root` | Calls `source_tree_digest(source)`. | Replace with source-root payload digest for live source roots only. |
| Isolated build copy | `copy_source_tree_for_build` | Walks source and skips only through `source_digest_skip_path`. | Consume the exact same live-source-root membership list as digest. |
| Post-build mutation check | `stage_source_root_toolchain` | Calls `source_tree_digest(source)` again; reports generic source-root dirtiness. | Compare source-root payload digests and report source-payload mutation. |
| Shared shallow skip | `source_digest_skip_path` | Skips only first component `.git` or `target`; includes `.agents/`, nested `target/`, and gitignored state. | Replace/fence behind explicit source-root/source-archive policy helpers. |
| Source archive metadata | `SourceArchiveReleaseMetadata::*` | Requires/validates `release-source.toml` for archive-like paths depending on call site. | Preserve fail-closed attestation and keep archive digest policy separate. |
| Source install record | `write_source_install_record` / `SourceInstallRecordInput` | Writes `source_archive_digest` when input provides it; no source-root payload digest policy field. | Add live source-root payload fields and do not overload `source_archive_digest`. |

## Source input classification frozen for TASK-989

TASK-989 must classify source inputs into at least these policy buckets before digesting/copying:

1. **Live git source root**
   - A source-shaped directory inside a git worktree with a usable git identity/membership query.
   - Payload membership command: `git ls-files --cached --others --exclude-standard -z` from the source root.
   - Failure behavior: once a root is treated as git-like, git membership failures are fatal; do not fall back to broad filesystem walking through `git_output_optional`.
   - Payload files are the null-delimited relative paths returned by git membership, filtered to files that still exist when digested/copied. Nonignored untracked files remain members and make the root dirty through git status.

2. **Live non-git source root**
   - A source-shaped directory without git identity and without source-archive attestation requiring archive semantics.
   - Payload membership uses a conservative built-in local-state exclusion set: top-level `.git`, nested `target/`, `.agents/`, `tools/agent-pipeline/.agents/`, `.worktrees/`, `.codex/`, and any explicitly added local-output paths documented in TASK-989.
   - The built-in policy must avoid broad substring matching.

3. **Source-shaped archive / release-source source**
   - A source-shaped directory that carries `release-source.toml` and is being installed as an attested release source, including extracted archives that also satisfy `is_source_root`.
   - It keeps source-archive attestation and digest semantics. Do not apply gitignore/local-state source-root filtering unless SPEC-074 is amended with archive-specific attestation semantics.

4. **Non-source-root source archive**
   - A source path that does not satisfy `is_source_root` and goes through the existing archive branch.
   - Preserve `SourceArchiveReleaseMetadata::read_from_source_archive` behavior and archive digest policy.

## Selected membership strategy

TASK-989 must use git CLI membership for live git source roots:

```bash
git ls-files --cached --others --exclude-standard -z
```

Parsing rules:

- Treat stdout as NUL-delimited relative paths.
- Reject absolute paths and paths containing parent traversal components.
- Sort the final relative file list deterministically before digesting.
- Use the same sorted list for digest and isolated copy.
- If the command exits nonzero for a git-like root, return an error that mentions source payload membership and the git stderr text.

The existing optional helper `git_output_optional` is not sufficient for membership because it converts nonzero git exits into `Ok(None)`.

## Digest/copy invariant

TASK-989 must introduce one membership producer for live source-root payload files, for example:

```rust
fn source_root_payload_files(source: &Path, classification: SourceInputKind) -> Result<Vec<PathBuf>>;
```

Both source-root digest and isolated build copy must consume that exact list. Separate filesystem walks with duplicated predicates are not sufficient evidence for A74-6.

Source-archive digesting must have a separate function/policy, for example:

```rust
fn source_archive_digest(source: &Path) -> Result<String>;
fn source_root_payload_digest(source: &Path, classification: SourceInputKind) -> Result<String>;
```

Names may differ; the required property is separate source-root and source-archive policy selection.

## Install-record metadata decision

TASK-989 must add source-root payload metadata for live source-root installs:

```toml
source_payload_digest_policy = "source-root-v2-gitignore-local-state"
source_payload_digest = "sha256:..."
```

`source_archive_digest` remains reserved for the digest of an attested source archive payload and must not be used for the gitignore-filtered live source-root payload digest. If an extracted source archive is classified as source-shaped archive, its archive attestation path may continue to populate `source_archive_digest`.

## Focused TASK-989 tests frozen

All tests should live in:

```text
crates/ashgrove/tests/task_989_source_payload_ignore.rs
```

Required test names:

1. `task_989_gitignored_agents_state_can_change_during_source_install`
   - Create a git source fixture.
   - Add and commit `.gitignore` containing `/.agents/` before install.
   - Put fake `cargo` earlier in `PATH`.
   - Fake cargo receives the original source root through a test-controlled environment variable, mutates `.agents/status/dashboard.json` in the original root, records its `$PWD` to an observation file, asserts/records that `$PWD/.agents/status/dashboard.json` is absent from the isolated copy, writes executable `ash` and `ashgrove` under `$CARGO_TARGET_DIR/debug`, and exits 0.
   - `ashgrove install --from source --path <fixture>` succeeds without `--allow-dirty-source`.

2. `task_989_gitignored_nested_target_is_excluded_from_digest_and_copy`
   - Create and commit a `.gitignore` rule for `crates/ash-bench/target/` or equivalent fixture path.
   - Fake cargo mutates that ignored nested target path in the original root.
   - Fake cargo records that the nested target file is absent from the isolated copy.
   - Install succeeds and ignored nested target churn does not affect payload digest.

3. `task_989_nonignored_payload_mutation_fails_before_publish`
   - Fake cargo mutates a nonignored source payload file in the original root during build.
   - Install fails with a source-payload-changed diagnostic.
   - No final toolchain directory is published.

4. `task_989_nonignored_dirty_source_still_rejects_without_override`
   - Create a nonignored untracked or modified file before install.
   - Install rejects without `--allow-dirty-source`.

5. `task_989_update_from_source_uses_same_payload_policy_as_install`
   - Exercise `ashgrove update --from source --path <fixture> --to <expected-id>` with ignored local-state churn.
   - Either reuse the fake cargo harness or run a narrowed command that proves update routes through the same source-root payload membership.

6. `task_989_source_archive_digest_policy_does_not_use_source_root_ignores`
   - Use an attested source-shaped archive fixture or equivalent source archive fixture.
   - Prove source-archive release metadata remains fail-closed and that source-archive digest/record behavior is not replaced by source-root payload metadata.

## Exact verification commands frozen

TASK-989 focused commands:

```bash
git diff --check
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

TASK-990 closeout commands:

```bash
git diff --check
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --all-targets -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
cargo fmt --all --check
python3 -c "from pathlib import Path; audit=Path('docs/plan/audits/TASK-990-ashgrove-source-payload-local-state-closeout.md'); assert audit.exists(), audit; text=audit.read_text(); required=['A74-1','A74-8','independent review','cargo']; missing=[s for s in required if s not in text]; assert not missing, missing; print('TASK-990 closeout artifact verified')"
```

The `task_989_source_payload_ignore` target does not exist before TASK-989 writes it; the command is intentionally frozen now so TASK-989 cannot proceed with placeholder verification.

## Existing non-regression tests to preserve

Existing executable source-archive coverage includes:

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_977_source_archive_release_metadata -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture
```

The broader `source_archive` filter currently covers tests across source-archive metadata, trust/signing, and release/deployment acceptance. TASK-989 may add a more focused source-archive digest-policy test, but it must still keep the filtered non-regression command green.

## TASK-988 verification

This audit is considered complete only if:

1. this file exists;
2. it contains the source-root/source-archive classification above;
3. TASK-989 and TASK-990 verification blocks contain no `false # TASK-988` placeholders;
4. TASK-989 names the six focused tests above;
5. SPEC-074 A74-6 ownership includes TASK-989;
6. TASK-989 dependencies/status show the audit gate complete and implementation ready.
