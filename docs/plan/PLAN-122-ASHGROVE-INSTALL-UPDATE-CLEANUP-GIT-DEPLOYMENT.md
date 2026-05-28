# PLAN-122: Ashgrove Install, Update, Cleanup, and Git Deployment

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. This phase creates install/deploy tooling. Do not build a package registry, release-channel resolver, global installer, or mandatory signing system unless a later spec expands scope.

**Goal:** Implement `ashgrove` as the user-local Ash toolchain/deployment manager for source installs, binary tarball installs, updates, removal/cleanup, and git-pinned Ash project deployment.

**Architecture:** SPEC-073 defines a thin pre-registry package substrate. `ashgrove` installs immutable XDG-local toolchain bundles containing `ash`, `ashgrove`, stdlib, selected standard tooling, metadata, and release/runtime support; updates install new toolchains rather than mutating old ones; project dependencies live in `ash.toml` as git URL + tag/rev entries resolved into exact commits in `ash.lock`; compiler/module loading must consume the locked dependency-root map so `ash check` and `ash run` can use fetched or vendored packages.

**Tech Stack:** Rust CLI crate/binary, existing workspace crates, git command/process integration or git library after audit, tarball unpacking, TOML metadata with reserved-field preservation, XDG path resolution, stdlib/release staging, existing Ash CLI/runtime tests, Markdown specs/plans/tasks/reference updates.

---

## 1. Status

**Status:** ⚠️ Partial second slice after TASK-974 report; SPEC-073 remains Draft pending deferred acceptance rows
**Spec:** [SPEC-073](../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
**Task range:** [TASK-964](tasks/TASK-964-ashgrove-install-policy-packet.md) through [TASK-974](tasks/TASK-974-ashgrove-closeout-acceptance.md)

TASK-964 creates the docs/spec/plan/task packet. TASK-965 is a hard audit gate and must patch exact downstream verification commands before Rust implementation starts.

## 2. Scope

### In scope

- `ashgrove <command>` public command naming.
- Source install from a git checkout/source archive into an immutable XDG-local toolchain directory.
- Binary tarball production contract and binary tarball install into the same toolchain layout.
- Toolchain metadata and install records.
- Bundled stdlib and standard tooling policy.
- Stable launcher dispatch for project-pinned/user-default toolchain selection.
- User default and project toolchain selection via lower-case `ash.toml`.
- Update, list, current, default, remove, cleanup commands.
- Project `ash.toml` dependency metadata for git dependencies.
- Project `ash.lock` exact commit resolution.
- Git fetch and vendor/offline deployment support.
- Module-loader/CLI integration so locked dependencies and installed stdlib roots are actually used by `ash check` and `ash run`.
- Reserved signing/trust metadata without mandatory enforcement.

### Out of scope

- Hosted package registry.
- Release-channel/version-index discovery.
- Global/system install roots.
- OS package-manager integration.
- Mandatory package signing or transparency log enforcement.
- Independent stdlib updates outside toolchain updates.
- Editor plugin management.
- Arbitrary SemVer dependency solving across registry packages.
- A separate semantic `ashd` daemon surface; daemon control remains `ash daemon ...` from SPEC-070.

## 3. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-964](tasks/TASK-964-ashgrove-install-policy-packet.md) | Create SPEC-073/PLAN-122/task packet and register Phase 127 | 4 | ✅ Complete |
| [TASK-965](tasks/TASK-965-ashgrove-live-install-audit-gate.md) | Audit live CLI/build/release/stdlib/daemon/XDG/git seams before implementation | 8 | ✅ Complete |
| [TASK-966](tasks/TASK-966-ashgrove-cli-crate-and-command-skeleton.md) | Add `ashgrove` command skeleton and shared reporting/errors | 8 | ✅ First slice |
| [TASK-967](tasks/TASK-967-toolchain-metadata-and-xdg-layout.md) | Implement metadata schemas, XDG paths, launcher dispatch, selectors, stdlib metadata, trust preservation, staging/publish helpers | 14 | ⚠️ Partial |
| [TASK-968](tasks/TASK-968-source-install-flow.md) | Implement source install path and installed-stdlib root use | 14 | ⚠️ Partial |
| [TASK-969](tasks/TASK-969-binary-tarball-install-flow.md) | Implement conforming tarball production/validation/install path | 14 | ⚠️ Partial |
| [TASK-970](tasks/TASK-970-update-default-list-current-flow.md) | Implement update/default/list/current flows | 10 | ⚠️ Partial |
| [TASK-971](tasks/TASK-971-remove-cleanup-flow.md) | Implement remove and cleanup policy, including daemon/running-manager protection | 12 | ⚠️ Partial |
| [TASK-972](tasks/TASK-972-ash-manifest-lock-git-fetch.md) | Implement `ash.toml`, `ash.lock`, git fetch, lock checking, trust preservation, and dependency-root module-loader integration | 18 | ⚠️ Partial second slice |
| [TASK-973](tasks/TASK-973-vendor-and-deployable-git-project-flow.md) | Implement vendor/offline deployment flow for git projects | 12 | ⚠️ Partial second slice |
| [TASK-974](tasks/TASK-974-ashgrove-closeout-acceptance.md) | Close out SPEC-073 with acceptance matrix and broad verification | 8 | ⚠️ Reported |

## 4. Decision gates

- **D1:** Alpha installs are user-local and XDG-compatible. Global installs are deferred.
- **D2:** A toolchain install includes `ash`, `ashgrove`, stdlib, metadata, and the TASK-965-frozen standard tooling list for that release.
- **D3:** The daemon surface is `ash daemon ...`; `ashd` is not required unless a future task explicitly adds a compatibility shim.
- **D4:** The stdlib is a toolchain component, not a normal third-party dependency in alpha.
- **D5:** Installed toolchain directories are immutable; update installs a new toolchain and changes selectors only when requested.
- **D6:** Source install and binary tarball install publish the same required toolchain shape.
- **D7:** Lower-case `ash.toml` is the canonical project/package manifest for this phase; `.ash.toml` is legacy/compat configuration and must not silently conflict.
- **D8:** Git tags are user intent; lockfiles record exact commits as execution truth.
- **D9:** Cleanup/remove must be conservative and dry-run visible before destructive deletion.
- **D10:** Signing/trust fields are reserved and preserved, but mandatory enforcement is deferred.
- **D11:** TASK-965 must bind every implementation task to exact live files, focused tests, and zero-test-safe commands before implementation starts.

## 5. Tracks

- **Track A — Packet and audit:** TASK-964/TASK-965 freeze policy, file ownership, dependencies, and exact verification seams.
- **Track B — Toolchain manager substrate:** TASK-966/TASK-967 create `ashgrove`, metadata, XDG roots, launcher dispatch, selectors, stdlib metadata, trust preservation, and atomic installation helpers.
- **Track C — Install/update lifecycle:** TASK-968/TASK-969/TASK-970 implement source install, tarball production/install, and toolchain update/selection flows.
- **Track D — Cleanup/removal lifecycle:** TASK-971 implements conservative removal and cleanup, including live daemon and running-manager protections.
- **Track E — Git deployment substrate:** TASK-972/TASK-973 implement manifests, lockfiles, git fetch, module-loader dependency roots, and vendor/offline deployment.
- **Track F — Closeout:** TASK-974 maps SPEC-073 A73-1 through A73-12 to focused/broad verification and independent review remediation.

## 6. Implementation feasibility gates

TASK-965 must produce an audit artifact before Rust implementation begins. The audit must inspect and bind:

1. Existing workspace binary/crate layout and whether `ashgrove` should be a new crate or additional binary. Preferred first-slice default is `crates/ashgrove` as a sibling tool crate.
2. Existing `ash-cli` command boundaries so `ashgrove` does not duplicate language commands.
3. Existing `ash daemon ...` control-plane surfaces from SPEC-070 and exact state fields needed for live toolchain protection.
4. Current stdlib source layout and current workspace-relative stdlib discovery seams in `ash-engine`/`ash-cli`.
5. Current build/release scripts and the missing/concrete tarball package producer path.
6. Current metadata/TOML dependencies and whether a shared metadata crate is warranted.
7. Current test harness patterns for CLI integration tests and temp XDG roots.
8. Whether git integration should shell out to `git` or use a Rust crate for the first slice.
9. Tarball/unpack dependencies, compression format, safe extraction constraints, and HTTP/download handling for `--url` if URL installs remain in first slice.
10. XDG path implementation strategy.
11. Exact public standard-tool list for the alpha bundle.
12. First-slice toolchain-id scheme, SemVer/version parsing strategy, and deterministic same-version collision behavior.
13. Exact command/test names each downstream task must run.

Implementation tasks TASK-966 through TASK-973 contain fail-closed placeholder verification until TASK-965 replaces them with exact commands. TASK-974 uses broad closeout gates and does not use the placeholder mechanism.

## 7. Required behavior by task

### TASK-966: command skeleton

Must add a public `ashgrove` binary with subcommands present but not all behavior complete. Incomplete destructive commands must fail closed with “not implemented” until their owning tasks land.

### TASK-967: metadata and XDG substrate

Must implement typed path resolution, stable launcher dispatch, toolchain metadata, first-slice toolchain-id/collision helpers, stdlib metadata staging, selector metadata, trust-field preservation for metadata read/write, and atomic staging/publish helpers using isolated test roots. Tests must not touch the developer's real `$HOME`, `$XDG_*`, or installed Ash state.

### TASK-968: source install

Must stage, build/copy, verify shape, and atomically publish a source-built toolchain. Dirty/unidentified source behavior must be explicit and recorded. The installed `ash` must use the selected toolchain stdlib rather than workspace `std/src`.

### TASK-969: binary tarball install

Must define and produce a conforming tarball fixture or release artifact, verify tarball shape, executable presence/permissions, metadata schema, archive-version/target match, stdlib manifest presence, safe extraction, and digest recording before publish.

### TASK-970: update/select flows

Must prove update does not mutate the old toolchain; first install/default/current/list work; `update --switch` changes the default; update without `--switch` preserves the current default.

### TASK-971: remove/cleanup

Must prove default/current/live daemon/running-manager toolchain deletion is refused by default and `cleanup --dry-run` is non-destructive. Must add or consume the minimal daemon toolchain registry/status needed for live protection.

### TASK-972: git lock/fetch

Must prove tag resolution writes exact commits, `lock --check` detects manifest/lock drift, reserved trust fields are preserved, unpinned deps fail closed, and fetched dependency roots are visible to `ash check`/`ash run` through module-loader integration. Fetch-only behavior is insufficient.

### TASK-973: vendor/deployment

Must prove a git-based project can be materialized for offline or reproducible deployment from lockfile data without fetching stdlib as a dependency. `vendor --check` must validate the default `vendor/ash/` directory or an explicit `--output PATH` without writing/fetching.

## 8. Verification strategy

Docs-only packet verification:

```bash
git diff --check
python3 - <<'PY'
from pathlib import Path
required = [
    'docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md',
    'docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md',
    'docs/plan/tasks/TASK-964-ashgrove-install-policy-packet.md',
    'docs/plan/tasks/TASK-965-ashgrove-live-install-audit-gate.md',
    'docs/plan/tasks/TASK-966-ashgrove-cli-crate-and-command-skeleton.md',
    'docs/plan/tasks/TASK-967-toolchain-metadata-and-xdg-layout.md',
    'docs/plan/tasks/TASK-968-source-install-flow.md',
    'docs/plan/tasks/TASK-969-binary-tarball-install-flow.md',
    'docs/plan/tasks/TASK-970-update-default-list-current-flow.md',
    'docs/plan/tasks/TASK-971-remove-cleanup-flow.md',
    'docs/plan/tasks/TASK-972-ash-manifest-lock-git-fetch.md',
    'docs/plan/tasks/TASK-973-vendor-and-deployable-git-project-flow.md',
    'docs/plan/tasks/TASK-974-ashgrove-closeout-acceptance.md',
]
for rel in required:
    assert Path(rel).exists(), rel
PY
```

Implementation-task verification adds focused tests named by TASK-965 plus repo-native broad gates:

```bash
bash scripts/check-rust-format.sh
bash scripts/check-rust-clippy.sh
bash scripts/check-rust-tests.sh --workspace --all-targets
bash scripts/check-doc-tests.sh
git diff --check
```

Closeout must also run a scoped docs/link/metadata sweep for all SPEC-073/PLAN-122/TASK-964..974 references.

## 9. Acceptance mapping

TASK-974 owns the final acceptance matrix. It must map SPEC-073 A73-1 through A73-12 to concrete tests, command output, or documented deferrals. No row may be marked accepted by prose alone.

## 10. Changelog

### 2026-05-28

- Initial PLAN-122 packet for `ashgrove` install/update/remove/cleanup and git deployment tooling.
