---
id: ref.tools.index
title: Tools Reference Index
kind: index
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: toolchain
last_verified: 2026-06-03
verified_against:
  git_commit: 7cf576d
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-1019-reference-ash-test-daily-use.md
  code:
    - crates/ash-cli/src/main.rs
    - crates/ashgrove/src/main.rs
  tests:
    - cargo run -p ash-cli -- --help
    - cargo run -p ash-cli -- test --help
    - cargo run -p ashgrove -- --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.tools.cli
    - ref.tools.test
    - ref.tools.ashgrove
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ash-cli/src/** changes
  - crates/ashgrove/src/** changes
  - reference/tools/** changes
---

# Tools Reference Index

This section maps the current Alpha command-line tools. It is a reference surface, not a substitute for the implementation specs.

Command surfaces were checked against live help on 2026-06-03:

- `cargo run -p ash-cli -- --help`
- `cargo run -p ash-cli -- test --help`
- `cargo run -p ashgrove -- --help`

Use `cargo run -p ... --` from a repository checkout. After installing an Ash toolchain and putting the user-local launcher directory on `PATH`, the same command shapes are available as `ash ...` and `ashgrove ...`.

## Tools

- [Ash command map](cli.md): language CLI commands for checking, running, tracing, testing, REPL, graph output, and local daemon control.
- [Ash test](test.md): daily-use `ash test` reference for authored tests, metadata directives, filtering, output, property/small-world controls, and synthesized-test boundaries.
- [Ashgrove overview](ashgrove.md): user-local Ash toolchain and deployment manager.

## Ashgrove Procedures

- [Install](ashgrove/install.md)
- [Update](ashgrove/update.md)
- [List, current, and default](ashgrove/list-current-default.md)
- [Remove and cleanup](ashgrove/remove-cleanup.md)
- [Project dependencies](ashgrove/project-dependencies.md)
- [Vendor and deploy](ashgrove/vendor-deploy.md)
- [Trust and signing](ashgrove/trust-and-signing.md)
- [Source payload and local state](ashgrove/source-payload.md)

## Boundaries

Ashgrove is local-first and fail-closed. These pages do not claim support for a hosted registry, global/system installs, OS package-manager integration, arbitrary SemVer dependency solving, broad source-ignore glob flags, or unsigned release-index lookup.
