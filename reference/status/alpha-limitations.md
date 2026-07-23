---
id: ref.status.alpha_limitations
title: Alpha Limitations
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 01bafb4
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    []
  tests:
    - frontmatter full reference validation
    - staleness reference-slice-2 audit
  examples:
    []
related:
  depends_on:
    - ref.status.feature_matrix
    - ref.status.ashgrove
    - ref.status.runtime_kernel
    - ref.stdlib.index
  explains:
    - ref.status.known_limitations
  supersedes:
    - ref.status.known_limitations
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/tools/** changes
  - reference/runtime/** changes
  - reference/stdlib/** changes
  - Phase 130 closeout changes accepted scope
---

# Alpha Limitations

This page records Reference Slice 2 alpha boundaries. It is a status surface, not a tutorial and not a replacement for the linked specs.

## Reference corpus limits

- Reference Slice 2 covers getting-started, Ash CLI/Ashgrove, RuntimeKernel, maintenance, status,
  and agent-card pages. Historical tower pages are retained only for old links after Phase 201. It
  does not migrate the entire historical `docs/` corpus.
- Reference pages are canonical-adjacent summaries. Normative behavior remains in specs and implementation evidence.
- The maintenance checker is path-based. It derives `needs-inspection` from evidence paths and refresh triggers; it is not a semantic staleness proof.
- Version and release metadata remain advisory until an Alpha release line exists. `verified_against.git_commit` is the primary freshness anchor.

## Toolchain and Ashgrove limits

- No hosted registry service is claimed.
- No global or system install root is claimed.
- No OS package-manager integration is claimed.
- No arbitrary SemVer solver is claimed.
- No signed release-index-as-digest resolver is claimed beyond the current fail-closed trust/signing evidence.
- Source-payload policy excludes ignored local checkout state from source-root payload identity, but nonignored source payload changes remain fail-closed.

## RuntimeKernel limits

- The daemon is local-only. No remote or multi-user RuntimeKernel daemon API is claimed.
- No distributed scheduling is claimed.
- No production init-system integration is claimed.
- Reload affects future starts; it does not hot-swap already admitted running instances.
- File presence is not execution, and provider/resource existence is not authority.

## Historical tower limits

- Older tower pages are historical after Phase 201 and are not target-source guidance.
- Historical tower terms must not be copied into new Ash source examples.
- `Result<T, E>` models domain success/failure. `Err` is not operational bottom, and `fail` is not implicit `Err` construction.
- Full generated API extraction remains out of scope.
