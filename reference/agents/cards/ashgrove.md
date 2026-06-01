---
id: ref.agents.card.ashgrove
title: Ashgrove Card
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
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
    - crates/ash-engine/src/lib.rs
  tests:
    - cargo run -p ashgrove -- --help
    - cargo run -p ashgrove -- install --help
    - cargo run -p ashgrove -- update --help
    - cargo run -p ashgrove -- lock --help
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
    - ref.status.ashgrove
  explains:
    - ref.tools.cli
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
refresh_trigger:
  - reference/tools/ashgrove.md changes
  - reference/tools/ashgrove/** changes
  - reference/status/ashgrove.md changes
  - crates/ashgrove/src/** changes
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
---

# Ashgrove Card

canonical_page: ref.tools.ashgrove
canonical_page_path: ../../tools/ashgrove.md
dependency_order: tools-runtime-2
warning: Read the canonical page and relevant procedure subpage first. This card is derivative and must not invent Ashgrove policy.

## Use

Retrieve the canonical page first, then read the relevant procedure page before editing install, update, selector, cleanup, dependency, vendor, trust, or source-payload claims.

## Retrieval tags

- ash
- reference-slice-2
- ashgrove
- toolchain-manager
- install
- update
- cleanup
- lock
- fetch
- vendor
- fail-closed
- source-payload

## Must check before editing

- ../../tools/ashgrove.md
- ../../tools/ashgrove/install.md
- ../../tools/ashgrove/update.md
- ../../tools/ashgrove/list-current-default.md
- ../../tools/ashgrove/remove-cleanup.md
- ../../tools/ashgrove/project-dependencies.md
- ../../tools/ashgrove/vendor-deploy.md
- ../../tools/ashgrove/trust-and-signing.md
- ../../tools/ashgrove/source-payload.md
- ../../status/ashgrove.md
- ../../../crates/ashgrove/src/lib.rs
- ../../../crates/ashgrove/src/main.rs
- ../../../docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
- ../../../docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md

## Forbidden stale claims

- Ashgrove provides a hosted registry service.
- Ashgrove installs into global/system roots or integrates with OS package managers.
- Ashgrove supports arbitrary SemVer dependency solving from a registry.
- Bare version install/update or hosted release-channel discovery is implemented.
- Release-index signature metadata is accepted as digest evidence.
- Source-root ignore policy is a broad arbitrary user ignore-glob CLI.
- Ashgrove updates stdlib independently from the selected toolchain.
- Agent cards are normative specs.
