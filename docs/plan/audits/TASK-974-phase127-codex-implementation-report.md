# TASK-974 Phase 127 Codex Implementation Report

**Date:** 2026-05-28
**Worktree:** `/home/dikini/Projects/ash/.worktrees/phase-127-ashgrove`
**Status:** Partial follow-up slice; SPEC-073 remains Draft.

## Summary

This run salvaged and continued the live Phase 127 diff without starting over or touching another checkout. The resulting tree contains a new `ashgrove` workspace crate, focused tests for TASK-966 through TASK-973, an installed-stdlib/dependency-root module-loader seam in `ash-engine`, and status documentation that does not promote SPEC-073 to Implemented MVP.

The implementation is intentionally reported as partial. Focused tests now exercise real local source-root builds plus prepared source-shaped directory inputs, but several SPEC-073 acceptance rows still require source archive release metadata, concrete runtime-support payload metadata, release tarball production/schema validation, full trust metadata preservation, and direct fetched-cache dependency-root integration. Follow-up slices added executable-bit tarball validation, current-project and XDG daemon-state removal protections, XDG git cache checkouts at exact lockfile commits, vendor package content materialization, `ash check`/explicit ordinary-file `ash run` discovery of default vendored locked dependencies, and real temp-root `ash`/`ashgrove` launcher shims backed by typed ashgrove dispatch through a stable user-local dispatcher copy, but broader producer/cleanup/CLI integration behavior is still missing.

## Completed Or Partially Completed Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| TASK-966 | First slice | `ashgrove` crate exists; command help lists install/update/default/list/current/remove/cleanup/fetch/lock/vendor; bare version install fails closed. |
| TASK-967 | Complete | XDG path defaults/overrides, first-slice `ToolchainId` validation, typed manifest/install-record metadata, selector trust preservation, staged publish/collision helpers, typed launcher dispatch, real temp-root `ash`/`ashgrove` launcher shim installation/execution through a stable user-local `.ashgrove-dispatcher` copy, transparent selected-tool exit-code preservation, selected-root symlink rejection, hardened shim temp-file writes, and symlink/path traversal fail-closed coverage exist. |
| TASK-968 | Partial | Source-root install builds `ash`/`ashgrove` from an isolated cache copy, preserves clean no-lock source roots, fails closed when git-like roots cannot report `HEAD` or dirty status, distinguishes dirty override payloads with dirty tree digests in metadata/toolchain IDs, keeps prepared source-shaped directory coverage for archive-shaped inputs, and routes launcher-selected `ash` to the selected stdlib root. Source archive release metadata and concrete runtime-support payload metadata remain deferred. |
| TASK-969 | Partial | Tarball install validates basic safe extraction and required path shape, records a digest, rejects unsafe symlink entries, and rejects required binaries without executable bits; release producer, schema validation, archive-version policy, path/URL recording, and atomic publish remain deferred. |
| TASK-970 | Partial | Default/list/current/update-from-existing selector flows are covered; launcher behavior and full source/tarball update semantics remain deferred. |
| TASK-971 | Partial | Remove protects user default, current-project pins, `ASHGROVE_RUNNING_TOOLCHAIN`, and TOML daemon state under `$XDG_STATE_HOME/ash/daemon/`; cleanup dry-run is non-destructive for old-toolchain planning. Cleanup execution/cache/orphan/project planning remains deferred. |
| TASK-972 | Partial follow-up slice | Lower-case `ash.toml` dependencies reject unpinned git entries, resolve local git tags/revs to exact commits in `ash.lock`, `lock --check` detects drift, `fetch` materializes local git dependencies into XDG cache checkouts keyed by exact lockfile commits, and `ash check` plus explicit ordinary-file `ash run` discover validated vendored lock roots; trust preservation and direct fetched-cache root discovery remain deferred. |
| TASK-973 | Partial stdlib-separation slice | `vendor` copies package content from locked XDG cache checkouts into `vendor/ash/<package>/`, writes provenance entries, rejects unsafe package names and non-40-hex lock commits, `vendor --check` is read-only, offline `ash check src/main.ash` plus `ash run src/main.ash:main` have focused smoke coverage, and selected/explicit stdlib roots take precedence over the auto-discovered `vendor/ash` dependency namespace even when it contains a stdlib-shaped package. |
| TASK-974 | Reported | This report maps acceptance status, verification, review findings, changed files, and deferrals. |

## Acceptance Matrix

| Acceptance | Status | Evidence / Deferral |
| --- | --- | --- |
| A73-1 source install | Partial | `cargo test -p ashgrove --test task_968_source_install -- --nocapture`; real local source-root build, staged publish, dirty/unidentified rejection, git-like corrupt metadata fail-closed behavior, post-review clean-source/status-failure/dirty-digest regressions, and prepared source-shaped directory inputs are covered. Source archive release metadata and concrete runtime-support payload metadata remain deferred. |
| A73-2 binary tarball install | Partial | `cargo test -p ashgrove task_969 -- --nocapture`; fixture tarball only; executable-bit validation is covered, but release producer, full schema/archive-version validation, path/URL recording, and atomic publish remain deferred. |
| A73-3 equivalent toolchain contents | Partial | Source/tarball fixtures require `bin/ash`, `bin/ashgrove`, stdlib manifest/src, manifest, install record; no runtime/support metadata equivalence proof. |
| A73-4 immutable update | Partial | `cargo test -p ashgrove task_970 -- --nocapture`; update-from-existing selector test preserves old manifest, but real update install path remains thin. |
| A73-5 default switches metadata | Partial | `cargo test -p ashgrove task_970 -- --nocapture`; `cargo test -p ashgrove --test task_967_layout -- --nocapture` now proves real temp-root launcher shim execution honors explicit override before project pins before defaults, routes through a stable user-local dispatcher copy, and preserves selected-tool exit status. Project rewrite proof and packaged dispatcher lifecycle remain deferred. |
| A73-6 remove protections | Partial | `cargo test -p ashgrove task_971 -- --nocapture`; default, current-project, live-daemon, and running-manager protections are covered, including non-overridable daemon/running-manager cases. Broader cleanup policy and explicit configured known-project roots remain deferred. |
| A73-7 cleanup dry-run | Partial | `cargo test -p ashgrove task_971 -- --nocapture`; dry-run old-toolchain planning only. |
| A73-8 tag resolves to exact commit | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; local file git tags resolve to full commits and `fetch` materializes the exact lockfile commit under XDG cache even if a tag later moves. |
| A73-9 lock drift detection | Partial | `cargo test -p ashgrove task_972 -- --nocapture`; drift detected after manifest tag change. |
| A73-10 selected toolchain stdlib | Partial | `cargo test -p ash-engine task_968 -- --nocapture`; `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution cli_uses_explicit_stdlib_root_when_vendor_dependency_has_stdlib_module_name -- --exact --nocapture`; `cargo test -p ashgrove --test task_968_source_install -- --nocapture`; explicit stdlib root override works, auto-discovered project vendor packages cannot shadow that root, and the launcher public path passes the selected installed stdlib root to `ash`. Concrete runtime-support payload metadata remains deferred. |
| A73-11 trust/signing reserved metadata | Deferred | No preservation test or model yet. |
| A73-12 locked dependencies visible to `ash check`/`ash run` | Partial | `cargo test -p ash-engine task_972 -- --nocapture` proves explicit dependency roots are visible to the loader, `cargo test -p ashgrove task_973 -- --nocapture` proves vendored package content comes from locked cache checkouts, and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture` proves `ash check src/main.ash` and explicit ordinary-file `ash run src/main.ash:main` discover validated project `ash.lock`/`vendor/ash` roots without dependency-root env vars while preserving selected stdlib precedence over auto-discovered vendor roots. Direct fetched-cache roots remain deferred. |

## Focused Verification

Commands run in this worktree:

- `cargo test -p ashgrove task_966 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove task_967 -- --nocapture` - passed, 2 tests.
- `cargo test -p ashgrove --test task_968_source_install -- --nocapture` - passed, 17 tests after adding review regressions for real local source-root builds, clean no-lock roots, git status failure, corrupt git metadata, dirty digest install IDs, and launcher-selected stdlib routing.
- `cargo test -p ash-engine task_968 -- --nocapture` - passed, 1 matching test.
- `cargo test -p ashgrove task_969 -- --nocapture` - initially failed because the unsafe tar fixture used `Header::set_path("../escape")`, which the tar crate rejects while building the fixture; after changing the fixture to a symlink entry, passed with 2 tests.
- `cargo test -p ashgrove task_970 -- --nocapture` - passed, 1 test.
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

## Independent Review Findings

An independent review agent inspected the diff and returned findings. Two code-level blockers have been remediated in this worktree:

- `ashgrove vendor` now validates lockfile package names before deriving vendor paths, and `task_973_vendor_rejects_lockfile_package_name_path_traversal` proves `../escape` is rejected without creating an escaped provenance file.
- `Manifest::lock_text` now serializes typed lockfile structs through `toml::to_string`, and `task_972_lock_serializes_dependency_values_without_toml_injection` proves crafted dependency values remain escaped data instead of malformed TOML or injected package tables.

The remaining findings are accepted as current deferred gaps, not dismissed:

- TASK-974 closeout evidence was missing before this report.
- Source install now builds real local source roots and publishes through the staged toolchain path, but source archive release metadata and concrete runtime-support payload metadata remain undefined.
- Git dependency work now materializes XDG cache checkouts and vendor package content for local git fixtures, and `ash check` plus explicit ordinary-file `ash run` consume project `ash.toml`/`ash.lock` plus default `vendor/ash` roots; direct fetched-cache root discovery remains deferred.
- Remove/cleanup safety now has focused current-project and live-daemon state protection, but cleanup execution/cache/orphan/project planning remains incomplete.
- Tarball validation now checks executable permissions for required binaries on Unix, but does not yet check full schemas or provide a producer.
- TASK-967 real launcher shims are now implemented for temp-root/user-local `ash` and `ashgrove` scripts through a stable user-local dispatcher copy. Launcher-selected stdlib routing has TASK-968 coverage; release packaged dispatcher lifecycle remains tracked under later release packaging work.
- Lock/vendor format is still too thin for full reproducible offline deployment beyond local git cache checkout, package content copy, remediated escaping, package-name validation, and full-commit validation.
- Status/changelog surfaces were stale before the current reconciliation edits.

## Deferred Gaps

- Source archive release metadata and concrete runtime-support payload metadata for source installs.
- Public release tarball producer plus full schema, version/id, path/URL recording, digest, and atomic publish validation.
- Release packaging and lifecycle policy for the stable user-local dispatcher copy.
- Full metadata models preserving reserved trust/signing fields.
- Full cleanup execution, cache/orphan handling, and configured known-project root protection.
- Remote-authenticated git fetch policy, trust metadata preservation, and direct fetched-cache dependency root discovery.
- Direct fetched-cache dependency-root discovery without vendoring.
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
