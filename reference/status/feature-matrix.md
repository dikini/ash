---
id: ref.status.feature_matrix
title: Reference Slice 2 Feature Matrix
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 710340f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-998-reference-agent-cards-and-context-pack.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
    - ref.runtime.index
    - ref.stdlib.result
    - reference/status/alpha-limitations.md
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - SPEC-075 changes
  - reference/tools/** changes
  - reference/runtime/** changes
  - reference/stdlib/** changes
  - reference/agents/** changes
  - Phase closeout changes reference policy
---

# Reference Slice 2 Feature Matrix

| Feature | Slice 2 status | Stability | Reference page | Evidence note |
| --- | --- | --- | --- | --- |
| Pure functions | current-partial | alpha | [functions](../language/functions.md) | Basic pure behavior only; not full language manual. |
| Effect rows and provider admission | current-partial | alpha | [runtime admission](../runtime/admission.md) | Target authority is projected through admitted rows/provider profiles, not public tower carriers. |
| Process/channel helpers | current-partial | alpha | [RuntimeKernel](../runtime/README.md) | Current process guidance is through checked helpers and runtime evidence, not public `Proc` tower APIs. |
| Application runtime reports | current-partial | alpha | [RuntimeKernel](../runtime/README.md) | Entry execution reports are application runtime metadata over checked target functions. |
| Productive stdlib helpers | current-partial | alpha | [examples](../examples/README.md) | Current stdlib guidance comes from checked target files/examples; historical tower pages are not current APIs. |
| Result | current-partial | alpha | [Result stdlib](../stdlib/result.md) | Domain `Ok`/`Err` values remain separate from operational bottom. |
| Getting-started journey | current-partial | alpha | [getting started](../getting-started/README.md) | Thin reader journey links into subsystem pages and avoids copying full policy. |
| Ash CLI | current-partial | alpha | [Ash CLI](../tools/cli.md) | Command reference is evidence-bound to current help and docs surfaces. |
| Ashgrove toolchain manager | current-partial | alpha | [Ashgrove](../tools/ashgrove.md) | Install/update/list/current/default/remove/cleanup/project/vendor/trust/source-payload procedures documented with non-goals. |
| RuntimeKernel | current-partial | alpha | [RuntimeKernel](../runtime/README.md) | One-shot and local daemon reference pages preserve authority, artifact, admission, reload, and policy-profile boundaries. |
| Reference maintenance | current | alpha | [maintenance](../maintenance/README.md) | Metadata, staleness, refresh, triage, release, and agent-card procedures exist under `reference/maintenance/`. |
| Agent cards | current | alpha | [agent context pack](../agents/context-pack-index.md) | Derivative cards link back to canonical pages and common-confusion warnings. |
| Alpha limitations | current | alpha | [alpha limitations](alpha-limitations.md) | Centralized limitations status for Slice 2 no-overclaim boundaries. |
| Reference metadata validator | current | alpha | [verification evidence](verification-evidence.md) | Frontmatter/path/link/ID checks cover the full reference tree; staleness checker supports `--slice reference-slice-2`. |

Historical pages retained for old links: [Act](../language/effects-act.md),
[Proc](../language/processes-proc.md), [Workflow](../language/workflows.md),
[generalized do](../language/generalized-do.md), [tower](../language/tower.md), and
[stdlib tower](../stdlib/README.md). They are superseded reference records after Phase 201 and
must not be used as target-source guidance.
