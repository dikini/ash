---
id: ref.agents.card.ash_cli
title: Ash CLI Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 7fc92f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    - crates/ash-cli/src/main.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - cargo run -p ash-cli -- --help
    - cargo run -p ash-cli -- run --help
    - cargo run -p ash-cli -- daemon --help
  examples:
    []
related:
  depends_on:
    - ref.tools.cli
  explains:
    - ref.runtime.kernel
    - ref.tools.ashgrove
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - reference/tools/cli.md changes
  - crates/ash-cli/src/** changes
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
---

# Ash CLI Card

canonical_page: ref.tools.cli
canonical_page_path: ../../tools/cli.md
dependency_order: tools-runtime-1
warning: Read the canonical page first. This card is derivative and must not expand the CLI beyond live-help evidence.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- ash
- reference-slice-2
- ash-cli
- check
- run
- trace
- test
- repl
- dot
- daemon
- runtime-kernel

## Must check before editing

- ../../tools/cli.md
- ../../runtime/README.md
- ../../runtime/kernel.md
- ../../runtime/daemon.md
- ../../status/runtime-kernel.md
- ../../../crates/ash-cli/src/main.rs
- ../../../crates/ash-cli/src/commands/run.rs
- ../../../crates/ash-cli/src/commands/daemon.rs
- ../../../docs/spec/SPEC-005-CLI.md
- ../../../docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md

## Forbidden stale claims

- `ash` defines Ashgrove install/update/dependency policy.
- `ash daemon` is a remote, multi-user, distributed, or production init daemon surface.
- File presence executes code.
- Provider or resource inventory grants authority without admission.
- CLI help-derived placeholder forms are copy/paste examples with concrete paths.
- Agent cards are normative specs.
