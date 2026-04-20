# SPEC-045: Ash Wiki Knowledge Substrate

**Status:** Draft
**Date:** 2026-04-20
**Related:** DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE, FUTURE-004, SPEC-026, SPEC-040, SPEC-041, SPEC-042, SPEC-043

## 1. Summary

This specification defines the Ash wiki as a static-first, human/AI shared knowledge substrate over the Ash project corpus.

The Ash wiki does not replace canonical specifications. Instead, it layers explicit metadata, authority and lifecycle semantics, supersession tracking, documentation-integrity audit records, computed indexes/views, and service/export surfaces over the repository corpus so that the same body of knowledge can support:

1. static reading and rendering
2. documentation-integrity and drift auditing
3. conversational explanation and analysis
4. onboarding and library-style knowledge services for humans and AI agents

## 2. Motivation

Ash is simultaneously:

- a programming language and execution substrate
- a documentation-heavy language/tooling project
- an exploration of deep human-AI interaction
- a future host for agent-native workflows and services

A file tree of markdown alone is not enough to support those goals. The project needs explicit answers to questions such as:

- what is authoritative?
- what is current?
- what is historical?
- what has been superseded, and to what scope?
- what is stale versus intentionally historical?
- how do specs, plans, tasks, code, and tests relate?
- how can a fresh human or agent onboard without replaying all history?

This spec defines the minimum contract necessary for the corpus to become serviceable.

## 3. Definitions

### 3.1 Canonical Artifact

A document whose repository role already defines project truth or official engineering intent, such as a spec, reference document, design, plan, or task.

### 3.2 Wiki Artifact

Any artifact managed as part of the Ash wiki substrate, including canonical artifacts, synthesized pages, audit records, computed views, registries, and service exports.

### 3.3 Synthesized Page

A human-readable page that summarizes, organizes, or cross-links canonical artifacts without replacing their authority.

### 3.4 Computed View

A generated output derived from wiki artifacts, typically as markdown, JSON, or YAML.

### 3.5 Audit Record

A durable record describing a documentation-integrity or drift finding, including evidence and remediation state.

### 3.6 Service Export

A structured output or bundle prepared for a consumer such as a browser UI, agent skill, onboarding tool, or future Ash-native knowledge service.

## 4. Core Principles

### 4.1 Static First

The wiki corpus MUST remain useful as ordinary files in the repository. No critical meaning MAY exist only in a running application.

### 4.2 Canonical Separation

Canonical specifications and references remain authoritative. Synthesized pages and service exports MUST NOT silently supersede canonical artifacts.

### 4.3 Explicit Authority

Authority, lifecycle status, and alignment/health MUST be modeled explicitly rather than inferred from filenames or folders alone.

### 4.4 Evidence-Backed Audits

Audit and drift findings MUST carry concrete evidence and scope. The wiki MUST distinguish findings from guesses.

### 4.5 Human/AI Shared Corpus

The same substrate MUST support both human and AI consumers, though they MAY use different interfaces or exports.

### 4.6 Graceful Degradation

If query services, browser chat, or agent tooling are unavailable, the corpus MUST still remain navigable and meaningful via static pages and computed views.

### 4.7 Shared Analysis Substrate, Separate Products

The Ash wiki SHOULD reuse a shared document/corpus analysis substrate with:

- the spec processor / repository-audit workflow (`PLAN-090`, `DESIGN-SPEC-PROCESSOR`)
- compatible finding/evidence conventions with `ash-lint` where practical (`SPEC-041`)

That shared substrate SHOULD own reusable primitives such as corpus discovery, markdown/frontmatter extraction, normalized artifact identity, relationship graphs, and shared evidence/finding base models.

However, these products MUST remain distinct:

- the spec processor is a CI/repository-audit product
- the Ash wiki is the broader corpus-semantic, audit, onboarding, and query/service product
- `ash-lint` is the source-code lint product

See [DESIGN-NOTE: Shared Document / Corpus Analysis Substrate](../design/DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md).

## 5. Layer Model

The Ash wiki consists of five logical layers.

### 5.1 Layer 1: Canonical Artifact Layer

Source artifacts already present in the repository, including:

- specs
- reference docs
- designs
- plans
- tasks
- ideas / notes
- historical records

### 5.2 Layer 2: Wiki Knowledge Layer

Markdown artifacts that organize or synthesize the corpus, including:

- subsystem pages
- topic pages
- history pages
- authority maps
- onboarding guides
- drift and supersession pages

### 5.3 Layer 3: Computed Index Layer

Derived machine-usable outputs, including:

- document registry
- anchor registry
- authority/status registry
- supersession graph
- drift registry
- traceability registry
- onboarding manifests

### 5.4 Layer 4: Workflow / Query Layer

Structured workflows that operate over Layers 1-3, such as:

- explain topic
- trace history
- show normative surface
- audit drift
- map lineage
- find examples
- onboard agent

### 5.5 Layer 5: Service / Interface Layer

Consumer-facing surfaces over the corpus and workflows, such as:

- browser conversation UI
- static HTML with query hooks
- agent skills/tools
- future Ash-native knowledge services

## 6. Artifact Taxonomy

Every wiki-managed artifact MUST classify its role via `type`.

### 6.1 Required `type` Values

At minimum, the following types MUST be supported:

- `spec`
- `reference`
- `design`
- `plan`
- `task`
- `idea`
- `note`
- `audit`
- `query`
- `comparison`
- `topic`
- `history`
- `index`
- `registry`
- `service-export`

Additional types MAY be introduced if documented centrally.

## 7. Authority / Status / Health Model

### 7.1 Authority

Every wiki-managed artifact MUST declare one of:

- `normative`
- `advisory`
- `exploratory`
- `historical`

Rules:

1. Only `spec` and `reference` artifacts MAY declare `authority: normative`.
2. An artifact with `authority: historical` MUST NOT also declare itself current normative truth.
3. Synthesized pages SHOULD normally use `advisory` unless they are purely archival/historical.

### 7.2 Status

Every wiki-managed artifact MUST declare one of:

- `draft`
- `current`
- `active`
- `blocked`
- `done`
- `superseded`
- `archived`

Rules:

1. `superseded` indicates intentional replacement.
2. `archived` indicates retired or preserved material not expected to receive ongoing maintenance.
3. `current` is valid for long-lived docs; `active` is valid for currently in-progress engineering artifacts.
4. `done` is valid for completed tasks/plans or published audit/report artifacts.

### 7.3 Health

Every wiki-managed artifact SHOULD declare one of:

- `aligned`
- `known-drift`
- `stale`
- `needs-review`

Rules:

1. `stale` means the artifact is still intended to matter but is no longer aligned with current reality.
2. `historical` is not a health state.
3. `known-drift` requires a linked audit record or explicit evidence section.

## 8. Required Metadata Contract

### 8.1 Carrier Model

The Ash wiki uses a hybrid carrier model:

1. YAML frontmatter is the preferred carrier for human-authored wiki-managed markdown artifacts.
2. Registry fallback records are allowed for legacy artifacts that have not yet been normalized to frontmatter.
3. Generated views/registries MAY carry metadata either as frontmatter (for markdown outputs) or as explicit structured records.

Frontmatter is preferred for new or materially revised wiki-managed documents. Registry fallback is an adoption mechanism, not the preferred steady-state authoring mode.

The concrete carrier rules, normalized field guidance, and adoption policy are defined in:

- [Ash Wiki Metadata Schema](../reference/ash-wiki-metadata-schema.md)

### 8.2 Minimum Logical Fields

All wiki-managed artifacts MUST normalize to the following minimum metadata fields, regardless of carrier:

- `id`
- `title`
- `type`
- `authority`
- `status`
- `updated`

### 8.3 Recommended Common Fields

The following fields SHOULD be provided wherever applicable:

- `health`
- `subsystem`
- `phase`
- `tags`
- `related`
- `supersedes`
- `superseded_by`
- `depends_on`
- `derived_from`
- `related_specs`
- `related_plans`
- `related_tasks`
- `related_code`
- `related_tests`
- `canonical_for`
- `aliases`
- `summary`
- `intended_audience`

### 8.4 Conditionally Required Fields

The following fields are REQUIRED when the corresponding condition applies:

- `supersession_scope` when supersession is partial
- `residual_authority` when supersession is partial and the predecessor retains authority in some scope
- `drift_status` and `drift_causes` when `health: known-drift`
- `evidence` for every audit artifact

### 8.5 Normalization Rule

Tooling MUST normalize frontmatter-carried metadata and registry-fallback metadata into one logical schema before:

- validation
- registry generation
- computed views
- audit/lint checks
- query/service exports

## 9. Supersession Model

### 9.1 Full Supersession

If an artifact is fully replaced by another artifact:

- predecessor MUST declare `status: superseded`
- predecessor MUST declare `superseded_by`
- successor SHOULD declare `supersedes`

### 9.2 Partial Supersession

If only part of an artifact is replaced, the predecessor MUST declare:

- `superseded_by`
- `supersession_scope`
- `residual_authority`

`supersession_scope` MUST describe the replaced surface explicitly, for example by:

- section / anchor identifiers
- named topics
- implementation surfaces
- subsystem slices

### 9.3 Historical Presentation

Superseded or historical artifacts MUST carry a visible presentation marker in rendered/static output so they are not mistaken for current authoritative documents.

## 10. Audit Record Model

Audit records are first-class artifacts with `type: audit`.

### 10.1 Required Fields for Audit Records

- `id`
- `title`
- `subject`
- `authority: advisory` or `historical`
- `status`
- `updated`
- `severity`
- `finding_type`
- `evidence`

### 10.2 Supported Audit Families

At minimum, the wiki MUST support these audit families:

1. corpus-state audit
2. cross-document consistency audit
3. spec-implementation drift audit
4. onboarding coverage audit
5. traceability audit

### 10.3 Drift Cause Taxonomy

For drift-related findings, the wiki MUST support at least these cause classes:

- `implementation-missing`
- `implementation-ahead`
- `design-conflict`
- `parser-gap`
- `substrate-gap`
- `doc-lag`
- `verification-gap`
- `ambiguous-spec`

### 10.4 Evidence Requirement

Every audit record MUST cite concrete evidence such as:

- document paths and anchors
- code file paths and symbols
- tests or commands
- dated observations

## 11. Computed Views and Registries

The wiki MUST support computed views and registries as inspectable outputs.

### 11.1 Minimum Required Computed Views

- current normative surface
- supersession map
- drift dashboard
- active work / lineage map
- onboarding map

### 11.2 Minimum Required Registries

- document registry
- authority/status registry
- supersession registry
- audit/drift registry

Computed views SHOULD be materialized as markdown and MAY also be emitted as JSON/YAML.

## 12. Query and Service Contracts

The wiki MUST support structured query workflows even if the initial UI is conversational.

### 12.1 Required Query Classes

At minimum, the system MUST support workflows equivalent to:

- explain a topic
- trace history / supersession
- show current authority surface
- audit drift / stale state
- map lineage from concept/spec to plan/task/code/test
- produce onboarding bundles

### 12.2 Output Trust Requirements

Query/service outputs MUST distinguish between:

- canonical claims
- synthesized summaries
- computed inferences
- unresolved ambiguity

Whenever possible, outputs SHOULD cite the artifacts they used.

## 13. Onboarding / Library-Service Exports

The Ash wiki MUST support exports or bundles intended for onboarding humans and agents.

### 13.1 Minimum Onboarding Bundle Contents

- project overview
- subsystem map
- terminology/glossary
- authority model
- active work snapshot
- known drift / unresolved issues snapshot
- example starting points

### 13.2 Consumer Modes

The same source corpus MAY produce different exports for:

- human readers
- browser conversation
- external AI agents
- Ash-native knowledge services

## 14. Static Rendering Requirements

The wiki MUST remain renderable as static documents.

Static-rendered output SHOULD preserve or surface:

- visible title and identity
- authority/status/health badges or equivalents
- supersession banners
- backlinks/related links where available
- query hook links or references where available

The static-rendered form MUST remain useful even if interactive services are absent.

## 15. Initial Implementation Tasks

This spec anticipates at least the following workstreams:

1. metadata schema introduction
2. corpus classification and authority mapping
3. supersession model and records
4. audit record model and initial lints
5. computed view/registry generation
6. onboarding/library-service exports
7. browser and agent query surfaces

These tasks are expected to be decomposed in a separate implementation plan and task files.

## 16. Changelog

### 2026-04-20

- Initial draft of the Ash wiki knowledge substrate specification.
