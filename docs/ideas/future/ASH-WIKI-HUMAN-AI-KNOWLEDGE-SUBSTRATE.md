---
status: drafting
created: 2026-04-20
last-revised: 2026-04-20
related-plan-tasks: []
tags: [wiki, documentation, metadata, lint, ai, onboarding, knowledge-base, static-site, obsidian, pandoc, future]
---

# FUTURE-004: Ash Wiki as Human/AI Shared Knowledge Substrate

## Problem Statement

Ash already has a large corpus of specs, design notes, plans, tasks, ideas, and historical documents. The corpus is rich and cross-linked, but it does not yet provide a single coherent answer to questions such as:

- what is current versus historical?
- what is normative versus exploratory?
- what has been superseded, partially superseded, or merely gone stale?
- where does implementation drift from spec, and why?
- how should a human or an AI agent onboard into Ash today?
- how can the same corpus remain useful both as static documentation and as a substrate for AI-native workflows?

A plain wiki is not enough. A chat-over-docs layer is also not enough. Ash needs a static-first knowledge substrate that remains readable as ordinary markdown while also supporting machine-auditable integrity checks, conversational explanation, and agent-facing onboarding/service surfaces.

## Scope

- **In scope:**
  - a static-first wiki layer over the existing Ash corpus
  - metadata, navigation, supersession tracking, and synthesis pages
  - explicit documentation-integrity and drift-lint questions
  - AI-queryable / workflow-queryable interfaces over the same corpus
  - human and agent onboarding surfaces derived from the same source material
  - compatibility with Obsidian, Knot/Knotty, Pandoc, and browser rendering
- **Out of scope:**
  - replacing canonical `docs/spec/` contracts with wiki summaries
  - requiring a database-backed web application to make the corpus usable
  - committing to one browser UI, one vector store, or one retrieval implementation
  - immediate implementation of all query, lint, and rendering services
- **Related but separate:**
  - AI-native workflow substrate work in [FUTURE-002](AI-NATIVE-WORKFLOWS.md)
  - agent benchmark and pressure-test families in [FUTURE-003](AGENTIC-WORKFLOW-EXEMPLARS.md)
  - spec processor / Pandoc-compatible markdown work in the spec-processor phases

## Current Understanding

### What we know

- Ash should keep canonical specifications separate from higher-level synthesis and navigation.
- The wiki should be static-first: durable markdown files, git-friendly, tool-friendly, and renderable without a running service.
- The wiki must answer stronger questions than ordinary documentation navigation, especially around staleness, supersession, implementation drift, and ownership of unresolved gaps.
- Querying and analysis should be done by Ash/AI workflows over the corpus, not by baking dynamic behavior into page content itself.
- The same corpus should serve both humans and AI agents, potentially through different interfaces.
- AI friendliness must be structural: encoded in metadata, indexes, and stable interfaces, not only in hidden prompts.
- The Karpathy-style LLM wiki pattern is useful as inspiration because it preserves static markdown and compounding knowledge, but Ash needs a stronger emphasis on authority, auditability, and serviceable exports.

### What we're uncertain about

- What the minimal metadata schema is for useful linting without creating high-friction authoring.
- Which parts of the wiki should be hand-authored versus computed views.
- Whether onboarding bundles should be persisted as files, computed on demand, or both.
- How much of the query/runtime surface should be Ash-native first versus external tooling first.
- How to stage rollout so that static usability improves immediately, before the full service/query layer exists.

## Design Dimensions

| Dimension | Option A | Option B | Option C |
|-----------|----------|----------|----------|
| Corpus organization | Keep existing folders, add metadata only | Add dedicated wiki synthesis directories alongside canonical docs | Move everything into a single wiki tree |
| Query model | Plain search over markdown | AI workflows over static corpus + derived indexes | App-specific database / RAG-first layer |
| Audit model | Style/frontmatter lint only | Documentation-integrity + drift audits + evidence records | Fully inferred truth from retrieval/runtime heuristics |
| Human/AI interface split | One identical interface for all clients | Same corpus, different human and AI surfaces | AI-only service layer with human docs as fallback |
| Render strategy | Obsidian only | Markdown source + HTML/Pandoc render + browser chat hooks | Web app as primary artifact |

## Proposed Approaches

### Approach 1: Static-First Shared Knowledge Substrate

**Description:**
Keep the existing corpus as plain markdown in the repo. Add explicit metadata, supersession tracking, audit records, and computed index/view artifacts. Use AI/Ash workflows to query, explain, and audit the corpus without making the static files depend on a runtime-only application.

**Pros:**
- Preserves repo and editor friendliness.
- Makes authoritative and historical distinctions explicit.
- Supports Obsidian/Knot/Pandoc/static HTML naturally.
- Gives agents a stable, inspectable substrate.
- Degrades gracefully when AI/runtime services are unavailable.

**Cons:**
- Requires discipline around metadata maintenance.
- Needs derived indexes/views to stay synchronized with source docs.
- Query quality depends on the quality of the graph/registry, not just page text.

**Questions:**
- Which metadata must be authored versus generated?
- What is the minimum viable derived index set?

### Approach 2: App-First Interactive Knowledge Base

**Description:**
Treat the wiki primarily as a dynamic browser application, with static markdown becoming a secondary export.

**Pros:**
- Faster path to rich in-browser interaction.
- Easier to hide internal complexity behind custom UI.

**Cons:**
- Violates the requirement that the corpus stay statically usable.
- Risks making the system opaque to both humans and agents.
- Harder to audit and preserve as durable repo artifacts.

**Questions:**
- Can app-only features ever remain trustworthy without file-backed outputs?

## Candidate Product / Platform Framing

The Ash wiki should be treated as:

1. a documentation product
2. an audit/integrity product
3. a conversational/analytic product
4. an AI onboarding / library-service product
5. an internal platform primitive for Ash-native tooling

These are distinct uses of one corpus, not five unrelated systems.

## Initial Service Surfaces

Promising first-class query surfaces include:

- explain a topic for a given audience
- trace historical evolution and supersession
- show current normative surface for a subsystem
- audit documented drift and stale artifacts
- map spec -> plan -> task -> implementation/test lineage
- produce onboarding bundles for fresh agents or contributors
- surface examples and implementation suggestions grounded in the corpus

## Related Explorations

- [FUTURE-002: AI-Native Workflows and Generated Ash Programs](AI-NATIVE-WORKFLOWS.md)
- [FUTURE-003: Agentic Workflow Exemplars](AGENTIC-WORKFLOW-EXEMPLARS.md)
- [FUTURE-001: First-Class Workflows](FIRST-CLASS-WORKFLOWS.md)
- [OTP-002: Ash OTP Design Considerations](../otp/OTP-002-ash-otp-design.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-20 | Treat the wiki as a stronger substrate than static cross-linked pages. | Ash needs shared human/AI project memory and queryable integrity, not only browsing. |
| 2026-04-20 | Keep the wiki static-first and canonical-adjacent rather than replacing canonical docs. | Canonical specs must remain authoritative and directly inspectable. |
| 2026-04-20 | Treat lint/audit as first-class wiki outputs. | Staleness, supersession, and drift are central Ash questions, not secondary hygiene checks. |

## Next Steps

- [ ] Draft a formal Ash wiki design document covering layers, interfaces, and trust model.
- [ ] Draft a spec for metadata, authority states, supersession, audit findings, and service exports.
- [ ] Draft an implementation plan that stages static corpus improvements before dynamic workflow services.
- [ ] Decide whether the first implementation target is a repository-local registry generator, a lint/audit engine, or an onboarding/query service.
