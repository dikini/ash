# DESIGN-042: Reference Corpus and Documentation Governance

**Status:** Draft design note — promoted to normative draft by [SPEC-071](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md) and rollout plan [PLAN-120](../plan/PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
**Date:** 2026-05-23
**Related:** [DESIGN-029](DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE.md), [DESIGN-035](DESIGN-035-DOCUMENTATION-CORPUS-GOVERNANCE.md), [SPEC-045](../spec/SPEC-045-ASH-WIKI.md), [SPEC-071](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)

## 1. Summary

Ash needs two documentation surfaces with different jobs. The existing `docs/` tree remains the working and historical corpus: design notes, specs, plans, tasks, audits, rationale, supersession trails, and implementation evidence. A separate top-level `reference/` corpus will provide the current curated reading surface for humans and AI agents.

The reference corpus is not a replacement for `docs/`. It is a maintained projection of live code, implemented specs, examples, and known limitations. It must be structured, metadata-rich, cross-linked, verifiable, and written in a conservative tone that separates current truth from historical rationale and future aspiration.

## 2. Motivation

The Ash documentation corpus has high value because it preserves reasoning traces, phase decisions, failed alternatives, status reconciliations, and evidence. That value makes `docs/` a poor user-facing reference surface:

- current behavior is mixed with historical plans;
- implemented MVP status is spread across specs, plan rows, tasks, changelog, examples, and code;
- design notes may remain useful while no longer being normative;
- AI agents can retrieve stale or superseded context unless authority and freshness are explicit;
- humans need a coherent route through the language without replaying 100+ phases of project history.

The solution is not to rewrite history. The solution is a separate curated reference spine backed by explicit metadata, crosslinks, and maintenance tooling.

## 3. Design principles

1. **Preserve `docs/` as history and working memory.** Do not flatten design history into current truth.
2. **Create a separate reference surface.** Use a top-level `reference/` tree for curated human/agent material.
3. **Reference is canonical-adjacent, not canonical replacement.** Specs remain normative where current; reference pages project specs plus live implementation and limitations.
4. **One semantic spine, multiple audience affordances.** Humans and agents share the same concept pages; agent packs add retrieval metadata and common-confusion warnings rather than duplicating semantics.
5. **Metadata is the maintenance substrate.** Every reference page carries stable identity, authority, status, verification, dependency, and refresh metadata.
6. **Crosslinks are typed.** A link to a normative spec is not the same as a link to historical rationale or implementation evidence.
7. **Docs are developed artifacts.** Public semantic changes require reference impact classification and drift checks during task closeout.
8. **Tone is conservative.** State what Ash is now, what is implemented MVP, what is alpha, what is known incomplete, and what is historical.

## 4. Corpus split

| Corpus | Location | Purpose | Audience | Mutation style | Authority |
| --- | --- | --- | --- | --- | --- |
| Working/historical corpus | `docs/` | Ideas, design notes, specs, plans, tasks, audits, historical rationale, implementation evidence | implementers, reviewers, auditors, future archaeology | append, patch, supersede, annotate | mixed: specs may be normative; plans/tasks are execution history |
| Curated reference corpus | `reference/` | Current readable Ash language/library/tool/runtime reference and agent context packs | humans and AI agents | maintained, regenerated, verified | canonical-adjacent projection of specs + code + tests + limitations |

`docs/` remains the place for design packets and phase work. `reference/` becomes the place a new human or agent starts when asking “what is Ash now?”

## 5. Proposed `reference/` layout

```text
reference/
  README.md
  INDEX.md
  META.md
  methodology.md
  style-guide.md
  authority.md
  status.md

  language/
    README.md
    modules-and-imports.md
    functions.md
    values-and-types.md
    pattern-matching.md
    effects-act.md
    processes-proc.md
    workflows.md
    generalized-do.md
    policies-and-capabilities.md
    type-computation.md
    errors-and-failure.md

  stdlib/
    README.md
    prelude.md
    act.md
    proc.md
    workflow.md
    result.md
    option.md
    list.md
    map.md
    string.md
    json.md
    http.md
    llm.md
    runtime.md

  tools/
    README.md
    cli.md
    repl.md
    lsp.md
    mcp.md
    lint.md
    formatter.md
    test-runner.md
    daemon.md
    diagnostics.md

  runtime/
    README.md
    runtime-kernel.md
    artifacts.md
    admission-authority.md
    daemon-control.md
    providers.md
    provenance.md

  implementation/
    README.md
    crate-map.md
    parser.md
    typechecker.md
    engine.md
    interpreter.md
    amir-bytecode.md
    testing.md

  agents/
    README.md
    context-pack-index.md
    ash-concept-map.md
    task-orientation.md
    common-confusions.md
    retrieval-policy.md
    cards/

  examples/
    README.md
    hello-world.md
    act-proc-workflow.md
    capability-provider.md
    daemon-workflow.md
    type-computation.md

  status/
    README.md
    feature-matrix.md
    known-limitations.md
    drift-report.md
    verification-evidence.md
```

This layout is a target, not a one-shot migration requirement. PLAN-120 pilots a vertical slice first.

## 6. Authority model

Reference pages must expose their authority chain. The default precedence is:

1. live code plus passing tests for actual behavior;
2. current implemented-MVP specs for intended normative behavior;
3. reference pages for curated explanation;
4. plans/tasks for implementation history and evidence;
5. design notes and idea documents for rationale and alternatives;
6. changelog for release-facing change history.

When live behavior and a current spec disagree, the reference page must not silently choose one. It must mark a drift finding and link to a follow-up task or drift report.

## 7. Metadata model

Each reference page should carry frontmatter with at least:

```yaml
---
id: ref.language.act
title: Act
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: runtime-tower
last_verified: YYYY-MM-DD
verified_against:
  git_commit: <commit>
  specs: []
  tasks: []
  code: []
  tests: []
  examples: []
related:
  depends_on: []
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale: []
refresh_trigger: []
agent:
  retrieval_tags: []
  common_confusions: []
---
```

SPEC-071 owns the exact field contract, allowed values, and validation rules.

## 8. Crosslinking model

Links must be typed by role:

| Link role | Meaning |
| --- | --- |
| Normative spec | Contract that should define intended behavior |
| Current implementation | Code path that implements or exposes behavior |
| Evidence | Tests, task closeout, audits, or commands proving status |
| Historical rationale | Design notes, ideas, rejected alternatives, phase decisions |
| Derivative | Teaching pages, agent cards, generated indexes |
| Limitation | Known incomplete, alpha-only, deferred, or drift item |

A page may contain many links, but it must not make readers infer which link is authoritative.

## 9. Human and agent audiences

Human reference pages should prioritize:

- concise conceptual summary;
- current syntax and semantics;
- small normative examples;
- practical limitations;
- links to deeper rationale.

Agent material should prioritize:

- stable concept IDs;
- dependency order;
- retrieval tags;
- common confusion warnings;
- “must check before editing” links;
- forbidden stale claims;
- context packs generated from the same reference spine.

Agent material must not fork semantics. It points back to canonical reference pages.

## 10. Methodology and tone

Reference prose should be:

- current-tense for current behavior;
- explicit about status and stability;
- conservative about implementation claims;
- short, structured, and terminal-readable;
- free of phase-story unless history is necessary;
- direct about limitations and drift;
- careful not to market Ash as a different thing than it is.

Ash is the programming language. Governance, auditability, business processes, and mixed human/agent actors are motivating requirements and design pressure, not a replacement identity.

## 11. Maintenance tooling direction

The first toolkit should be static and repo-local:

```text
tools/reference/
  inventory.py
  check_frontmatter.py
  check_links.py
  check_authority.py
  check_code_paths.py
  check_examples.py
  check_cli_docs.py
  generate_stdlib_index.py
  generate_feature_matrix.py
  stale_derivatives.py
```

The tools should initially validate structure and detect drift. They should not require a dynamic service, database, or browser UI.

## 12. Rollout strategy

Start with a pilot slice rather than a corpus rewrite. The recommended pilot is the Pure / Act / Proc / Workflow tower because it exercises language semantics, stdlib, runtime, examples, and agent confusion risks.

PLAN-120 sequences the rollout:

1. packet creation;
2. inventory and metadata pilot;
3. reference skeleton and authority/style guides;
4. Pure/Act/Proc/Workflow pilot pages;
5. agent concept cards/context-pack index;
6. static validator MVP;
7. example/status classification;
8. closeout drift report and next-slice recommendation.

## 13. Non-goals

- Do not move or rewrite the existing `docs/` corpus wholesale.
- Do not make `reference/` a wiki replacement or dynamic knowledge service in the first slice.
- Do not duplicate semantic explanations separately for humans and agents.
- Do not claim stale historical docs are wrong merely because they are historical.
- Do not require the reference corpus to stabilize all Ash APIs before it can start.

## 14. Open questions for PLAN-120

1. Should `reference/` be the permanent top-level name, or should the pilot compare `reference/`, `manual/`, and `knowledge/`? DESIGN-042 recommends `reference/` unless the pilot finds a blocker.
2. Should reference pages use only YAML frontmatter or allow sidecar metadata files for generated pages? SPEC-071 should allow sidecars for generated pages but require equivalent fields.
3. Which examples are normative enough for the first reference slice? TASK-952 should classify rather than assume.
4. How strict should drift gates be before reference pages block implementation closeout? PLAN-120 should start with warning/report mode, then promote selected checks to gates after the pilot.
