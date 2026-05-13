# DESIGN-035: Documentation Corpus Governance

## Status

Draft design note. This document is a methodology and architecture proposal for later formal specification and task planning. It does not by itself create the canonical reference/teaching corpus or implement tooling.

## Summary

Ash documentation should be treated as two related but distinct systems:

1. `docs/` remains the evolving work corpus: plans, specs, ideas, design notes, task evidence, audits, historical reasoning, stale-but-useful artifacts, and active WIP.
2. A separate top-level curated corpus, provisionally called `knowledge/`, becomes the new-style surface for current reference, teaching resources, LLM packs, catalog metadata, and extraction manifests.

Git commits and tags are the actual snapshots of project state. Named documentation snapshots should therefore be lightweight manifests that name a git state, phase range, extraction profile, and verification evidence, not hand-maintained duplicate directory trees.

The central maintenance invariant is:

> A document may be current, stale, superseded, exploratory, historical, rejected, or derived. It must not be ambiguously current.

This design complements [DESIGN-029: Ash Wiki Knowledge Substrate](DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE.md) and [DESIGN-NOTE: Shared Document / Corpus Analysis Substrate](DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md). DESIGN-029 establishes the broader static-first knowledge substrate; this note narrows the operational governance model: where curated material lives, how the existing `docs/` corpus is preserved, how frontmatter/catalog metadata accumulates, how drift is managed, and what librarian/editor/verifier tooling should do.

## Motivation

Ash is now usable enough to need reliable reference and teaching resources, but the language and libraries are still moving quickly. The existing `docs/` tree is chaotic in a natural and valuable way: it records evolving methods, exploratory decisions, plans, task evidence, partially superseded reasoning, and implementation history.

A one-shot cleanup would damage that history and still fail to keep up with development velocity. The project instead needs a documentation governance layer that can gradually organize, classify, and extract reliable views from the corpus while preserving development history.

The desired end state is not a cleaned-up `docs/` directory. The desired end state is:

- a preserved development corpus in `docs/`;
- curated current reference and teaching documents outside `docs/`;
- machine-readable catalog metadata for both old and new artifacts;
- named git-state snapshot manifests for important project states;
- tooling that lets librarian, editor, verifier, and publisher agents maintain the corpus incrementally.

## Goals

1. Preserve `docs/` as the WIP and development-history corpus for the foreseeable future.
2. Establish a separate top-level curated corpus for new-style reference, teaching, LLM packs, catalog indexes, and extraction manifests.
3. Support gradual cataloging without requiring immediate movement or rewriting of old documents.
4. Preserve document history and reasoning traces while making currentness, authority, and stale status explicit.
5. Treat git commits/tags as the real snapshots and use named manifests to identify meaningful project states.
6. Define frontmatter and sidecar metadata sufficient for later classification, drift detection, extraction, and publishing tools.
7. Define a maintenance methodology that fits Ash's fast-moving development model.
8. Define agent roles and tool responsibilities for librarian/editor/verifier/publisher workflows.
9. Provide a bridge to later formal specs, implementation plans, and task files.

## Non-goals

This design does not:

- reorganize the existing `docs/` tree;
- declare the final name of the curated top-level directory as normative;
- replace existing specs, plans, tasks, or changelog policy;
- require every historical document to receive frontmatter in the first pass;
- require a vector database, web UI, or service runtime;
- define a full command-line interface for the future toolkit;
- guarantee that all current docs are accurate before the catalog exists;
- make teaching or LLM-pack documents normative sources of Ash semantics.

## Core design decisions

### D1: Preserve `docs/` as the work corpus

`docs/` remains the primary project workbench for:

- canonical specs under `docs/spec/`;
- plans and task files under `docs/plan/`;
- design notes under `docs/design/`;
- ideas and exploratory history under `docs/ideas/`;
- notes, audits, closeouts, and implementation evidence.

The project should not try to make every file in `docs/` globally current. Instead, every file should eventually become interpretable: current, stale, superseded, exploratory, historical, rejected, merged, or unknown.

This preserves the reasoning archaeology that is necessary for future design work.

### D2: Put curated current products in a separate top-level directory

New-style canonical-adjacent products should not live under `docs/` by default. A separate top-level directory gives humans and agents a clear boundary between development history and curated reading surfaces.

The working name in this design is `knowledge/`:

```text
knowledge/
  README.md
  policy/
  catalog/
  reference/
  teaching/
  llm-packs/
  manifests/
```

The final name should be decided in the later spec/plan packet. Candidate names include:

| Name | Strength | Risk |
|------|----------|------|
| `knowledge/` | Fits shared human/LLM substrate and multiple products | Weaker signal of canonical currentness |
| `reference/` | Clear current reading surface | Awkward for teaching and LLM packs |
| `canon/` | Strong boundary from `docs/` | May overstate normativity of derived material |
| `manual/` | Human-friendly | Too teaching/book-like for agents and metadata |

This design uses `knowledge/` as the provisional name because it best covers reference, teaching, catalog, and LLM-pack products without claiming that every page is normative.

### D3: Keep canonical authority separate from curated usefulness

The curated corpus can contain high-quality current explanations, but authority still comes from explicit sources:

- specs define normative behavior;
- implementation and tests provide realized behavior;
- plans/tasks provide scoped development intent and evidence;
- design notes provide rationale;
- curated reference pages synthesize and route to authority;
- teaching documents derive from reference pages and examples;
- LLM packs derive from reference pages, examples, and current-limitation notes.

A reference page may be the best place to read about a topic, but it must cite the authority that backs its claims.

### D4: Git stores states; manifests name states

The project should not maintain copied snapshot trees as source artifacts. Git already stores immutable project states.

A named snapshot is a manifest that records:

- snapshot ID;
- title and purpose;
- git commit/tag/branch state;
- phase or task range;
- extraction profiles;
- verification evidence;
- known exclusions;
- refresh or regeneration instructions.

Example:

```yaml
id: snapshot.phase-115
kind: named-snapshot
name: phase-115-associated-family-mvp
git:
  commit: <sha>
  tag: null
phase_range:
  through: 115
meaning:
  - Associated type-family computation implemented MVP.
  - SPEC-063 and PLAN-111 closeout state included.
extract:
  profiles:
    - reference-core
    - llm-core-semantics
verification:
  date: 2026-05-13
  evidence:
    - linkcheck
    - status-check
    - selected-example-check
known_exclusions:
  - Legacy tutorials not refreshed.
```

Materialized snapshot outputs are generated artifacts. They may be published or cached, but they should not become hand-maintained source-of-truth copies.

### D5: Catalog gradually with frontmatter and sidecars

Catalog metadata should accumulate incrementally during research, auditing, and document maintenance.

Preferred order:

1. Inventory existing artifacts without moving them.
2. Add sidecar catalog entries where direct edits are too invasive.
3. Add frontmatter to documents when they are touched for legitimate reasons.
4. Generate indexes from frontmatter plus sidecar metadata.
5. Use tooling reports to prioritize missing metadata.

This avoids a high-churn metadata migration while still creating a future tooling substrate.

### D6: Stale is a valid state; ambiguous currentness is not

It is acceptable for a document to be stale if that is explicit. It is unacceptable for stale or historical documents to appear current to humans or agents.

The governance model therefore separates:

- artifact kind;
- authority level;
- lifecycle status;
- health/currentness;
- supersession state;
- verification state.

A document can be `historical-useful` and `aligned` for its historical purpose while not being valid for current syntax generation.

### D7: Maintenance must be event-driven by development tasks

Every Ash or Ash-library development task should classify its documentation impact. Public semantic/API changes must either update affected curated surfaces or mark them stale with a refresh trigger.

Impact classes:

- `no-doc-change`;
- `spec-change`;
- `design-change`;
- `reference-change`;
- `library-api-doc-change`;
- `example-change`;
- `teaching-change`;
- `llm-pack-change`;
- `catalog-change`;
- `drift-repair`.

Task closeout should record which classes apply and which surfaces were updated, deferred, or marked stale.

### D8: Different agent roles need different authority

The system should distinguish agent roles:

| Role | Primary function | Must not do alone |
|------|------------------|-------------------|
| Librarian | Inventory, classify, link, propose metadata | Rewrite current truth without verifier evidence |
| Editor | Distill reference/teaching pages | Certify implementation correctness alone |
| Verifier | Compare docs to specs/code/tests/examples | Rewrite history for clarity |
| Archivist | Mark superseded/stale/rejected/merged states | Delete reasoning traces by default |
| Publisher | Build named snapshot manifests and extracts | Treat generated extracts as source truth |

This separation prevents a single pass from both reinterpreting history and certifying current truth.

## Proposed repository layout

The initial layout should be minimal:

```text
docs/
  ... existing development corpus remains ...

knowledge/
  README.md
  policy/
    documentation-policy.md
    archive-policy.md
    artifact-schema.md
    extraction-policy.md
    librarian-editor-verifier-workflow.md
  catalog/
    artifacts.yaml
    concept-map.yaml
    source-map.yaml
    archive-index.yaml
    drift-index.yaml
  reference/
    concepts/
    libraries/
    diagnostics/
  teaching/
    paths/
    tutorials/
  llm-packs/
    profiles/
  manifests/
    snapshots/
    extraction-profiles/
```

`knowledge/` is source material for curated products and policy. Generated extracts should either be ignored, placed under a build/output directory, or clearly marked as generated.

## Artifact model

A future spec should define a common artifact record. This design proposes the following core fields.

```yaml
id: ash.doc.<stable-id>
kind: idea | design | spec | plan | task | audit | reference | teaching | llm-pack | manifest | generated-view
title: Human-readable title
path: repository/relative/path.md
authority: normative | canonical-adjacent | implementation-evidence | advisory | exploratory | historical | derived
lifecycle: draft | active | implemented-mvp | implemented | partial | deferred | superseded | rejected | merged | archived
health: current-verified | current-unverified | likely-stale | known-stale | historical-only | unknown
owner_subsystem: workflow | type-system | runtime | parser | stdlib | docs | tooling | unknown
introduced_in:
  phase: null
  task: null
  commit: null
verified_against:
  commit: null
  specs: []
  plans: []
  tasks: []
  code: []
supersedes: []
superseded_by: []
archive:
  status: null
  reason: null
  valid_for: []
  do_not_use_for: []
refresh_triggers: []
extraction:
  include_in_profiles: []
  exclude_from_profiles: []
  notes: null
```

The actual frontmatter schema may be smaller for MVP. The important design point is that authority, lifecycle, and health are distinct.

## Frontmatter policy

### Mandatory for new curated documents

Every new document under the curated top-level corpus should include frontmatter from the start.

Minimum:

```yaml
---
id: ash.reference.concept.act
kind: reference
title: Act
status: draft
authority: canonical-adjacent
health: current-unverified
verified_against:
  specs:
    - SPEC-047
    - SPEC-054
source_links:
  - docs/spec/SPEC-047-ACT-MONAD.md
refresh_triggers:
  - act-runtime-semantics-change
  - do-notation-target-change
---
```

### Gradual for existing `docs/` artifacts

Existing `docs/` artifacts do not need immediate frontmatter. They may be cataloged through sidecars first.

When an old file is edited for a real reason, the editor should consider adding frontmatter, but should not churn the tree solely for metadata unless a task is explicitly scoped as catalog migration.

### Sidecar catalog entries

Sidecars can represent metadata for untouched files:

```yaml
id: ash.idea.future.first-class-workflows
path: docs/ideas/future/FIRST-CLASS-WORKFLOWS.md
kind: idea
authority: historical
lifecycle: superseded
health: historical-only
superseded_by:
  - docs/spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md
classification_notes:
  - Unary Workflow<A> implemented a reduced version of the original idea.
```

Tooling should merge sidecar and frontmatter metadata with predictable precedence.

## Archive policy

Archive is a retrieval and interpretation state, not deletion.

Archive statuses:

| Status | Meaning | Retrieval policy |
|--------|---------|------------------|
| `historical-useful` | Old but still useful for rationale | Searchable with historical warning |
| `superseded` | Replaced by newer artifact | Route readers to successor first |
| `stale-needs-refresh` | Intended to be current but no longer verified | Exclude from current packs unless explicitly requested |
| `obsolete-do-not-use` | Known wrong for current Ash | Exclude from generation and warn loudly |
| `rejected` | Deliberately abandoned | Preserve rationale; do not use as plan |
| `merged` | Absorbed into another artifact | Redirect to successor |

Archive records should include scope. Supersession may apply to an entire document, a section, a concept, or a use case such as syntax generation.

## Curated reference and teaching policy

### Reference pages

Reference pages should be concise, current, authority-linked, and concept/API focused.

Each page should include:

- one-sentence meaning;
- current status;
- authority links;
- implementation links if relevant;
- what it is / what it is not;
- examples marked passing, illustrative, aspirational, or stale;
- common confusions;
- related concepts;
- LLM generation constraints when relevant.

### Teaching pages

Teaching pages are downstream derivatives. They should cite reference pages and may not introduce new semantics.

Each teaching page should include:

- audience;
- prerequisites;
- learning objectives;
- concepts covered;
- exercises or examples;
- reference dependencies;
- verification state.

### LLM packs

LLM packs are compressed, status-aware context bundles generated or maintained from reference pages, examples, and current-limitation notes.

They should be explicit about:

- implemented vs draft vs aspirational behavior;
- syntax that should not be generated;
- current examples;
- authority links;
- freshness and extraction profile.

## Development-time documentation impact

Every task that changes Ash or Ash libraries should include a documentation impact section.

Example:

```markdown
## Documentation Impact

Classification:
- [ ] no-doc-change
- [ ] spec-change
- [ ] design-change
- [ ] reference-change
- [ ] library-api-doc-change
- [ ] example-change
- [ ] teaching-change
- [ ] llm-pack-change
- [ ] catalog-change
- [ ] drift-repair

Required updates:
- [ ] CHANGELOG.md
- [ ] docs/spec/...
- [ ] docs/design/...
- [ ] knowledge/reference/...
- [ ] knowledge/catalog/...
- [ ] examples/...
- [ ] stale markers added where refresh is deferred

No-doc-change rationale:
...
```

Public semantic/API changes must not close with unknown documentation impact.

## Toolkit architecture

The toolkit should be static-first and initially local. A future service may wrap it, but no critical meaning should depend on a service.

### Shared substrate

Reuse the shared document/corpus analysis substrate from [DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE](DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md):

- corpus discovery;
- markdown/frontmatter extraction;
- normalized artifact identity;
- relationship graph construction;
- evidence/finding base model.

### Librarian commands

Candidate commands:

```text
ash-doc inventory
ash-doc classify
ash-doc id-resolve SPEC-063
ash-doc frontmatter check
ash-doc archive-report
```

Responsibilities:

- scan corpus;
- extract frontmatter and sidecar metadata;
- infer artifact kind/status from path and content;
- detect missing IDs;
- propose classification patches;
- report unknown/stale/superseded candidates.

### Editor commands

Candidate commands:

```text
ash-doc reference scaffold concept.act
ash-doc reference check concept.act
ash-doc teaching plan semantic-tower
ash-doc llm-pack build ash-core-semantics
```

Responsibilities:

- scaffold reference pages;
- ensure required sections exist;
- build teaching paths from concept dependencies;
- build LLM packs from reference/profile inputs;
- mark generated outputs clearly.

### Verifier commands

Candidate commands:

```text
ash-doc linkcheck
ash-doc status-check
ash-doc code-check
ash-doc example-check
ash-doc lib-surface-check
ash-doc drift --since snapshot.phase-115
```

Responsibilities:

- verify local links;
- compare status fields against specs/plans/tasks;
- check referenced source files/symbols exist;
- run examples marked passing;
- compare public library surfaces against docs;
- find affected curated docs since a named git state.

### Publisher commands

Candidate commands:

```text
ash-doc snapshot create phase-115 --commit HEAD
ash-doc extract --snapshot phase-115 --profile llm-core-semantics
ash-doc extract --snapshot phase-115 --profile reference-core
```

Responsibilities:

- create named snapshot manifests;
- materialize views from git states and extraction profiles;
- record verification evidence;
- prevent generated views from becoming hand-maintained source truth.

## Drift model

Drift should be represented as evidence, not only as a failure.

Drift categories:

| Category | Meaning |
|----------|---------|
| `status-drift` | Document status disagrees with plan/task/spec state |
| `authority-drift` | Derived/current doc cites a superseded or advisory source as authoritative |
| `implementation-drift` | Code/tests differ from documented current behavior |
| `example-drift` | Example marked passing no longer passes |
| `syntax-drift` | Current syntax differs from tutorial/reference examples |
| `library-surface-drift` | Public library item lacks or contradicts docs |
| `link-drift` | Links or source-backed identifiers no longer resolve |
| `snapshot-drift` | Named snapshot manifest cannot be extracted or verified |

Findings should include:

```yaml
id: drift.<stable-id>
category: example-drift
severity: warning | error | blocking
artifact: knowledge/reference/concepts/act.md
evidence:
  command: ash-doc example-check concept.act
  output: ...
related:
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
recommendation: Mark example stale or update to current syntax.
```

## Named snapshot manifests

Named snapshots should live under the curated corpus, not under `docs/`.

Proposed path:

```text
knowledge/manifests/snapshots/<name>.yaml
```

A manifest names a git state and extraction profiles. It should not copy the source tree.

Minimum fields:

```yaml
id: snapshot.<name>
kind: named-snapshot
title: ...
git:
  commit: ...
  tag: ...
phase_range: ...
created_at: ...
created_by: ...
meaning: []
extract:
  profiles: []
verification:
  evidence: []
known_exclusions: []
```

An extraction profile defines what to include, exclude, transform, and verify.

```yaml
id: extract.llm-core-semantics
include:
  - knowledge/reference/concepts/semantic-tower.md
  - knowledge/reference/concepts/act.md
  - knowledge/reference/concepts/proc.md
exclude:
  - '**/historical-only/**'
transforms:
  - strip_editor_notes
  - include_authority_links
verification:
  - linkcheck
  - status-check
```

## Maintenance cadence

### Per task

- classify documentation impact;
- update specs/plans/tasks/changelog as required;
- update curated reference if public current truth changed;
- mark teaching/LLM derivatives stale if not refreshed;
- update catalog/frontmatter for touched artifacts;
- run scoped checks.

### Per phase

- run corpus drift report for the phase;
- reconcile PLAN/task/spec status;
- update idea/design-to-implementation cross-links;
- create or update named snapshot manifest for major milestones;
- refresh high-priority LLM packs if public syntax/semantics changed.

### Periodic

- run full inventory;
- review unknown/stale/superseded candidates;
- publish updated current reference bundles;
- regenerate selected LLM packs;
- triage archive policy gaps.

## Initial pilot

The first implementation phase should avoid bulk migration. It should pilot the model on a small, high-value slice.

Recommended pilot slice:

```text
Pure < Act < Proc < Workflow
```

Pilot deliverables:

```text
knowledge/README.md
knowledge/policy/artifact-schema.md
knowledge/policy/archive-policy.md
knowledge/policy/extraction-policy.md
knowledge/catalog/artifacts.yaml
knowledge/reference/concepts/semantic-tower.md
knowledge/reference/concepts/act.md
knowledge/reference/concepts/proc.md
knowledge/reference/concepts/workflow.md
knowledge/llm-packs/profiles/ash-core-semantics.yaml
knowledge/manifests/snapshots/phase-115.yaml
```

Pilot verifier scope:

- all links resolve;
- every reference page has authority links;
- every example is marked passing/illustrative/aspirational/stale;
- snapshot manifest names a real git state;
- extraction profile can produce a deterministic file list;
- stale or excluded areas are explicit.

## Relationship to existing docs

### DESIGN-029

DESIGN-029 remains the broader knowledge-substrate design. This document narrows and updates the operational stance:

- `docs/` remains the development corpus;
- curated products live outside `docs/`;
- git commits/tags are the actual snapshots;
- named snapshots are manifests;
- frontmatter/sidecar metadata accumulates gradually.

### DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE

The shared substrate remains the correct lower layer for inventory, markdown parsing, identity extraction, relationship graphs, and findings. This design adds product policy above that substrate.

### SPEC-045

A later spec should decide whether SPEC-045 should be amended, superseded, or complemented by a narrower documentation-governance spec. This design should not silently rewrite SPEC-045's authority.

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| `knowledge/` becomes another stale docs tree | Require verified-against metadata and stale markers |
| Metadata migration becomes high-churn busywork | Allow sidecars and gradual frontmatter |
| Generated extracts get edited by hand | Mark generated outputs and regenerate from manifests |
| `docs/` stays uninterpretable | Inventory and archive reports prioritize classification gaps |
| Teaching docs drift from reference | Teaching docs cite reference dependencies and carry refresh triggers |
| LLM packs learn obsolete syntax | Packs derive from reference/current-limitation profiles and exclude stale docs by default |
| Tooling overcommits before schema is proven | Pilot on semantic tower slice only |

## Spec and plan starting points

Later implementation documentation should split this design into at least the following artifacts.

### Proposed spec: Documentation artifact governance

Normative topics:

- artifact kinds;
- authority/lifecycle/health vocabulary;
- frontmatter and sidecar merge rules;
- archive/supersession semantics;
- documentation impact classification;
- named snapshot manifest schema;
- generated extract provenance rules.

### Proposed plan: Documentation corpus organization rollout

Suggested task groups:

1. Create `knowledge/` skeleton and policy docs.
2. Define artifact metadata schema and sidecar precedence rules.
3. Implement corpus inventory and ID resolver.
4. Implement link/status/archive report checks.
5. Pilot frontmatter/sidecar classification on 20-30 artifacts.
6. Create semantic tower reference pilot.
7. Create phase-115 named snapshot manifest and extraction profile.
8. Add task-template documentation impact section.
9. Add public Ash-library surface doc inventory check.
10. Close out with drift report and spec/plan status reconciliation.

### Proposed task-template amendment

Add a `Documentation Impact` section to all future Ash task files once the governance spec is accepted.

## Open questions

1. Should the curated top-level directory be named `knowledge/`, `reference/`, `canon/`, or something else?
2. Should sidecar metadata be one central `artifacts.yaml`, one file per artifact, or both?
3. Which fields are mandatory for MVP frontmatter versus later strict mode?
4. Should generated LLM packs be committed, gitignored, or committed only for named releases?
5. Which examples should be part of the first `example-check` gate?
6. How should public Ash library surface extraction work before the standard library stabilizes further?
7. Should named snapshot manifests be tied to git tags, phase numbers, or both?

## Recommended next step

Create a formal spec and implementation plan for documentation corpus governance. The spec should freeze the metadata vocabulary and named-snapshot manifest model. The plan should start with a low-risk pilot: `knowledge/` skeleton, metadata schema, inventory tool, and semantic tower reference slice.
