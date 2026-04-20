# DESIGN-029: Ash Wiki Knowledge Substrate

## Status: Draft

## Overview

Design a static-first knowledge substrate over the Ash project corpus that remains usable as ordinary markdown documentation while also supporting structured metadata, supersession tracking, documentation-integrity audits, onboarding bundles, and AI/Ash workflow query surfaces.

The design is deliberately canonical-adjacent rather than canonical-replacing. `docs/spec/` remains the normative contract surface. The wiki layers navigation, synthesis, historical archaeology, drift/audit evidence, and serviceable exports over the broader corpus.

This design is influenced by the LLM-wiki pattern of durable markdown knowledge bases, but extends it with stronger requirements around authority, trust, auditability, and human/AI dual use.

## Problem Statement

Ash now has enough specs, plans, tasks, notes, and historical reasoning that navigation by folder structure alone is no longer sufficient. The project needs a corpus model that can answer:

- which artifacts are normative, current, historical, exploratory, or stale?
- which artifacts supersede which others, and at what scope?
- which implementation surfaces correspond to a spec or plan?
- where do spec, design, plan, task, and code drift apart?
- how should a new contributor or agent onboard into the project?
- how can the same material remain useful in Obsidian, plain markdown, static HTML, and browser-based AI conversation?

A conventional wiki solves only the cross-linking problem. Ash needs a stronger substrate that also supports explicit audit and service layers.

## Goals

1. Preserve static usability of the corpus in markdown-first tools.
2. Keep canonical specifications authoritative and distinguish them from synthesis or commentary.
3. Make status, authority, health, and supersession explicit and machine-usable.
4. Support documentation-integrity and drift audits as first-class artifacts.
5. Support AI/human onboarding and query services from the same source corpus.
6. Provide a path to browser conversation and Ash-native query workflows without making the corpus dependent on a single runtime.

## Non-Goals

This design does not attempt to:

- replace `docs/spec/` with wiki summaries
- require a vector database or RAG service to make the corpus useful
- define a single UI implementation for browser chat
- fully solve semantic code/spec drift inference in the first iteration
- collapse all document categories into one flat wiki namespace

## Design Decisions

### D1: Static-First, Serviceable Corpus

The core artifact remains a versioned directory of markdown files inside the repository. The corpus must remain useful under these conditions:

- opened directly in an editor
- browsed in Obsidian or Knot/Knotty
- searched with grep/ripgrep-like tools
- rendered to static HTML/PDF via Pandoc or equivalent
- inspected by AI agents via file access and structured indexes

No critical meaning should exist only inside a running web application.

### D2: Canonical-Adjacent, Not Canonical-Replacing

Canonical documents retain their existing roles:

- `docs/spec/` and selected `docs/reference/` artifacts define normative behavior
- `docs/design/` and `docs/plans/` capture rationale and intended execution
- `docs/plan/tasks/` capture concrete work items
- `docs/ideas/` and `docs/notes/` capture exploratory or future material

The wiki layer does not replace those categories. Instead, it overlays:

- metadata
- cross-links
- supersession records
- synthesis/topic pages
- audit/drift records
- onboarding and query bundles

### D3: Five-Layer Architecture

The Ash wiki is organized conceptually into five layers.

#### Layer 1: Canonical Artifact Layer

The source corpus: specs, reference docs, designs, plans, tasks, notes, ideas, historical records.

#### Layer 2: Wiki Knowledge Layer

Human-readable organization and synthesis over the corpus:

- topic pages
- subsystem pages
- historical narratives
- current normative maps
- supersession pages
- drift/audit pages
- onboarding pages

These remain markdown artifacts.

#### Layer 3: Computed Index Layer

Derived machine-usable structures generated from the corpus:

- document registry
- anchor/section registry
- term/concept index
- authority/status registry
- supersession graph
- drift finding registry
- implementation trace registry
- onboarding bundle manifests

These may be emitted as JSON/YAML/markdown but are not the canonical source of truth.

#### Layer 4: Workflow / Query Layer

Ash workflows and AI workflows that operate over the corpus and derived indexes:

- explain topic
- trace history
- show authority surface
- audit drift
- map lineage
- find examples
- suggest implementation
- onboard agent

#### Layer 5: Service / Interface Layer

Consumer-facing surfaces that invoke the workflow/query layer:

- browser conversation UI
- static HTML pages with query hooks
- agent-facing tools / skills / API surfaces
- future Ash-native knowledge services

### D3.5: Shared Corpus Analysis Substrate, Separate Products

The Ash wiki SHOULD reuse a shared document/corpus analysis substrate with the spec processor and, where practical, compatible finding/evidence conventions with `ash-lint`.

That substrate should cover reusable analysis primitives such as:

- corpus discovery
- frontmatter/markdown extraction
- normalized artifact identity
- relationship graph construction
- evidence/finding base models

However, the products MUST remain distinct:

- the spec processor remains a repository-audit / CI-facing tool (`DESIGN-SPEC-PROCESSOR`, `PLAN-090`)
- the Ash wiki remains the broader corpus-semantic, navigation, audit, onboarding, and query/service layer
- `ash-lint` remains the source-code lint product (`SPEC-041`)

See [DESIGN-NOTE: Shared Document / Corpus Analysis Substrate](DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md).

### D4: Human and AI Consumers Share the Corpus but Not Necessarily the Interface

The same corpus must serve two classes of clients.

#### Human-facing needs

- readable pages
- navigation
- historical understanding
- explanatory synthesis
- static rendering
- browser conversation

#### AI-facing needs

- stable schema and registry exports
- authority-aware retrieval
- onboarding packs
- traceability links
- drift evidence
- example bundles
- predictable query contracts

Different interfaces are expected and desirable, but they must share one underlying truth-maintained corpus.

### D5: Authority, Status, and Health Must Be Distinct

The wiki must avoid collapsing orthogonal dimensions into a single "status" string.

#### Authority

Defines whether an artifact is normative, advisory, exploratory, or historical.

#### Status

Defines lifecycle state such as draft, current, active, blocked, done, superseded, archived.

#### Health

Defines alignment state such as aligned, stale, known-drift, or needs-review.

This separation allows the wiki to represent cases like:

- current + advisory + aligned
- superseded + historical + aligned
- current + normative + known-drift
- draft + exploratory + needs-review

### D6: Supersession Must Support Partial Scope

Full supersession is insufficient for Ash documentation. A newer artifact may supersede only part of an older one.

The design therefore requires explicit scope on supersession records, for example:

- entire document
- sections / anchors
- named topics
- implementation surface subsets

Residual authority must remain expressible when a document is only partially superseded.

### D7: Lint/Audit Is a First-Class Product

Documentation lint is not merely style checking. The wiki must support first-class audit questions, including:

- stale or misclassified documents
- unresolved or undocumented supersession
- broken lineage from spec -> plan -> task -> implementation
- current artifacts citing superseded materials without explanation
- documented drift between spec and implementation
- undocumented drift inferred from audit findings
- missing onboarding or authority surfaces for active subsystems

Lint findings themselves should become durable wiki artifacts or registry entries, not only terminal output.

### D8: Computed Views Are First-Class Outputs

Not every useful wiki page should be hand-authored. The design explicitly allows computed views, such as:

- current normative surface by subsystem
- supersession map
- drift dashboard
- active work map
- topic index
- onboarding reading paths
- implementation trace matrix

These computed views should preferably materialize as markdown and/or JSON so they stay inspectable in static workflows.

### D9: Browser Conversation Invokes Typed Query Workflows

The browser conversation model should not be a free-form "ask an LLM over docs" pattern only. It should be grounded in typed corpus workflows such as:

- `explain_topic(topic, audience, detail_level)`
- `trace_history(subject)`
- `show_normative_surface(subsystem)`
- `audit_drift(subject)`
- `map_lineage(subject)`
- `find_examples(concept)`
- `onboard_agent(profile)`

Natural-language interaction can route into these workflows, but the underlying operations should remain structured.

### D10: Onboarding and Library Service Are Core, Not Extras

The Ash wiki should act as a library-style knowledge service for both humans and agents.

Initial service products should include:

- project orientation bundle
- subsystem maps
- terminology/glossary pack
- authority map
- active work map
- known drift / unresolved issues pack
- implementation example pack

This is especially important because Ash aims to become a substrate for human-AI interaction and agent-native development.

## Proposed Artifact Families

### A. Source Artifacts

Existing repo materials: specs, designs, plans, tasks, ideas, notes, references.

### B. Synthesized Wiki Pages

Markdown pages that summarize or organize source artifacts without replacing them.

Examples:
- subsystem overviews
- history pages
- topic maps
- onboarding guides
- current-state summaries

### C. Audit Records

Structured findings about drift, staleness, supersession, or traceability gaps.

### D. Registries and Indexes

Machine-readable outputs that enable reliable tooling and query workflows.

### E. Service Exports

Bundles or structured outputs prepared for humans or agents, such as onboarding manifests or glossary packs.

## Trust Model

The design requires each artifact/view/export to be legible in terms of trust.

- canonical documents define official contract surfaces
- synthesized pages summarize but do not outrank canonical sources
- computed indexes derive from corpus state and may become stale if regeneration fails
- audit findings are evidence-backed and explicitly scoped
- conversational answers should cite the corpus pages or computed views they rely on

A browser or workflow answer must be able to distinguish between:

- canonical claim
- synthesized summary
- computed inference
- unresolved ambiguity

## Relationship to the LLM-Wiki Pattern

Useful adopted ideas:

- persistent markdown knowledge base
- cross-linked pages
- compounding synthesis
- explicit indexes and logs
- static tool compatibility

Necessary Ash-specific extensions:

- stronger authority model
- stronger supersession model
- stronger drift/audit model
- explicit service exports for onboarding and agents
- deeper coupling to implementation traceability and project workflows

## Rollout Strategy

A staged rollout is required.

### Stage 1: Corpus Semantics

- define metadata schema
- classify authority/status/health
- define supersession and drift record shapes
- create initial topic and subsystem maps

### Stage 2: Computed Views

- generate document registry
- generate authority/supersession/drift indexes
- publish static computed views

### Stage 3: Audit Workflows

- implement corpus-state lint
- implement cross-document consistency lint
- implement drift registry checks

### Stage 4: Query and Onboarding Services

- define query contracts
- generate onboarding bundles
- expose browser and agent interfaces

### Stage 5: Ash-Native Integration

- route selected query and audit workflows through Ash-native services where appropriate
- reuse the substrate for future agentic or library-service workloads

## Open Questions

1. Which metadata fields should be mandatory on all artifacts versus only on wiki-managed ones?
2. Should drift/audit records live inside source-document frontmatter, companion records, or dedicated audit directories?
3. Which computed views should be regenerated in CI versus on-demand locally?
4. Which initial query workflows are highest leverage: onboarding, authority lookup, or drift audit?
5. How should static HTML pages surface conversation hooks while remaining tool-agnostic?

## Recommended Next Step

Promote this design into:

- a formal spec for artifact semantics, metadata, supersession, audits, and service exports
- an implementation plan that stages the substrate without prematurely committing to one UI or retrieval backend
