---
id: ref.status.ashgrove
title: Ashgrove Status
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: ashgrove
last_verified: 2026-06-01
verified_against:
  git_commit: 710340f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
    - crates/ash-engine/src/lib.rs
    - crates/ash-cli/src/main.rs
  tests:
    - cargo run -p ashgrove -- --help
    - cargo run -p ash-cli -- --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.status.index
    - ref.tools.ashgrove
    - ref.stdlib.index
  explains:
    - ref.tools.ashgrove.install
    - ref.tools.ashgrove.update
    - ref.tools.ashgrove.remove_cleanup
    - ref.tools.ashgrove.source_payload
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/** changes
  - reference/stdlib/** changes
  - reference/status/ashgrove.md changes
---

# Ashgrove Status

Ashgrove is documented here as an Alpha implemented MVP reference surface. The status is current for the command/help and policy boundaries checked in TASK-995, but still Alpha: behavior remains evidence-bound to the current repository commit and specs.

## Current Claims

| Area | Status | Evidence boundary |
| --- | --- | --- |
| Command surface | current | Live `ashgrove --help` exposes install/update/default/list/current/remove/cleanup/fetch/lock/vendor. |
| User-local toolchains | current | SPEC-073 Implemented MVP; no global/system install roots. |
| Source install/update | current | SPEC-074 source payload/local-state policy; nonignored payload changes fail closed. |
| Tarball install/update | current | Local tarball and explicit-digest URL boundaries; unsafe or mismatched archives fail before publish. |
| Selectors | current | List/current/default are local selector operations; project pins and defaults are fail-closed. |
| Remove/cleanup | current | Conservative local deletion with protected toolchains and bounded known-project reachability. |
| Git dependencies | current | Git-pinned `ash.toml` dependencies resolve to exact `ash.lock` commits and fetched/vendored roots. |
| Trust/signing | current MVP | Required evidence fails closed at implemented release/download/git boundaries; release-index-as-digest is not claimed. |

## Explicit Non-Goals

Ashgrove currently does not provide:

- hosted registry service;
- global/system installs;
- OS package-manager integration;
- arbitrary SemVer dependency solving;
- hosted release-channel discovery or bare version install/update;
- signed release-index-as-digest evidence;
- broad source-ignore glob CLI;
- automatic project rewriting during update;
- independent stdlib updates.

## Fail-Closed Boundaries

Ashgrove fails closed rather than guessing across trust/source boundaries: dirty nonignored source payload, source-archive attestation gaps, tarball digest/signature mismatch, unsafe archive entries, untrusted git protocols, credential-bearing origins, missing or mismatched lock signature evidence, missing selected toolchains, and unsupported release-index lookup.

## Open Follow-Ups

TASK-997 completed the older stdlib tower reference expansion. Phase 201 keeps those pages only as
historical routing records. TASK-998 derived agent cards from the canonical stdlib, Ashgrove, CLI,
and RuntimeKernel pages. TASK-999 owns final closeout validation and must verify derivative cards
still point back to canonical pages without forking semantics.
