# TASK-974 Phase 127 Codex Implementation Report

**Date:** 2026-05-28
**Worktree:** `/home/dikini/Projects/ash/.worktrees/phase-127-ashgrove`
**Status:** Partial first slice; SPEC-073 remains Draft.

## Summary

This run salvaged and continued the live Phase 127 diff without starting over or touching another checkout. The resulting tree contains a new `ashgrove` workspace crate, focused tests for TASK-966 through TASK-973, an installed-stdlib/dependency-root module-loader seam in `ash-engine`, and status documentation that does not promote SPEC-073 to Implemented MVP.

The implementation is intentionally reported as partial. Focused tests exercise a fixture-based first slice, but several SPEC-073 acceptance rows still require real source builds, atomic publish, release tarball production/schema validation, launcher shims, daemon-state removal protection, trust metadata preservation, real git fetch/vendor content materialization, and `ash check`/`ash run` lockfile integration.

## Completed Or Partially Completed Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| TASK-966 | First slice | `ashgrove` crate exists; command help lists install/update/default/list/current/remove/cleanup/fetch/lock/vendor; bare version install fails closed. |
| TASK-967 | Partial | XDG path defaults/overrides and first-slice `ToolchainId` validation exist; launcher dispatch, typed metadata preservation, and atomic publish remain deferred. |
| TASK-968 | Partial | Fixture-shaped source install copies required shape, rejects dirty/unidentified source without overrides, and `ash-engine` can load from an explicit installed stdlib root; real build-from-source and atomic staging remain deferred. |
| TASK-969 | Partial | Tarball install validates basic safe extraction and required path shape, records a digest, and rejects unsafe symlink entries; release producer, schema validation, executable permission checks, and atomic publish remain deferred. |
| TASK-970 | Partial | Default/list/current/update-from-existing selector flows are covered; launcher behavior and full source/tarball update semantics remain deferred. |
| TASK-971 | Partial | Remove protects user default and `ASHGROVE_RUNNING_TOOLCHAIN`; cleanup dry-run is non-destructive. Project-selected/current/live-daemon protection and cleanup execution policy remain deferred. |
| TASK-972 | Partial | Lower-case `ash.toml` dependencies reject unpinned git entries, resolve local git tags/revs to exact commits in `ash.lock`, and `lock --check` detects drift; real fetch/cache checkout, trust preservation, and `ash-cli` lockfile integration remain deferred. |
| TASK-973 | Partial | `vendor` writes provenance entries and `vendor --check` is read-only for those entries; package content materialization and offline `ash check`/`ash run` smoke tests remain deferred. |
| TASK-974 | Reported | This report maps acceptance status, verification, review findings, changed files, and deferrals. |

## Acceptance Matrix

| Acceptance | Status | Evidence / Deferral |
| --- | --- | --- |
| A73-1 source install | Partial | `cargo test -p ashgrove task_968 -- --nocapture`; fixture copy only, no real source build/atomic publish. |
| A73-2 binary tarball install | Partial | `cargo test -p ashgrove task_969 -- --nocapture`; fixture tarball only, no release producer/schema/permission validation. |
| A73-3 equivalent toolchain contents | Partial | Source/tarball fixtures require `bin/ash`, `bin/ashgrove`, stdlib manifest/src, manifest, install record; no runtime/support metadata equivalence proof. |
| A73-4 immutable update | Partial | `cargo test -p ashgrove task_970 -- --nocapture`; update-from-existing selector test preserves old manifest, but real update install path remains thin. |
| A73-5 default switches metadata | Partial | `cargo test -p ashgrove task_970 -- --nocapture`; no launcher rewrite/project rewrite proof beyond selector file. |
| A73-6 remove protections | Partial | `cargo test -p ashgrove task_971 -- --nocapture`; no `$XDG_STATE_HOME/ash/daemon` live daemon protection evidence. |
| A73-7 cleanup dry-run | Partial | `cargo test -p ashgrove task_971 -- --nocapture`; dry-run old-toolchain planning only. |
| A73-8 tag resolves to exact commit | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; local file git tags resolve to full commits. |
| A73-9 lock drift detection | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; drift detected after manifest tag change. |
| A73-10 selected toolchain stdlib | Partial | `cargo test -p ash-engine task_968 -- --nocapture`; explicit stdlib root override works, but launcher-selected installed `ash` flow remains deferred. |
| A73-11 trust/signing reserved metadata | Deferred | No preservation test or model yet. |
| A73-12 locked dependencies visible to `ash check`/`ash run` | Partial | `cargo test -p ash-engine task_972 -- --nocapture` proves explicit dependency roots are visible to the loader; manifest/lock-driven `ash-cli` integration remains deferred. |

## Focused Verification

Commands run in this worktree:

- `cargo test -p ashgrove task_966 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_967 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_968 -- --nocapture` - passed, 2 tests.
- `cargo test -p ash-engine task_968 -- --nocapture` - passed, 1 matching test.
- `cargo test -p ashgrove task_969 -- --nocapture` - initially failed because the unsafe tar fixture used `Header::set_path("../escape")`, which the tar crate rejects while building the fixture; after changing the fixture to a symlink entry, passed with 2 tests.
- `cargo test -p ashgrove task_970 -- --nocapture` - passed, 1 test.
- `cargo test -p ashgrove task_971 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_972 -- --nocapture` - initially failed because fixture git operations could invoke user signing/editor policy; after forcing non-signing git config in the fixture, passed with 2 tests.
- `cargo test -p ash-engine task_972 -- --nocapture` - passed, 1 matching test.
- `cargo test -p ashgrove task_973 -- --nocapture` - passed, 1 test.

## Broad Gates

Commands run in this worktree:

- `cargo fmt --check` - passed after applying `cargo fmt`.
- `cargo check --workspace` - failed under the configured `sccache` wrapper with `Operation not permitted (os error 1)` before project code compiled.
- `RUSTC_WRAPPER= cargo check --workspace` - passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed.
- `git diff --check` - passed.
- `python3 tools/reference/check_frontmatter.py` - passed, `checked=33 pilot=False`.

## Independent Review Findings

An independent review agent inspected the diff and returned findings. Two code-level blockers have been remediated in this worktree:

- `ashgrove vendor` now validates lockfile package names before deriving vendor paths, and `task_973_vendor_rejects_lockfile_package_name_path_traversal` proves `../escape` is rejected without creating an escaped provenance file.
- `Manifest::lock_text` now serializes typed lockfile structs through `toml::to_string`, and `task_972_lock_serializes_dependency_values_without_toml_injection` proves crafted dependency values remain escaped data instead of malformed TOML or injected package tables.

The remaining findings are accepted as current deferred gaps, not dismissed:

- TASK-974 closeout evidence was missing before this report.
- Source install does not build from source or atomically stage/publish.
- Git dependency work is metadata-only; `fetch()` calls `lock()`, and `ash-cli` does not yet consume `ash.toml`/`ash.lock`.
- Remove/cleanup safety lacks project-selected/current and live-daemon state protection.
- Tarball validation does not yet check full schemas or executable permissions.
- Launcher dispatch is not implemented.
- Lock/vendor format is still too thin for reproducible offline deployment beyond the remediated escaping and package-name validation.
- Status/changelog surfaces were stale before the current reconciliation edits.

## Deferred Gaps

- Real source build/stage/install flow with source URL/rev/build profile/target triple and atomic publish.
- Public release tarball producer plus schema, version/id, permission, digest, and safe extraction validation.
- Stable launchers under `$HOME/.local/bin` resolving explicit override, project pin, then default.
- Full metadata models preserving reserved trust/signing fields.
- Daemon-state integration under `$XDG_STATE_HOME/ash/daemon/` and non-overridable live daemon removal protection.
- Real git fetch/checkout into XDG cache and lockfile-driven dependency root derivation.
- Vendor package content materialization and offline deployment smoke tests through `ash check` and `ash run`.
- Broader acceptance evidence before SPEC-073 can move beyond Draft.

## Changed Files

- `CHANGELOG.md`
- `Cargo.toml`
- `crates/ash-engine/src/entry.rs`
- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-engine/tests/module_import_resolution_tests.rs`
- `crates/ash-engine/tests/task_968_installed_stdlib.rs`
- `crates/ashgrove/Cargo.toml`
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/src/main.rs`
- `crates/ashgrove/tests/support/mod.rs`
- `crates/ashgrove/tests/task_966_ashgrove_cli.rs`
- `crates/ashgrove/tests/task_967_layout.rs`
- `crates/ashgrove/tests/task_968_source_install.rs`
- `crates/ashgrove/tests/task_969_tarball_install.rs`
- `crates/ashgrove/tests/task_970_selectors.rs`
- `crates/ashgrove/tests/task_971_remove_cleanup.rs`
- `crates/ashgrove/tests/task_972_manifest_lock_git.rs`
- `crates/ashgrove/tests/task_973_vendor.rs`
- `docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/plan/audits/TASK-965-ashgrove-live-install-audit-gate.md`
- `docs/plan/audits/TASK-974-phase127-codex-implementation-report.md`
- `docs/plan/tasks/TASK-964-ashgrove-install-policy-packet.md`
- `docs/plan/tasks/TASK-965-ashgrove-live-install-audit-gate.md`
- `docs/plan/tasks/TASK-966-ashgrove-cli-crate-and-command-skeleton.md`
- `docs/plan/tasks/TASK-967-toolchain-metadata-and-xdg-layout.md`
- `docs/plan/tasks/TASK-968-source-install-flow.md`
- `docs/plan/tasks/TASK-969-binary-tarball-install-flow.md`
- `docs/plan/tasks/TASK-970-update-default-list-current-flow.md`
- `docs/plan/tasks/TASK-971-remove-cleanup-flow.md`
- `docs/plan/tasks/TASK-972-ash-manifest-lock-git-fetch.md`
- `docs/plan/tasks/TASK-973-vendor-and-deployable-git-project-flow.md`
- `docs/plan/tasks/TASK-974-ashgrove-closeout-acceptance.md`
- `docs/spec/README.md`
- `docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md`
