---
id: ref.tools.ashgrove
title: Ashgrove Toolchain Manager
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
    - cargo run -p ashgrove -- --help
    - cargo run -p ashgrove -- install --help
    - cargo run -p ashgrove -- update --help
    - cargo run -p ashgrove -- cleanup --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.index
  explains:
    - ref.tools.ashgrove.install
    - ref.tools.ashgrove.update
    - ref.tools.ashgrove.list_current_default
    - ref.tools.ashgrove.remove_cleanup
    - ref.tools.ashgrove.project_dependencies
    - ref.tools.ashgrove.vendor_deploy
    - ref.tools.ashgrove.trust_signing
    - ref.tools.ashgrove.source_payload
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/** changes
---

# Ashgrove Toolchain Manager

`ashgrove` is Ash's user-local toolchain and local deployment manager. It installs coherent Ash toolchain bundles, switches selectors, removes or cleans local state conservatively, resolves git-pinned project dependencies, and vendors locked dependencies for offline deployment.

It is intentionally not a second language execution CLI. Use `ash` for checking, running, tracing, testing, REPL, graph output, and daemon control.

## Live Command Surface

Checked on 2026-06-01:

```bash
cargo run -p ashgrove -- --help
```

Help-derived installed form:

```bash
ashgrove <COMMAND>
```

Current commands:

| Command | Help summary | Detail page |
| --- | --- | --- |
| `install` | Install a toolchain from source or tarball | [Install](ashgrove/install.md) |
| `update` | Install a new toolchain and optionally switch the default | [Update](ashgrove/update.md) |
| `default` | Set the user default toolchain | [List, current, and default](ashgrove/list-current-default.md) |
| `list` | List installed toolchains | [List, current, and default](ashgrove/list-current-default.md) |
| `current` | Print the selected toolchain | [List, current, and default](ashgrove/list-current-default.md) |
| `remove` | Remove an installed toolchain | [Remove and cleanup](ashgrove/remove-cleanup.md) |
| `cleanup` | Plan or perform conservative cleanup | [Remove and cleanup](ashgrove/remove-cleanup.md) |
| `fetch` | Fetch git dependencies recorded in `ash.toml` | [Project dependencies](ashgrove/project-dependencies.md) |
| `lock` | Resolve or check `ash.lock` | [Project dependencies](ashgrove/project-dependencies.md) |
| `vendor` | Materialize locked dependencies for offline deployment | [Vendor and deploy](ashgrove/vendor-deploy.md) |

## Installed Shape

Ashgrove installs immutable toolchain directories under the user-local XDG data root. A valid toolchain includes `ash`, `ashgrove`, bundled stdlib metadata/source, runtime-support metadata, `manifest.toml`, and `install-record.toml`.

Stable launcher shims under the user-local bin directory dispatch to the selected installed toolchain. Selection is ordered by explicit override, project pin, then user default. Broken or incomplete selected toolchains fail closed.

## Procedure Pages

- [Install](ashgrove/install.md)
- [Update](ashgrove/update.md)
- [List, current, and default](ashgrove/list-current-default.md)
- [Remove and cleanup](ashgrove/remove-cleanup.md)
- [Project dependencies](ashgrove/project-dependencies.md)
- [Vendor and deploy](ashgrove/vendor-deploy.md)
- [Trust and signing](ashgrove/trust-and-signing.md)
- [Source payload and local state](ashgrove/source-payload.md)

## Explicit Non-Goals

The current Alpha/MVP boundary excludes:

- hosted Ash registry service;
- global/system install roots;
- OS package-manager integration;
- arbitrary SemVer dependency solving from a registry;
- hosted release-channel discovery or bare version install/update;
- signed release-index-as-digest evidence;
- broad arbitrary source-ignore glob CLI;
- automatic user-project rewriting during toolchain updates;
- independent stdlib updates decoupled from toolchain updates.

Commands that would require those policies fail closed rather than guessing, crawling broad user state, or performing ad hoc network lookup.
