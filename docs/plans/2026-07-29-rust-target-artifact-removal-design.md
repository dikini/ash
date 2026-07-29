# Rust Target Artifact Removal Design

## Goal

Remove every tracked Cargo build-output directory from the repository and prevent any future
`target/` directory from being added to version control.

## Current State

The repository tracks two Cargo-output trees: `crates/ash-bench/target/` (3 files) and
`crates/ash-fuzz/target/` (582 files). The root `target/` directory is already ignored, but
the nested paths are not covered consistently. A modified tracked build-metadata file therefore
blocked the pre-push hook.

## Design

1. Replace the path-specific benchmark ignore rule with one repository-wide `target/` rule.
   Git ignore patterns without a slash match directories at every level, so this covers the root
   workspace target and all present or future crate-local Cargo targets.
2. Remove the two existing target trees from Git's index while preserving the local files as
   ignored build cache.
3. Add a focused repository-policy test that fails when any tracked path is inside a `target/`
   directory and confirms the global ignore rule ignores both root and nested paths.
4. Record the maintenance change in a task record, the plan index, and `CHANGELOG.md`.

## Non-goals

- Removing directories merely named `target` that are not Cargo build output.
- Deleting local build caches from developers' worktrees.
- Changing Cargo's target-directory configuration.

## Verification

The new test is written first and observed failing while tracked target files remain. After the
index removal and ignore-rule update, it must pass together with the repository's relevant
documentation and pre-commit checks. `git status` must show no tracked `target/` modifications.
