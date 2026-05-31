# PLAN-124: Ashgrove Source Payload Local-State Ignore Implementation Plan

> **For Hermes:** Use subagent-driven-development for Rust implementation tasks. TASK-987 is docs/planning only; TASK-988 is the mandatory audit gate before implementation; TASK-989 owns code/tests; TASK-990 owns closeout and independent review.

**Goal:** Fix `ashgrove install --from source` so local ignored state in a developer checkout cannot falsely trip the post-build source-payload mutation check.

**Architecture:** Introduce one policy-aware source-payload membership layer and make both source-root digesting and isolated build-copying consume it. Git source roots use git-compatible ignore membership; non-git source roots use a conservative built-in local-state ignore set; source archives keep SPEC-073 attestation behavior separate.

**Tech Stack:** Rust 2024, ashgrove crate, git CLI or equivalent ignore-compatible walker, cargo integration tests with fake cargo fixtures, SPEC-073/SPEC-074 docs.

---

## 1. Background

The observed failure:

```text
source cargo build dirtied source root /home/dikini/Projects/ash; aborting before publish
```

was reproduced even with a clean `git status --porcelain`. The changing file was ignored local agent state:

```text
.agents/status/dashboard.json
```

The source-install code currently skips only top-level `.git/` and top-level `target/` in `source_digest_skip_path`. That means ignored local runtime state and nested crate `target/` outputs can participate in the source payload digest and isolated build copy. SPEC-074 defines the corrected boundary.

## 2. Scope

### In scope

- Source-root payload membership for source install and source update.
- Shared digest/copy file selection.
- Gitignore-aware exclusion of local state in git source roots.
- Conservative built-in local-state exclusion for non-git source roots.
- Deterministic regression tests for ignored state mutation during build.
- Diagnostics and metadata updates needed for implementation-grade provenance.

### Out of scope

- General arbitrary user ignore globs.
- Hosted release index behavior.
- Source-archive integrity relaxation.
- Reworking tarball install semantics.

## 3. Current implementation seams

| Seam | File/function | Required change |
| --- | --- | --- |
| Source-root identity | `crates/ashgrove/src/lib.rs::SourceRootMetadata::inspect` | Preserve git dirty/unidentified rejection. |
| Pre-build payload digest | `install_from_source_root` | Use policy-aware source-root payload digest. |
| Isolated build copy | `copy_source_tree_for_build` | Use the exact same source-root payload membership as digest. |
| Post-build mutation check | `stage_source_root_toolchain` | Compare policy-aware payload digests and report source-payload mutation. |
| Shallow skip predicate | `source_digest_skip_path` | Replace or fence with policy-aware walker. |
| Source archive path | `install_from_source` archive branch | Preserve release-source metadata and source-archive digest semantics. |

## 4. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-987](tasks/TASK-987-ashgrove-source-payload-local-state-packet.md) | Create SPEC-074/PLAN-124/TASK packet and register Phase 129 | 3 | ✅ Complete |
| [TASK-988](tasks/TASK-988-ashgrove-source-payload-audit-gate.md) | Audit source payload membership, freeze implementation choice, and replace focused verification placeholders | 5 | 🟡 Ready |
| [TASK-989](tasks/TASK-989-ashgrove-source-payload-ignore-implementation.md) | Implement source-root payload walker, digest/copy sharing, metadata/diagnostics, and focused regressions | 10 | 📝 Planned |
| [TASK-990](tasks/TASK-990-ashgrove-source-payload-local-state-closeout.md) | Run composed acceptance, independent review, status reconciliation, and broad gates | 5 | 📝 Planned |

Total estimate: 23 hours.

## 5. Decision gates

- D1: SPEC-073 stays Implemented MVP; SPEC-074 is a targeted amendment for a post-MVP correctness bug.
- D2: Source-root payload membership and source-archive integrity are separate policies.
- D3: Git source-root payload membership must align with git ignore semantics; git-clean ignored state must not trigger `--allow-dirty-source`.
- D4: Payload digest and isolated build copy must share one file-selection implementation.
- D5: Nonignored source payload mutation during build remains fail-closed before publish.
- D6: No broad arbitrary `--ignore`/`--exclude` CLI is introduced in the first fix.
- D7: TASK-988 must replace any placeholder focused verification with exact non-zero commands before TASK-989 starts.

## 6. Implementation approach

### 6.1 Source-payload walker

Introduce a single source-payload membership abstraction in `crates/ashgrove/src/lib.rs`. The implementation may be private for the first slice.

Suggested shapes:

```rust
enum SourcePayloadKind {
    SourceRoot,
    SourceArchive,
}

struct SourcePayloadPolicy {
    kind: SourcePayloadKind,
    git_root: Option<PathBuf>,
}
```

or a simpler private helper:

```rust
fn source_root_payload_files(source: &Path) -> Result<Vec<PathBuf>>;
fn source_archive_payload_files(source: &Path) -> Result<Vec<PathBuf>>;
```

The important invariant is not the type shape; it is shared use by digest and copy.

### 6.2 Git membership strategy

TASK-988 must choose one git-compatible strategy for git source roots:

1. `git ls-files --cached --others --exclude-standard -z`; or
2. a Rust ignore-compatible walker dependency that matches git's standard exclude behavior for the tested source roots.

A narrow built-in policy is acceptable only for non-git source roots. Git-compatible membership is not optional for git roots unless SPEC-074 is explicitly weakened first.

Preferred first implementation: use git CLI membership for git source roots because ashgrove already shells out to git for source identity and dirty checks.

### 6.3 Deterministic test strategy

Avoid slow real builds where possible. Use a fake `cargo` executable earlier in `PATH` that:

1. records that ashgrove invoked cargo from the isolated source-build copy;
2. mutates the original source root ignored local-state file, e.g. `.agents/status/dashboard.json`;
3. writes executable fixture `ash` and `ashgrove` binaries to `$CARGO_TARGET_DIR/debug/`; and
4. exits 0.

Then assert source install succeeds without `--allow-dirty-source` and that installed/copy payload does not include ignored state.

Add a second fake-cargo test that mutates a nonignored source payload file and assert publish fails before toolchain publication.

## 7. Verification strategy

Focused commands are intentionally finalized by TASK-988. Expected broad closeout gates:

```bash
git diff --check
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --all-targets -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

If the offline cache is incomplete, closeout must record that honestly and run the equivalent non-offline commands or the repo's standard Rust gate scripts.

## 8. Documentation closeout

TASK-990 must update:

- `docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md` status/evidence if implemented.
- `docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md` amendment note if behavior changes are completed.
- `docs/spec/README.md` status row for SPEC-074.
- `docs/plan/PLAN-INDEX.md` Phase 129 task statuses.
- `CHANGELOG.md` under `[Unreleased]`.

## 9. Completion criteria

The phase is complete only when:

- ignored local state mutation during source-root install is covered by a focused regression;
- nested target/local-state exclusion is covered by a focused regression;
- nonignored source payload mutation still fails before publish;
- source archive trust/attestation behavior has a non-regression test or cited existing focused test;
- ashgrove clippy/tests/fmt pass; and
- an independent review reports no blocking spec or code-quality findings.
