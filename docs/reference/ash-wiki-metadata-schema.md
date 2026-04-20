# Ash Wiki Metadata Schema

## Status

Phase 94 metadata-carrier decision and reference schema.

## Purpose

This document defines the concrete metadata carrier model for the Ash wiki described by:

- [SPEC-045: Ash Wiki Knowledge Substrate](../spec/SPEC-045-ASH-WIKI.md)
- [DESIGN-029: Ash Wiki Knowledge Substrate](../design/DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE.md)
- [2026-04-20 Ash Wiki Implementation Plan](../plans/2026-04-20-ash-wiki-implementation-plan.md)

It answers the Phase 1 question left open in the initial packet: how wiki metadata is stored for canonical documents, synthesized pages, audit records, and generated outputs.

## Decision

Use a hybrid carrier model:

1. YAML frontmatter is the preferred carrier for human-authored wiki-managed artifacts.
2. Registry fallback records are allowed for legacy or externally constrained artifacts that do not yet carry frontmatter.
3. Generated outputs may emit metadata either as frontmatter (for markdown views) or as explicit structured records (for JSON/YAML registries).

This model preserves static markdown ergonomics while allowing gradual adoption across the existing Ash corpus.

## Why Hybrid Rather Than Frontmatter-Only

A frontmatter-only migration would impose high churn across the current corpus and create unnecessary friction for early adoption. Ash needs machine-usable metadata soon, but it does not need a flag day that rewrites every document at once.

A hybrid model allows:

- newly created or materially revised docs to adopt frontmatter immediately
- older docs to remain readable while being registered through external metadata
- deterministic tooling over one normalized logical schema
- static rendering and editor workflows to keep working without special infrastructure

## Carrier Rules

### Rule 1: Frontmatter Preferred for Human-Authored Managed Docs

The following artifact families SHOULD use YAML frontmatter when created or materially revised:

- wiki topic/history/index pages
- new specs, designs, plans, tasks, references created under the Ash wiki rollout
- audit records written as markdown
- onboarding/service markdown exports intended for static browsing

### Rule 2: Registry Fallback Allowed for Legacy Corpus

Legacy docs MAY be represented in a registry when one or more of the following is true:

- the document has not yet been normalized to frontmatter
- adding frontmatter would create disproportionate churn during early rollout
- the file format or generation path makes inline frontmatter awkward

Registry fallback is a migration mechanism, not the long-term preferred authoring mode.

### Rule 3: One Logical Schema Across Carriers

Regardless of carrier, the logical metadata fields are the same. Tooling MUST normalize frontmatter and registry entries into one common in-memory/document-registry shape.

### Rule 4: Frontmatter Wins on Direct Conflict

If both frontmatter and a registry fallback record exist for the same artifact and disagree on the same field, frontmatter is the source of truth unless the field is explicitly designated generated-only.

### Rule 5: Generated Fields Must Be Distinguished

Generated or derived fields SHOULD be clearly distinguished from authored fields in registry outputs. Examples include:

- `path`
- `links_to`
- `linked_from`
- `registry_generated_at`
- `render_targets`

## Minimum Logical Schema

Every wiki-managed artifact must normalize to the following minimum record shape:

```yaml
id: string
title: string
type: string
authority: normative | advisory | exploratory | historical
status: draft | current | active | blocked | done | superseded | archived
updated: YYYY-MM-DD
```

Recommended common fields:

```yaml
health: aligned | known-drift | stale | needs-review
summary: string
subsystem: string | [string]
phase: string | integer | [string]
tags: [string]
related: [string]
supersedes: [string]
superseded_by: [string]
depends_on: [string]
derived_from: [string]
related_specs: [string]
related_plans: [string]
related_tasks: [string]
related_code: [string]
related_tests: [string]
canonical_for: [string]
aliases: [string]
intended_audience: [string]
```

Conditionally required fields:

```yaml
supersession_scope: object | [object]
residual_authority: string | [string]
drift_status: none | known | accepted-temporary | unresolved
drift_causes: [implementation-missing, implementation-ahead, design-conflict, parser-gap, substrate-gap, doc-lag, verification-gap, ambiguous-spec]
evidence: [object]
```

## Frontmatter Shape

Preferred frontmatter format:

```yaml
---
id: "SPEC-045"
title: "Ash Wiki Knowledge Substrate"
type: spec
authority: normative
status: draft
health: aligned
updated: 2026-04-20
summary: "Static-first knowledge substrate and service contract over the Ash corpus."
related: [DESIGN-029, FUTURE-004]
related_plans: [PLAN-AWK-001]
tags: [wiki, metadata, audit, ai]
---
```

Notes:

- `id` SHOULD match the canonical document identifier where one exists.
- `title` SHOULD match the visible H1 title or canonical title string.
- `updated` is the wiki metadata freshness field; it is not a promise that all linked artifacts are aligned.
- Arrays SHOULD be used rather than comma-separated strings.

## Registry Fallback Shape

Registry fallback entries SHOULD live in a generated or maintained registry file and key by repository-relative path.

Example:

```yaml
path: docs/design/THREADING_MODEL.md
id: DESIGN-THREADING-MODEL
title: Threading Model
type: design
authority: advisory
status: current
health: needs-review
updated: 2026-04-20
summary: Historical and current design material for runtime threading.
carrier: registry-fallback
```

Required registry-only fields:

- `path`
- `carrier` (`frontmatter`, `registry-fallback`, or `generated`)

Recommended registry-only derived fields:

- `links_to`
- `linked_from`
- `anchors`
- `render_targets`
- `registry_generated_at`

## Required Field Guidance by Artifact Type

### Spec / Reference

Must include:

- `id`
- `title`
- `type`
- `authority`
- `status`
- `updated`

Strongly recommended:

- `health`
- `canonical_for`
- `related_code`
- `related_tests`
- `supersedes`
- `superseded_by`

Rules:

- Only `spec` and `reference` may declare `authority: normative`.
- A superseded normative artifact must still remain clearly discoverable historically.

### Design / Plan / Task

Must include:

- `id`
- `title`
- `type`
- `authority`
- `status`
- `updated`

Strongly recommended:

- `health`
- `related_specs`
- `related_plans`
- `related_tasks`
- `depends_on`
- `phase`
- `subsystem`

Rules:

- design/plan/task artifacts SHOULD normally be `authority: advisory`.
- completed tasks may use `status: done` while remaining `authority: advisory`.

### Idea / Note

Must include:

- `id`
- `title`
- `type`
- `authority`
- `status`
- `updated`

Strongly recommended:

- `summary`
- `related`
- `tags`

Rules:

- ideas and exploratory notes SHOULD normally use `authority: exploratory`.
- if preserved only for history, they MAY use `authority: historical`.

### Audit

Must include:

- all minimum fields
- `severity`
- `finding_type`
- `subject`
- `evidence`

Strongly recommended:

- `drift_status`
- `drift_causes`
- `related_code`
- `related_tests`
- `related_specs`
- `related_plans`
- `related_tasks`

### Generated Index / Registry / Service Export

Must include enough metadata to identify the output clearly:

- `id`
- `title`
- `type`
- `authority`
- `status`
- `updated`
- `carrier: generated` in the normalized registry

Recommended:

- `derived_from`
- `render_targets`
- `summary`

Rules:

- generated artifacts are typically `authority: advisory`.
- generated artifacts must link back to the source contract they summarize.

## Validation Rules

### Legal Combinations

1. `authority: normative` is legal only for `type: spec|reference`.
2. `authority: historical` must not be paired with `status: current`.
3. `status: superseded` requires `superseded_by`.
4. partial supersession requires both `supersession_scope` and `residual_authority`.
5. `health: known-drift` requires `drift_status`, `drift_causes`, and evidence or a linked audit record.
6. `type: audit` requires `evidence`.
7. `id` must be unique within the normalized registry.

### Recommended Lint Warnings

1. `status: current` with no `summary`
2. canonical artifacts with no `related_specs` / `canonical_for` / lineage fields where obviously applicable
3. large legacy slices remaining on registry fallback long after surrounding docs adopted frontmatter
4. stale `updated` values that lag behind linked successor artifacts or audit findings

## Adoption Policy

### Phase 1 Adoption Rule

During the first Ash wiki rollout slice:

- new Ash wiki docs MUST adopt frontmatter or an equivalent explicit metadata block if they live in the ideas/live-note space that already uses frontmatter
- legacy docs MAY stay without frontmatter if represented in the pilot registry
- the pilot registry MUST record which artifacts are still on fallback representation

### Promotion Rule

When a legacy document is materially revised for substantive Ash wiki work, it SHOULD be promoted from registry fallback to frontmatter unless there is a specific reason not to do so.

### No Flag Day Rule

The project MUST NOT require full-corpus frontmatter conversion before:

- registry generation
- initial linting
- initial onboarding bundles
- initial query workflows

## Example Normalized Records

### Example A: Current normative spec

```yaml
id: SPEC-045
title: Ash Wiki Knowledge Substrate
type: spec
authority: normative
status: draft
health: aligned
updated: 2026-04-20
related: [DESIGN-029, FUTURE-004]
canonical_for: [ash-wiki, authority-model, supersession-model, audit-model]
carrier: frontmatter
path: docs/spec/SPEC-045-ASH-WIKI.md
```

### Example B: Historical exploration retained for archaeology

```yaml
id: FUTURE-002
title: AI-Native Workflows and Generated Ash Programs
type: idea
authority: exploratory
status: draft
health: aligned
updated: 2026-04-20
related: [FUTURE-003, FUTURE-004]
carrier: frontmatter
path: docs/ideas/future/AI-NATIVE-WORKFLOWS.md
```

### Example C: Registry fallback for a legacy document

```yaml
id: DESIGN-THREADING-MODEL
title: Threading Model
type: design
authority: advisory
status: current
health: needs-review
updated: 2026-04-20
carrier: registry-fallback
path: docs/design/THREADING_MODEL.md
```

## Relationship to Future Audit Work

This schema is intentionally narrower than the full audit/lint contract. It defines how metadata is carried and normalized. Future audit specs and lints should treat this document as the carrier/validation baseline rather than redefining field shapes ad hoc.
