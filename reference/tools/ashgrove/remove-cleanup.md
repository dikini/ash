---
id: ref.tools.ashgrove.remove_cleanup
title: Ashgrove Remove and Cleanup
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: ashgrove
last_verified: 2026-06-01
verified_against:
  git_commit: e06944a
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - cargo run -p ashgrove -- remove --help
    - cargo run -p ashgrove -- cleanup --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.getting_started.cleanup
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/remove-cleanup.md changes
---

# Ashgrove Remove and Cleanup

`ashgrove remove` deletes a selected installed toolchain when safety checks allow it. `ashgrove cleanup` plans or performs conservative cleanup of Ash-owned local state.

Live help checked:

```bash
cargo run -p ashgrove -- remove --help
cargo run -p ashgrove -- cleanup --help
```

## Remove

Help-derived form:

```bash
ashgrove remove [OPTIONS] TOOLCHAIN_ID
```

Reference-only examples:

```bash
ashgrove remove TOOLCHAIN_ID
ashgrove remove --force TOOLCHAIN_ID
```

Removal refuses protected toolchains:

- the user default unless `--force` is provided and confirmation succeeds;
- the current project-pinned toolchain unless `--force` is provided and confirmation succeeds;
- a toolchain used by a live daemon;
- the toolchain that provides the running Ashgrove manager.

`--force` does not override live-daemon or running-manager protection. Those boundaries fail closed.

## Cleanup

Help-derived form:

```bash
ashgrove cleanup [OPTIONS]
```

Reference-only forms:

```bash
ashgrove cleanup --dry-run --cache --orphans --old-toolchains
ashgrove cleanup --project PROJECT --dry-run --cache
```

Cleanup options:

| Option | Meaning |
| --- | --- |
| `--project PROJECT` | Include the supplied project in selector and lockfile reachability analysis. |
| `--dry-run` | Print the plan without deleting. |
| `--cache` | Delete safe Ash-owned cache children such as rebuildable downloads/build/module/git cache entries. |
| `--orphans` | Delete invalid or unreachable Ash-owned entries inside Ash roots. |
| `--old-toolchains` | Delete unprotected old toolchains after confirmation checks. |

Cleanup preserves project-local `ash.toml` and `ash.lock`, project-pinned toolchains, default toolchains, live-daemon toolchains, running-manager toolchains, lockfile-referenced fetched checkouts, and vendor-provenance referenced cache state.

## Scope Boundary

Known projects are bounded to the current/supplied project plus registered project roots. Cleanup does not crawl the user's filesystem looking for Ash projects and does not act as a hosted registry garbage collector, remote rollback mechanism, or project rewrite tool.

## Non-Goals

Remove and cleanup do not provide global/system uninstall, OS package-manager integration, hosted registry garbage collection, arbitrary SemVer solving, broad source-ignore glob cleanup, or automatic project migration.
