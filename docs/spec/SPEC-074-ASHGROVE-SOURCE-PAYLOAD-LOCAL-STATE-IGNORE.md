# SPEC-074: Ashgrove Source Payload and Local-State Ignore Policy

**Status:** Draft
**Date:** 2026-05-31
**Amends:** [SPEC-073](SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
**Plan:** [PLAN-124](../plan/PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md)
**Implementation Tasks:** [TASK-987](../plan/tasks/TASK-987-ashgrove-source-payload-local-state-packet.md) through [TASK-990](../plan/tasks/TASK-990-ashgrove-source-payload-local-state-closeout.md)

## 1. Summary

SPEC-073 requires `ashgrove install --from source` to build a source root in an isolated cache/staging area and fail closed if the source payload changes during the build. The current MVP implementation computes that check over too much of a developer checkout: it skips only top-level `.git/` and top-level `target/`, so ignored local state such as `.agents/status/dashboard.json` or nested crate `target/` directories can change during the isolated build and cause a false failure.

This spec defines the missing source-payload boundary. Ashgrove source-root installs must distinguish reproducible source payload from local state. Local state must not affect source-root payload identity, must not be copied into the isolated build root, and must not require `--allow-dirty-source`. Nonignored source payload changes remain fail-closed.

## 2. Motivation

A clean git checkout can currently fail source install with:

```text
source cargo build dirtied source root /path/to/ash; aborting before publish
```

The failure can happen even when `git status --porcelain` is empty. In the observed checkout, `.agents/status/dashboard.json` changed during the source build. Git correctly treats this file as ignored local state, but ashgrove's payload digest included it. That makes the source-install reproducibility check stricter than git cleanliness and couples installer success to unrelated local daemons or agent dashboards.

The alpha installer needs both properties:

1. fail closed when actual source payload changes while building; and
2. ignore known local state that is intentionally outside the source payload.

## 3. Normative terms

- **Source root:** A directory accepted by ashgrove as a buildable source checkout, currently detected by `Cargo.toml` plus `std/src/`.
- **Source archive:** A prepared source-shaped directory governed by release-source attestation semantics. It can be non-source-root-shaped or source-shaped after extraction; a source-shaped archive may satisfy source-root detection but must still keep source-archive digest/attestation policy unless classified as a live source root by the TASK-988 audit rules.
- **Source payload:** The files ashgrove considers part of the reproducible source-root build/install identity.
- **Local state:** Files under a source root that are intentionally outside the source payload, including VCS metadata, build outputs, local runtime state, agent dashboards, caches, worktrees, and gitignored artifacts.
- **Payload digest:** The deterministic digest over the source payload, used to detect source-payload changes during source-root install.
- **Payload copy:** The file set copied into ashgrove's isolated build root before invoking cargo.
- **Ignore policy:** The rule used to decide source-payload membership.

## 4. Scope

### 4.1 In scope

1. Source-root payload membership for `ashgrove install --from source --path PATH`.
2. Source-root payload membership for `ashgrove update --from source --path PATH`.
3. Shared source-root file selection for both payload digest and isolated build copy.
4. Local-state exclusion for git source roots and conservative non-git source roots.
5. Regression tests for ignored local state changing during source install.
6. Diagnostics that distinguish source-payload mutation from ignored local-state churn.
7. Install-record provenance for the digest policy when new metadata fields are added.

### 4.2 Out of scope

1. Hosted release-channel lookup or signed release-index resolution.
2. A general user-supplied arbitrary ignore-glob CLI for source install.
3. Weakening source-archive attestation requirements from SPEC-073.
4. Treating dirty nonignored source files as reproducible without `--allow-dirty-source`.
5. Ignoring package-manager lockfiles or source files merely because they are locally inconvenient.

## 5. Current implementation facts

The live implementation has these relevant seams:

| Concern | Current location | Current behavior |
| --- | --- | --- |
| CLI source flags | `crates/ashgrove/src/lib.rs` `InstallArgs` / `UpdateArgs` | Supports `--allow-dirty-source` and `--allow-unidentified-source`; no payload-ignore policy knob. |
| Source-root metadata | `SourceRootMetadata::inspect` | Uses `git status --porcelain`, so ignored files do not mark the root dirty. |
| Pre-build digest | `install_from_source_root` | Calls `source_tree_digest(source)` before build. |
| Isolated build | `build_source_binaries` | Copies source to XDG cache build root and sets `CARGO_TARGET_DIR` outside the original root. |
| Post-build digest | `stage_source_root_toolchain` | Calls `source_tree_digest(source)` again and aborts on mismatch. |
| Digest/copy skip | `source_digest_skip_path` | Skips only first path component `.git` or `target`; includes `.agents/`, nested `target/`, and other ignored state. |

This spec changes the live source-root digest/copy membership contract, not the immutable toolchain layout or tarball install contract. TASK-989 must explicitly classify live git source roots, live non-git source roots, source-shaped archives carrying `release-source.toml`, and non-source-root source archives before choosing a digest policy.

## 6. Source-root payload membership

### 6.1 General rule

For source roots, ashgrove MUST compute payload digest and payload copy from the same source-payload file set.

The following invariant is mandatory:

```text
payload_digest_inputs(source_root) == payload_copy_inputs(source_root)
```

modulo the fact that digest reads bytes and copy creates files.

If a file is excluded from the payload digest, ashgrove MUST NOT copy it into the isolated build root. If a file is copied into the isolated build root, it MUST participate in the payload digest unless a later task explicitly classifies it as generated-at-build-time metadata.

### 6.2 Git source roots

For a source root inside a git work tree, ashgrove MUST use git-compatible ignore semantics for local state:

1. Files ignored by git's standard exclude rules MUST NOT participate in source-root payload digest.
2. Files ignored by git's standard exclude rules MUST NOT be copied into the isolated build root.
3. Nonignored untracked files remain source-payload candidates and therefore make `git status --porcelain` dirty.
4. Modified/deleted tracked files remain dirty and are rejected unless `--allow-dirty-source` is explicit.
5. Ignored local state MUST NOT require `--allow-dirty-source`.
6. Ignored local state MUST NOT make the install non-reproducible.

Implementation may obtain this membership with `git ls-files --cached --others --exclude-standard` or an equivalent `.gitignore`-compatible walker. If the chosen implementation shells out to git, git failures in git-like source roots remain fail-closed rather than silently falling back to broad filesystem walking. The first implementation uses `git ls-files --cached --others --exclude-standard -z` for live git source roots; helper paths that convert nonzero git exits into `None` are not sufficient for membership selection.

### 6.3 Non-git source roots

For source roots that are not git work trees, ashgrove MUST apply a conservative built-in local-state ignore set. The first implementation MUST exclude at least:

```text
.git/
.git
*/target/
.agents/
tools/agent-pipeline/.agents/
.worktrees/
.codex/
```

The built-in policy may grow, but it MUST stay narrowly focused on local state and build/runtime outputs. It MUST NOT exclude ordinary source directories by broad substring matching.

### 6.4 Source archives

Source archives remain governed by SPEC-073 source-archive release metadata and attestation rules. Ashgrove MUST NOT silently weaken source-archive integrity by applying broad developer-checkout ignore rules to arbitrary archives, including extracted source-shaped archives that contain `Cargo.toml`, `std/src/`, and `release-source.toml`.

The first implementation SHOULD keep source-archive digest behavior separate from source-root payload digest behavior. These are distinct concepts and call paths, not a requirement that two arbitrary digest strings must always differ:

```text
source_root_payload_digest_policy != source_archive_digest_policy
```

A source archive producer should avoid packaging local state before release. If later work wants source-archive-local-state exclusion, it must amend this spec with archive-specific attestation semantics.

## 7. Install-record and provenance

If the implementation changes install-record metadata, it MUST record the source payload digest policy explicitly. Recommended field names:

```toml
source_payload_digest_policy = "source-root-v2-gitignore-local-state"
source_payload_digest = "sha256:..."
```

Existing `source_archive_digest` MUST remain the digest of an attested source archive payload and MUST NOT be overloaded to mean the gitignore-filtered source-root payload digest.

When `--allow-dirty-source` is used, dirty nonignored source payload still marks the install non-reproducible and contributes to the dirty digest/toolchain identity. Ignored local state MUST NOT contribute to dirty source identity.

## 8. Diagnostics

When post-build payload digest changes, ashgrove MUST report source-payload mutation, not generic source-root dirtiness. The diagnostic SHOULD include the source root path and SHOULD mention ignored local state is excluded.

Required diagnostic distinction:

| Condition | Required outcome |
| --- | --- |
| Ignored `.agents/status/dashboard.json` changes during build | install succeeds; no dirty-source override required |
| Ignored nested `target/` file changes during build | install succeeds; ignored file not copied |
| Nonignored source file changes during build | install fails before publish |
| Nonignored untracked file exists before build | install rejects without `--allow-dirty-source` |
| Git cannot determine payload membership for git-like source root | fail closed |

## 9. CLI policy

The first implementation MUST NOT add a broad arbitrary `--ignore` or `--exclude` CLI. Arbitrary user-defined source ignore globs can hide real source changes and would need to become part of the install identity. The default source-root behavior should be safe enough for ordinary developer checkouts.

A later explicit flag is allowed only if it is narrow, recorded in install metadata, and reflected in toolchain identity. Example of an acceptable future direction:

```text
--source-payload-policy gitignore-local-state
```

This future flag is not required by this spec.

## 10. Acceptance criteria

| ID | Requirement | Evidence owner |
| --- | --- | --- |
| A74-1 | Gitignored local state under `.agents/` can change during source-root install without aborting publish. | TASK-989 |
| A74-2 | Nested `target/` directories are excluded from payload digest and isolated build copy. | TASK-989 |
| A74-3 | Nonignored source payload mutation during build still fails before publish. | TASK-989 |
| A74-4 | Nonignored dirty source roots remain rejected unless `--allow-dirty-source` is explicit. | TASK-988/TASK-989 |
| A74-5 | Source archive release metadata and attestation behavior remains fail-closed and unchanged unless explicitly amended. | TASK-989 |
| A74-6 | Payload digest and payload copy share one membership policy. | TASK-988/TASK-989 |
| A74-7 | Install/update source paths use the same source-root payload policy. | TASK-989 |
| A74-8 | Closeout proves the reported local Ash checkout failure mode no longer reproduces or is covered by an equivalent deterministic regression. | TASK-990 |

## 11. Implementation notes

A practical implementation path is:

1. Introduce a policy-aware source payload walker in `crates/ashgrove/src/lib.rs`.
2. Split live source-root payload digesting from source-archive digesting, then make source-root digest and `copy_source_tree_for_build` consume one shared source-root membership list instead of each walking with a shallow skip predicate.
3. For live git source roots, use `git ls-files --cached --others --exclude-standard -z` or equivalent membership so git cleanliness and payload membership agree, failing closed on git membership errors.
4. Preserve fail-closed git-source identity checks in `SourceRootMetadata::inspect`, but fence the legacy `.dirty` sentinel so gitignored `.dirty` files in git roots cannot bypass gitignore-aware cleanliness.
5. Add deterministic tests with a fake `cargo` earlier in `PATH`; the fake cargo receives the original source root through a test environment variable, records its isolated-copy working directory, verifies ignored local state is absent from that copy, mutates ignored or nonignored files in the original source root, and writes executable fixture binaries under `$CARGO_TARGET_DIR/debug`.

## 12. Changelog

### 2026-05-31

- Initial draft specifying source-root payload membership, local-state exclusion, source-archive boundary preservation, and TASK-987 through TASK-990 implementation plan.
- Review remediation tightened source-shaped archive classification, git membership fail-closed rules, `.dirty` sentinel handling, update-path parity evidence, A74-6 ownership, and fake-cargo regression requirements.
