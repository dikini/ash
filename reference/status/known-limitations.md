---
id: ref.status.known_limitations
title: Known Limitations
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 710340f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.status.feature_matrix
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-070 changes
  - reference/runtime/** changes
  - reference/stdlib/** changes
  - Phase closeout changes reference policy
---

# Known Limitations

## Language/reference pilot

- Phase 124 covers only a vertical slice, not the full Ash language, stdlib, tools, or runtime reference.
- Reference pages are canonical-adjacent summaries, not replacements for specs.
- Full generated stdlib/API extraction is not implemented in this phase.

## Current behavior caveats preserved by the pilot

- Pure < Act < Proc < Workflow is the pilot reading order; no implicit tower lifts are claimed.
- Act is opaque runtime-managed state-threading effect, not Result.
- Effectful operations go through CapabilityProvider/runtime provider machinery.
- Generalized do does not provide blanket final-expression returns or automatic target inference.
- Historical examples are not silently promoted to normative-pass.
- Stdlib tower API pages document current public `std/src` names only; they do not invent implicit lifts, hidden constructors, or full generated API extraction.
- `Result<T, E>` is a domain value type. `Err` is not operational bottom, and `fail` is not implicit `Err` construction.

## RuntimeKernel Alpha limits

- File presence is not execution; daemon indexing and source discovery do not start workflow bodies by themselves.
- Provider/resource existence is not authority; RuntimeKernel admission must grant authority before user body execution.
- Verified RuntimeKernel artifacts are source/check-summary based Alpha summaries, not production deployment packages.
- Daemon reload affects future starts and does not hot-swap already admitted running instances.
- The Alpha daemon is local-only: no remote/multi-user daemon API, distributed scheduling, or production init-system integration is claimed.
