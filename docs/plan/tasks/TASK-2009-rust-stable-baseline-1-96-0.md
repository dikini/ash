# TASK-2009: Shared Rust and Verus Baseline 1.96.0

**Status:** Complete

## Description

Set Ash and the pinned official Verus release to one common Rust 1.96.0 baseline for the workspace
and its separately managed fuzzing and benchmark crates.

## Requirements

- Set every active Ash `rust-version` requirement to `1.96.0`.
- Pin the workspace's normal Cargo invocation to Rust 1.96.0 without changing the user's global
  rustup default.
- Pin the isolated Verus release to the same Rust 1.96.0 requirement.
- Do not rewrite historical audit evidence.
- Verify workspace metadata and the affected standalone manifests with Rust 1.96.0.

## Completion Checklist

- [x] Workspace, fuzz, and benchmark manifests declare 1.96.0.
- [x] `rust-toolchain.toml` selects 1.96.0 for normal workspace commands.
- [x] Cargo metadata accepts each active manifest under Rust 1.96.0.
- [x] CHANGELOG and PLAN-INDEX record the baseline update.
- [x] Rust 1.96 Clippy accepts the workspace after nine behavior-preserving, version-exposed
  idiom cleanups (three match-guard collapses, three key sorts, panic-string token loop rewrite,
  constructor-key iteration, and decreases-option zip).

## Verification Evidence

- Focused Rust 1.96 `cargo clippy -- -D warnings` checks for `ash-doc-tests`, `ash-core`,
  `ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`, and `ash-cli` initially exposed the
  nine idiom lints addressed above; all pass after the minimal rewrites.
