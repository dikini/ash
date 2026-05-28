# TASK-965: Ashgrove live install audit gate

## Status: Complete

## Summary

TASK-965 binds Phase 127 implementation to live repository seams before Rust changes. The implementation home is a new workspace member, `crates/ashgrove`, because the existing `crates/ash-cli` binary is the language CLI (`ash`) and already owns `ash daemon ...`, `check`, `run`, `trace`, `test`, `repl`, and `dot`.

The first slice uses user-local XDG roots, immutable toolchain directories, lower-case `ash.toml`, lockfile commit truth, and fail-closed destructive behavior. Network release discovery, registry dependencies, global installs, mandatory signing, and independent stdlib updates remain out of scope.

## Live Seams

| Area | Live files | Phase 127 binding |
| --- | --- | --- |
| Workspace binary layout | `Cargo.toml`, `crates/ash-cli/Cargo.toml`, `crates/ash-cli/src/main.rs` | Add `crates/ashgrove` as a workspace crate with public binary `ashgrove`. Keep language commands in `ash-cli`. |
| Ash CLI command boundary | `crates/ash-cli/src/main.rs`, `crates/ash-cli/src/commands/*` | `ashgrove` must not duplicate `ash check`, `ash run`, or daemon language/runtime commands. |
| Daemon control plane | `crates/ash-cli/src/commands/daemon.rs` | Daemon remains `ash daemon ...`. TASK-971 may add minimal toolchain identity/state sidecars under XDG state for removal protection, without adding an `ashd` semantic surface. |
| Stdlib source layout | `std/src/**`, `std/Cargo.toml` | Installed toolchains stage stdlib to `lib/ash/std/src` and generate `lib/ash/std/ash.toml`. |
| Current stdlib discovery | `crates/ash-engine/src/module_loader.rs`, `crates/ash-engine/src/entry.rs`, `crates/ash-cli/src/commands/run.rs`, `crates/ash-cli/src/commands/check.rs` | Replace hardcoded `env!("CARGO_MANIFEST_DIR")/../../std/src` assumptions with an explicit selected stdlib root seam. Continue supporting workspace fallback for developer runs. |
| Module imports | `crates/ash-engine/src/module_loader.rs` | Add locked dependency roots to import search alongside entry root, `ASH_LIBRARY_PATH`, and selected stdlib. |
| Build/release scripts | `scripts/check-rust-format.sh`, `scripts/check-rust-clippy.sh`, `scripts/check-rust-tests.sh`, `scripts/check-doc-tests.sh` | Add `scripts/package-ash-toolchain.sh` as the release-side tarball producer for TASK-969. |
| CLI tests | `crates/ash-cli/tests/*` | Add focused `crates/ashgrove/tests/task_966_*.rs` through `task_973_*.rs` using temp XDG roots and `assert_cmd`. |
| Metadata/TOML | no workspace TOML dependency today | Add workspace `toml` dependency and keep metadata structs in `crates/ashgrove` for first slice. Extract later only if another crate needs ownership. |
| Git | no git library dependency today | Shell out to `git` for TASK-972/TASK-973. Tests use local repositories, not network. |
| Tarballs | no tar/flate dependency today | Add workspace `tar` and `flate2` for `.tar.gz` package validation/production. Implement safe extraction in `crates/ashgrove`. |
| HTTP download | `url` exists, no HTTP client workspace dependency | First slice accepts `--url` syntax but rejects it as unsupported until a later authenticated download policy. Local `--path` tarballs are implemented. |
| XDG | no shared XDG helper today | Implement explicit `AshgrovePaths` in `crates/ashgrove`, honoring `HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_STATE_HOME`, and test overrides. |

## Frozen Policy

- Public standard-tool list for alpha: `ash` and `ashgrove` only.
- Developer/test binaries such as `ash-doc-tests`, fuzz targets, benches, and helper apps are not standard user tools.
- Existing sibling tools such as `ash-lsp`, `ash-lint`, and `ash-mcp` are not part of the required first-slice bundle; they may be staged later by explicit release policy.
- Daemon control remains `ash daemon ...`; no separate `ashd` binary is required.
- Toolchain id scheme: `ash-<package-version>+<target-triple>.<source-kind>.<short-commit-or-digest>`, for example `ash-0.1.0+x86_64-unknown-linux-gnu.source.abcdef123456`.
- Same exact toolchain id reinstall is a deterministic already-installed no-op. Same package version with different source commit/digest produces a distinct id. If an existing directory has the same id but mismatched metadata, installation fails closed.
- Dirty source and unidentified source installs are rejected unless their explicit override flags are present, and override state is recorded in `install-record.toml`.
- Trust/signing metadata fields are parsed as reserved data and preserved, but not enforced.

## Dependencies and Commands

Add these workspace dependencies when the owning Rust task starts:

- `clap` with derive support for the `ashgrove` CLI.
- `anyhow`, `thiserror`, `serde`, `serde_json`, `chrono`, `sha2`, `tempfile`, and `walkdir` from workspace where already available.
- `toml` for manifest/lock/toolchain metadata.
- `tar` and `flate2` for tarball packaging and extraction.
- `assert_cmd` and `predicates` as `crates/ashgrove` dev-dependencies.

Git operations use the system `git` command. Focused tests that need git must skip or fail with an explicit prerequisite diagnostic if `git` is unavailable; no network repository is required.

## Downstream Focused Verification

- TASK-966: `cargo test -p ashgrove task_966 -- --nocapture`
- TASK-967: `cargo test -p ashgrove task_967 -- --nocapture`
- TASK-968: `cargo test -p ashgrove task_968 -- --nocapture` and `cargo test -p ash-engine task_968 -- --nocapture`
- TASK-969: `cargo test -p ashgrove task_969 -- --nocapture`
- TASK-970: `cargo test -p ashgrove task_970 -- --nocapture`
- TASK-971: `cargo test -p ashgrove task_971 -- --nocapture` and `cargo test -p ash-cli task_971 -- --nocapture`
- TASK-972: `cargo test -p ashgrove task_972 -- --nocapture` and `cargo test -p ash-engine task_972 -- --nocapture`
- TASK-973: `cargo test -p ashgrove task_973 -- --nocapture`

Cheap per-task gates after focused tests:

```bash
cargo fmt --check
cargo check -p ashgrove
git diff --check
```

Closeout still owns broad workspace gates and must not mark inconclusive commands as passing.

## TASK-964 Packet Verification

TASK-964 packet files exist for SPEC-073, PLAN-122, TASK-964 through TASK-974, the spec index, PLAN-INDEX, and CHANGELOG. The packet is coherent as a planning/audit handoff and is not stale relative to this worktree; no redo is required.

## Checklist

- [x] Created `docs/plan/audits/TASK-965-ashgrove-live-install-audit-gate.md`.
- [x] Chose `crates/ashgrove` as the implementation home.
- [x] Mapped install, tarball, update, cleanup, remove, lock, fetch, and vendor seams to live files.
- [x] Froze the alpha public standard-tool list as `ash` and `ashgrove`.
- [x] Bound installed-stdlib refactor to `ash-engine` and `ash-cli` seams.
- [x] Bound daemon live-protection work to `ash-cli` daemon state and `ashgrove` removal planning.
- [x] Selected TOML/XDG/git/tar/HTTP/version policies for the first slice.
- [x] Chose toolchain id and collision policy.
- [x] Replaced TASK-966 through TASK-973 placeholder verification commands with focused non-zero commands.
