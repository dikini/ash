---
id: ref.status.known_limitations
title: Known Limitations
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
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
