# TASK-974 Phase 127 Codex Implementation Report

**Date:** 2026-05-28
**Worktree:** `/home/dikini/Projects/ash/.worktrees/phase-127-ashgrove`
**Status:** Partial Phase 127 after TASK-973 alpha offline vendor/deployable flow completion; SPEC-073 remains Draft.

## Summary

This run salvaged and continued the live Phase 127 diff without starting over or touching another checkout. The resulting tree contains a new `ashgrove` workspace crate, focused tests for TASK-966 through TASK-973, an installed-stdlib/dependency-root module-loader seam in `ash-engine`, and status documentation that does not promote SPEC-073 to Implemented MVP.

The implementation is intentionally reported as partial. Focused tests now exercise real local source-root builds plus prepared source-shaped directory inputs, and the TASK-969 follow-up adds a repository release tarball producer plus schema-versioned local tarball install proof. Several SPEC-073 acceptance rows still require source archive release metadata, concrete runtime-support payload metadata, manifest rewrite trust preservation, authenticated URL install policy, packaged dispatcher lifecycle policy, and mandatory trust/signing enforcement. Follow-up slices added executable-bit tarball validation, current-project and XDG daemon-state removal protections, explicit confirmation for protected force-removal and old-toolchain cleanup, conservative cleanup execution for cache/orphan/old-toolchain flags, XDG git cache checkouts at exact lockfile commits, vendor package content materialization, `ash check`/explicit ordinary-file `ash run` discovery of default vendored locked dependencies and direct fetched-cache locked dependencies, and real temp-root `ash`/`ashgrove` launcher shims backed by typed ashgrove dispatch through a stable user-local dispatcher copy. Broader lockfile/cache cleanup reachability remains deferred by the SPEC-073 alpha boundary.

## Completed Or Partially Completed Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| TASK-966 | First slice | `ashgrove` crate exists; command help lists install/update/default/list/current/remove/cleanup/fetch/lock/vendor; bare version install fails closed. |
| TASK-967 | Complete | XDG path defaults/overrides, first-slice `ToolchainId` validation, typed manifest/install-record metadata, selector trust preservation, staged publish/collision helpers, typed launcher dispatch, real temp-root `ash`/`ashgrove` launcher shim installation/execution through a stable user-local `.ashgrove-dispatcher` copy, transparent selected-tool exit-code preservation, selected-root symlink rejection, hardened shim temp-file writes, and symlink/path traversal fail-closed coverage exist. |
| TASK-968 | Partial | Source-root install builds `ash`/`ashgrove` from an isolated cache copy, preserves clean no-lock source roots, fails closed when git-like roots cannot report `HEAD` or dirty status, distinguishes dirty override payloads with dirty tree digests in metadata/toolchain IDs, keeps prepared source-shaped directory coverage for archive-shaped inputs, and routes launcher-selected `ash` to the selected stdlib root. Source archive release metadata and concrete runtime-support payload metadata remain deferred. |
| TASK-969 | Complete | `scripts/package-ash-toolchain.sh` packages a coherent repository release tarball with required tools, stdlib, typed manifest/install-record metadata, and `archive_schema_version = 1`; local tarball install validates schema/version, safe extraction, required executable bits, stdlib shape, identity/version match, staged publish, and local path/digest/install-time recording. Authenticated URL download remains deferred outside TASK-969. |
| TASK-970 | Complete for alpha local updates | Default/list/current validate selector state against installed manifest/install metadata; `default` requires exact installed immutable ids; source update builds/stages real source roots and records source metadata; local tarball update accepts producer-compatible payloads and records tarball path/digest/install time; update preserves or switches defaults according to `--switch`; bare/network update remains deferred by SPEC-073. |
| TASK-971 | Complete for SPEC-073 alpha remove/cleanup policy | Remove protects user default, current-project pins, `ASHGROVE_RUNNING_TOOLCHAIN`, and TOML daemon state under `$XDG_STATE_HOME/ash/daemon/`; `--force` overrides only default/current-project pin protection after explicit stdin confirmation; live-daemon and running-manager protection remain non-overridable; bare `cleanup --project PATH --dry-run` is a non-destructive planner; `--cache`, `--orphans`, and `--old-toolchains` execute conservatively under isolated XDG roots, with old-toolchain deletion requiring explicit stdin confirmation before any combined cleanup deletion. Broader lockfile/cache reachability remains deferred. |
| TASK-972 | Complete for SPEC-073 alpha git lock/fetch and dependency-root integration | Lower-case `ash.toml` dependencies reject unpinned git entries, reject legacy `.ash.toml` metadata conflicts, resolve local git tags/revs to exact commits in `ash.lock`, expand accepted abbreviated revs, preserve existing lockfile `[trust]` metadata, `lock --check` detects drift, `fetch` materializes local git dependencies into XDG cache checkouts keyed by exact lockfile commits, and `ash check` plus explicit ordinary-file `ash run` discover validated vendored lock roots and direct fetched-cache roots without dependency-root environment variables. Direct fetched roots fail closed when missing or when git `HEAD` does not match the lock commit, and selected stdlib roots keep precedence over stdlib-shaped locked packages. Manifest rewrite trust preservation and mandatory trust/signing enforcement remain deferred. |
| TASK-973 | Complete for SPEC-073 alpha offline vendor/deployable git project flow | `vendor` copies every locked package from exact XDG cache checkouts into default `vendor/ash/<package>/` or explicit `--output PATH`, writes provenance entries, rejects unsafe package names and non-40-hex lock commits, `vendor --check` is read-only and fails on provenance/content/cache drift without fetch writes, offline `ash check src/main.ash` plus explicit ordinary-file `ash run src/main.ash:main` resolve default vendored dependencies without dependency-root env vars or usable XDG fetched cache, and selected/explicit stdlib roots take precedence over the auto-discovered `vendor/ash` dependency namespace even when it contains a stdlib-shaped package. |
| TASK-974 | Reported | This report maps acceptance status, verification, review findings, changed files, and deferrals. |

## Acceptance Matrix

| Acceptance | Status | Evidence / Deferral |
| --- | --- | --- |
| A73-1 source install | Partial | `cargo test -p ashgrove --test task_968_source_install -- --nocapture`; real local source-root build, staged publish, dirty/unidentified rejection, git-like corrupt metadata fail-closed behavior, post-review clean-source/status-failure/dirty-digest regressions, and prepared source-shaped directory inputs are covered. Source archive release metadata and concrete runtime-support payload metadata remain deferred. |
| A73-2 binary tarball install | Complete for local tarballs | `cargo test -p ashgrove --test task_969_tarball_install -- --nocapture`; producer-output tarball installs under temporary XDG/home roots, executable-bit validation is covered, schema/archive-version validation is enforced, local path/digest/install-time recording is covered, and install publishes through staging. Authenticated URL download remains deferred. |
| A73-3 equivalent toolchain contents | Partial | Source/tarball fixtures require `bin/ash`, `bin/ashgrove`, stdlib manifest/src, manifest, install record; no runtime/support metadata equivalence proof. |
| A73-4 immutable update | Complete for alpha local source/tarball updates | `RUSTC_WRAPPER= cargo test -p ashgrove --test task_970_selectors -- --nocapture`; real source-root update and producer-compatible local tarball update install new immutable toolchains, require `--to` payload identity matches, and preserve old manifest/install-record metadata. Bare release-index/network update remains deferred. |
| A73-5 default switches metadata | Partial | `cargo test -p ashgrove task_970 -- --nocapture`; `cargo test -p ashgrove --test task_967_layout -- --nocapture` now proves real temp-root launcher shim execution honors explicit override before project pins before defaults, routes through a stable user-local dispatcher copy, and preserves selected-tool exit status. Project rewrite proof and packaged dispatcher lifecycle remain deferred. |
| A73-6 remove protections | Complete for alpha | `RUSTC_WRAPPER= cargo test -p ashgrove --test task_971_remove_cleanup -- --nocapture`; default, current-project, live-daemon, and running-manager protections are covered, including explicit confirmation before `--force` overrides default/current-project pins and non-overridable daemon/running-manager cases. |
| A73-7 cleanup dry-run | Complete for alpha | `RUSTC_WRAPPER= cargo test -p ashgrove --test task_971_remove_cleanup -- --nocapture`; bare `cleanup --project PATH --dry-run` is non-destructive, cache/orphan/old-toolchain dry-runs report without deleting, non-dry-run old-toolchain deletion requires explicit confirmation before any combined cleanup deletion, and execution flags are conservatively constrained. Broader lockfile/cache reachability remains deferred. |
| A73-8 tag resolves to exact commit | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; local file git tags resolve to full commits and `fetch` materializes the exact lockfile commit under XDG cache even if a tag later moves. |
| A73-9 lock drift detection | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; drift detected after manifest tag change. |
| A73-10 selected toolchain stdlib | Partial | `cargo test -p ash-engine task_968 -- --nocapture`; `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name -- --exact --nocapture`; `cargo test -p ashgrove --test task_968_source_install -- --nocapture`; explicit stdlib root override works, auto-discovered project vendor packages cannot shadow that root, and the launcher public path passes the selected installed stdlib root to `ash`. Concrete runtime-support payload metadata remains deferred. |
| A73-11 trust/signing reserved metadata | Deferred | No preservation test or model yet. |
| A73-12 locked dependencies visible to `ash check`/`ash run` | Complete for TASK-972/TASK-973 alpha | `cargo test -p ash-engine task_972 -- --nocapture` proves explicit dependency roots and direct fetched-cache roots are visible to the loader, `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_973_vendor -- --nocapture` proves the default vendor layout copies every locked package from exact locked cache checkouts and separately proves explicit `--output PATH` provenance/check behavior, and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture` proves `ash check src/main.ash` and explicit ordinary-file `ash run src/main.ash:main` discover validated project `ash.lock` roots through both `vendor/ash` and direct `$XDG_CACHE_HOME/ash/git/checkouts/.../<commit>/` without dependency-root env vars while preserving selected stdlib precedence over auto-discovered dependency roots. |

## Focused Verification

Commands run in this worktree:

- `cargo test -p ashgrove task_966 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_967 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove --test task_968_source_install -- --nocapture` - passed, 17 tests after adding review regressions for real local source-root builds, clean no-lock roots, git status failure, corrupt git metadata, dirty digest install IDs, and launcher-selected stdlib routing.
- `cargo test -p ash-engine task_968 -- --nocapture` - passed, 1 matching test.
- `cargo test -p ashgrove task_969 -- --nocapture` - initially failed because the unsafe tar fixture used `Header::set_path("../escape")`, which the tar crate rejects while building the fixture; after changing the fixture to a symlink entry, passed with 2 tests.
- `cargo test -p ashgrove task_970 -- --nocapture` - passed, 1 test.
- `RUSTC_WRAPPER= cargo test -p ashgrove --test task_970_selectors -- --nocapture` - passed, 10 tests after replacing the stale update-from-existing shortcut with real source-root and producer-compatible local tarball update coverage.
- `cargo test -p ashgrove task_971 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_972 -- --nocapture` - initially failed because fixture git operations could invoke user signing/editor policy; after forcing non-signing git config in the fixture, passed with 2 tests.
- `cargo test -p ash-engine task_972 -- --nocapture` - passed, 1 matching test.
- `cargo test -p ashgrove task_973 -- --nocapture` - passed, 1 test.
- Follow-up slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_969_tarball_install_rejects_non_executable_required_binary -- --nocapture` - initially failed because tarball validation accepted a non-executable required binary.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_969 -- --nocapture` - passed, 3 tests after executable-bit validation and executable fixture permissions were added.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_971_remove_force_protects_live_daemon_state -- --nocapture` - initially failed because `remove --force` ignored daemon state.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_971_remove_protects_current_project_pin_without_force -- --nocapture` - initially failed because `remove` ignored the current project's `ash.toml` toolchain pin.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_971 -- --nocapture` - passed, 4 tests after live-daemon and current-project protection were added.
  - `RUSTC_WRAPPER= cargo test -p ashgrove --test task_971_remove_cleanup -- --nocapture` - passed, 21 tests, after adding regressions for force override confirmation, default-only protection, non-overridable running-manager protection, exact bare project dry-run planning, cache/orphan dry-run non-destruction, allowlisted cache cleanup, toolchain-root-only orphan cleanup, and project-pin preservation plus confirmation before old-toolchain and combined cleanup deletion.

## Broad Gates

Commands run in this worktree:

- `cargo fmt --check` - passed after applying `cargo fmt`.
- `cargo check --workspace` - failed under the configured `sccache` wrapper with `Operation not permitted (os error 1)` before project code compiled.
- `RUSTC_WRAPPER= cargo check --workspace` - passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed.
- `git diff --check` - passed.
- `python3 tools/reference/check_frontmatter.py` - passed, `checked=33 pilot=False`.
- Follow-up slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo fmt --check` - initially failed on formatting in `crates/ashgrove/src/lib.rs`; after `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo fmt`, passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo check --workspace` - passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --tests -- --nocapture` - passed, 19 tests.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine task_968 -- --nocapture` - passed, 1 matching test.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine task_972 -- --nocapture` - passed, 1 matching test.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli task_971 -- --nocapture` - passed with 0 matching tests.
  - `python3 tools/reference/check_frontmatter.py` - passed, `checked=33 pilot=False`.
  - `git diff --check` - passed.
- Current second slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_972_fetch_materializes_exact_lock_commit_in_xdg_cache -- --nocapture` - initially failed because `fetch()` only wrote metadata and did not create the expected XDG cache checkout; after adding git mirror/checkout materialization, passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_973_vendor_materializes_package_content_from_locked_cache_commit -- --nocapture` - initially failed because `vendor()` only wrote provenance and did not copy package content; after copying from locked cache checkouts, passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_972 -- --nocapture` - passed, 4 tests.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_973 -- --nocapture` - passed, 3 tests.
- Current follow-up slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli task_972_973 --test task_972_973_project_vendor_roots -- --nocapture` - initially failed because `ash check` could not discover `vendor/ash` roots and malformed lock package names were reported only as unresolved modules; the attempted `ash run --dry-run` test also exposed the separate entry-runtime import limitation.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture` - passed, 2 tests, after moving discovery into `ash-engine` module-root search and scoping the run limitation to a documented deferral.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine task_972 -- --nocapture` - passed, 1 matching test.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_973 -- --nocapture` - passed, 4 matching tests.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo fmt --check` - passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo check --workspace` - passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy --workspace --all-targets --all-features -- -D warnings` - passed.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --tests -- --nocapture` - passed, 23 tests.
  - `python3 tools/reference/check_frontmatter.py` - passed, `checked=33 pilot=False`.
  - `git diff --check` - passed.
- Current explicit `ash run` vendored-root slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution run_discovers_locked_vendored_dependency_without_dependency_root_env -- --nocapture` - initially failed with exit 2 because `ash run /tmp/.../src/main.ash:main` routed an ordinary dependency import through the entry-runtime import filter: `unsupported entry runtime import 'helper::{HelperToken}'`.
  - After routing explicit non-entry `FILE:WORKFLOW` ordinary files through the module-loader-backed ordinary path, the same command passed, 1 test.
- Current stdlib/vendor separation slice:
  - `cargo test -p ash-cli --test phase127_vendored_dependency_resolution cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name -- --exact --nocapture` - initially failed with 1 failed test because `option::{SelectedOption}` resolved through locked `vendor/ash/option` instead of the explicit stdlib root.
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name -- --exact --nocapture` - passed, 1 test, after the auto-discovered project `vendor/ash` dependency namespace was moved after the selected stdlib root in module resolution.
- Current TASK-967 launcher shim slice:
  - `cargo test -p ashgrove --test task_967_layout -- --nocapture` - initially failed at compile time because no real launcher shim installation API existed. After adding `install_launcher_shims`, hidden `__launcher-dispatch`, `ASH_TOOLCHAIN` explicit override handling, contained tool-path validation, and real temp-root shim execution regressions, passed with 16 tests.
- Current TASK-967 launcher blocker remediation slice:
  - `cargo test -p ashgrove --test task_967_layout -- --nocapture` - initially failed 4 new regressions because selected-tool exit status collapsed to a wrapper error, source/tarball install shims embedded the transient `current_exe()` path, selected toolchain-root symlinks were accepted, and predictable shim temp files followed attacker-controlled symlinks. After adding Unix `exec`/non-Unix exit-code preservation, stable user-local `.ashgrove-dispatcher` copies, pre-canonicalization toolchain-root symlink rejection, and hardened temp-file writes, passed with 20 tests.
- Current TASK-973 completion reconciliation slice:
  - `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_973_vendor -- --nocapture` - passed, 7 tests, after adding evidence for multi-package default vendoring, explicit `--output PATH` provenance/check behavior, and read-only `vendor --check` failures for missing vendor/cache evidence. No production-code change was needed for these regressions.

## Independent Review Findings

An independent review agent inspected the diff and returned findings. Two code-level blockers have been remediated in this worktree:

- `ashgrove vendor` now validates lockfile package names before deriving vendor paths, and `task_973_vendor_rejects_lockfile_package_name_path_traversal` proves `../escape` is rejected without creating an escaped provenance file.
- `Manifest::lock_text` now serializes typed lockfile structs through `toml::to_string`, and `task_972_lock_serializes_dependency_values_without_toml_injection` proves crafted dependency values remain escaped data instead of malformed TOML or injected package tables.

The remaining findings are accepted as current deferred gaps, not dismissed:

- TASK-974 closeout evidence was missing before this report.
- Source install now builds real local source roots and publishes through the staged toolchain path, but source archive release metadata and concrete runtime-support payload metadata remain undefined.
- Git dependency work now materializes XDG cache checkouts and vendor package content for local git fixtures, and `ash check` plus explicit ordinary-file `ash run` consume project `ash.toml`/`ash.lock` plus both default `vendor/ash` roots and direct fetched-cache roots.
- Remove/cleanup safety now has focused current-project and live-daemon state protection, explicit confirmation for protected force-removal and old-toolchain cleanup, plus conservative cache/orphan/old-toolchain execution coverage. Broader lockfile/cache reachability remains deferred.
- Tarball validation checks executable permissions for required binaries on Unix, enforces first-slice archive schema version, records local tarball provenance, and has a repository producer-output install proof.
- TASK-967 real launcher shims are now implemented for temp-root/user-local `ash` and `ashgrove` scripts through a stable user-local dispatcher copy. Launcher-selected stdlib routing has TASK-968 coverage; release packaged dispatcher lifecycle remains tracked under later release packaging work.
- Lock/vendor format is still intentionally thin beyond the verified alpha offline vendor flow: local git cache checkout, default/explicit vendor package content copy, package provenance, read-only check, remediated escaping, package-name validation, full-commit validation, and default-layout CLI consumption.
- Status/changelog surfaces were stale before the current reconciliation edits.

## Deferred Gaps

- Source archive release metadata and concrete runtime-support payload metadata for source installs.
- Authenticated tarball URL download/recording and release-index trust policy.
- Release packaging and lifecycle policy for the stable user-local dispatcher copy.
- Full metadata models preserving reserved trust/signing fields.
- Broader cleanup reachability across lockfiles/cache metadata.
- Remote-authenticated git fetch policy, manifest rewrite trust metadata preservation, and mandatory trust/signing enforcement.
- Broader acceptance evidence before SPEC-073 can move beyond Draft.

## Changed Files

- `CHANGELOG.md`
- `Cargo.toml`
- `crates/ash-engine/Cargo.toml`
- `crates/ash-engine/src/entry.rs`
- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-cli/tests/phase127_vendored_dependency_resolution.rs`
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
- `scripts/package-ash-toolchain.sh`
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
